#[macro_use]
extern crate tokio_gql;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use serde_json as json;
use tokio_gql::resolver::*;

mod star_wars {
    #[allow(dead_code)]
    #[derive(SomethingCompletelyDifferent)]
    #[SomethingCompletelyDifferent(path = "tests/star_wars_schema.graphql")]
    struct ComplexSchema;
}

struct StarWarsResolver;

impl Resolver for StarWarsResolver {
    type Schema = star_wars::Schema;

    fn resolve(&self, request: Self::Schema) -> ResolverFuture {
        unimplemented!();
    }
}

fn test_response(req: &star_wars::Schema, expected_response: json::Value) {}

#[test]
fn basic_sync_field() {
    unimplemented!();
}
