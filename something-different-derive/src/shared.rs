use graphql_parser;
use proc_macro2::{Span, Term};
use quote;

pub fn extract_inner_name(ty: &graphql_parser::query::Type) -> &str {
    use graphql_parser::query::Type::*;

    match ty {
        NamedType(name) => name,
        ListType(inner) => extract_inner_name(inner),
        NonNullType(inner) => extract_inner_name(inner),
    }
}

pub fn gql_type_to_json_type(gql_type: &graphql_parser::query::Type) -> quote::Tokens {
    use graphql_parser::query::Type::*;

    match gql_type {
        NamedType(name) => match name.as_str() {
            "Boolean" => quote!(Option<bool>),
            _ => {
                let ident = Term::new(name, Span::call_site());
                quote!(Option<#ident>)
            }
        },
        ListType(inner) => {
            let inner_converted = gql_type_to_json_type(&inner);
            quote!(Vec<#inner_converted>)
        }
        NonNullType(inner) => {
            let inner_converted = gql_type_to_json_type(&inner);
            quote!(#inner_converted)
        }
    }
}
