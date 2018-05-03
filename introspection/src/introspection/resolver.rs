use futures::prelude::*;
use introspection::schema;
use serde_json as json;
use tokio_gql::introspection::Schema;
use tokio_gql::resolver::Resolver;
use tokio_gql::response::Response;

pub struct IntrospectionResolver {
    schema_root: Schema,
}

impl IntrospectionResolver {
    pub fn new(schema_root: Schema) -> IntrospectionResolver {
        IntrospectionResolver { schema_root }
    }
}

#[derive(Debug, PartialEq)]
pub struct Error;

impl Resolver for IntrospectionResolver {
    type Schema = schema::Operation;

    fn resolve(
        &self,
        request: Self::Schema,
    ) -> Box<Future<Item = json::Value, Error = Vec<::tokio_gql::errors::ResolverError>>> {
        unimplemented!();
    }
}
