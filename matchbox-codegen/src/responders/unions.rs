use super::traits::ImplResponder;
use context::DeriveContext;
use graphql_parser::schema;
use proc_macro2::{Span, Term};
use quote;

impl ImplResponder for schema::UnionType {
    fn impl_responder(&self, context: &DeriveContext) -> quote::Tokens {
        let responder_name = Term::new(
            &::shared::schema_name_to_responder_name(&self.name),
            Span::call_site(),
        );

        quote! {
            #[derive(Debug, PartialEq)]
            pub struct #responder_name;
            trivial_default_impl!(#responder_name, #responder_name);
        }
    }
}
