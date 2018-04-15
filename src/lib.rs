#[macro_use]
extern crate failure;
extern crate futures;
#[macro_use]
extern crate matches;
extern crate hyper;
extern crate serde;
#[macro_use]
extern crate serde_json as json;

pub extern crate graphql_parser;

pub mod coercion;
pub mod errors;
pub mod identifiable;
pub mod query_validation;
pub mod response;
pub mod service;
mod shared;
pub mod standalone_server;

#[allow(unused_imports)]
#[macro_use]
extern crate something_different_derive;
#[doc(hidden)]
pub use something_different_derive::*;
