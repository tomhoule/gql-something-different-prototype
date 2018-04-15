use coercion::CoercionError;
use graphql_parser::query::ParseError;
use std::convert::From;

#[derive(Debug, Fail)]
pub enum GqlError<ResolverError> {
    #[fail(display = "Invalid request")]
    InvalidRequest,
    #[fail(display = "Invalid query")]
    InvalidQuery,
    #[fail(display = "Resolver error")]
    ResolverError(ResolverError),
    #[fail(display = "Invalid error")]
    InternalError,
}

impl<ResolverError> From<CoercionError> for GqlError<ResolverError> {
    fn from(_err: CoercionError) -> Self {
        GqlError::InternalError
    }
}

impl<ResolverError> From<ParseError> for GqlError<ResolverError> {
    fn from(_err: ParseError) -> Self {
        GqlError::InvalidQuery
    }
}
