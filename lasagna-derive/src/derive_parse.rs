use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, parse_quote,
    spanned::Spanned,
    Attribute, Data, DeriveInput, Fields, FieldsNamed, FieldsUnnamed, Path, Token, Type,
};

const NO_FIELDS: &str = "type must have a least one Spanned field";

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

    let mut token = None;
    let (parse, start) = parse(input.data, attrs.clone(), &mut token);

    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics ::lasagna::Parse for #name #type_generics #where_clause {
            type Token = #token;

            const START: ::lasagna::ParseStart<Self::Token> = #start;

            fn parse(
                parser: &mut impl ::lasagna::Parser<Self::Token>
            ) -> Result<Self, ::lasagna::Error> {
                #parse
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}

fn parse(data: Data, attrs: Attributes, token: &mut Option<Type>) -> (TokenStream, TokenStream) {
    match data {
        Data::Enum(data) => {
            let parse = data.variants.iter().map(|variant| {
                let is_next = variant
                    .fields
                    .iter()
                    .next()
                    .map(|field| {
                        let ty = &field.ty;

                        quote!(<#ty as ::lasagna::Parse>::is_next(parser))
                    })
                    .expect(NO_FIELDS);

                let variant_name = &variant.ident;

                let parse_variant = match variant.fields {
                    Fields::Named(ref named) => {
                        let parse_fields = parse_fields_named(named, token);

                        quote! {
                            ::std::result::Result::Ok(
                                Self::#variant_name {
                                    #(#parse_fields),*
                                }
                            )
                        }
                    }
                    Fields::Unnamed(ref unnamed) => {
                        let parse_fields = parse_fields_unnamed(unnamed, token);

                        quote! {
                            ::std::result::Result::Ok(
                                Self::#variant_name(#(#parse_fields),*)
                            )
                        }
                    }
                    Fields::Unit => unimplemented!("{}", NO_FIELDS),
                };

                quote! {
                    match #is_next {
                        ::std::option::Option::Some(true) => return #parse_variant,
                        ::std::option::Option::Some(false) => {},
                        _ => {},
                    }
                }
            });

            let start = data.variants.iter().map(|variant| {
                variant
                    .fields
                    .iter()
                    .next()
                    .map(|field| {
                        let ty = &field.ty;

                        quote!(<#ty as ::lasagna::Parse>::START)
                    })
                    .expect(NO_FIELDS)
            });

            let parse = quote! {
                #(#parse)*

                let span = parser.span(0);

                if let Some(tok) = parser.peek()? {
                    ::std::result::Result::Err(
                        ::lasagna::Error::expected_one(span, &Self::START.to_vec(), tok)
                    )
                } else {
                    ::std::result::Result::Err(
                        ::lasagna::Error::expected_one(span, &Self::START.to_vec(), "eof")
                    )
                }
            };

            let start = quote!(&::lasagna::StartTokens::Any(&[#(#start),*]));

            (parse, start)
        }
        Data::Struct(data) => match data.fields {
            Fields::Named(named) => {
                let parse_fields = parse_fields_named(&named, token);

                let parse = quote! {
                    Ok(Self {
                        #(#parse_fields),*
                    })
                };

                let is_next = named
                    .named
                    .first()
                    .map(|field| {
                        let ty = &field.ty;

                        quote!(<#ty as ::lasagna::Parse>::START)
                    })
                    .expect(NO_FIELDS);

                (parse, is_next)
            }
            Fields::Unnamed(unnamed) => {
                let parse_fields = parse_fields_unnamed(&unnamed, token);

                let parse = quote! {
                    Ok(Self(#(#parse_fields),*))
                };

                let is_next = unnamed
                    .unnamed
                    .first()
                    .map(|field| {
                        let ty = &field.ty;

                        quote!(<#ty as ::lasagna::Parse>::START)
                    })
                    .expect(NO_FIELDS);

                (parse, is_next)
            }
            Fields::Unit => unimplemented!("{}", NO_FIELDS),
        },
        _ => unimplemented!(),
    }
}

fn parse_fields_named<'a>(
    fields: &'a FieldsNamed,
    token: &'a mut Option<Type>,
) -> impl Iterator<Item = TokenStream> + 'a {
    fields.named.iter().map(move |field| {
        let name = field.ident.as_ref().unwrap();
        let ty = &field.ty;

        if token.is_none() {
            *token = Some(parse_quote!(<#ty as ::lasagna::Parse>::Token));
        }

        quote_spanned! {name.span()=>
            #name: <#ty as ::lasagna::Parse>::parse(parser)?
        }
    })
}

fn parse_fields_unnamed<'a>(
    fields: &'a FieldsUnnamed,
    token: &'a mut Option<Type>,
) -> impl Iterator<Item = TokenStream> + 'a {
    fields.unnamed.iter().map(move |field| {
        let ty = &field.ty;

        if token.is_none() {
            *token = Some(parse_quote!(<#ty as ::lasagna::Parse>::Token));
        }

        quote_spanned! {field.span()=>
            <#ty as ::lasagna::Parse>::parse(parser)?
        }
    })
}
