extern crate included_library;

fn main() {
    println!("included schema: {}", included_library::THE_SCHEMA);
}

#[cfg(test)]
mod tests {
    use included_library::THE_SCHEMA;
    use std::io::prelude::*;

    #[test]
    fn it_includes_the_right_schema() {
        let mut schema =
            ::std::fs::File::open("./included_library/src/local_schema.graphql").unwrap();
        let mut out = String::new();
        schema.read_to_string(&mut out).unwrap();
        assert_eq!(THE_SCHEMA, out);
    }
}
