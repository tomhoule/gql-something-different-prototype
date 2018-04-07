extern crate graphql_parser;
extern crate heck;
extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use std::fs::File;
use std::io::prelude::*;

use heck::*;
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
    let mut file = File::open(schema_path).expect("File not found");
    let mut the_schema_string = String::new();
    file.read_to_string(&mut the_schema_string).unwrap();

    let parsed_schema = graphql_parser::parse_schema(&the_schema_string).expect("parse error");
    let schema_as_string_literal = syn::Lit::from(the_schema_string);
    let definitions = gql_document_to_rs(&parsed_schema);

    quote! {
        pub const THE_SCHEMA: &'static str = #schema_as_string_literal;

        #definitions
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

fn gql_document_to_rs(document: &graphql_parser::schema::Document) -> quote::Tokens {
    use graphql_parser::schema::*;

    let mut definitions: Vec<quote::Tokens> = Vec::with_capacity(document.definitions.len());
    for definition in document.definitions.iter() {
        let tokens = match definition {
            Definition::TypeDefinition(ref type_def) => match type_def {
                TypeDefinition::Object(ref object_type) => gql_type_to_rs(object_type),
                TypeDefinition::Enum(ref enum_type) => gql_enum_to_rs(enum_type),
                _ => unimplemented!(),
            },
            _ => unimplemented!(),
        };
        definitions.push(tokens);
    }
    quote!(#(#definitions)*)
}

fn gql_enum_to_rs(enum_type: &graphql_parser::schema::EnumType) -> quote::Tokens {
    let name: syn::Ident = enum_type.name.as_str().into();
    let values: Vec<syn::Ident> = enum_type
        .values
        .iter()
        .map(|v| v.name.to_camel_case().into())
        .collect();
    let doc_attr: quote::Tokens = if let Some(ref doc_string) = enum_type.description {
        let str_literal: syn::Lit = doc_string.as_str().into();
        quote!(#[doc = #str_literal])
    } else {
        quote!()
    };
    quote!{
        #doc_attr
        pub enum #name {
            #(#values),* ,
        }
    }
}

fn gql_type_to_rs(object_type: &graphql_parser::schema::ObjectType) -> quote::Tokens {
    let enum_name: syn::Ident = format!("{}Field", object_type.name).into();
    let struct_name: syn::Ident = object_type.name.as_str().into();
    // let struct_name_lit: syn::Lit = object_type.name.as_str().into();
    let field_names: Vec<quote::Tokens> = object_type
        .fields
        .iter()
        .map(|f| {
            let ident: syn::Ident = f.name.clone().into();
            let args: Vec<quote::Tokens> = f.arguments
                .iter()
                .map(|arg| {
                    let field_name: syn::Ident = arg.name.clone().into();
                    let field_type = gql_type_to_json_type(&arg.value_type);
                    quote!( #field_name: #field_type )
                })
                .collect();
            let sub_field_set: Option<syn::Ident> =
                Some(format!("{}Field", f.field_type).to_camel_case().into());
            let sub_field_set: Option<quote::Tokens> =
                sub_field_set.map(|set| quote!{ field_set: Vec<#set>, });
            if sub_field_set.is_some() || !args.is_empty() {
                quote!{#ident { #sub_field_set #(#args),* }}
            } else {
                quote!(#ident)
            }
        })
        .collect();
    let doc_attr: quote::Tokens = if let Some(ref doc_string) = object_type.description {
        let str_literal: syn::Lit = doc_string.as_str().into();
        quote!(#[doc = #str_literal])
    } else {
        quote!()
    };

    quote!(
        pub enum #enum_name {
            #(#field_names),*
        }

        #doc_attr
        pub struct #struct_name {
            selected_fields: Vec<#enum_name>,
        }

    )
    // impl ::tokio_gql::FromQueryField for #struct_name {
    //     type Arguments = (#(#arguments),)

    //     fn from_query_field(field: ::tokio_gql::query::Field) -> Result<Self, ::tokio_gql::QueryValidationError> {
    //         if field.name != #struct_name_lit {
    //             return Err(::tokio_gql::QueryValidationError::InvalidField { got: field.name.clone(), expected: #struct_name_lit })
    //         }

    //         let args = <Self::Arguments as tokio_gql::FromQueryArguments>::from_arguments()?;
    //     }
    // }
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
            gql_document_to_rs(&parsed),
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
            gql_document_to_rs(&parsed),
            quote!{
                pub enum PastaField { shape { strict: Option<bool> }, ingredients { filter: Option<String> } }
                pub struct Pasta { selected_fields: Vec<PastaField>, }
            }
        )
    }

    #[test]
    fn object_derive_with_description_string() {
        let gql = r##"
        """
        Represents a point on the plane.
        """
        type Point {
            x: Int!
            y: Int!
        }
        "##;

        let parsed = parse_schema(gql).unwrap();
        let expanded = gql_document_to_rs(&parsed);
        assert_eq!(
            expanded,
            quote!{
                pub enum PointField { x, y }
                #[doc = "Represents a point on the plane.\n"]
                pub struct Point { selected_fields: Vec<PointField>, }
            }
        )
    }

    #[test]
    fn object_derive_with_nested_field() {
        let gql = r##"
        type Dessert {
            name: String!
            contains_chocolate: Boolean
        }

        type Cheese {
            name: String!
            blue: Boolean
        }

        type Meal {
            main_course: String!
            cheese(vegan: Boolean): Cheese
            dessert: Dessert!
        }
        "##;

        let parsed = parse_schema(gql).unwrap();
        let expanded = gql_document_to_rs(&parsed);

        assert_eq!(
            expanded,
            quote!{
                enum DessertField {
                    Name,
                    ContainsChocolate,
                }

                enum CheeseField {
                    Name,
                    Blue,
                }

                enum MealField {
                    MainCourse,
                    Cheese ,
                    Dessert,
                }
            }
        )
    }

    #[test]
    fn enum_derive() {
        let gql = r##"
        enum Dog {
            GOLDEN
            CHIHUAHUA
            CORGI
        }
        "##;
        let parsed = parse_schema(gql).unwrap();
        assert_eq!(
            gql_document_to_rs(&parsed),
            quote!(pub enum Dog {
                Golden,
                Chihuahua,
                Corgi,
            })
        )
    }

    #[test]
    fn enum_derive_with_docs() {
        let gql = r##"
        """
        The bread kinds supported by this app.

        [Bread](https://en.wikipedia.org/wiki/bread) on wikipedia.
        """
        enum BreadKind {
            WHITE
            FULL_GRAIN
        }
        "##;
        let parsed = parse_schema(gql).unwrap();
        assert_eq!(
            gql_document_to_rs(&parsed),
            quote!(
                #[doc = "The bread kinds supported by this app.\n\n[Bread](https://en.wikipedia.org/wiki/bread) on wikipedia.\n"]
                pub enum BreadKind {
                    White,
                    FullGrain,
                }
            )
        )
    }
}
