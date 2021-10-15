mod derive_parse;

#[proc_macro_derive(Parse, attributes(parse))]
pub fn derive_parse(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    derive_parse::derive_parse(input)
}
