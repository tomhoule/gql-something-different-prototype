use futures::prelude::*;
use response::Response;
use serde_json as json;

pub trait GqlService {
    type Schema;
    type Error;

    fn resolve(
        &self,
        request: Self::Schema,
        response: Response,
    ) -> Box<Future<Item = json::Value, Error = Self::Error>>;
}
