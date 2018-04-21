#![recursion_limit = "256"]

extern crate matchbox_codegen;

extern crate proc_macro;
extern crate proc_macro2;
extern crate quote;
extern crate syn;

use proc_macro2::{Span, Term};
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

    matchbox_codegen::expand_schema(&the_schema_string)
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
