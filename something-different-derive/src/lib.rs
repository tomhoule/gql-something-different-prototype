#![recursion_limit = "256"]

extern crate graphql_parser;
extern crate heck;
extern crate proc_macro;
extern crate proc_macro2;
extern crate syn;
#[macro_use]
extern crate quote;

mod coercion;
mod context;
mod enums;
mod inputs;
mod objects;
mod shared;
mod unions;

use coercion::*;
use context::DeriveContext;
use proc_macro2::{Literal, Span, Term};
use std::fs::File;
use std::io::prelude::*;

use proc_macro::TokenStream;

#[proc_macro_derive(SomethingCompletelyDifferent, attributes(SomethingCompletelyDifferent))]
pub fn and_now_for_something_completely_different(input: TokenStream) -> TokenStream {
    let input = proc_macro2::TokenStream::from(input);
    let ast = syn::parse2(input).expect("Derive input is well formed");
    let gen = impl_something_different(&ast);
    gen.into()
}

fn impl_something_different(ast: &syn::DeriveInput) -> quote::Tokens {
    let schema_path = extract_path(&ast.attrs).expect("path not specified");
    let cargo_manifest_dir =
        ::std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR env variable is defined");
    // We need to qualify the schema with the path to the crate it is part of
    let schema_path = format!("{}/{}", cargo_manifest_dir, schema_path);
    let mut file = File::open(schema_path).expect("File not found");
    let mut the_schema_string = String::new();
    file.read_to_string(&mut the_schema_string)
        .expect("Could not read schema.");

    let parsed_schema = graphql_parser::parse_schema(&the_schema_string).expect("parse error");
    let schema_as_string_literal = Literal::string(&the_schema_string);
    let mut context = DeriveContext::new();
    extract_definitions(&parsed_schema, &mut context);
    let mut definitions = Vec::new();
    gql_document_to_rs(&mut definitions, &context);
    let coerce_impls = coerce_impls(&context);

    quote! {
        pub const THE_SCHEMA: &'static str = #schema_as_string_literal;

        #(#definitions)*

        #(#coerce_impls)*
    }
}

fn coerce_impls(context: &DeriveContext) -> Vec<quote::Tokens> {
    let mut coerce_impls = Vec::new();

    for object_type in context.object_types.iter() {
        coerce_impls.push(object_type.impl_coerce(&context));
    }

    coerce_impls.push(
        context
            .get_schema()
            .expect("Schema is present")
            .impl_coerce(&context),
    );

    coerce_impls
}

fn extract_path(attributes: &[syn::Attribute]) -> Option<String> {
    let path_ident = Term::new("path", Span::call_site());
    for attr in attributes.iter() {
        if let syn::Meta::List(items) = &attr.interpret_meta().expect("Attribute is well formatted")
        {
            for item in items.nested.iter() {
                if let syn::NestedMeta::Meta(syn::Meta::NameValue(name_value)) = item {
                    let syn::MetaNameValue {
                        ident,
                        eq_token: _,
                        lit,
                    } = name_value;
                    if ident == &path_ident.to_string() {
                        if let syn::Lit::Str(lit) = lit {
                            return Some(lit.value());
                        }
                    }
                }
            }
        }
    }
    None
}

fn extract_definitions(document: &graphql_parser::schema::Document, context: &mut DeriveContext) {
    use graphql_parser::schema::*;

    for definition in document.definitions.iter() {
        match definition {
            Definition::TypeDefinition(ref type_def) => match type_def {
                TypeDefinition::Object(ref object_type) => {
                    context.insert_object(object_type.clone());
                }
                TypeDefinition::Enum(ref enum_type) => {
                    context.insert_enum(enum_type.clone());
                }
                TypeDefinition::InputObject(ref input_object_type) => {
                    context.insert_input_type(input_object_type.clone());
                }
                TypeDefinition::Scalar(ref scalar_type) => {
                    context.insert_scalar(scalar_type.name.to_string());
                }
                TypeDefinition::Union(ref union_type) => {
                    context.insert_union(union_type.clone());
                }
                TypeDefinition::Interface(interface_type) => {
                    context.insert_interface(interface_type.clone());
                }
            },
            Definition::DirectiveDefinition(_) => unimplemented!(),
            Definition::SchemaDefinition(schema_definition) => {
                context.set_schema(schema_definition.clone())
            }
            Definition::TypeExtension(_) => unimplemented!(),
        };
    }
}

