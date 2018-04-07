extern crate graphql_parser;
extern crate heck;
extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use std::fs::File;
use std::io::prelude::*;

use proc_macro::TokenStream;

#[proc_macro_derive(SomethingCompletelyDifferent, attributes(SomethingCompletelyDifferent))]
pub fn and_now_for_something_completely_different(input: TokenStream) -> TokenStream {
    let s = input.to_string();
    let ast = syn::parse_derive_input(&s).unwrap();
    let gen = impl_something_different(&ast);
    gen.parse().unwrap()
}

fn impl_something_different(ast: &syn::DeriveInput) -> quote::Tokens {
    let schema_path = extract_path(&ast.attrs).expect("path not specified");
    let cargo_manifest_dir =
        ::std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR env variable is defined");
    // We need to qualify the schema with the path to the crate it is part of
    let schema_path = format!("{}/{}", cargo_manifest_dir, schema_path);
    // panic!("schema_path: {}", schema_path,);
    let mut file = File::open(schema_path).expect("File not found");
    let mut the_schema_string = String::new();
    file.read_to_string(&mut the_schema_string).unwrap();
    let parsed_schema = graphql_parser::parse_schema(&the_schema_string).expect("parse error");
    let the_schema = syn::Lit::from(the_schema_string);

    let object_types = extract_object_types(&parsed_schema);
    let object_types: Vec<quote::Tokens> =
        object_types.iter().map(|ty| gql_type_to_rs(ty)).collect();

    quote! {
        pub const THE_SCHEMA: &'static str = #the_schema;


        #(#object_types)*
    }
}

fn extract_path(attributes: &[syn::Attribute]) -> Option<String> {
    let path_ident: syn::Ident = "path".into();
    for attr in attributes.iter() {
        if let syn::MetaItem::List(_ident, items) = &attr.value {
            for item in items.iter() {
                if let syn::NestedMetaItem::MetaItem(syn::MetaItem::NameValue(
                    name,
                    syn::Lit::Str(value, _),
                )) = item
                {
                    if name == &path_ident {
                        return Some(value.to_string());
                    }
                }
            }
        }
    }
    None
}

fn gql_type_to_rs(object_type: &graphql_parser::schema::ObjectType) -> quote::Tokens {
    let enum_name: syn::Ident = format!("{}Field", object_type.name).into();
    let struct_name: syn::Ident = object_type.name.as_str().into();
    let field_names: Vec<quote::Tokens> = object_type
        .fields
        .iter()
        .map(|f| {
            let ident: syn::Ident = f.name.clone().into();
            if f.arguments.is_empty() {
                quote!(#ident)
            } else {
                let args: Vec<quote::Tokens> = f.arguments
                    .iter()
                    .map(|arg| {
                        let field_name: syn::Ident = arg.name.clone().into();
                        let field_type = gql_type_to_json_type(&arg.value_type);
                        quote!( #field_name: #field_type )
                    })
                    .collect();
                quote!{#ident { #(#args),* }}
            }
        })
        .collect();
    quote!(
        pub enum #enum_name {
            #(#field_names),*
        }

        pub struct #struct_name {
            selected_fields: Vec<#enum_name>,
        }
    )
}

// fn extract_query(
//     document: &graphql_parser::schema::Document,
// ) -> Option<&graphql_parser::schema::ObjectType> {
//     use graphql_parser::schema::*;

//     for definition in document.definitions.iter() {
//         if let Definition::TypeDefinition(TypeDefinition::Object(obj)) = definition {
//             if obj.name == "Query" {
//                 return Some(obj);
//             }
//         }
//     }
//     None
// }

fn extract_object_types(
    document: &graphql_parser::schema::Document,
) -> Vec<&graphql_parser::schema::ObjectType> {
    use graphql_parser::schema::*;

    document
        .definitions
        .iter()
        .filter_map(|def| {
            if let Definition::TypeDefinition(TypeDefinition::Object(obj)) = def {
                Some(obj)
            } else {
                None
            }
        })
        .collect()
}

fn gql_type_to_json_type(gql_type: &graphql_parser::query::Type) -> quote::Tokens {
    use graphql_parser::query::Type::*;

    match gql_type {
        NamedType(name) => match name.as_str() {
            "Boolean" => quote!(Option<bool>),
            _ => {
                let ident: syn::Ident = name.as_str().into();
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

#[cfg(test)]
mod tests {
    use super::*;
    use graphql_parser::schema::*;

    #[test]
    fn basic_object_derive() {
        let gql = r#"
        type Pasta {
            shape: String!
            ingredients: [String!]!
        }
        "#;
        let parsed = parse_schema(gql).unwrap();
        assert_eq!(
            gql_type_to_rs(
                parsed
                    .definitions
                    .iter()
                    .filter_map(|d| {
                        if let Definition::TypeDefinition(TypeDefinition::Object(ty)) = d {
                            Some(ty)
                        } else {
                            None
                        }
                    })
                    .next()
                    .unwrap()
            ),
            quote!{
                pub enum PastaField { shape, ingredients }
                pub struct Pasta { selected_fields: Vec<PastaField>, }
            }
        )
    }

    #[test]
    fn object_derive_with_scalar_input() {
        let gql = r#"
        type Pasta {
            shape(strict: Boolean): String!
            ingredients(filter: String!): [String!]!
        }
        "#;
        let parsed = parse_schema(gql).unwrap();
        assert_eq!(
            gql_type_to_rs(
                parsed
                    .definitions
                    .iter()
                    .filter_map(|d| {
                        if let Definition::TypeDefinition(TypeDefinition::Object(ty)) = d {
                            Some(ty)
                        } else {
                            None
                        }
                    })
                    .next()
                    .unwrap()
            ),
            quote!{
                pub enum PastaField { shape { strict: Option<bool> }, ingredients { filter: Option<String> } }
                pub struct Pasta { selected_fields: Vec<PastaField>, }
            }
        )
    }
}
