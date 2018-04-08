#[macro_use]
extern crate tokio_gql;
#[macro_use]
extern crate serde_json;

extern crate graphql_parser;
use graphql_parser::query::*;

use tokio_gql::coercion::*;

#[derive(SomethingCompletelyDifferent)]
#[SomethingCompletelyDifferent(path = "tests/basic_schema.graphql")]
struct BasicSchema;

#[test]
fn query_coercion_works() {
    let query = r##"
    query {
        lastName
        greeting
    }
    "##;
    let context = tokio_gql::query_validation::ValidationContext::new();
    let query = parse_query(query).unwrap();
    let user_fields = Schema::coerce(&query, &context);

    assert_eq!(user_fields, Schema { query: vec![User::LastName, User::Greeting] });
}
