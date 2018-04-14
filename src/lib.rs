#[macro_use]
extern crate failure;
extern crate futures;
#[macro_use]
extern crate matches;
extern crate serde;
#[macro_use]
extern crate serde_json as json;

pub extern crate graphql_parser;

pub mod coercion;
pub mod identifiable;
pub mod query_validation;
pub mod response;
mod shared;

#[macro_use]
extern crate something_different_derive;
#[doc(hidden)]
pub use something_different_derive::*;

use futures::prelude::*;

// Take an arbitrary error type as input to `refine_schema`

struct DataLoader<Identifier, Output, Error> {
    ids: Vec<Identifier>,
    _output: ::std::marker::PhantomData<Output>,
    _error: ::std::marker::PhantomData<Error>,
    resolve: Fn(Vec<Identifier>) -> Box<Future<Item = Vec<Output>, Error = Error>>,
}

//
// object!({
//   title: "meow",
//   age: 33,
//   recipes: some_computation_returning_a_future()
// })

// trait FromQueryField: Sized {
//     /// Tuple of input types, most of the time
//     type Arguments: FromQueryArguments;

//     fn from_query_field(field: query::Field) -> Result<Self, QueryValidationError>;
// }

// trait FromQueryArguments: Sized {
//     fn from_arguments(args: &[(String, query::Value)]) -> Result<Self, QueryValidationError>;
// }

// impl FromQueryArguments for () {
//     fn from_arguments(args: &[(String, query::Value)]) -> Result<Self, QueryValidationError> {
//         Ok(())
//     }
// }

// impl<T1, T2> FromQueryArguments for (T1, T2)
// where
//     T1: FromQueryArguments,
//     T2: FromQueryArguments,
// {
//     fn from_arguments(args: &[(String, query::Value)]) -> Result<Self, QueryValidationError> {
//         let a1 = T1::from_arguments(args)?;
//         let a2 = T2::from_arguments(args)?;
//         Ok((a1, a2))
//     }
// }
