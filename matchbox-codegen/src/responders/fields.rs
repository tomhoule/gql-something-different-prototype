use super::traits::ImplResponder;
use context::DeriveContext;
use graphql_parser::schema;
use heck::*;
use proc_macro2::{Span, Term};
use quote;

impl ImplResponder for schema::Field {
    fn impl_responder(&self, context: &DeriveContext) -> quote::Tokens {
        let responder_name = Term::new(
            &::shared::schema_name_to_responder_name(&self.name),
            Span::call_site(),
        );

        let expanded = field_impl_inner(&responder_name, &self.field_type, context, false);

        quote!{ #expanded }
    }
}

fn field_impl_inner(responder_name: &Term, ty: &schema::Type, context: &DeriveContext, non_nullable: bool) -> quote::Tokens {
    match ty {
        schema::Type::NonNullType(inner) => field_impl_inner(responder_name, inner, context, true),
        schema::Type::ListType(inner) => list_responder_impl(responder_name, inner),
        schema::Type::NamedType(name) => unimplemented!();
    }
}

fn list_responder_impl(responder_name: &Term, inner_type: &schema::Type) -> quote::Tokens {
    unimplemented!();
}
