//! The mental model is the following:
//!
//! Very often a graphql object contains fields that are usually fetched together or trivial to get. We can just always set them in the Response and they will be pruned when the actual JSON response is sent. For other, more intricate fields (those that take an argument or require fetching more data), we want to have special handlers (but still go the simple route for the simple fields).
//!
//! This is achieved with the `on` and `merge` methods on `Response` and judicious usage of Rust's pattern matching and iterators.

use futures::prelude::*;
use json;
use std::collections::HashMap;
use std::fmt::Debug;

pub trait PathFragment {
    fn as_path_fragment(&self) -> &'static str;
}

impl<'a, T: PathFragment> PathFragment for &'a T {
    fn as_path_fragment(&self) -> &'static str {
        (*self).as_path_fragment()
    }
}

pub struct Response<Error> {
    async_fields: Vec<ResponseFut<Error>>,
    resolved_fields: HashMap<String, json::Value>,
    path: Vec<&'static str>,
}

type ResponseFut<Error> = Box<Future<Item = json::Value, Error = Error>>;

impl<Error: Debug + PartialEq + 'static> Response<Error> {
    pub fn new() -> Response<Error> {
        Response::create(&[])
    }

    pub fn create(prefix: &[&'static str]) -> Response<Error> {
        Response {
            async_fields: Vec::new(),
            resolved_fields: HashMap::new(),
            path: prefix.to_owned(),
        }
    }

    /// Passing all fields to `on` should be avoided unless all of them are asynchronous, since this allocates (to store the resulting future). Values that are not futures should go through `merge`.
    pub fn on<T: PathFragment>(
        mut self,
        fields: impl IntoIterator<Item = T>,
        handler: impl Fn(&T, Response<Error>) -> ResponseValue<Error>,
    ) -> ResponseValue<Error> {
        for field in fields {
            let path_suffix = field.as_path_fragment();
            let mut full_path = self.path.clone();
            full_path.push(path_suffix);
            let response_value = handler(&field, Response::create(&full_path));

            match response_value {
                ResponseValue::Node(fut) => self.async_fields.push(fut),
                ResponseValue::Skip => (),
            }
        }
        ResponseValue::Node(self.into_future())
    }

    /// Sets the value on `key` at the current path in the response tree.
    pub fn set(mut self, key: &str, value: json::Value) -> Self {
        self.resolved_fields.insert(key.to_string(), value);
        self
    }

    /// Merge a value at the current path in the response tree. The value must be a `serde_json::Value::Object` or it will be ignored.
    pub fn merge(mut self, value: json::Value) -> Self {
        if let json::Value::Object(map) = value {
            self.resolved_fields.extend(map)
        }
        self
    }
}

impl<Error: 'static> IntoFuture for Response<Error> {
    type Item = json::Value;
    type Error = Error;
    type Future = Box<Future<Item = Self::Item, Error = Self::Error>>;

    fn into_future(self) -> Self::Future {
        let Response {
            async_fields,
            mut resolved_fields,
            ..
        } = self;

        Box::new(
            ::futures::future::join_all(async_fields).and_then(move |deferred_fields| {
                for field in deferred_fields.into_iter() {
                    if let json::Value::Object(map) = field {
                        resolved_fields.extend(map)
                    }
                }

                Ok(json::to_value(&resolved_fields).expect("the response map is valid JSON"))
            }),
        )
    }
}

// impl<Error: 'static> From<Response<Error>> for ResponseValue<Error> {
//     fn from(res: Response<Error>) -> ResponseValue<Error> {
//         let key = res.path.get(res.path.len() - 1).unwrap_or(&"data").clone();
//         rest(res.into_future().map(move |value| json!({ *key: value })))
//     }
// }

pub enum ResponseValue<Error> {
    Node(Box<Future<Item = json::Value, Error = Error>>),
    Skip,
}

pub fn leaf<Error, Field: PathFragment>(
    field: Field,
    async_value: impl IntoFuture<Item = json::Value, Error = Error> + 'static,
) -> ResponseValue<Error> {
    let key = field.as_path_fragment();
    let fut = async_value
        .into_future()
        .map(move |value| json!({ key: value }));
    ResponseValue::Node(Box::new(fut))
}

pub fn rest<Error>(
    async_value: impl IntoFuture<Item = json::Value, Error = Error> + 'static,
) -> ResponseValue<Error> {
    ResponseValue::Node(Box::new(async_value.into_future()))
}

pub fn skip<Error>() -> ResponseValue<Error> {
    ResponseValue::Skip
}

