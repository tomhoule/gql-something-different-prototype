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
    let name = &ast.ident;
    let schema_path = extract_path(&ast.attrs).expect("path not specified");
    // panic!(
    //     "schema_path {:?} (cwd: {:?}",
    //     &schema_path,
    //     &std::env::current_dir()
    // );
    let mut file = File::open(schema_path).expect("File not found");
    let mut the_schema_string = String::new();
    file.read_to_string(&mut the_schema_string).unwrap();
    let parsed_schema = graphql_parser::parse_schema(&the_schema_string).expect("parse error");
    let the_schema = syn::Lit::from(the_schema_string);
    let query = extract_query(&parsed_schema).expect("Could not find Query type");
    let query_field_names: Vec<quote::Tokens> = query
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
                        let field_type: syn::Ident = "SomeType".into();
                        quote!( #field_name: #field_type )
                    })
                    .collect();
                quote!{#ident { #(#args),* }}
            }
        })
        .collect();

    println!("field names {:?}", query_field_names);
    quote! {
        const THE_SCHEMA: &'static str = #the_schema;

        enum QueryField {
            #(#query_field_names),*
        }

        struct Query {
            selected_fields: Vec<QueryField>,
        }
    }
}

fn extract_path(attributes: &[syn::Attribute]) -> Option<String> {
    let outer_ident: syn::Ident = "SomethingCompletelyDifferent".into();
    let path_ident: syn::Ident = "path".into();
    for attr in attributes.iter() {
        if let syn::MetaItem::List(ident, items) = &attr.value {
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

fn extract_query(
    document: &graphql_parser::schema::Document,
) -> Option<&graphql_parser::schema::ObjectType> {
    use graphql_parser::schema::*;

    for definition in document.definitions.iter() {
        if let Definition::TypeDefinition(TypeDefinition::Object(obj)) = definition {
            if obj.name == "Query" {
                return Some(obj);
            }
        }
    }
    None
}

fn gql_type_to_json_type(gql_type: graphql_parser::query::Type) -> syn::Ident {
    use graphql_parser::query::Type::*;

    match gql_type {
        NamedType(name) => name.into(),
    }
}
