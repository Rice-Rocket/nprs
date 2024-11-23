use proc_macro2::TokenStream;
use quote::quote;

pub(crate) fn expand_parse_pass(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    let vis = input.vis;
    let ident = input.ident;
    let ident_str = ident.to_string();

    Ok(quote! {
        impl #ident {
            #vis const PASS_NAME: &'static str = #ident_str;
        }

        inventory::submit! {
            <dyn ::nprs::pass::Pass>::register_pass(
                #ident_str,
                (|value| {
                    ::nprs::__private::Result::Ok(::nprs::__private::Box::new(<#ident as ::nprs::parser::FromParsedValue>::from_parsed_value(value)?))
                }) as ::nprs::pass::RegistrationValueParser,
            )
        }
    })
}
