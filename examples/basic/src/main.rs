extern crate graphql_parser;

#[macro_use]
extern crate tokio_gql;

#[derive(SomethingCompletelyDifferent)]
#[SomethingCompletelyDifferent(path = "./examples/basic/src/simple_schema.gql")]
struct MySchema;

fn main() {
    println!("o, hi");
    // let schema = include_str!("./simple_schema.gql");
    // let parsed_schema = graphql_parser::parse_schema(schema);
    // println!("parsed: {:?}", parsed_schema);
}

// trait Query {
//     fn resolve_pizzas() -> Future<Vec<Pizza>, MyError>;
// }
