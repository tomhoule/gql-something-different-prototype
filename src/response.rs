use futures::prelude::*;
use identifiable::Identifiable;
use json;
use serde::*;
use std::collections::HashMap;
use std::fmt::Debug;

pub trait PathFragment {
    fn as_path_fragment() -> &'static str;
}

enum ResponseNodeValue<Error> {
    Unset,
    Immediate(json::Value),
    Delayed(Box<Future<Item = json::Value, Error = Error>>),
}

pub struct ResponseNode<Error> {
    value: ResponseNodeValue<Error>,
    prefix: Vec<&'static str>,
}

struct ResponseRoot {}

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
    fn new(prefix: Vec<&'static str>) -> Self {
        ResponseNode {
            value: ResponseNodeValue::Unset,
            prefix,
        }
    }

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

pub struct Response<Error> {
    _err: ::std::marker::PhantomData<Error>,
    data_fields: Vec<Box<Future<Item = json::Value, Error = Error>>>,
}

pub trait Handler<T, Error>
where
    Error: Debug + PartialEq + 'static,
{
    fn handle(&self, arg: T) -> Box<Future<Item = json::Value, Error = Error>>;
}

impl<T, F, R, Error: Debug + PartialEq + 'static> Handler<T, Error> for F
where
    R: IntoFuture<Item = json::Value, Error = Error> + 'static,
    F: Fn(T) -> R,
{
    fn handle(&self, arg: T) -> Box<Future<Item = json::Value, Error = Error>> {
        Box::new(self(arg).into_future())
    }
}

impl<Error: Debug + PartialEq + 'static> Response<Error> {
    pub fn new() -> Response<Error> {
        Response {
            _err: ::std::marker::PhantomData,
            data_fields: Vec::new(),
        }
    }

    pub fn on<T: PathFragment>(&mut self, field: Option<T>, handler: impl Handler<T, Error>) {
        if let Some(field) = field {
            self.data_fields.push(handler.handle(field));
        }
    }

    pub fn resolve(self) -> impl Future<Item = json::Value, Error = Error> {
        ::futures::future::join_all(self.data_fields).and_then(|data_fields| {
            let mut data_map = HashMap::new();

            for field in data_fields.iter() {
                if let json::Value::Object(map) = field {
                    data_map.extend(map.iter())
                }
            }

            Ok(json!({ "data": data_map }))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    enum QueryRoot {
        Cats { name: String },
    }

    impl PathFragment for QueryRoot {
        fn as_path_fragment() -> &'static str {
            "QueryRoot"
        }
    }

    #[derive(Debug, PartialEq)]
    struct Error;

    fn resolve_query_root(q: QueryRoot) -> Result<json::Value, Error> {
        Ok(json!({ "weight": 4 }))
    }

    #[test]
    fn basic_json_object() {
        let cats_request = QueryRoot::Cats {
            name: "Leopold".to_string(),
        };
        let mut response = Response::<Error>::new();

        response.on(Some(cats_request), resolve_query_root);

        assert_eq!(
            ::futures::executor::block_on(response.resolve()).unwrap(),
            json!({ "data": { "weight": 4 } })
        );
    }

    #[test]
    fn basic_future() {
        let cats_request = QueryRoot::Cats {
            name: "Leopold".to_string(),
        };
        let mut response = Response::<Error>::new();

        response.on(Some(cats_request), |_req| {
            ::futures::future::ok(json!({ "weight": 5 } ))
        });

        let resolved = ::futures::executor::block_on(response.resolve()).unwrap();

        assert_eq!(resolved, json!({ "data": { "weight": 5 } }));
    }
}
