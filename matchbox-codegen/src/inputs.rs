use context::DeriveContext;
use graphql_parser::schema::InputObjectType;
use proc_macro2::{Literal, Span, Term};
use quote;
use shared;

pub fn gql_input_to_rs(input_type: &InputObjectType, _context: &DeriveContext) -> quote::Tokens {
    let name = Term::new(&input_type.name, Span::call_site());
    let values: Vec<Term> = input_type
        .fields
        .iter()
        .map(|v| Term::new(&v.name, Span::call_site()))
        .collect();
    let types: Vec<_> = input_type
        .fields
        .iter()
        .map(|v| shared::gql_type_to_json_type(&v.value_type))
        .collect();
    let doc_attr: quote::Tokens = if let Some(ref doc_string) = input_type.description {
        let str_literal = Literal::string(&doc_string);
        quote!(#[doc = #str_literal])
    } else {
        quote!()
    };

    quote!{
        #doc_attr
        #[derive(Debug, PartialEq, Deserialize)]
        pub struct #name {
            #(#values: #types),* ,
        }
    }
}

#[cfg(test)]
mod tests {
    use context::DeriveContext;

    #[test]
    fn simple_input_object_derive() {
        assert_expands_to! {
            r##"
            """
            A point in 2, 3 or 4 dimensions, because why not?
            """
            input Point {
                x: Int!
                y: Int!
                z: Int
                zZ: Boolean!
            }
            "## => {
                #[doc = "A point in 2, 3 or 4 dimensions, because why not?\n"]
                #[derive(Debug, PartialEq, Deserialize)]
                pub struct Point {
                    x: i32,
                    y: i32,
                    z: Option<i32>,
                    zZ: bool,
                }
            }
        }
    }
}
