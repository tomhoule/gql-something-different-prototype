use coercion::traits::*;
use context::DeriveContext;
use graphql_parser::schema::*;
use proc_macro2::{Span, Term};
use quote;
use shared;

impl ImplCoerce for EnumType {
    fn impl_coerce(&self, _context: &DeriveContext) -> quote::Tokens {
        let name_term = Term::new(&self.name, Span::call_site());

        quote! {
            impl ::tokio_gql::coercion::CoerceScalar for #name_term {
                fn coerce(
                    query: &::tokio_gql::graphql_parser::query::Value,
                ) -> Result<#name_term, ::tokio_gql::coercion::CoercionError> {
                    unimplemented!("enum coercion");
                }
            }
        }
    }
}
