use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod parsed_value;
mod parse_pass;

#[proc_macro_derive(FromParsedValue, attributes(nprs))]
pub fn derive_from_parsed_value(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    parsed_value::expand_from_parsed_value(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(ParsePass)]
pub fn derive_parse_pass(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    parse_pass::expand_parse_pass(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
