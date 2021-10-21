use proc_macro2::{Ident, TokenStream};
use quote::{quote, quote_spanned};
use syn::{
    parse_macro_input, spanned::Spanned, Data, DeriveInput, Fields, FieldsNamed, FieldsUnnamed,
};

pub fn derive_spanned(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;

    let span = span(&input.data);

    let expanded = quote! {
        impl lasagna::Spanned for #name {
            #[inline]
            fn span(&self) -> lasagna::Span {
                #span
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}

fn span(data: &Data) -> TokenStream {
    match data {
        Data::Enum(data) => {
            let mut variants = Vec::new();

            for variant in &data.variants {
                let variant_name = &variant.ident;

                match &variant.fields {
                    Fields::Unnamed(unnamed) => {
                        let mut i = 0;
                        let names = unnamed.unnamed.iter().map(move |field| {
                            let ident = Ident::new(&format!("_{}", i), field.span());
                            i += 1;

                            quote!(#ident)
                        });

                        let fields = names.clone();

                        variants.push(quote_spanned! {variant_name.span()=>
                            Self::#variant_name(#(#names),*) => #(#fields.span())|*
                        });
                    }
                    Fields::Named(named) => {
                        let names = named.named.iter().map(|field| {
                            let ident = field.ident.as_ref().unwrap();

                            quote!(#ident)
                        });

                        let fields = names.clone();

                        variants.push(quote_spanned! {variant_name.span()=>
                            Self::#variant_name { #(#names),* } => #(#fields.span())|*
                        });
                    }
                    _ => unimplemented!(),
                }
            }

            quote! {
                match self {
                    #(#variants),*
                }
            }
        }
        Data::Struct(data) => match &data.fields {
            Fields::Named(named) => {
                let fields = fields_named(named);

                quote! {
                    #(#fields)|*
                }
            }
            Fields::Unnamed(unnamed) => {
                let fields = fields_unnamed(unnamed);

                quote! {
                    #(#fields)|*
                }
            }
            _ => unimplemented!("spanned structs must have fields"),
        },
        _ => unimplemented!(),
    }
}

fn fields_unnamed(fields: &FieldsUnnamed) -> impl Iterator<Item = TokenStream> + '_ {
    let mut i = 0;

    fields.unnamed.iter().map(move |field| {
        let ident = Ident::new(&format!("{}", i), field.span());
        i += 1;

        quote_spanned! {ident.span()=>
            self.#ident.span()
        }
    })
}

fn fields_named(fields: &FieldsNamed) -> impl Iterator<Item = TokenStream> + '_ {
    fields.named.iter().map(|field| {
        let ident = field.ident.as_ref().unwrap();

        quote_spanned! {ident.span()=>
            self.#ident.span()
        }
    })
}
