use proc_macro2::{Ident, TokenStream};
use quote::{quote, quote_spanned};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    spanned::Spanned,
    Attribute, Data, DataStruct, DeriveInput, Fields, LitStr, Token,
};

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
                fn lex(lexer: &mut impl lasagna::Lexer<Output = char>) -> Result<Self, lasagna::Error> {
                    #match_string

                    Ok(Self)
                }
            }

            impl lasagna::Parse for #name {
                type Source = char;

                #[inline]
                fn parse(
                    parser: &mut impl lasagna::Parser<Source = Self::Source>
                ) -> Result<Self, lasagna::Error> {
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
                    return Err(lasagna::Error::expected(
                        lasagna::Lexer::span(lexer, 0),
                        next_char,
                        c,
                    ));
                }
            } else {
                return Err(lasagna::Error::expected(
                    lasagna::Lexer::span(lexer, 0),
                    "<eof>",
                    c,
                ));
            }
        }
    }
}

fn source_token(input: DeriveInput) -> TokenStream {
    let name = input.ident;
    let kind_name = Ident::new(&format!("{}Kind", name), name.span());

    match input.data {
        Data::Enum(data) => {
            let mut variant_matches = Vec::new();
            let mut lex_variant = Vec::new();
            let mut token_variants = Vec::new();
            let mut display_variants = Vec::new();
            let mut variant_names = Vec::new();
            let mut kind_names = Vec::new();

            for variant in data.variants {
                let mut attrs = Attributes::default();
                attrs.from_attrs(&variant.attrs);

                let variant_ident = variant.ident;
                let variant_name = variant_ident.to_string();
                variant_names.push(variant_name.clone());

                kind_names.push(variant_ident.clone());

                if let Some(string) = attrs.match_string {
                    variant_matches.push(quote!(Self::#variant_ident));

                    display_variants.push(quote_spanned! {variant_ident.span()=>
                        Self::#variant_ident => write!(f, "{}", #string)
                    });

                    let match_string = match_string(&string);

                    lex_variant.push(quote_spanned! {variant_ident.span()=>
                        let mut fork = lexer.fork();

                        {
                            fn variant(
                                lexer: &mut impl lasagna::Lexer<Output = char>
                            ) -> Result<(), lasagna::Error> {
                                #match_string

                                Ok(())
                            }

                            if variant(&mut fork).is_ok() {
                                *lexer = fork;

                                return Ok(Self::#variant_ident);
                            }
                        }
                    });

                    token_variants.push(quote_spanned! {variant_ident.span()=>
                        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
                        pub struct #variant_ident(lasagna::Span);

                        impl lasagna::Spanned for #variant_ident {
                            #[inline]
                            fn span(&self) -> lasagna::Span {
                                self.0
                            }
                        }

                        impl lasagna::Parse for #variant_ident {
                            type Token = #name;

                            const START: ::lasagna::ParseStart<Self::Token> =
                                &::lasagna::StartTokens::Token(&#kind_name::#variant_ident);

                            #[inline]
                            fn parse(
                                parser: &mut impl Parser<Self::Token>,
                            ) -> Result<Self, lasagna::Error> {
                                let span = parser.span(0);
                                let token = parser.next()?;
                                let span = span | parser.span(0);

                                match token {
                                    #name::#variant_ident => Ok(Self(span)),
                                    _ => ::std::result::Result::Err(
                                        ::lasagna::Error::expected(span, #variant_name, token),
                                    ),
                                }
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

                    variant_matches.push(quote!(Self::#variant_ident(_)));

                    let field_ty = &field.ty;

                    display_variants.push(quote_spanned! {field_ty.span()=>
                        Self::#variant_ident(_) => write!(f, "{}", #variant_name)
                    });

                    lex_variant.push(quote_spanned! {variant_ident.span()=>
                        let mut fork = lexer.fork();

                        if let Ok(tok) = <#field_ty as lasagna::Lex<char>>::lex(&mut fork) {
                            *lexer = fork;

                            return Ok(Self::#variant_ident(tok));
                        }
                    });

                    token_variants.push(quote_spanned! {variant_ident.span()=>
                        impl lasagna::Parse for #field_ty {
                            type Token = #name;

                            const START: ::lasagna::ParseStart<Self::Token> =
                                &::lasagna::StartTokens::Token(&#kind_name::#variant_ident);

                            #[inline]
                            fn parse(
                                parser: &mut impl Parser<Self::Token>,
                            ) -> Result<Self, lasagna::Error> {
                                let span = parser.span(0);
                                let token = parser.next()?;
                                let span = span | parser.span(0);

                                match token {
                                    #name::#field_ty(var) => Ok(var),
                                    _ => ::std::result::Result::Err(
                                        ::lasagna::Error::expected(span, #variant_name, token)
                                    ),
                                }
                            }
                        }
                    });
                }
            }

            quote! {
                #[derive(Clone, Copy, Debug, PartialEq, Eq)]
                pub enum #kind_name {
                    #(#kind_names,)*
                }

                impl ::lasagna::TokenKind for #kind_name {
                    fn name(&self) -> &str {
                        match self {
                            #(Self::#kind_names => #variant_names,)*
                        }
                    }
                }

                impl std::fmt::Display for #name {
                    #[inline]
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        match self {
                            #(#display_variants),*
                        }
                    }
                }

                impl lasagna::Token<char> for #name {
                    type Kind = #kind_name;

                    #[inline]
                    fn kind(&self) -> Self::Kind {
                        match self {
                            #(#variant_matches => #kind_name::#kind_names,)*
                        }
                    }
                }

                impl ::lasagna::Lex<char> for #name {
                    #[inline]
                    fn lex(lexer: &mut impl Lexer<Output = char>) -> Result<Self, lasagna::Error> {
                        #(#lex_variant)*

                        Err(lasagna::Error::expected(lexer.span(0), "token", "eof"))
                    }
                }

                impl lasagna::Parse for #name {
                    type Token = Self;

                    const START: ::lasagna::ParseStart<Self::Token> = &::lasagna::StartTokens::All;

                    #[inline]
                    fn parse(
                        parser: &mut impl Parser<Self::Token>
                    ) -> Result<Self, lasagna::Error> {
                        parser.next()
                    }
                }

                #(#token_variants)*
            }
        }
        _ => unimplemented!("can only derive Token for enums"),
    }
}
