extern crate graphql_parser;
extern crate heck;
extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use std::fs::File;
use std::io::prelude::*;

use graphql_parser::schema::{EnumType, InputObjectType, InterfaceType, ObjectType,
                             SchemaDefinition, UnionType};
use heck::*;
use proc_macro::TokenStream;
use std::collections::{HashMap, HashSet};

struct DeriveContext {
    enum_types: HashMap<String, EnumType>,
    input_types: HashMap<String, InputObjectType>,
    interface_types: HashMap<String, InterfaceType>,
    object_types: Vec<ObjectType>,
    scalar_types: HashSet<String>,
    union_types: HashMap<String, UnionType>,
    schema_type: Option<SchemaDefinition>,
}

impl DeriveContext {
    pub fn new() -> DeriveContext {
        let mut scalar_types = HashSet::new();

        // See https://graphql.org/learn/schema/#scalar-types
        scalar_types.insert("Int".to_string());
        scalar_types.insert("Float".to_string());
        scalar_types.insert("String".to_string());
        scalar_types.insert("Boolean".to_string());

        let object_types = Vec::new();
        let input_types = HashMap::new();
        let enum_types = HashMap::new();
        let union_types = HashMap::new();
        let interface_types = HashMap::new();

        DeriveContext {
            enum_types,
            input_types,
            interface_types,
            object_types,
            scalar_types,
            schema_type: None,
            union_types,
        }
    }

    pub fn insert_object(&mut self, object_type: ObjectType) {
        self.object_types.push(object_type);
    }

    pub fn insert_enum(&mut self, enum_type: EnumType) {
        self.enum_types.insert(enum_type.name.clone(), enum_type);
    }

    pub fn insert_input_type(&mut self, input_type: InputObjectType) {
        self.input_types.insert(input_type.name.clone(), input_type);
    }

    pub fn insert_scalar(&mut self, scalar_type: String) {
        self.scalar_types.insert(scalar_type);
    }

    pub fn is_scalar(&self, type_name: &str) -> bool {
        self.scalar_types.contains(type_name)
    }

    pub fn insert_union(&mut self, union_type: UnionType) {
        self.union_types.insert(union_type.name.clone(), union_type);
    }

    pub fn insert_interface(&mut self, interface_type: InterfaceType) {
        self.interface_types
            .insert(interface_type.name.clone(), interface_type);
    }
}

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
    let mut context = DeriveContext::new();
    extract_definitions(&parsed_schema, &mut context);
    let mut definitions = Vec::new();
    gql_document_to_rs(&mut definitions, &context);
    let extractor_impls = extractor_impls(&context);

    quote! {
        pub const THE_SCHEMA: &'static str = #schema_as_string_literal;

        #(definitions)*

        #(#extractor_impls)*
    }
}

fn extractor_impls(context: &DeriveContext) -> Vec<quote::Tokens> {
    Vec::new()
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
                context.schema_type = Some(schema_definition.clone())
            }
            Definition::TypeExtension(_) => unimplemented!(),
        };
    }
}

