use super::traits::ImplResponder;
use context::DeriveContext;
use graphql_parser::schema;
use heck::CamelCase;
use proc_macro2::{Span, Term};
use quote;

impl ImplResponder for schema::InterfaceType {
    fn impl_responder(&self, context: &DeriveContext) -> quote::Tokens {
        let responder_name = Term::new(
            &::shared::schema_name_to_responder_name(&self.name),
            Span::call_site(),
        );
        let variant_name = Term::new(&self.name, Span::call_site());
        let name = &self.name;

        let field_impls = self.fields.iter().map(|field| {
            let field_responder_name = format!("{}{}Responder", name, field.name.to_camel_case());
            super::fields::impl_field(
                &field_responder_name,
                &field.name,
                &field.field_type,
                context,
            )
        });

        quote! {
            #[derive(Debug, PartialEq)]
            pub struct #responder_name;
            trivial_default_impl!(#responder_name, #responder_name);

            impl #responder_name {
                pub fn to<Resolver>(
                    selection: Vec<#variant_name>,
                    resolver: Resolver,
                ) -> impl ::futures::future::Future<Item = ::tokio_gql::response::Response, Error = ::tokio_gql::errors::ResolverError>
                where
                    Resolver: Fn(#variant_name) -> Result<::tokio_gql::response::Response, ::tokio_gql::errors::ResolverError> {

                    use futures::prelude::Future;

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
                        Ok(::tokio_gql::response::Response::Immediate((#name, ::serde_json::Value::Object(result))))
                    })
                }
            }

            #(#field_impls)*
        }
    }
}
