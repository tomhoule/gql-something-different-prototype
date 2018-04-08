#[macro_use]
extern crate failure;
extern crate futures;
extern crate serde;
extern crate serde_json as json;

pub extern crate graphql_parser;

pub mod coercion;
pub mod identifiable;
pub mod query_validation;

#[macro_use]
extern crate something_different_derive;
#[doc(hidden)]
pub use something_different_derive::*;

use futures::prelude::*;

use serde::Serialize;

// Find the Query type
// For each field, generate either a prefixed type or a module (maybe more a module?)
//

use graphql_parser::query;
use identifiable::Identifiable;

// Take an arbitrary error type as input to `refine_schema`

enum ResponseNodeValue<Error> {
    Immediate(json::Value),
    Delayed(Box<Future<Item = json::Value, Error = Error>>),
}

struct DataLoader<Identifier, Output, Error> {
    ids: Vec<Identifier>,
    _output: ::std::marker::PhantomData<Output>,
    _error: ::std::marker::PhantomData<Error>,
    resolve: Fn(Vec<Identifier>) -> Box<Future<Item = Vec<Output>, Error = Error>>,
}

pub struct ResponseNode<Error> {
    value: ResponseNodeValue<Error>,
    prefix: Vec<String>,
}

pub struct ResponseBuilder<Error> {
    tree: ResponseNode<Error>,
}

impl<Error> ResponseBuilder<Error> {
    pub fn new() -> ResponseBuilder<Error> {
        ResponseBuilder {
            tree: ResponseNode {
                prefix: Vec::new(),
                value: ResponseNodeValue::Immediate(json::Value::Null),
            },
        }
    }
}

impl<Error> ResponseNode<Error> {
    /// Sets the current scope value to the `value` argument.
    pub fn set<S: Serialize>(&mut self, value: S) {
        self.value = ResponseNodeValue::Immediate(
            json::to_value(value).expect("value can be converted to JSON"),
        );
    }

    /// Sets `key` to `value` in the current scope. If the current scope is not an object. it becomes one with only that key-value.
    pub fn set_kv<S: Serialize>(key: &str, value: S) {
        // match self.value {
        //   ResponseNodeValue::Immediate(json_value) => {
        //      match json_value {
        //        json::Value::Object(obj) => {
        //           ...insert
        //        }
        //        _ => { self.value = ResponseNodeValue::Immediate(json!({ [key]: value })),
        //
        //      }
        //   }
        //   ResponseNodeValue::Deferred(deferred_value) => { unimplemented!() }
        // }
        //
        unimplemented!();
    }

    /// Registers
    pub fn set_deferred<'a, Resource: Identifiable>(ids: impl AsRef<&'a [&'a str]>) {
        unimplemented!();
    }
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
