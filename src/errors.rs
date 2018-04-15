use coercion::CoercionError;
use graphql_parser::query::ParseError;
use std::convert::From;

#[derive(Debug, Fail)]
pub enum GqlError {
    #[fail(display = "Invalid request")]
    InvalidRequest,
    #[fail(display = "Invalid query")]
    InvalidQuery,
    #[fail(display = "Resolver error")]
    ResolverError,
    #[fail(display = "Invalid error")]
    InternalError,
}

impl From<CoercionError> for GqlError {
    fn from(err: CoercionError) -> GqlError {
        GqlError::InternalError
    }
}

impl From<ParseError> for GqlError {
    fn from(err: ParseError) -> GqlError {
        GqlError::InvalidQuery
    }
}
