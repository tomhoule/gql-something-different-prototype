use graphql_parser::schema::EnumType;
use heck::*;
use proc_macro2::{Literal, Span, Term};
use quote;

pub fn gql_enum_to_rs(enum_type: &EnumType) -> quote::Tokens {
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
        #[derive(Debug, PartialEq, Deserialize)]
        #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
        pub enum #name {
            #(#values),* ,
        }
    }
}

#[cfg(test)]
mod tests {
    use context::DeriveContext;

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
                #[derive(Debug, PartialEq, Deserialize)]
                #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
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
                #[derive(Debug, PartialEq, Deserialize)]
                #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
                pub enum BreadKind {
                    White,
                    FullGrain,
                }
            }
        }
    }

}
