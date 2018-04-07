#[macro_use]
extern crate tokio_gql;

#[derive(SomethingCompletelyDifferent)]
#[SomethingCompletelyDifferent(path = "src/local_schema.graphql")]
struct MySchema;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
