use context::DeriveContext;
use graphql_parser::schema::InputObjectType;
use heck::CamelCase;
use proc_macro2::{Literal, Span, Term};
use quote;

pub fn gql_input_to_rs(input_type: &InputObjectType, _context: &DeriveContext) -> quote::Tokens {
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
}
