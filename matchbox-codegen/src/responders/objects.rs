use super::traits::ImplResponder;
use context::DeriveContext;
use graphql_parser::schema;
use heck::*;
use proc_macro2::{Span, Term};
use quote;

impl ImplResponder for schema::ObjectType {
    fn impl_responder(&self, context: &DeriveContext) -> quote::Tokens {
        let responder_name = Term::new(
            &::shared::schema_name_to_responder_name(&self.name),
            Span::call_site(),
        );
        let variant_name = Term::new(&self.name.to_camel_case(), Span::call_site());
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
        // let field_names: Vec<_> = self.fields
        //     .iter()
        //     .map(|field| Term::new(&field.name, Span::call_site()))
        //     .collect();
        // let field_types = self.fields
        //     .iter()
        //     .map(|field| ::shared::graphql_type_to_response_type(&field.field_type, context));

        // here:
        // first implement the responder for the thing itself
        // then delegate for each field
        // -> there should be an impl_responder for Field
        quote! {
            #[derive(Debug, PartialEq)]
            pub struct #responder_name;
            trivial_default_impl!(#responder_name, #responder_name);

            impl #responder_name {
                pub fn to<LoaderFuture, Loader, Data, Resolver>(
                    &self,
                    selection: Vec<#variant_name>,
                    loader: Loader,
                    resolver: Resolver,
                ) -> Box<::futures::Future<Item = ::serde_json::Value, Error = ::tokio_gql::errors::ResolverError>
                where
                    Loader: Fn(&[#variant_name]) -> LoaderFuture,
                    LoaderFuture: ::futures::Future<Item = Data, Error = ::tokio_gql::errors::ResolverError>,
                    Resolver: Fn(#variant_name, &Data) -> ::tokio_gql::response::Response {

                    use ::futures::prelude::Future;

                    loader(&selection).and_then(move |data| {
                        let mut result = ::serde_json::Map::with_capacity(selection.len());
                        let mut async_fields: Vec<_> = Vec::new();

                        for field in selection.into_iter() {
                            let response = resolver(&data, field);
                            match response {
                                ::tokio_gql::response::Response::Immediate(kv) => {
                                    result.insert(kv.0.to_string(), kv.1);
                                }
                                ::tokio_gql::response::Response::Async(fut) => {
                                    async_fields.push(fut)
                                }
                            }
                        }

                        Box::new(
                            ::futures::future::join_all(async_fields)
                                .map(move |r| (result, r))
                                .and_then(move |(mut result, resolved)| {
                                    result.extend(resolved.into_iter().map(|(k, v)| (k.to_string(), v)));
                                    Ok(::serde_json::Value::Object(result))
                                })
                        )
                    })
                }
            }

            #(#field_impls)*
        }
    }
}
