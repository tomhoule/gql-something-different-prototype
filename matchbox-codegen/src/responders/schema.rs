use super::traits::ImplResponder;
use context::DeriveContext;
use graphql_parser::schema;
use proc_macro2::{Span, Term};
use quote;

fn optional_ty_to_response(ty: &Option<String>) -> Term {
    Term::new(
        &match ty {
            Some(ref ty) => ::shared::schema_name_to_response_name(&ty),
            None => "()".to_string(),
        },
        Span::call_site(),
    )
}

impl ImplResponder for schema::SchemaDefinition {
    fn impl_responder(&self, _context: &DeriveContext) -> quote::Tokens {
        let response_name = Term::new(
            &::shared::schema_name_to_response_name("Operation"),
            Span::call_site(),
        );
        let query_response = optional_ty_to_response(&self.query);
        let mutation_response = optional_ty_to_response(&self.mutation);
        let subscription_response = optional_ty_to_response(&self.subscription);

        quote! {
            #[derive(Serialize)]
            struct #response_name {
                #[serde(serialize_with = "serialize_response_field")]
                query: Option<Option<#query_response>>,
                #[serde(serialize_with = "serialize_response_field")]
                mutation: Option<Option<#mutation_response>>,
                #[serde(serialize_with = "serialize_response_field")]
                subscription: Option<Option<#subscription_response>>,
            }
        }
    }
}
