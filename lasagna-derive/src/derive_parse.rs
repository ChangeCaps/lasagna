use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    parse_macro_input, parse_quote,
    punctuated::Punctuated,
    token::Impl,
    Attribute, Data, DeriveInput, Error, Expr, Fields, Generics, Path, Token, Type, TypeParamBound,
};

struct Param {
    generics: Option<Generics>,
    token: Path,
}

impl Parse for Param {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let generics = if Impl::parse(input).is_ok() {
            let generics = Generics::parse(input)?;

            Some(generics)
        } else {
            None
        };

        Ok(Self {
            generics,
            token: Path::parse(input)?,
        })
    }
}

syn::custom_keyword!(specific);
syn::custom_keyword!(expect);

enum Arg {
    Specific(Param),
    Expect(Punctuated<Expr, Token![,]>),
}

impl Parse for Arg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if specific::parse(input).is_ok() {
            let content;

            parenthesized!(content in input);

            Ok(Arg::Specific(Param::parse(&content)?))
        } else if expect::parse(input).is_ok() {
            let content;

            parenthesized!(content in input);

            Ok(Arg::Expect(Punctuated::parse_terminated(&content)?))
        } else {
            Err(Error::new(Span::call_site(), "invalid argument"))
        }
    }
}

#[derive(Default)]
struct Attributes {
    expect: Option<Vec<Expr>>,
    types: Vec<Param>,
}

impl Attributes {
    fn parse_from_attrs(attrs: &[Attribute]) -> Self {
        let mut attributes = Attributes::default();

        for attr in attrs {
            if let Some(ident) = attr.path.get_ident() {
                if ident == "parse" {
                    let args = attr
                        .parse_args_with(|input: ParseStream| {
                            Punctuated::<Arg, Token!(,)>::parse_terminated(input)
                        })
                        .unwrap();

                    for arg in args {
                        match arg {
                            Arg::Expect(expr) => {
                                attributes
                                    .expect
                                    .get_or_insert_with(Default::default)
                                    .append(&mut expr.into_iter().collect());
                            }
                            Arg::Specific(param) => {
                                attributes.types.push(param);
                            }
                        }
                    }
                }
            }
        }

        attributes
    }
}

pub fn derive_parse(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;

    let attrs = Attributes::parse_from_attrs(&input.attrs);

    let data = &input.data;
    let generics = input.generics;

    let expanded = if attrs.types.is_empty() {
        let param = Param {
            generics: Some(parse_quote!(<__Token>)),
            token: parse_quote!(__Token),
        };

        let parse = parse_token(
            &name,
            data,
            &generics,
            &param,
            attrs.expect.as_ref().map(|expect| expect.first().unwrap()),
            true,
        );

        quote! {
            #parse
        }
    } else {
        let impls = attrs.types.iter().enumerate().map(|(i, param)| {
            parse_token(
                &name,
                data,
                &generics,
                param,
                attrs.expect.as_ref().map(|expect| &expect[i]),
                false,
            )
        });

        quote! {
            #(#impls)*
        }
    };

    proc_macro::TokenStream::from(expanded)
}

