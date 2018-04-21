use coercion::CoercionError;
use graphql_parser::query::ParseError;
use serde_json as json;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::convert::From;

#[derive(Debug)]
pub struct ResolverError {
    message: String,
    path: VecDeque<String>,
}

#[derive(Debug, Fail)]
pub enum GqlError {
    #[fail(display = "Invalid request")]
    InvalidRequest,
    #[fail(display = "Invalid query")]
    InvalidQuery,
    #[fail(display = "Resolver error")]
    ResolverError(ResolverError),
    #[fail(display = "Invalid error")]
    InternalError,
}

struct Pos {
    line: u16,
    column: u16,
}

/// The errors as returned in the response.
///
/// This is part of the [official spec](https://github.com/facebook/graphql/blob/master/spec/Section%207%20--%20Response.md).
struct ResponseError {
    message: String,
    locations: Option<Vec<Pos>>,
    path: Option<Vec<String>>,
    extensions: Option<HashMap<String, json::Value>>,
}

impl From<CoercionError> for GqlError {
    fn from(_err: CoercionError) -> Self {
        GqlError::InternalError
    }
}

impl From<ParseError> for GqlError {
    fn from(_err: ParseError) -> Self {
        GqlError::InvalidQuery
    }
}
