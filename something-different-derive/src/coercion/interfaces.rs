use coercion::arguments::ArgumentsContext;
use coercion::traits::*;
use context::DeriveContext;
use graphql_parser::schema::*;
use proc_macro2::{Span, Term};
use quote;

impl ImplCoerce for InterfaceType {
    fn impl_coerce(&self, context: &DeriveContext) -> quote::Tokens {
        let name_term: Term = Term::new(&self.name, Span::call_site());

        let field_matchers = ArgumentsContext {
            fields: self.fields.iter().map(|f| f.clone().into()).collect(),
            object_name: name_term.clone(),
        }.impl_coerce(&context);

        quote! {
            impl ::tokio_gql::coercion::CoerceSelection for #name_term {
                fn coerce(
                    query: &::tokio_gql::graphql_parser::query::SelectionSet,
                    context: &::tokio_gql::query_validation::ValidationContext,
                ) -> Result<Vec<#name_term>, ::tokio_gql::coercion::CoercionError> {
                    for selection in query.items.iter() {
                        match selection {
                            ::tokio_gql::graphql_parser::query::Selection::Field(field) => {
                                let mut result = Vec::new();
                                #field_matchers
                                return Ok(result)
                            },
                            ::tokio_gql::graphql_parser::query::Selection::FragmentSpread(_) => unimplemented!("fragment spread on interface"),
                            ::tokio_gql::graphql_parser::query::Selection::InlineFragment(_) => unimplemented!("inline fragment on interface"),

                        }
                    }
                    Ok(Vec::new())
                }
            }
        }
    }
}
