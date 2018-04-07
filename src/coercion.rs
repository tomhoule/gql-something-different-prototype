///! This module contains the traits that are auto-implemented on the derived tree to extract it from a parsed request.
use graphql_parser::query::*;

/// This should be implemented by the schema. It coerces a Schema struct from a query root, recursively coercing fields.
trait CoerceQuery {
    fn coerce(query: Document) -> Self;
}

/// Coerces a selection into the corresponding object, interface or union type
trait CoerceSelection: Sized {
    fn coerce(query: SelectionSet) -> Vec<Self>;
}
