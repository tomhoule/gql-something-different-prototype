///! This module contains the traits that are auto-implemented on the derived tree to extract it from a parsed request.
use graphql_parser::query::*;
use query_validation::ValidationContext;

#[derive(Debug, PartialEq)]
pub struct CoercionError;

/// This should be implemented by the schema. It coerces a Schema struct from a query root, recursively coercing fields.
pub trait CoerceQueryDocument: Sized {
    fn coerce(query: &Document, context: &ValidationContext) -> Result<Self, CoercionError>;
}

/// Coerces a selection into the corresponding object, interface or union type
pub trait CoerceSelection: Sized {
    fn coerce(
        query: &SelectionSet,
        context: &ValidationContext,
    ) -> Result<Vec<Self>, CoercionError>;
}

/// Coerces a response to match the query type.
pub trait CoerceResponse {
    fn coerce(query: &Document, response: ::json::Value) -> ::json::Value;
}
