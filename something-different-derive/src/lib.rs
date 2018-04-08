#![recursion_limit = "128"]

extern crate graphql_parser;
extern crate heck;
extern crate proc_macro;
extern crate proc_macro2;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro2::{Literal, Span, Term};
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
        scalar_types.insert("ID".to_string());

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
    file.read_to_string(&mut the_schema_string).unwrap();

    let parsed_schema = graphql_parser::parse_schema(&the_schema_string).expect("parse error");
    let schema_as_string_literal = Literal::string(&the_schema_string);
    let mut context = DeriveContext::new();
    extract_definitions(&parsed_schema, &mut context);
    let mut definitions = Vec::new();
    gql_document_to_rs(&mut definitions, &context);
    let extractor_impls = extractor_impls(&context);

    quote! {
        pub const THE_SCHEMA: &'static str = #schema_as_string_literal;

        #(#definitions)*

        #(#extractor_impls)*
    }
}

fn extractor_impls(context: &DeriveContext) -> Vec<quote::Tokens> {
    let mut coerce_impls = Vec::new();

    for object_type in context.object_types.iter() {
        let name = Term::new(&object_type.name, Span::call_site());
        let field_matchers = object_type.fields.iter().map(|field| {
            let variant_name = Term::new(&field.name.to_camel_case(), Span::call_site());
            let variant_name_literal = &field.name;
            let argument_idents: Vec<Term> = Vec::new();
            let argument_literals: Vec<Literal> = Vec::new();
            quote! {
                if field.name == #variant_name_literal {
                    #(
                        let #argument_idents = field.arguments.find(|arg| arg.name == #argument_literals).unwrap();
                    )
                    result.push(#name::#variant_name { #(#argument_idents),* })
                }
            }
        });
        let implementation = quote! {
            impl ::tokio_gql::coercion::CoerceSelection for #name {
                fn coerce(
                    query: ::tokio_gql::graphql_parser::query::SelectionSet,
                    context: &::tokio_gql::query_validation::ValidationContext,
                ) -> Vec<#name> {
                    let mut result = Vec::new();

                    for item in query.items.iter() {
                        match item {
                            ::tokio_gql::graphql_parser::query::Selection::Field(field) => {
                                #(#field_matchers)*
                            }
                            ::tokio_gql::graphql_parser::query::Selection::FragmentSpread(_) => unimplemented!(),
                            ::tokio_gql::graphql_parser::query::Selection::InlineFragment(_) => unimplemented!(),

                        }
                    }

                    result
                }
            }
        };

        coerce_impls.push(implementation);
    }

    coerce_impls.push(impl_schema_coerce(
        &context.schema_type.clone().expect("Schema is present"),
        context,
    ));

    coerce_impls
}

struct FieldVariantDescriptor {
    arguments: Vec<graphql_parser::schema::InputValue>,
    field_type: Option<String>,
}

fn impl_schema_coerce(
    schema: &graphql_parser::schema::SchemaDefinition,
    _context: &DeriveContext,
) -> quote::Tokens {
    let mut field_values: Vec<Term> = Vec::new();
    let mut field_names: Vec<Term> = Vec::new();

    if let Some(ref name) = schema.query {
        let name = Term::new(name.as_str(), Span::call_site());
        field_values.push(name);
        field_names.push(Term::new("query", Span::call_site()));
    }

    if let Some(ref name) = schema.mutation {
        let name = Term::new(name.as_str(), Span::call_site());
        field_values.push(name);
        field_names.push(Term::new("mutation", Span::call_site()));
    }

    if let Some(ref name) = schema.subscription {
        let name = Term::new(name.as_str(), Span::call_site());
        field_values.push(name);
        field_names.push(Term::new("subscription", Span::call_site()));
    }

    let node_types: Vec<Term> = field_names
        .iter()
        .map(|name| Term::new(&format!("{}", name).to_camel_case(), Span::call_site()))
        .collect();
    let field_names_2 = field_names.clone();

    quote! {
        impl ::tokio_gql::coercion::CoerceQueryDocument for Schema {
            fn coerce(
                document: &::tokio_gql::graphql_parser::query::Document,
                context: &::tokio_gql::query_validation::ValidationContext
            ) -> Self {
                use ::tokio_gql::graphql_parser::query::*;

                #(
                    let #field_names = document.definitions
                        .iter()
                        .filter_map(|op| {
                            if let ::tokio_gql::graphql_parser::query::Definition::Operation(::tokio_gql::graphql_parser::query::OperationDefinition::#node_types(ref definition)) = op {
                                return Some(#field_values::coerce(definition.clone().selection_set, context))
                            }
                            None
                        })
                        .next()
                        .unwrap();
                )*

                Schema {
                    #(#field_names_2),*
                }
            }
        }
    }
}

