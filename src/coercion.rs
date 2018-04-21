///! This module contains the traits that are auto-implemented on the derived tree to extract it from a parsed request.
use graphql_parser::query::*;
use graphql_parser::schema::Value;
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
/// TODO: Figure out if we still need that. Probably not \o/
pub trait CoerceResponse {
    fn coerce(query: &Document, response: ::serde_json::Value) -> ::serde_json::Value;
}

pub trait CoerceScalar: Sized {
    fn coerce(value: &Value) -> Result<Self, CoercionError>;
}

impl CoerceScalar for String {
    fn coerce(value: &Value) -> Result<String, CoercionError> {
        match value {
            Value::String(ref s) => Ok(s.to_string()),
            _ => Err(CoercionError),
        }
    }
}

impl CoerceScalar for i32 {
    fn coerce(value: &Value) -> Result<i32, CoercionError> {
        match value {
            Value::Int(i) => Ok(i.as_i64().unwrap() as i32),
            _ => Err(CoercionError),
        }
    }
}

impl CoerceScalar for bool {
    fn coerce(value: &Value) -> Result<bool, CoercionError> {
        match value {
            Value::Boolean(b) => Ok(*b),
            _ => Err(CoercionError),
        }
    }
}

impl<T> CoerceScalar for Option<T>
where
    T: CoerceScalar,
{
    fn coerce(value: &Value) -> Result<Option<T>, CoercionError> {
        Ok(T::coerce(value).ok())
    }
}

impl<T> CoerceScalar for Vec<T>
where
    T: CoerceScalar,
{
    fn coerce(value: &Value) -> Result<Vec<T>, CoercionError> {
        match value {
            Value::List(elems) => elems.iter().map(T::coerce).collect(),
            _ => Err(CoercionError),
        }
    }
}
