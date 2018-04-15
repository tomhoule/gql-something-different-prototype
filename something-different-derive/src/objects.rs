use context::DeriveContext;
use graphql_parser;
use heck::*;
use proc_macro2::{Literal, Span, Term};
use quote;
use shared;

pub fn gql_type_to_rs(
    object_type: &graphql_parser::schema::ObjectType,
    context: &DeriveContext,
) -> quote::Tokens {
    let name = Term::new(object_type.name.as_str(), Span::call_site());
    // let struct_name_lit: syn::Lit = object_type.name.as_str().into();
    let field_names: Vec<quote::Tokens> = get_field_names(&object_type.fields, context);
    let doc_attr: quote::Tokens = if let Some(ref doc_string) = object_type.description {
        let str_literal = Literal::string(doc_string.as_str());
        quote!(#[doc = #str_literal])
    } else {
        quote!()
    };

    quote!(
        #doc_attr
        #[derive(Debug, PartialEq)]
        pub enum #name {
            #(#field_names),*
        }
    )
}

pub fn get_field_names<'a>(
    fields: impl IntoIterator<Item = &'a graphql_parser::schema::Field>,
    context: &DeriveContext,
) -> Vec<quote::Tokens> {
    fields
        .into_iter()
        .map(|f| {
            let ident = Term::new(&f.name.to_camel_case(), Span::call_site());
            let args: Vec<quote::Tokens> = f.arguments
                .iter()
                .map(|arg| {
                    let field_name =
                        Term::new(arg.name.to_mixed_case().as_str(), Span::call_site());
                    let field_type = shared::gql_type_to_json_type(&arg.value_type);
                    quote!( #field_name: #field_type )
                })
                .collect();
            let field_type_name = shared::extract_inner_name(&f.field_type);
            let sub_field_set: Option<Term> =
                if (context.is_scalar(field_type_name) || context.is_enum(field_type_name)) {
                    None
                } else {
                    Some(Term::new(
                        field_type_name.to_camel_case().as_str(),
                        Span::call_site(),
                    ))
                };
            let sub_field_set: Option<quote::Tokens> =
                sub_field_set.map(|set| quote!{ selection: Vec<#set>, });
            if sub_field_set.is_some() || !args.is_empty() {
                quote!{#ident { #sub_field_set #(#args),* }}
            } else {
                quote!(#ident)
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn object_type_with_input_object_argument() {
        assert_expands_to! {
            r#"
            input SpecialInstructions {
                cooking_time: Int
                this_is_redundant: String
            }

            input CookingInstructions {
                temperature: Int!
                pressure: Boolean
                salt: Boolean!
                additional: SpecialInstructions
            }

            type Pasta {
                ingredients(filter: String!): [String!]!
                instructions(filter: CookingInstructions): String
                name: String!
            }
            "# => {
                #[derive(Debug, PartialEq)]
                pub enum Pasta {
                    Ingredients { filter: String },
                    Instructions { filter: Option<CookingInstructions> },
                    Name
                }

                #[derive(Debug, PartialEq)]
                pub struct CookingInstructions {
                    temperature: i32,
                    pressure: Option<bool>,
                    salt: bool,
                    additional: Option<SpecialInstructions>,
                }

                #[derive(Debug, PartialEq)]
                pub struct SpecialInstructions {
                    cooking_time: Option<i32>,
                    this_is_redundant: Option<String>,
                }
            }
        }
    }
}