fn extract_path(attributes: &[syn::Attribute]) -> Option<String> {
    let path_ident = Term::new("path", Span::call_site());
    for attr in attributes.iter() {
        if let syn::MetaItem::List(_ident, items) = &attr.value {
            for item in items.iter() {
                if let syn::NestedMetaItem::MetaItem(syn::MetaItem::NameValue(
                    name,
                    syn::Lit::Str(value, _),
                )) = item
                {
                    if name == &path_ident.to_string() {
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

    for _interface_type in context.interface_types.values() {
        unimplemented!();
    }

    if let Some(ref schema_definition) = context.schema_type {
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

fn gql_union_to_rs(union_type: &UnionType, _context: &DeriveContext) -> quote::Tokens {
    let name = Term::new(union_type.name.as_str(), Span::call_site());
    let united_types = union_type.types.iter().map(|ty| {
        let ident = Term::new(&format!("on{}", ty.as_str()), Span::call_site());
        let selection_type = Term::new(ty.as_str(), Span::call_site());
        quote!(#ident(Vec<#selection_type>))
    });
    quote! {
        #[derive(Debug, PartialEq)]
        pub enum #name {
            #(#united_types),*
        }
    }
}

fn gql_input_to_rs(input_type: &InputObjectType, _context: &DeriveContext) -> quote::Tokens {
    let name = Term::new(&input_type.name, Span::call_site());
    let values: Vec<Term> = input_type
        .fields
        .iter()
        .map(|v| Term::new(&v.name.to_camel_case(), Span::call_site()))
        .collect();
    let doc_attr: quote::Tokens = if let Some(ref doc_string) = input_type.description {
        let str_literal = Literal::string(&doc_string);
        quote!(#[doc = #str_literal])
    } else {
        quote!()
    };

    quote!{
        #doc_attr
        #[derive(Debug, PartialEq)]
        pub enum #name {
            #(#values),* ,
        }
    }
}

fn gql_enum_to_rs(enum_type: &graphql_parser::schema::EnumType) -> quote::Tokens {
    let name = Term::new(enum_type.name.as_str(), Span::call_site());
    let values: Vec<Term> = enum_type
        .values
        .iter()
        .map(|v| Term::new(v.name.to_camel_case().as_str(), Span::call_site()))
        .collect();
    let doc_attr: quote::Tokens = if let Some(ref doc_string) = enum_type.description {
        let str_literal = Literal::string(doc_string.as_str());
        quote!(#[doc = #str_literal])
    } else {
        quote!()
    };
    quote!{
        #doc_attr
        #[derive(Debug, PartialEq)]
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
    let name = Term::new(object_type.name.as_str(), Span::call_site());
    // let struct_name_lit: syn::Lit = object_type.name.as_str().into();
    let field_names: Vec<quote::Tokens> = object_type
        .fields
        .iter()
        .map(|f| {
            let ident = Term::new(&f.name.to_camel_case(), Span::call_site());
            let args: Vec<quote::Tokens> = f.arguments
                .iter()
                .map(|arg| {
                    let field_name =
                        Term::new(arg.name.to_mixed_case().as_str(), Span::call_site());
                    let field_type = gql_type_to_json_type(&arg.value_type);
                    quote!( #field_name: #field_type )
                })
                .collect();
            let field_type_name = extract_inner_name(&f.field_type);
            let sub_field_set: Option<Term> = if context.is_scalar(field_type_name) {
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
        .collect();
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

fn gql_type_to_json_type(gql_type: &graphql_parser::query::Type) -> quote::Tokens {
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
    fn enum_derive() {
        assert_expands_to! {
            r##"
            enum Dog {
                GOLDEN
                CHIHUAHUA
                CORGI
            }
            "## => {
                #[derive(Debug, PartialEq)]
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
                #[derive(Debug, PartialEq)]
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
                #[derive(Debug, PartialEq)]
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

    #[test]
    fn unions() {
        assert_expands_to! {
            r##"
            union SearchResult = Human | Droid | Starship
            "## => {
                #[derive(Debug, PartialEq)]
                pub enum SearchResult {
                    onHuman(Vec<Human>),
                    onDroid(Vec<Droid>),
                    onStarship(Vec<Starship>)
                }
            }
        }
    }
}