fn gql_document_to_rs(buf: &mut Vec<quote::Tokens>, context: &DeriveContext) {
    for object in context.object_types.iter() {
        buf.push(gql_type_to_rs(object, &context));
    }

    for enum_type in context.enum_types.values() {
        buf.push(gql_enum_to_rs(enum_type));
    }

    for input_type in context.input_types.values() {
        buf.push(gql_input_to_rs(input_type, &context));
    }

    for union_type in context.union_types.values() {
        buf.push(gql_union_to_rs(union_type, &context));
    }

    for interface_type in context.interface_types.values() {
        unimplemented!();
    }

    if let Some(ref schema_definition) = context.schema_type {
        let mut fields: Vec<quote::Tokens> = Vec::new();
        if let Some(ref query) = schema_definition.query {
            let object_name: syn::Ident = query.as_str().into();
            fields.push(quote!(query: #object_name));
        }

        if let Some(ref mutation) = schema_definition.mutation {
            let object_name: syn::Ident = mutation.as_str().into();
            fields.push(quote!(mutation: #object_name));
        }

        if let Some(ref subscription) = schema_definition.subscription {
            let object_name: syn::Ident = subscription.as_str().into();
            fields.push(quote!(subscription: #object_name));
        }

        buf.push(quote!{
            pub struct Schema {
                #(#fields),*,
            }
        })
    }
}

fn gql_union_to_rs(union_type: &UnionType, _context: &DeriveContext) -> quote::Tokens {
    let name: syn::Ident = union_type.name.as_str().into();
    let united_types = union_type.types.iter().map(|ty| {
        let ident: syn::Ident = format!("on{}", ty.as_str()).into();
        let selection_type: syn::Ident = ty.as_str().into();
        quote!(#ident(Vec<#selection_type>))
    });
    quote! {
        pub enum #name {
            #(#united_types),*
        }
    }
}

fn gql_input_to_rs(input_type: &InputObjectType, _context: &DeriveContext) -> quote::Tokens {
    let name: syn::Ident = input_type.name.as_str().into();
    let values: Vec<syn::Ident> = input_type
        .fields
        .iter()
        .map(|v| v.name.to_camel_case().into())
        .collect();
    let doc_attr: quote::Tokens = if let Some(ref doc_string) = input_type.description {
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

fn extract_inner_name(ty: &graphql_parser::query::Type) -> &str {
    use graphql_parser::query::Type::*;

    match ty {
        NamedType(name) => name,
        ListType(inner) => extract_inner_name(inner),
        NonNullType(inner) => extract_inner_name(inner),
    }
}

fn gql_type_to_rs(
    object_type: &graphql_parser::schema::ObjectType,
    context: &DeriveContext,
) -> quote::Tokens {
    let name: syn::Ident = object_type.name.as_str().into();
    // let struct_name_lit: syn::Lit = object_type.name.as_str().into();
    let field_names: Vec<quote::Tokens> = object_type
        .fields
        .iter()
        .map(|f| {
            let ident: syn::Ident = f.name.to_camel_case().into();
            let args: Vec<quote::Tokens> = f.arguments
                .iter()
                .map(|arg| {
                    let field_name: syn::Ident = arg.name.to_mixed_case().into();
                    let field_type = gql_type_to_json_type(&arg.value_type);
                    quote!( #field_name: #field_type )
                })
                .collect();
            let field_type_name = extract_inner_name(&f.field_type);
            let sub_field_set: Option<syn::Ident> = if context.is_scalar(field_type_name) {
                None
            } else {
                Some(field_type_name.to_camel_case().into())
            };
            let sub_field_set: Option<quote::Tokens> =
                sub_field_set.map(|set| quote!{ selection: Vec<#set>, });
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
        #doc_attr
        pub enum #name {
            #(#field_names),*
        }
    )
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
                pub enum Pasta { Shape { strict: Option<bool> }, Ingredients { filter: Option<String> } }
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
                pub enum DessertDescriptor {
                    Name,
                    ContainsChocolate
                }

                pub enum Cheese {
                    Name,
                    Blue
                }

                pub enum Meal {
                    MainCourse,
                    Cheese { selection: Vec<Cheese>, vegan: Option<bool> },
                    Dessert { selection: Vec<DessertDescriptor>, }
                }
            }
        }
    }

    #[test]
    fn enum_derive() {
        assert_expands_to! {
            r##"
            enum Dog {
                GOLDEN
                CHIHUAHUA
                CORGI
            }
            "## => {
                pub enum Dog {
                    Golden,
                    Chihuahua,
                    Corgi,
                }
            }
        }
    }

    #[test]
    fn enum_derive_with_docs() {
        assert_expands_to! {
            r##"
            """
            The bread kinds supported by this app.

            [Bread](https://en.wikipedia.org/wiki/bread) on wikipedia.
            """
            enum BreadKind {
                WHITE
                FULL_GRAIN
            }
            "## => {
                #[doc = "The bread kinds supported by this app.\n\n[Bread](https://en.wikipedia.org/wiki/bread) on wikipedia.\n"]
                pub enum BreadKind {
                    White,
                    FullGrain,
                }
            }
        }
    }

    #[test]
    fn simple_input_object_derive() {
        assert_expands_to! {
            r##"
            """
            A point in 2, 3 or 4 dimensions, because why not?
            """
            input Point {
                X: Int!
                Y: Int!
                Z: Int!
                ZZ: Int!
            }
            "## => {
                #[doc = "A point in 2, 3 or 4 dimensions, because why not?\n"]
                pub enum Point {
                    X,
                    Y,
                    Z,
                    Zz,
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
                pub struct Schema {
                    query: MyQuery,
                    mutation: AMutation,
                    subscription: TheSubscription,
                }
            }
        }
    }

    #[test]
    fn unions() {
        assert_expands_to! {
            r##"
            union SearchResult = Human | Droid | Starship
            "## => {
                pub enum SearchResult {
                    onHuman(Vec<Human>),
                    onDroid(Vec<Droid>),
                    onStarship(Vec<Starship>)
                }
            }
        }
    }
}
