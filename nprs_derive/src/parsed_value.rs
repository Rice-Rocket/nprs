use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{DataEnum, Field, Fields, FieldsNamed, FieldsUnnamed, Ident};

pub fn expand_from_parsed_value(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    let ident = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let body = match input.data {
        syn::Data::Struct(data) => expand_struct(quote! { #ident }, data.fields)?,
        syn::Data::Enum(data) => expand_enum(ident, data)?,
        syn::Data::Union(_) => return Err(syn::Error::new(ident.span(), "cannot derive `FromParseValue` for union")),
    };

    Ok(quote! {
        impl #impl_generics ::nprs::parser::FromParsedValue for #ident #ty_generics #where_clause {
            fn from_parsed_value(__value: ::nprs::parser::interpreter::ParsedValue) -> Result<Self, ::nprs::parser::ParseValueError> {
                #body
            }
        }
    })
}

fn expand_struct(ident: TokenStream, fields: Fields) -> syn::Result<TokenStream> {
    let body = expand_struct_fields(ident, fields)?;

    Ok(quote! {
        let __type_name = __value.type_name();
        let Some((__name, __fields)) = __value.struct_properties() else {
            return Err(::nprs::parser::ParseValueError::WrongType(String::from("struct, tuple struct, or unit struct"), __type_name));
        };

        #body
    })
}

fn expand_struct_fields(ident: TokenStream, fields: Fields) -> syn::Result<TokenStream> {
    match fields {
        syn::Fields::Named(named_fields) => Ok(expand_named_fields(ident, named_fields)?),
        syn::Fields::Unnamed(unnamed_fields) => Ok(expand_unnamed_fields(ident, unnamed_fields)?),
        syn::Fields::Unit => Ok(quote! { Ok(#ident) }),
    }
}

fn expand_enum(ident: &Ident, data: DataEnum) -> syn::Result<TokenStream> {
    let variant_names: Vec<_> = data.variants.iter()
        .map(|variant| variant.ident.to_string()).collect();

    let variant_values = data.variants.into_iter()
        .map(|variant| {
            let variant_ident = variant.ident;
            expand_struct_fields(quote! { #ident::#variant_ident }, variant.fields)
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(quote! {
        let __type_name = __value.type_name();
        let Some((__name, __fields)) = __value.struct_properties() else {
            return Err(::nprs::parser::ParseValueError::WrongType(String::from("struct variant, tuple struct variant or unit struct variant"), __type_name));
        };

        match __name.as_str() {
            #(
                #variant_names => {
                    #variant_values
                },
            )*
            _ => Err(::nprs::parser::ParseValueError::UnknownVariant(__name)),
        }
    })
}

fn expand_named_fields(ident: TokenStream, named_fields: FieldsNamed) -> syn::Result<TokenStream> {
    let fields: Vec<_> = named_fields.named.into_iter()
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

        Ok(#ident {
            #(#filled_fields)*
        })
    })
}

fn expand_unnamed_fields(ident: TokenStream, unnamed_fields: FieldsUnnamed) -> syn::Result<TokenStream> {
    let fields: Vec<_> = unnamed_fields.unnamed.into_iter()
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

        Ok(#ident(#(#filled_fields)*))
    })
}

fn expand_parsed_fields(fields: Vec<StructFieldData>) -> syn::Result<TokenStream> {
    let var_defs = fields.iter()
        .map(|field| {
            let varname = &field.var_name;
            let ty = &field.ty;

            quote! {
                let mut #varname: Option<#ty> = None;
            }
        });

    let name_matches = fields.iter()
        .map(|field| {
            let varname = &field.var_name;
            let ty = &field.ty;
            let string = &field.string;

            quote! {
                #string => {
                    if #varname.is_some() {
                        return Err(::nprs::parser::ParseValueError::DuplicateField(String::from(#string)));
                    };

                    #varname = Some(<#ty>::from_parsed_value(*__value)?);
                },
            }
        });

    let get_some_vars = fields.iter()
        .map(|field| {
            let varname = &field.var_name;
            let string = &field.string;

            quote! {
                let Some(#varname) = #varname else {
                    return Err(::nprs::parser::ParseValueError::MissingField(String::from(#string)));
                };
            }
        });

    Ok(quote! {
        #(#var_defs)*

        for (__param, __value) in __fields.into_iter() {
            match __param.as_str() {
                #(#name_matches)*
                _ => return Err(::nprs::parser::ParseValueError::UnknownField(__param.to_string())),
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
}

fn expand_struct_field(index: usize, field: Field) -> StructFieldData {
    let ty = field.ty;
    
    match &field.ident {
        Some(var_name_raw) => {
            let var_name = format_ident!("__{}", var_name_raw);
            let string = var_name_raw.to_string();

            StructFieldData {
                var_name_raw: Some(var_name_raw.clone()),
                var_name,
                string: quote! { #string },
                ty: quote! { #ty },
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
            }
        },
    }
}
