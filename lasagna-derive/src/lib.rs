mod derive_named;
mod derive_parse;
mod derive_spanned;
mod derive_token;

#[proc_macro_derive(Parse, attributes(token, parse))]
pub fn derive_parse(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    derive_parse::derive_parse(input)
}

#[proc_macro_derive(Token, attributes(token))]
pub fn derive_token(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    derive_token::derive_token(input)
}

#[proc_macro_derive(Named, attributes(name))]
pub fn derive_named(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    derive_named::derive_named(input)
}

#[proc_macro_derive(Spanned)]
pub fn derive_spanned(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    derive_spanned::derive_spanned(input)
}
