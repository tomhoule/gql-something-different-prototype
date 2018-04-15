use context;
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

/// Will return true for nullable and non-nullable list types
pub fn is_list_type(gql_type: &graphql_parser::query::Type) -> bool {
    use graphql_parser::query::Type;

    match gql_type {
        Type::NamedType(_) => false,
        Type::NonNullType(inner) => is_list_type(inner),
        Type::ListType(_) => true,
    }
}

/// The variant to extract for that type
pub fn value_variant_for_type(
    value_type: &graphql_parser::schema::Type,
    context: &context::DeriveContext,
) -> quote::Tokens {
    let inner_name = if is_list_type(&value_type) {
        "List"
    } else {
        let name = extract_inner_name(&value_type);

        if context.is_scalar(name) {
            name
        } else if context.is_enum(name) {
            "Enum"
        } else {
            "Object"
        }
    };
    let variant = Term::new(inner_name, Span::call_site());
    quote!(::tokio_gql::graphql_parser::schema::Value::#variant)
}

pub fn type_is_optional(value_type: &graphql_parser::schema::Type) -> bool {
    if let graphql_parser::schema::Type::NonNullType(_) = value_type {
        false
    } else {
        true
    }
}

#[cfg(test)]
macro_rules! assert_expands_to {
    ($gql_string:expr => $expanded:tt) => {
        let gql = $gql_string;
        let parsed = ::graphql_parser::parse_schema(gql).unwrap();
        let mut buf = Vec::new();
        let mut context = DeriveContext::new();
        ::extract_definitions(&parsed, &mut context);
        ::gql_document_to_rs(&mut buf, &context);
        let got = quote!(#(#buf)*);
        let expected = quote! $expanded ;
        assert_eq!(expected, got);
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_list_type_works() {
        use graphql_parser::query::Type;

        assert!(is_list_type(&Type::ListType(Box::new(Type::NamedType(
            "meow".to_string()
        )))));

        assert!(is_list_type(&Type::NonNullType(Box::new(Type::ListType(
            Box::new(Type::NamedType("meow".to_string()))
        )))));

        assert!(!is_list_type(&Type::NonNullType(Box::new(
            Type::NamedType("meow".to_string())
        ))));
    }
}
