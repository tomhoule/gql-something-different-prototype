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

    let field_names = get_field_names(&interface_type.fields, context, &name);
    let implementor_names: Vec<_> = context
        .object_types
        .iter()
        .filter_map(|obj| {
            if obj.implements_interfaces
                .iter()
                .any(|iface| iface.as_str() == interface_type.name.as_str())
            {
                Some(&obj.name)
            } else {
                None
            }
        })
        .collect();
    let implementor_variants = implementor_names
        .iter()
        .map(|name| Term::new(&format!("On{}", name), Span::call_site()));
    let implementor_extractors = implementor_names.iter().map(|name| {
        let name_term = Term::new(name, Span::call_site());
        quote! { Vec<#name_term> }
    });

    quote!(
        #doc_attr
        #[derive(Debug, PartialEq)]
        pub enum #name {
            #(#field_names,)*
            #(#implementor_variants(#implementor_extractors),)*
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
                Id { respond: CharacterIdResponder, },
                Name { respond: CharacterNameResponder, },
                WithWittyComment { respond: CharacterWithWittyCommentResponder, meh: Option<bool>, },
                Friends { respond: CharacterFriendsResponder, selection: Vec<Character>, },
                AppearsIn { respond: CharacterAppearsInResponder, selection: Vec<Episode>, },
            }
        }
        }
    }

    #[test]
    fn interfaces_with_implementor() {
        assert_expands_to! {
        r##"
        type Wookie implements Character {
            id: ID!
            name: String!
            hairiness: Int!
        }

        interface Character {
          id: ID!
          name: String!
        }
        "## => {
            #[derive(Debug, PartialEq)]
            pub enum Wookie {
                Id { respond: WookieIdResponder, },
                Name { respond: WookieNameResponder, },
                Hairiness { respond: WookieHairinessResponder, },
            }

            #[derive(Debug, PartialEq)]
            pub enum Character {
                Id { respond: CharacterIdResponder, },
                Name { respond: CharacterNameResponder, },
                OnWookie(Vec<Wookie>),
            }
        }
        }
    }

}
