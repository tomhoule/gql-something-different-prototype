use errors::GqlError;
use futures::prelude::*;
use json;
use response::Response;

pub trait GqlService {
    type Schema;
    type Error;

    fn resolve(
        &self,
        request: Self::Schema,
        response: Response<Self::Error>,
    ) -> Box<Future<Item = json::Value, Error = Self::Error>>;

    fn handle_errors(&self, errors: GqlError<Self::Error>) -> json::Value;
}
