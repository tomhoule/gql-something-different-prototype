use errors::ResolverError;
use futures::prelude::*;
use serde_json as json;

pub type ResolverFuture = Box<Future<Item = json::Value, Error = Vec<ResolverError>>>;

pub trait Resolver {
    type Schema;

    fn resolve(&self, request: Self::Schema) -> ResolverFuture;
}
