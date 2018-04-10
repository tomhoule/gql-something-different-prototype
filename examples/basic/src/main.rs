#[macro_use]
extern crate tokio_gql;

#[derive(SomethingCompletelyDifferent)]
#[SomethingCompletelyDifferent(path = "src/simple_schema.gql")]
struct MySchema;

fn main() {
    println!("Schema:\n\n {}", THE_SCHEMA);
}
