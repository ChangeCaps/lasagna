use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{Attribute, Data, DataStruct, DeriveInput, Fields, LitStr, Token, parse::{Parse, ParseStream}, parse_macro_input, spanned::Spanned};

struct MatchString(LitStr);

impl Parse for MatchString {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        <Token![=]>::parse(input)?;

        Ok(Self(<LitStr as Parse>::parse(input)?))
    }
} 

#[derive(Default)]
struct Attributes {
    match_string: Option<LitStr>,
    extern_token: bool,
}

impl Attributes {
    pub fn from_attrs(&mut self, attrs: &[Attribute]) {
        for attr in attrs {
            if attr
                .path
                .get_ident()
                .map(|ident| ident == "token")
                .unwrap_or(false)
            {
                if let Ok(MatchString(match_string)) =
                    syn::parse::Parser::parse2(MatchString::parse, attr.tokens.clone())
                {
                    self.match_string = Some(match_string);
                } else {
                    self.extern_token = true;
                }
            }
        }
    }
}

pub fn derive_token(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let mut attrs = Attributes::default();
    attrs.from_attrs(&input.attrs);

    let expanded = if attrs.match_string.is_some() {
        match_string_token(input, attrs)
    } else {
        source_token(input)
    };

    proc_macro::TokenStream::from(expanded)
}

fn match_string_token(input: DeriveInput, attrs: Attributes) -> TokenStream {
    if let Data::Struct(DataStruct {
        fields: Fields::Unit,
        ..
    }) = input.data
    {
        let name = input.ident;
        let match_string = match_string(&attrs.match_string.unwrap());

        quote! {
            impl lasagna::Token<char> for #name {
                #[inline]
                fn lex(lexer: &mut impl lasagna::Lexer<Output = char>) -> Result<Self, lasagna::ParseError> {
                    #match_string

                    Ok(Self)
                }
            }

            impl lasagna::Parse for #name {
                type Source = char;

                #[inline]
                fn parse(
                    parser: &mut impl lasagna::Parser<Source = Self::Source>
                ) -> Result<Self, lasagna::ParseError> {
                    parser.next::<Self>()
                }
            }
        }
    } else {
        panic!("match string must be a unit struct")
    }
}

fn match_string(match_string: &LitStr) -> TokenStream {
    quote! {
        for c in #match_string.chars() {
            let next = lasagna::Lexer::next(lexer);

            if let Some(next_char) = next {
                if next_char != c {
                    return Err(lasagna::ParseError::Expected {
                        span: lasagna::Lexer::span(lexer, 0),
                        found: String::from(next_char),
                        expected: String::from(c),
                    });
                }
            } else {
                return Err(lasagna::ParseError::Expected {
                    span: lasagna::Lexer::span(lexer, 0),
                    found: String::from("<eof>"),
                    expected: String::from(c),
                });
            }
        }
    }
}

