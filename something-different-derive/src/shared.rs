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
    gql_type_to_json_type_inner(gql_type, false)
}

fn gql_type_to_json_type_inner(
    gql_type: &graphql_parser::query::Type,
    non_null: bool,
) -> quote::Tokens {
    use graphql_parser::query::Type::*;

    match gql_type {
        NamedType(name) => {
            let inner_name = Term::new(correspondant_type(name.as_str()), Span::call_site());
            if non_null {
                quote!(#inner_name)
            } else {
                quote!(Option<#inner_name>)
            }
        }
        ListType(inner) => {
            let inner_converted = gql_type_to_json_type_inner(&inner, false);
            if non_null {
                quote!(Vec<#inner_converted>)
            } else {
                quote!(Option<Vec<#inner_converted>>)
            }
        }
        NonNullType(inner) => {
            let inner_converted = gql_type_to_json_type_inner(&inner, true);
            quote!(#inner_converted)
        }
    }
}

/// Correspondance function between a GraphQL scalar type name(Int, String...) and rust types
pub fn correspondant_type(gql_type: &str) -> &str {
    match gql_type {
        "Int" => "i32",
        "String" => "String",
        "Double" => "f64",
        "Boolean" => "bool",
        other => other,
    }
}
