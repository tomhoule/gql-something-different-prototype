#[macro_use]
extern crate tokio_gql;

#[derive(SomethingCompletelyDifferent)]
#[SomethingCompletelyDifferent(path = "src/local_schema.graphql")]
struct MySchema;