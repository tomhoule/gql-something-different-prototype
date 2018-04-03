extern crate graphql_parser;

fn main() {
    let schema = include_str!("./simple_schema.gql");
    let parsed_schema = graphql_parser::parse_schema(schema);
    println!("parsed: {:?}", parsed_schema);
}

trait Query {
    fn resolve_pizzas() -> Future<Vec<Pizza>, MyError>;
}
