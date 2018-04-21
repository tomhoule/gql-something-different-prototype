use futures::prelude::*;
use introspection::schema;
use serde_json as json;
use tokio_gql::introspection::Schema;
use tokio_gql::response::Response;
use tokio_gql::service::GqlService;

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

impl GqlService for IntrospectionResolver {
    type Schema = schema::Schema;
    type Error = Error;

    fn resolve(
        &self,
        request: Self::Schema,
        response: Response<Self::Error>,
    ) -> Box<Future<Item = json::Value, Error = Self::Error>> {
        unimplemented!();
    }

    fn handle_errors(&self, errors: ::tokio_gql::errors::GqlError<Self::Error>) -> json::Value {
        unimplemented!();
    }
}
