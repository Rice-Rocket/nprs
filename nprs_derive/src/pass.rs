use proc_macro2::TokenStream;
use quote::quote;

pub(crate) fn expand_pass(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    let ident = input.ident;
    let ident_str = ident.to_string();

    Ok(quote! {
        inventory::submit! {
            <dyn ::nprs::pass::Pass>::register_pass(
                #ident_str,
                (|value| {
                    Ok(Box::new(<#ident as ::nprs::parser::FromParsedValue>::from_parsed_value(value)?))
                }) as ::nprs::pass::RegistrationValueParser,
            )
        }
    })
}
