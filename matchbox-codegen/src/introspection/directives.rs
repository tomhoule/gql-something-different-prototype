use super::traits::Introspectable;
use graphql_parser::schema;
use quote;

impl Introspectable for schema::DirectiveDefinition {
    fn introspect(&self) -> quote::Tokens {
        unimplemented!();
    }
}
