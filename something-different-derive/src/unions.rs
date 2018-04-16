use context::DeriveContext;
use graphql_parser::schema::UnionType;
use proc_macro2::{Span, Term};
use quote;

pub fn gql_union_to_rs(union_type: &UnionType, _context: &DeriveContext) -> quote::Tokens {
    let name = Term::new(union_type.name.as_str(), Span::call_site());
    let united_types = union_type.types.iter().map(|ty| {
        let ident = Term::new(&format!("On{}", ty.as_str()), Span::call_site());
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

    #[test]
    fn unions() {
        assert_expands_to! {
            r##"
            union SearchResult = Human | Droid | Starship
            "## => {
                #[derive(Debug, PartialEq)]
                pub enum SearchResult {
                    OnHuman(Vec<Human>),
                    OnDroid(Vec<Droid>),
                    OnStarship(Vec<Starship>)
                }
            }
        }
    }
}
