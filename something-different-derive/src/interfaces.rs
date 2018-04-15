use context::DeriveContext;
use graphql_parser;
use heck::*;
use proc_macro2::{Literal, Span, Term};
use quote;
use shared;

pub fn gql_interface_to_rs(
    interface_type: &graphql_parser::schema::InterfaceType,
    context: &DeriveContext,
) -> quote::Tokens {
    let name = Term::new(interface_type.name.as_str(), Span::call_site());
    let doc_attr: quote::Tokens = if let Some(ref doc_string) = interface_type.description {
        let str_literal = Literal::string(doc_string.as_str());
        quote!(#[doc = #str_literal])
    } else {
        quote!()
    };

    quote!(
        #doc_attr
        #[derive(Debug, PartialEq)]
        pub enum #name {
        }
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interfaces_are_implemented() {
        unimplemented!()
    }
}