fn parse_token(
    name: &Ident,
    data: &Data,
    generics: &Generics,
    param: &Param,
    expect: Option<&Expr>,
    field_bounds: bool,
) -> TokenStream {
    let mut generics = generics.clone();

    add_generics(&mut generics, param);

    let mut impl_generics = impl_generics(generics.clone(), &param);
    let (_, type_generics, _) = generics.split_for_impl();

    let token = &param.token;

    if let Some(expect) = expect {
        let (impl_generics, _, where_clause) = impl_generics.split_for_impl();

        quote! {
            impl #impl_generics lasagna::Parse<#token> for
                #name #type_generics #where_clause
            {
                fn parse<__Error, __Parser>(source: &mut __Parser) -> Result<Self, __Error>
                where
                    __Parser: lasagna::Parser<#token, __Error> + ?Sized,
                    __Error: lasagna::ParseError<#token>,
                {
                    source.expect(#expect)?;

                    Ok(Self)
                }
            }
        }
    } else {
        let (parse, types) = parse(data, token);

        if field_bounds {
            self::field_bounds(&mut impl_generics, token, types);
        }

        let (impl_generics, _, where_clause) = impl_generics.split_for_impl();

        quote! {
            impl #impl_generics lasagna::Parse<#token> for
                #name #type_generics #where_clause
            {
                fn parse<__Error, __Parser>(source: &mut __Parser) -> Result<Self, __Error>
                where
                    __Parser: lasagna::Parser<#token, __Error> + ?Sized,
                    __Error: lasagna::ParseError<#token>,
                {
                    #parse
                }
            }
        }
    }
}

fn add_generics(generics: &mut Generics, param: &Param) {
    let token = &param.token;

    for param in generics.type_params_mut() {
        param
            .bounds
            .push(TypeParamBound::Trait(parse_quote!(lasagna::Parse<#token>)));
    }
}

fn impl_generics(mut ty_generics: Generics, param: &Param) -> Generics {
    if let Some(ref generics) = param.generics {
        for param in &generics.params {
            ty_generics.params.push(param.clone());
        }
    }

    ty_generics
}

fn field_bounds(generics: &mut Generics, token: &Path, types: Vec<Type>) {
    let where_clause = generics.where_clause.get_or_insert(parse_quote!(where));

    for ty in types {
        where_clause
            .predicates
            .push(parse_quote!(#ty: lasagna::Parse<#token>));
    }
}

fn parse(data: &Data, token: &Path) -> (TokenStream, Vec<Type>) {
    let mut types = Vec::new();

    match data {
        Data::Enum(data) => {
            let mut variants = Vec::new();

            for variant in &data.variants {
                match variant.fields {
                    Fields::Named(ref named) => {
                        let parse = named.named.iter().map(|field| {
                            let name = field.ident.as_ref().unwrap();
                            let field_ty = &field.ty;

                            types.push(field_ty.clone());

                            quote! {
                                #name: <#field_ty as lasagna::Parse<#token>>::parse(parser)?
                            }
                        });

                        let name = &variant.ident;

                        variants.push(quote! {
                            |parser| {
                                Result::<Self, __Error>::Ok(Self::#name {
                                    #(#parse),*
                                })
                            }
                        });
                    }
                    Fields::Unnamed(ref unnamed) => {
                        let parse = unnamed.unnamed.iter().map(|field| {
                            let field_ty = &field.ty;

                            types.push(field_ty.clone());

                            quote! {
                                lasagna::Parse::parse(parser)?
                            }
                        });

                        let name = &variant.ident;

                        variants.push(quote! {
                            |parser| {
                                Result::<Self, __Error>::Ok(Self::#name(#(#parse),*))
                            }
                        });
                    }
                    _ => {}
                }
            }

            let parse = quote! {
                use lasagna::Parse;

                #(
                    let branch = source.try_parse_with(#variants);

                    if branch.is_ok() {
                        return branch;
                    }
                )*

                let tok = source.next();

                Err(__Error::expected(tok, &[]))
            };

            (parse, types)
        }
        Data::Struct(data) => match data.fields {
            Fields::Named(ref named) => {
                let fields = named.named.iter().map(|field| {
                    let ident = field.ident.as_ref().unwrap();
                    let field_ty = &field.ty;

                    types.push(field_ty.clone());

                    quote_spanned! {ident.span()=>
                        #ident: lasagna::Parse::parse(source)?
                    }
                });

                let parse = quote! {
                    Ok(Self {
                        #(#fields),*
                    })
                };

                (parse, types)
            }
            Fields::Unit => {
                let parse = quote! {
                    Ok(Self)
                };

                (parse, types)
            }
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    }
}
