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
        #[derive(Debug, PartialEq)]
        pub enum #name {
            #(#values),* ,
        }
    }
}

#[cfg(test)]
mod tests {
    use context::DeriveContext;
    use graphql_parser;

    /// This is repeated between test modules, we may have to create a test_support crate to overcome that limitation.
    macro_rules! assert_expands_to {
        ($gql_string:expr => $expanded:tt) => {
            let gql = $gql_string;
            let parsed = graphql_parser::parse_schema(gql).unwrap();
            let mut buf = Vec::new();
            let mut context = DeriveContext::new();
            ::extract_definitions(&parsed, &mut context);
            ::gql_document_to_rs(&mut buf, &context);
            let got = quote!(#(#buf)*);
            let expected = quote! $expanded ;
            assert_eq!(expected, got);
        };
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

}
