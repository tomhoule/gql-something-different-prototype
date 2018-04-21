use coercion::arguments::ArgumentsContext;
use coercion::traits::*;
use context::DeriveContext;
use graphql_parser::schema::*;
use proc_macro2::{Span, Term};
use quote;

impl ImplCoerce for ObjectType {
    fn impl_coerce(&self, context: &DeriveContext) -> quote::Tokens {
        let name = Term::new(&self.name, Span::call_site());
        let field_matchers = ArgumentsContext {
            fields: self.fields.iter().map(|i| i.clone().into()).collect(),
            object_name: Term::new(&self.name, Span::call_site()),
        }.impl_coerce(context);

        quote! {
            impl ::tokio_gql::coercion::CoerceSelection for #name {
                fn coerce(
                    query: &::tokio_gql::graphql_parser::query::SelectionSet,
                    context: &::tokio_gql::query_validation::ValidationContext,
                ) -> Result<Vec<#name>, ::tokio_gql::coercion::CoercionError> {
                    let mut result: Vec<#name> = Vec::new();

                    for item in query.items.iter() {
                        match item {
                            ::tokio_gql::graphql_parser::query::Selection::Field(ref field) => {
                                #field_matchers
                            }
                            ::tokio_gql::graphql_parser::query::Selection::FragmentSpread(_) => unimplemented!("fragment spreads are unimplemented"),
                            ::tokio_gql::graphql_parser::query::Selection::InlineFragment(_) => unimplemented!("inline fragments are unimplemented"),

                        }
                    }

                    Ok(result)
                }
            }
        }
    }
}
