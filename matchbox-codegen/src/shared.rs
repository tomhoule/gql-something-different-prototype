use context::DeriveContext;
use graphql_parser;
use heck::*;
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
        "ID" => "String",
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
    context: &DeriveContext,
) -> quote::Tokens {
    let inner_name = if is_list_type(&value_type) {
        "List"
    } else {
        let name = extract_inner_name(&value_type);

        if name == "ID" {
            "String"
        } else if context.is_scalar(name) {
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

pub fn query_value_to_tokens(value: &::graphql_parser::query::Value) -> quote::Tokens {
    use graphql_parser::query::Value;

    let prefix = quote!(::tokio_gql::graphql_parser::query::Value);

    match value {
        Value::Variable(_) => unimplemented!("variable as default value"),
        Value::Int(num) => {
            let num = num.as_i64();
            quote!(#prefix::Int(#num))
        }
        Value::Float(num) => quote!(#prefix::Float(#num)),
        Value::String(s) => quote!(#prefix::String(#s.to_string())),
        Value::Boolean(b) => quote!(#prefix::Boolean(#b)),
        Value::Null => unimplemented!("null as default value"),
        Value::Enum(en) => quote!(#prefix::Enum(#en.to_string())),
        Value::List(list) => {
            let inner: Vec<_> = list.iter().map(|v| query_value_to_tokens(v)).collect();
            quote!(vec![
                    #(#inner),*
                ])
        }
        Value::Object(obj) => {
            let inner: Vec<_> = obj.iter()
                .map(|(k, v)| (k, query_value_to_tokens(v)))
                .collect();
            let keys = inner.iter().map(|(k, _v)| k);
            let values = inner.iter().map(|(_k, v)| v);
            quote! {
                let mut map = ::std::collections::BTreeMap::new();
                #(
                    map.insert(#keys, #values);
                )*
                #prefix::Object(map)
            }
        }
    }
}

/// Used for implementing responders
pub fn schema_name_to_response_name(name: &str) -> String {
    format!("{}Response", name)
}

/// Figure out the name of the responder for a field
pub fn responder_type_name(field: graphql_parser::schema::Field) -> String {
    unimplemented!();
}

pub fn graphql_type_to_response_type(
    graphql_type: &graphql_parser::schema::Type,
    context: &DeriveContext,
) -> quote::Tokens {
    use graphql_parser::schema::Type;

    match graphql_type {
        Type::ListType(items) => graphql_type_to_response_type_inner(&graphql_type, context, false),
        Type::NamedType(ty) => graphql_type_to_response_type_inner(&graphql_type, context, false),
        Type::NonNullType(inner) => graphql_type_to_response_type_inner(&inner, context, true),
    }
}

fn graphql_type_to_response_type_inner(
    gql_type: &graphql_parser::query::Type,
    context: &DeriveContext,
    non_null: bool,
) -> quote::Tokens {
    use graphql_parser::schema::Type;
    let inner = match gql_type {
        Type::ListType(items) => {
            let inner = graphql_type_to_response_type_inner(&items, context, false);
            quote!(Vec<#inner>)
        }
        Type::NamedType(ty) => {
            if context.is_scalar(ty) {
                let ty = Term::new(correspondant_type(ty), Span::call_site());
                quote!(#ty)
            } else if context.is_enum(ty) {
                let ty = Term::new(&ty.to_camel_case(), Span::call_site());
                quote!(#ty)
            } else {
                let ty = Term::new(&schema_name_to_response_name(&ty), Span::call_site());
                quote!(#ty)
            }
        }
        Type::NonNullType(ty) => graphql_type_to_response_type_inner(&ty, context, true),
    };

    if non_null {
        inner
    } else {
        quote!(Option<#inner>)
    }
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
