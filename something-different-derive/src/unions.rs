use context::DeriveContext;
use graphql_parser::schema::UnionType;
use proc_macro2::{Span, Term};
use quote;

pub fn gql_union_to_rs(union_type: &UnionType, _context: &DeriveContext) -> quote::Tokens {
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

#[cfg(test)]
mod tests {
    use super::*;
    use graphql_parser::schema::*;

    /// This is repeated between test modules, we may have to create a test_support crate to overcome that limitation.
    macro_rules! assert_expands_to {
        ($gql_string:expr => $expanded:tt) => {
            let gql = $gql_string;
            let parsed = parse_schema(gql).unwrap();
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
