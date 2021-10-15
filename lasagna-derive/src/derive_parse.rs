use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    bracketed, parenthesized,
    parse::{Parse, ParseStream, Parser},
    parse_macro_input, parse_quote,
    punctuated::Punctuated,
    token::Bracket,
    Attribute, Data, DeriveInput, Error, Fields, Generics, Path, Token, TraitBound, Type,
    TypeParamBound,
};

struct Input {
    bracket: Bracket,
    types: Punctuated<Type, Token![,]>,
}

impl Parse for Input {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;

        Ok(Self {
            bracket: bracketed!(content in input),
            types: Punctuated::parse_terminated(&content)?,
        })
    }
}

#[derive(Default)]
struct Attributes {
    types: Vec<Type>,
}

impl Attributes {
    fn parse_from_attrs(attrs: &[Attribute]) -> Self {
        let mut attributes = Attributes::default();

        for attr in attrs {
            if let Some(ident) = attr.path.get_ident() {
                if ident == "parse" {
                    let parser = |input: ParseStream| Input::parse(input);

                    let input = parser.parse2(attr.tokens.clone()).unwrap();

                    for ty in input.types.into_iter() {
                        attributes.types.push(ty);
                    }
                }
            }
        }

        attributes
    }
}

pub fn derive_parse(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;

    let attrs = Attributes::parse_from_attrs(&input.attrs);

    add_generics(&mut input.generics, &attrs.types);

    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

    let data = &input.data;
    let impls = attrs.types.into_iter().map(|ty| {
        let (error, parse) = parse(data, &ty);

        quote! {
            impl #impl_generics lasagna::Parse<#ty> for #name #type_generics #where_clause {
                type Error = <#error as lasagna::Parse<#ty>>::Error;

                fn parse(source: #ty) -> Result<Self, Self::Error> {
                    #parse
                }
            }
        }
    });

    let expanded = quote! {
        #(#impls)*
    };

    println!("{}", expanded);

    proc_macro::TokenStream::from(expanded)
}

fn add_generics(generics: &mut Generics, types: &[Type]) {
    for param in generics.type_params_mut() {
        for ty in types {
            param
                .bounds
                .push(TypeParamBound::Trait(parse_quote!(lasagna::Parse<#ty>)));
        }
    }
}

fn parse(data: &Data, ty: &Type) -> (Type, TokenStream) {
    let mut error: Option<Type> = None;

    let parse = match data {
        Data::Enum(data) => {
            let mut variants = Vec::new();

            for variant in &data.variants {
                match variant.fields {
                    Fields::Named(ref named) => {
                        let parse = named.named.iter().map(|field| {
                            let name = field.ident.as_ref().unwrap();
                            let field_ty = &field.ty;

                            error = Some(field_ty.clone());

                            quote! {
                                #name: <#field_ty as lasagna::Parse<#ty>>::parse(source)?
                            }
                        });

                        let name = &variant.ident;

                        variants.push(quote! {
                            || Result::<Self, Self::Error>::Ok(Self::#name {
                                #(#parse),*
                            })
                        });
                    }
                    Fields::Unnamed(ref unnamed) => {
                        let parse = unnamed.unnamed.iter().map(|field| {
                            let field_ty = &field.ty;

                            error = Some(field_ty.clone());

                            quote! {
                                <#field_ty as lasagna::Parse<#ty>>::parse(source)?
                            }
                        });

                        let name = &variant.ident;

                        variants.push(quote! {
                            || Result::<Self, Self::Error>::Ok(Self::#name(#(#parse),*))
                        }); 
                    }
                    _ => todo!(),
                }
            }

            quote! {
                #(if let Ok(var) = (#variants)() {
                    return Ok(var);
                }),*

                panic!()
            }
        }
        _ => unimplemented!(),
    };

    (error.unwrap(), parse)
}