fn source_token(input: DeriveInput) -> TokenStream {
    let name = input.ident;

    match input.data {
        Data::Enum(data) => {
            let mut lex_variant = Vec::new();
            let mut token_variants = Vec::new();
            let mut display_variants = Vec::new();

            for variant in data.variants {
                let mut attrs = Attributes::default();
                attrs.from_attrs(&variant.attrs);

                let variant_name = variant.ident;

                if let Some(string) = attrs.match_string {
                    display_variants.push(quote_spanned! {variant_name.span()=> 
                        Self::#variant_name => write!(f, "{}", #string)
                    });

                    let match_string = match_string(&string);

                    lex_variant.push(quote_spanned! {variant_name.span()=>
                        let mut fork = lexer.fork();

                        {
                            fn variant(
                                lexer: &mut impl lasagna::Lexer<Output = char>
                            ) -> Result<(), lasagna::ParseError> {
                                #match_string

                                Ok(())
                            } 

                            if variant(&mut fork).is_ok() {
                                *lexer = fork;

                                return Ok(Self::#variant_name);
                            }
                        }
                    });

                    token_variants.push(quote_spanned! {variant_name.span()=>
                        #[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
                        pub struct #variant_name(lasagna::Span);

                        impl lasagna::Spanned for #variant_name {
                            #[inline]
                            fn span(&self) -> lasagna::Span {
                                self.0
                            }
                        }

                        impl lasagna::Named for #variant_name {
                            const NAME: &'static str = #string;
                        }

                        impl lasagna::Token<#name> for #variant_name {
                            #[inline]
                            fn lex(
                                lexer: &mut impl lasagna::Lexer<Output = #name>
                            ) -> Result<Self, lasagna::ParseError> {
                                let span = lexer.span(0);
                                let next = lexer.next();

                                if let Some(next_source) = next {
                                    if let #name::#variant_name = next_source {
                                        Ok(Self(span | lexer.span(0)))
                                    } else {
                                        Err(lasagna::ParseError::msg(
                                            span | lexer.span(0), 
                                            format!("expected '{}'", <#variant_name as lasagna::Named>::NAME)
                                        ))
                                    }
                                } else {
                                    Err(lasagna::ParseError::eof(span, stringify!(#variant_name)))
                                }
                            }
                        }

                        impl lasagna::Parse for #variant_name {
                            type Source = #name;

                            #[inline]
                            fn parse(
                                parser: &mut impl lasagna::Parser<Source = Self::Source>
                            ) -> Result<Self, lasagna::ParseError> {
                                parser.next()
                            }
                        }
                    });
                } else if attrs.extern_token {
                    let field = if let Fields::Unnamed(unnamed) = &variant.fields {
                        if unnamed.unnamed.len() == 1 {
                            &unnamed.unnamed[0]
                        } else { 
                            unimplemented!("extern tokens must have one field")
                        }
                    } else {
                        unimplemented!("extern tokens must be unnamed variant")  
                    };

                    let field_ty = &field.ty;

                    display_variants.push(quote_spanned! {field_ty.span()=>
                        Self::#variant_name(_) => write!(f, "{}", <#field_ty as lasagna::Named>::NAME)
                    });

                    lex_variant.push(quote_spanned! {variant_name.span()=>
                        let mut fork = lexer.fork();

                        if let Ok(tok) = <#field_ty as lasagna::Token<char>>::lex(&mut fork) {
                            *lexer = fork;

                            return Ok(Self::#variant_name(tok));
                        }
                    });

                    token_variants.push(quote_spanned! {variant_name.span()=>
                        impl lasagna::Token<#name> for #field_ty {
                            #[inline]
                            fn lex(
                                lexer: &mut impl Lexer<Output = #name>
                            ) -> Result<Self, lasagna::ParseError> {
                                let span = lexer.span(0);
                                let next = lexer.next();

                                if let Some(tok) = next {
                                    #[allow(irrefutable_let_patterns)]
                                    if let #name::#variant_name(var) = tok {
                                        Ok(var)
                                    } else {
                                        Err(lasagna::ParseError::msg(
                                            span | lexer.span(0),
                                            format!("expected '{}'", <#field_ty as lasagna::Named>::NAME),
                                        ))
                                    }
                                } else {
                                    Err(lasagna::ParseError::eof(span, <#field_ty as lasagna::Named>::NAME))
                                }
                            }
                        }

                        impl lasagna::Parse for #field_ty {
                            type Source = #name;

                            #[inline]
                            fn parse(
                                parser: &mut impl Parser<Source = Self::Source>,
                            ) -> Result<Self, lasagna::ParseError> {
                                parser.next()
                            }
                        }
                    });
                }
            }

            quote! {
                impl std::fmt::Display for #name {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        match self {
                            #(#display_variants),*
                        }
                    }
                }

                impl lasagna::Token<char> for #name {
                    #[inline]
                    fn lex(lexer: &mut impl Lexer<Output = char>) -> Result<Self, lasagna::ParseError> {
                        #(#lex_variant)*

                        Err(lasagna::ParseError::eof(lexer.span(0), <#name as lasagna::Named>::NAME))
                    }
                }

                impl lasagna::Parse for #name {
                    type Source = char;

                    #[inline]
                    fn parse(
                        parser: &mut impl Parser<Source = Self::Source>
                    ) -> Result<Self, lasagna::ParseError> {
                        parser.next()
                    }
                }

                #(#token_variants)*
            }
        }
        _ => unimplemented!("can only derive Token for enums"),
    }
}
