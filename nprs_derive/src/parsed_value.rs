use darling::{ast, FromDeriveInput, FromField, FromVariant};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Expr, Ident, Type};

#[derive(FromDeriveInput)]
#[darling(attributes(nprs))]
struct Input {
    ident: Ident,
    generics: syn::Generics,
    data: darling::ast::Data<InputVariant, InputField>,
    #[darling(default)]
    from: Option<Ident>,
}

#[derive(FromField)]
#[darling(attributes(nprs))]
struct InputField {
    ident: Option<Ident>,
    ty: Type,
    #[darling(default)]
    default: Option<Expr>,
    #[darling(default)]
    rename: Option<Ident>,
    #[darling(default)]
    alias: Option<Ident>,
}

#[derive(FromVariant)]
#[darling(attributes(nprs))]
struct InputVariant {
    ident: Ident,
    fields: darling::ast::Fields<InputField>,
}

pub(crate) fn expand_from_parsed_value(derive_input: syn::DeriveInput) -> syn::Result<TokenStream> {
    let input = Input::from_derive_input(&derive_input)?;

    let ident = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();


    if let Some(from_value) = input.from {
        Ok(quote! {
            impl #impl_generics ::nprs::parser::FromParsedValue for #ident #ty_generics #where_clause {
                fn from_parsed_value(__value: ::nprs::parser::interpreter::ParsedValue) -> ::nprs::__private::Result<Self, ::nprs::parser::ParseValueError> {
                    ::nprs::__private::Result::Ok(Self::from(<#from_value as ::nprs::parser::FromParsedValue>::from_parsed_value(__value)?))
                }
            }
        })
    } else {
        expand_standard(input)
    }
}

fn expand_standard(input: Input) -> syn::Result<TokenStream> {
    let ident = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let body = match input.data {
        ast::Data::Struct(data) => expand_struct(quote! { #ident }, data)?,
        ast::Data::Enum(data) => expand_enum(ident, data)?,
    };

    Ok(quote! {
        impl #impl_generics ::nprs::parser::FromParsedValue for #ident #ty_generics #where_clause {
            fn from_parsed_value(__value: ::nprs::parser::interpreter::ParsedValue) -> ::nprs::__private::Result<Self, ::nprs::parser::ParseValueError> {
                #body
            }
        }
    })
}

fn expand_struct(ident: TokenStream, fields: ast::Fields<InputField>) -> syn::Result<TokenStream> {
    let body = expand_struct_fields(ident, fields)?;

    Ok(quote! {
        let __type_name = __value.type_name();
        let ::nprs::__private::Option::Some((__name, __fields)) = __value.struct_properties() else {
            return ::nprs::__private::Result::Err(::nprs::parser::ParseValueError::WrongType(::nprs::__private::String::from("struct, tuple struct, or unit struct"), __type_name));
        };

        #body
    })
}

fn expand_struct_fields(ident: TokenStream, fields: ast::Fields<InputField>) -> syn::Result<TokenStream> {
    match fields.style {
        ast::Style::Struct => Ok(expand_named_fields(ident, fields.fields)?),
        ast::Style::Tuple => Ok(expand_unnamed_fields(ident, fields.fields)?),
        ast::Style::Unit => Ok(quote! { ::nprs::__private::Result::Ok(#ident) }),
    }
}

fn expand_enum(ident: &Ident, variants: Vec<InputVariant>) -> syn::Result<TokenStream> {
    let variant_names: Vec<_> = variants.iter()
        .map(|variant| variant.ident.to_string()).collect();

    let variant_values = variants.into_iter()
        .map(|variant| {
            let variant_ident = variant.ident;
            expand_struct_fields(quote! { #ident::#variant_ident }, variant.fields)
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(quote! {
        let __type_name = __value.type_name();
        let ::nprs::__private::Option::Some((__name, __fields)) = __value.struct_properties() else {
            return ::nprs::__private::Result::Err(::nprs::parser::ParseValueError::WrongType(::nprs::__private::String::from("struct variant, tuple struct variant or unit struct variant"), __type_name));
        };

        match __name.as_str() {
            #(
                #variant_names => {
                    #variant_values
                },
            )*
            _ => ::nprs::__private::Result::Err(::nprs::parser::ParseValueError::UnknownVariant(__name)),
        }
    })
}

fn expand_named_fields(ident: TokenStream, named_fields: Vec<InputField>) -> syn::Result<TokenStream> {
    let fields: Vec<_> = named_fields.into_iter()
        .enumerate()
        .map(|(i, field)| expand_struct_field(i, field))
        .collect();

    let fields_clone = fields.clone();
    let filled_fields = fields_clone.iter()
        .map(|field| {
            let varname_raw = &field.var_name_raw.as_ref().unwrap();
            let varname = &field.var_name;

            quote! {
                #varname_raw: #varname,
            }
        });

    let collect_fields = expand_parsed_fields(fields)?;

    Ok(quote! {
        #collect_fields

        ::nprs::__private::Result::Ok(#ident {
            #(#filled_fields)*
        })
    })
}

fn expand_unnamed_fields(ident: TokenStream, unnamed_fields: Vec<InputField>) -> syn::Result<TokenStream> {
    let fields: Vec<_> = unnamed_fields.into_iter()
        .enumerate()
        .map(|(i, field)| expand_struct_field(i, field))
        .collect();

    let fields_clone = fields.clone();
    let filled_fields = fields_clone.iter()
        .map(|field| {
            let varname = &field.var_name;

            quote! {
                #varname
            }
        });

    let collect_fields = expand_parsed_fields(fields)?;

    Ok(quote! {
        #collect_fields

        ::nprs::__private::Result::Ok(#ident(#(#filled_fields),*))
    })
}

fn expand_parsed_fields(fields: Vec<StructFieldData>) -> syn::Result<TokenStream> {
    let var_defs = fields.iter()
        .map(|field| {
            let varname = &field.var_name;
            let ty = &field.ty;

            quote! {
                let mut #varname: ::nprs::__private::Option<#ty> = ::nprs::__private::Option::None;
            }
        });

    let name_matches = fields.iter()
        .map(|field| {
            let varname = &field.var_name;
            let ty = &field.ty;
            let string = &field.string;
            let patterns = if let Some(alias) = &field.alias {
                quote! { #string | #alias }
            } else {
                quote! { #string }
            };

            quote! {
                #patterns => {
                    if #varname.is_some() {
                        return ::nprs::__private::Result::Err(::nprs::parser::ParseValueError::DuplicateField(::nprs::__private::String::from(#string)));
                    };

                    #varname = ::nprs::__private::Option::Some(<#ty as ::nprs::parser::FromParsedValue>::from_parsed_value(*__value)?);
                },
            }
        });

    let get_some_vars = fields.iter()
        .map(|field| {
            let varname = &field.var_name;
            let string = &field.string;

            if let Some(default) = &field.default {
                quote! {
                    let #varname = #varname.unwrap_or_else(|| #default);
                }
            } else {
                quote! {
                    let ::nprs::__private::Option::Some(#varname) = #varname else {
                        return ::nprs::__private::Result::Err(::nprs::parser::ParseValueError::MissingField(::nprs::__private::String::from(#string)));
                    };
                }
            }
        });

    Ok(quote! {
        #(#var_defs)*

        for (__param, __value) in __fields.into_iter() {
            match __param.as_str() {
                #(#name_matches)*
                _ => return ::nprs::__private::Result::Err(::nprs::parser::ParseValueError::UnknownField(::nprs::__private::String::from(__param))),
            }
        };
        
        #(#get_some_vars)*
    })
}

#[derive(Clone)]
struct StructFieldData {
    var_name_raw: Option<Ident>,
    var_name: Ident,
    string: TokenStream,
    ty: TokenStream,
    default: Option<TokenStream>,
    alias: Option<TokenStream>,
}

fn expand_struct_field(index: usize, field: InputField) -> StructFieldData {
    let ty = field.ty;

    let default = field.default.map(|default| quote! { #default });
    
    match &field.ident {
        Some(var_name_raw) => {
            let var_name = format_ident!("__{}", var_name_raw);
            let string = if let Some(rename) = field.rename {
                rename.to_string()
            } else {
                var_name_raw.to_string()
            };

            let alias = field.alias.map(|alias| {
                let alias_str = alias.to_string();
                quote! { #alias_str }
            });

            StructFieldData {
                var_name_raw: Some(var_name_raw.clone()),
                var_name,
                string: quote! { #string },
                ty: quote! { #ty },
                default,
                alias,
            }
        },
        None => {
            let var_name = format_ident!("__{}", index);
            let string = index.to_string();

            StructFieldData {
                var_name_raw: None,
                var_name,
                string: quote! { #string },
                ty: quote! { #ty },
                default,
                alias: None,
            }
        },
    }
}
