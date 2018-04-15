use context::DeriveContext;
use graphql_parser;
use objects::get_field_names;
use proc_macro2::{Literal, Span, Term};
use quote;

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

    let field_names = get_field_names(&interface_type.fields, context);

    quote!(
        #doc_attr
        #[derive(Debug, PartialEq)]
        pub enum #name {
            #(#field_names),*
        }
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interfaces_are_implemented() {
        assert_expands_to! {
        r##"
        interface Character {
          id: ID!
          name: String!
          withWittyComment(meh: Boolean): String
          friends: [Character]
          appearsIn: [Episode]!
        }
        "## => {
            #[derive(Debug, PartialEq)]
            pub enum Character {
                Id,
                Name,
                WithWittyComment { meh: Option<bool> },
                Friends { selection: Vec<Character>, },
                AppearsIn { selection: Vec<Episode>, }
            }
        }
        }
    }
}
