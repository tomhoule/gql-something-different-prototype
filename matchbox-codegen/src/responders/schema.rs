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
        let operations: Vec<_> = &[self.query, self.mutation, self.subscription]
            .iter()
            .filter_map(|op| op)
            .collect();
        let operation_responder_names: Vec<Term> = operations
            .iter()
            .map(|op| {
                Term::new(
                    &::shared::schema_name_to_responder_name(&op.name),
                    Span::call_site(),
                )
            })
            .collect();
        let operation_type_names = operations
            .iter()
            .map(|op| Term::new(&op.name, Span::call_site()));
        // let query_responder = optional_ty_to_responder_name(&self.query);
        // let mutation_responder = optional_ty_to_responder_name(&self.mutation);
        // let subscription_responder = optional_ty_to_responder_name(&self.subscription);

        // let responder_names = &[query_responder, mutation_responder, subscription_responder];

        quote!{
            #(
                pub struct #operation_responder_names {
                    fn to<Resolver>(
                        selection: Vec<#operation_type_names>,
                        resolver: Resolver
                    ) -> impl Future<Item = ::tokio_gql::response::Response, Error = ::tokio_gql::errors::ResolverError>
                    where
                        Resolver: FnOnce(#operation_type_names) -> ResolverFuture,
                        ResolverFuture: Future<Item = ::tokio_gql::response::Response, Error = ::tokio_gql::errors::ResolverError> {
                        let mut result = ::serde_json::Map::new();
                        let mut async_fields: Vec<_> = Vec::new();

                        for field in selection.into_iter() {
                            match field {
                                ::tokio_gql::response::Response::Immediate(kv) => {
                                    result.extend(kv);
                                }
                                ::tokio_gql::response::Response::Async(fut) => {
                                    async_fields.push(fut)
                                }
                            }
                        }

                        ::futures::future::join_all(async_fields).and_then(move |resolved| {
                            result.extend(resolved);
                            Ok(tokio_gql::response::Response::Immediate(result))
                        })
                    }
                }
            )*
        }
    }
}
