use super::traits::ImplResponder;
use context::DeriveContext;
use graphql_parser::schema;
use proc_macro2::{Span, Term};
use quote;

fn optional_ty_to_responder_name(ty: &Option<String>) -> Term {
    Term::new(
        &match ty {
            Some(ref ty) => ::shared::schema_name_to_responder_name(&ty),
            None => "()".to_string(),
        },
        Span::call_site(),
    )
}

impl ImplResponder for schema::SchemaDefinition {
    fn impl_responder(&self, _context: &DeriveContext) -> quote::Tokens {
        let responder_name = Term::new(
            &::shared::schema_name_to_responder_name("Operation"),
            Span::call_site(),
        );
        let operations: Vec<String> = vec![&self.query, &self.mutation, &self.subscription]
            .into_iter()
            .filter_map(|op| op.clone())
            .collect();
        let operation_literals = operations.clone();
        let operation_responder_names: Vec<Term> = operations
            .iter()
            .map(|op| {
                Term::new(
                    &::shared::schema_name_to_responder_name(&op),
                    Span::call_site(),
                )
            })
            .collect();
        let operation_responder_names_2 = operation_responder_names.clone();
        let operation_type_names = operations
            .iter()
            .map(|op| Term::new(&op, Span::call_site()));
        let operation_type_names_2 = operation_type_names.clone();
        // let query_responder = optional_ty_to_responder_name(&self.query);
        // let mutation_responder = optional_ty_to_responder_name(&self.mutation);
        // let subscription_responder = optional_ty_to_responder_name(&self.subscription);

        // let responder_names = &[query_responder, mutation_responder, subscription_responder];

        quote!{
            #(
                pub struct #operation_responder_names;

                impl #operation_responder_names_2 {
                    fn to<Resolver, ResolverFuture>(
                        selection: Vec<#operation_type_names>,
                        resolver: Resolver
                    ) -> impl ::futures::future::Future<Item = ::tokio_gql::response::Response, Error = ::tokio_gql::errors::ResolverError>
                    where
                        Resolver: Fn(#operation_type_names_2) -> Result<::tokio_gql::response::Response, ::tokio_gql::errors::ResolverError> {
                        use futures::prelude::*;
                        // ResolverFuture: ::futures::future::Future<Item = ::tokio_gql::response::Response, Error = ::tokio_gql::errors::ResolverError> {
                        let mut result = ::serde_json::Map::with_capacity(selection.len());
                        let mut async_fields: Vec<_> = Vec::new();

                        for field in selection.into_iter() {
                            let response = resolver(field);
                            match response.unwrap() {
                                ::tokio_gql::response::Response::Immediate(kv) => {
                                    result.insert(kv.0.to_string(), kv.1);
                                }
                                ::tokio_gql::response::Response::Async(fut) => {
                                    async_fields.push(fut)
                                }
                            }
                        }

                        ::futures::future::join_all(async_fields).and_then(move |resolved| {
                            result.extend(resolved.into_iter().map(|(k, v)| (k.to_string(), v)));
                            Ok(::tokio_gql::response::Response::Immediate((#operation_literals, ::serde_json::Value::Object(result))))
                        })
                    }
                }
            )*
        }
    }
}
