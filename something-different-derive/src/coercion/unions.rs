use coercion::traits::*;
use context::DeriveContext;
use graphql_parser::schema::*;
use proc_macro2::{Span, Term};
use quote;
use shared;

impl ImplCoerce for UnionType {
    fn impl_coerce(&self, context: &DeriveContext) -> quote::Tokens {
        unimplemented!()
    }
}
