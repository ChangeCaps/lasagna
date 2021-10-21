use quote::quote;
use syn::{
    parse::{Parse, ParseStream, Parser},
    parse_macro_input, DeriveInput, LitStr, Token,
};

struct Name(LitStr);

impl Parse for Name {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        <Token![=] as Parse>::parse(input)?;

        Ok(Self(Parse::parse(input)?))
    }
}

pub fn derive_named(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let mut name = None;

    for attr in &input.attrs {
        if attr
            .path
            .get_ident()
            .map(|ident| ident == "name")
            .unwrap_or(false)
        {
            name = Some(
                Name::parse
                    .parse2(attr.tokens.clone())
                    .expect("signature 'name = \"insert name\"'")
                    .0,
            );
        }
    }

    let ident = input.ident;

    let name = name.map(|lit| lit.value()).unwrap_or(ident.to_string());

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics lasagna::Named for #ident #ty_generics #where_clause {
            const NAME: &'static str = #name;
        }
    };

    proc_macro::TokenStream::from(expanded)
}