fn gql_document_to_rs(buf: &mut Vec<quote::Tokens>, context: &DeriveContext) {
    for object in context.object_types.iter() {
        buf.push(objects::gql_type_to_rs(object, &context));
    }

    for enum_type in context.enum_types.values() {
        buf.push(enums::gql_enum_to_rs(enum_type));
    }

    for input_type in context.input_types.values() {
        buf.push(inputs::gql_input_to_rs(input_type, &context));
    }

    for union_type in context.union_types.values() {
        buf.push(unions::gql_union_to_rs(union_type, &context));
    }

    for _interface_type in context.interface_types.values() {
        unimplemented!();
    }

    if let Some(ref schema_definition) = context.get_schema() {
        let mut fields: Vec<quote::Tokens> = Vec::new();
        if let Some(ref query) = schema_definition.query {
            let object_name = Term::new(query.as_str(), Span::call_site());
            fields.push(quote!(query: Vec<#object_name>));
        }

        if let Some(ref mutation) = schema_definition.mutation {
            let object_name = Term::new(mutation.as_str(), Span::call_site());
            fields.push(quote!(mutation: Vec<#object_name>));
        }

        if let Some(ref subscription) = schema_definition.subscription {
            let object_name = Term::new(subscription.as_str(), Span::call_site());
            fields.push(quote!(subscription: Vec<#object_name>));
        }

        buf.push(quote!{
            #[derive(Debug, PartialEq)]
            pub struct Schema {
                #(#fields),*,
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use graphql_parser::schema::*;
    /// This is repeated between test modules, we may have to create a test_support crate to overcome that limitation.

    macro_rules! assert_expands_to {
        ($gql_string:expr => $expanded:tt) => {
            let gql = $gql_string;
            let parsed = parse_schema(gql).unwrap();
            let mut buf = Vec::new();
            let mut context = DeriveContext::new();
            extract_definitions(&parsed, &mut context);
            gql_document_to_rs(&mut buf, &context);
            let got = quote!(#(#buf)*);
            let expected = quote! $expanded ;
            assert_eq!(expected, got);
        };
    }

    #[test]
    fn basic_object_derive() {
        assert_expands_to! {
            r#"
            type Pasta {
                shape: String!
                ingredients: [String!]!
            }
            "# => {
                #[derive(Debug, PartialEq)]
                pub enum Pasta { Shape, Ingredients }
            }
        }
    }

    #[test]
    fn object_derive_with_scalar_input() {
        assert_expands_to! {
            r#"
            type Pasta {
                shape(strict: Boolean): String!
                ingredients(filter: String!): [String!]!
            }
            "# => {
                #[derive(Debug, PartialEq)]
                pub enum Pasta {
                    Shape { strict: Option<bool> },
                    Ingredients { filter: String }
                }
            }
        }
    }

    #[test]
    fn object_derive_with_description_string() {
        assert_expands_to!{
            r##"
            """
            Represents a point on the plane.
            """
            type Point {
                x: Int!
                y: Int!
            }
            "## => {
                #[doc = "Represents a point on the plane.\n"]
                #[derive(Debug, PartialEq)]
                pub enum Point { X, Y }
            }
        }
    }

    #[test]
    fn object_derive_with_nested_field() {
        assert_expands_to! {
            r##"
                type DessertDescriptor {
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
                    dessert: DessertDescriptor!
                }
            "## => {
                #[derive(Debug, PartialEq)]
                pub enum DessertDescriptor {
                    Name,
                    ContainsChocolate
                }

                #[derive(Debug, PartialEq)]
                pub enum Cheese {
                    Name,
                    Blue
                }

                #[derive(Debug, PartialEq)]
                pub enum Meal {
                    MainCourse,
                    Cheese { selection: Vec<Cheese>, vegan: Option<bool> },
                    Dessert { selection: Vec<DessertDescriptor>, }
                }
            }
        }
    }

    #[test]
    fn schema_definition() {
        assert_expands_to! {
            r##"
            schema {
                query: MyQuery
                mutation: AMutation
                subscription: TheSubscription
            }
            "## => {
                #[derive(Debug, PartialEq)]
                pub struct Schema {
                    query: Vec<MyQuery>,
                    mutation: Vec<AMutation>,
                    subscription: Vec<TheSubscription>,
                }
            }
        }
    }

    #[test]
    fn partial_schema_definition() {
        assert_expands_to! {
            r##"
            schema {
                query: SomeQuery
            }
            "## => {
                #[derive(Debug, PartialEq)]
                pub struct Schema {
                    query: Vec<SomeQuery>,
                }
            }
        }
    }
}