impl<Error: 'static> IntoFuture for ResponseValue<Error> {
    type Item = json::Value;
    type Error = Error;
    type Future = Box<Future<Item = Self::Item, Error = Self::Error>>;

    fn into_future(self) -> Self::Future {
        match self {
            ResponseValue::Node(fut) => fut,
            ResponseValue::Skip => Box::new(::futures::future::ok(json!({}))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(dead_code)]
    #[derive(Debug, PartialEq)]
    enum DogSkills {
        CanJump,
        CanPlayDead,
        CanSwim,
        CanSit,
    }

    #[allow(dead_code)]
    #[derive(Debug, PartialEq)]
    enum Dog {
        Name,
        Age,
        Fluffiness,
        FetchBall,
        Skills { selection: Vec<DogSkills> },
    }

    #[derive(Debug, PartialEq)]
    enum SomeField {
        Cats {
            name: String,
        },
        Dogs {
            selection: Vec<Dog>,
            puppies_only: bool,
        },
    }

    impl PathFragment for Dog {
        fn as_path_fragment(&self) -> &'static str {
            match self {
                Dog::Name => "name",
                Dog::Age => "age",
                Dog::FetchBall => "fetch_ball",
                Dog::Fluffiness => "fluffiness",
                Dog::Skills { .. } => "skills",
            }
        }
    }
    impl PathFragment for DogSkills {
        fn as_path_fragment(&self) -> &'static str {
            match self {
                DogSkills::CanJump => "CanJump",
                DogSkills::CanPlayDead => "CanPlayDead",
                DogSkills::CanSwim => "CanSwim",
                DogSkills::CanSit => "CanSit",
            }
        }
    }

    impl PathFragment for SomeField {
        fn as_path_fragment(&self) -> &'static str {
            match self {
                SomeField::Cats { .. } => "cats",
                SomeField::Dogs { .. } => "dogs",
            }
        }
    }

    #[derive(Debug, PartialEq)]
    struct Error;

    fn resolve_query_root(_res: Response<Error>) -> Result<json::Value, Error> {
        Ok(json!({ "weight": 4 }))
    }

    #[test]
    fn basic_json_object() {
        let cats_request = vec![SomeField::Cats {
            name: "Leopold".to_string(),
        }];
        let response = Response::<Error>::new()
            .on(cats_request, |req, res| leaf(req, resolve_query_root(res)));

        assert_eq!(
            response.into_future().wait().unwrap(),
            json!({ "cats": { "weight": 4 } })
        );
    }

    #[test]
    fn basic_future() {
        let cats_request = vec![SomeField::Cats {
            name: "Leopold".to_string(),
        }];
        let response = Response::<Error>::new().on(cats_request, |req, _res| {
            leaf(req, ::futures::future::ok(json!({ "weight": 5 } )))
        });

        let resolved = response.into_future().wait().unwrap();

        assert_eq!(resolved, json!({ "cats": { "weight": 5 } }));
    }

    #[test]
    fn response_set_works() {
        let cats_request = vec![SomeField::Cats {
            name: "Leopold".to_string(),
        }];
        let response = Response::<Error>::new().on(cats_request, |req, res| {
            leaf(req, res.set("whiskers", json!(9000)))
        });

        let resolved = response.into_future().wait().unwrap();

        assert_eq!(resolved, json!({ "cats": { "whiskers": 9000 } }));
    }

    #[test]
    fn multi_field_nested_response() {
        let dogs_request = vec![SomeField::Dogs {
            selection: vec![
                Dog::Name,
                Dog::Fluffiness,
                Dog::FetchBall,
                Dog::Skills {
                    selection: vec![DogSkills::CanJump],
                },
            ],
            puppies_only: false,
        }];

        fn resolve_skills(res: Response<Error>, skills: &[DogSkills]) -> ResponseValue<Error> {
            res.on(skills, |req, res| match req {
                DogSkills::CanSit => leaf(req, Ok(json!(true))),
                DogSkills::CanSwim => leaf(req, Ok(json!(true))),
                DogSkills::CanJump => leaf(req, Ok(json!(true))),
                _ => skip(),
            })
        }

        let response = Response::<Error>::new().on(dogs_request, |req, res| match req {
            SomeField::Dogs { selection, .. } => res.on(selection, |req, res| match req {
                Dog::Skills { selection } => leaf(req, resolve_skills(res, selection)),
                Dog::FetchBall => leaf(req, Ok(json!("doing a bamboozle"))),
                _ => skip(),
            }),
            SomeField::Cats { .. } => unreachable!(),
        });

        let resolved = response.into_future().wait().unwrap();

        assert_eq!(
            resolved,
            json!({
                "skills": { "CanJump": true },
                "fetch_ball": "doing a bamboozle"
            })
        );
    }
}
