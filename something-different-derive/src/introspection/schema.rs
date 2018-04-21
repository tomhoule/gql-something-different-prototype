use super::traits::Introspectable;
use graphql_parser::schema;
use quote;

impl Introspectable for schema::SchemaDefinition {
    fn introspect(&self) -> quote::Tokens {
        unimplemented!();
    }
}
