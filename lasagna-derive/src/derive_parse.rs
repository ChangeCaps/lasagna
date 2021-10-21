use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, parse_quote,
    spanned::Spanned,
    Attribute, Data, DeriveInput, Fields, FieldsNamed, FieldsUnnamed, Path, Token, Type,
};

syn::custom_keyword!(source);

struct Source(Type);

impl Parse for Source {
    #[inline]
    fn parse(input: ParseStream) -> syn::Result<Self> {
        source::parse(input)?;

        <Token![=]>::parse(input)?;

        Ok(Self(Type::parse(input)?))
    }
}

#[derive(Clone, Default)]
struct Attributes {
    token: Option<Type>,
    source: Option<Type>,
}

impl Attributes {
    fn from_attrs(&mut self, attrs: &[Attribute]) {
        for attr in attrs {
            if attr
                .path
                .get_ident()
                .map(|ident| ident == "token")
                .unwrap_or(false)
            {
                if let Ok(ty) = attr.parse_args::<Type>() {
                    self.token = Some(ty);
                }
            }

            if attr
                .path
                .get_ident()
                .map(|ident| ident == "parse")
                .unwrap_or(false)
            {
                if let Ok(Source(ty)) = attr.parse_args::<Source>() {
                    self.source = Some(ty);
                }
            }
        }
    }
}

syn::custom_keyword!(peek);

struct Peek(Path);

impl Parse for Peek {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        peek::parse(input)?;

        <Token![=]>::parse(input)?;

        Ok(Self(Path::parse(input)?))
    }
}

#[derive(Default)]
struct VariantAttributes {
    peek: Option<Path>,
}

impl VariantAttributes {
    fn from_attrs(&mut self, attrs: &[Attribute]) {
        for attr in attrs {
            if attr
                .path
                .get_ident()
                .map(|ident| ident == "parse")
                .unwrap_or(false)
            {
                if let Ok(path) = attr.parse_args::<Peek>() {
                    self.peek = Some(path.0);
                }
            }
        }
    }
}

pub fn derive_parse(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let mut attrs = Attributes::default();
    attrs.from_attrs(&input.attrs);

    let name = input.ident;

    let mut source = None;
    let parse = parse(input.data, attrs.clone(), &mut source);

    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

    if attrs.source.is_some() {
        source = attrs.source;
    }

    let source = source.unwrap();

    let expanded = quote! {
        impl #impl_generics lasagna::Parse for #name #type_generics #where_clause {
            type Source = #source;

            fn parse(
                parser: &mut impl lasagna::Parser<Source = Self::Source>
            ) -> Result<lasagna::Spanned<Self>, lasagna::ParseError> {
                #parse
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}

fn parse(data: Data, attrs: Attributes, source: &mut Option<Type>) -> TokenStream {
    if let Some(ty) = attrs.token {
        *source = Some(ty);

        return quote! {
            parser.next::<Self>()
        };
    }

    match data {
        Data::Enum(data) => {
            let mut names = Vec::new();

            let variants = data.variants.iter().map(|variant| {
                let name = &variant.ident;

                names.push(name.to_string());

                let mut var_attrs = VariantAttributes::default();
                var_attrs.from_attrs(&variant.attrs);

                let var = match &variant.fields {
                    Fields::Named(named) => {
                        let fields = fields_named(&named, source);

                        quote_spanned! {variant.span()=>
                            Spanned::new(
                                Self::#name { #(#fields),* },
                                span,
                            )
                        }
                    }
                    Fields::Unnamed(unnamed) => {
                        let fields = fields_unnamed(&unnamed, source);

                        quote_spanned! {variant.span()=>
                            Spanned::new(
                                Self::#name(#(#fields),*),
                                span,
                            )
                        }
                    }
                    _ => unimplemented!(),
                };

                if let Some(peek) = &var_attrs.peek {
                    quote! {
                        let mut fork = parser.fork();

                        if fork.next::<#peek>().is_ok() {
                            #[allow(unused_mut)]
                            let mut span;

                            return Ok(#var);
                        }
                    }
                } else {
                    try_variant(var)
                }
            });

            quote! {
                #(#variants)*

                Err(lasagna::ParseError::ExpectedOne {
                    span: parser.span(0),
                    expected: vec![#(String::from(#names)),*],
                })
            }
        }
        Data::Struct(data) => match data.fields {
            Fields::Named(named) => {
                let fields = fields_named(&named, source);

                quote! {
                    let mut span;

                    Ok(Spanned::new(
                        Self {
                            #(#fields),*
                        },
                        span,
                    ))
                }
            }
            Fields::Unnamed(unnamed) => {
                let fields = fields_unnamed(&unnamed, source);

                quote! {
                    let mut span;

                    Ok(Spanned::new(
                        Self {
                            #(#fields),*
                        },
                        span,
                    ))
                }
            }
            Fields::Unit => {
                quote! {
                    parser.next::<Self>()
                }
            }
        },
        _ => unimplemented!(),
    }
}

fn try_variant(variant: TokenStream) -> TokenStream {
    quote! {
        let variant = |parser| {
            #[allow(unused_mut)]
            let mut span;

            Result::<_, lasagna::ParseError>::Ok(#variant)
        };

        let mut fork = parser.fork();

        let variant = variant(&mut fork);

        if variant.is_ok() {
            *parser = fork;

            return variant;
        }
    }
}

fn fields_named<'a>(
    fields: &'a FieldsNamed,
    source: &'a mut Option<Type>,
) -> impl Iterator<Item = TokenStream> + 'a {
    let mut span_set = false;

    fields.named.iter().map(move |field| {
        let name = field.ident.as_ref().unwrap();
        let ty = &field.ty;

        if source.is_none() {
            *source = Some(parse_quote!(<#ty as lasagna::Parse>::Source));
        }

        let span = if span_set {
            quote! {
                span |= field.span;
            }
        } else {
            span_set = true;

            quote! {
                span = field.span;
            }
        };

        quote_spanned! {name.span()=>
            #name: {
                let field = parser.parse::<#ty>()?;

                #span

                field.value
            }
        }
    })
}

fn fields_unnamed<'a>(
    fields: &'a FieldsUnnamed,
    source: &'a mut Option<Type>,
) -> impl Iterator<Item = TokenStream> + 'a {
    let mut span_set = false;

    fields.unnamed.iter().map(move |field| {
        let ty = &field.ty;

        if source.is_none() {
            *source = Some(parse_quote!(<#ty as lasagna::Parse>::Source));
        }

        let span = if span_set {
            quote! {
                span |= field.span;
            }
        } else {
            span_set = true;

            quote! {
                span = field.span;
            }
        };

        quote_spanned! {field.span()=>
            {
                let field = parser.parse::<#ty>()?;

                #span

                field.value
            }
        }
    })
}
