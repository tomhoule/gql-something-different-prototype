extern crate futures;
extern crate serde_json as json;

use futures::prelude::*;

use std::collections::HashMap;

// Find the Query type
// For each field, generate either a prefixed type or a module (maybe more a module?)
//

// Take an arbitrary error type as input to `refine_schema`

enum ResponseNodeValue<Error>  {
    Immediate(json::Value),
    Delayed(Box<Future<Item=json::Value, Error=Error>>)
}

struct DataLoader<Identifier, Output, Error> {
    ids: Vec<Identifier>,
    _output: ::std::marker::PhantomData<Output>,
    _error: ::std::marker::PhantomData<Error>,
    resolve: Fn(Vec<Identifier>) -> Box<Future<Item=Vec<Output>, Error=Error>>,
}

struct ResponseNode<Error>
{
    value: ResponseNodeValue<Error>,
    children: HashMap<&'static str, ResponseNodeValue<Error>>,
}

struct ResponseBuilder<Error> {
    tree: ResponseNode<Error>,
}

//
// object!({
//   title: "meow",
//   age: 33,
//   recipes: some_computation_returning_a_future()    
// })