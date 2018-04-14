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
    _err: ::std::marker::PhantomData<Error>,
    data_fields: Vec<Box<Future<Item = json::Value, Error = Error>>>,
    path: Vec<&'static str>,
}

impl<Error: Debug + PartialEq + 'static> Response<Error> {
    pub fn new() -> Response<Error> {
        Response::create(&["data"])
    }

    pub fn create(prefix: &[&'static str]) -> Response<Error> {
        Response {
            _err: ::std::marker::PhantomData,
            data_fields: Vec::new(),
            path: prefix.to_owned(),
        }
    }

    pub fn on<T: PathFragment, F: IntoFuture<Item = json::Value, Error = Error> + 'static>(
        mut self,
        field: T,
        handler: impl Fn(&T, Response<Error>) -> F,
    ) -> Self {
        let path_suffix = field.as_path_fragment();
        let mut full_path = self.path.clone();
        full_path.push(path_suffix);
        self.data_fields.push(Box::new(
            handler(&field, Response::create(&full_path)).into_future(),
        ));
        self
    }

    pub fn maybe_on<T: PathFragment, F: IntoFuture<Item = json::Value, Error = Error> + 'static>(
        mut self,
        field: Option<T>,
        handler: impl Fn(&T, Response<Error>) -> F,
    ) -> Self {
        if let Some(field) = field {
            let path_suffix = field.as_path_fragment();
            let mut full_path = self.path.clone();
            full_path.push(path_suffix);
            self.data_fields.push(Box::new(
                handler(&field, Response::create(&full_path))
                    .into_future()
                    .map(move |res| json!({ path_suffix: res })),
            ));
        }
        self
    }

    pub fn merge(mut self, value: json::Value) -> Self {
        self.data_fields
            .push(Box::new(::futures::future::ok(value)));
        self
    }

    pub fn resolve(self) -> impl Future<Item = json::Value, Error = Error> {
        let Response {
            path, data_fields, ..
        } = self;

        ::futures::future::join_all(data_fields).and_then(move |data_fields| {
            let mut data_map = HashMap::new();

            for field in data_fields.iter() {
                if let json::Value::Object(map) = field {
                    data_map.extend(map.iter())
                }
            }

            let key: &'static str = path[path.len() - 1];

            Ok(json!({ key: data_map }))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    enum DogSkills {
        CanJump,
        CanPlayDead,
        CanSwim,
        CanSit,
    }

    #[derive(Debug, PartialEq)]
    enum Dog {
        Name,
        Age,
        Fluffiness,
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
                Dog::Fluffiness => "fluffiness",
                Dog::Skills { .. } => "skills",
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

    fn resolve_query_root(q: &SomeField, res: Response<Error>) -> Result<json::Value, Error> {
        Ok(json!({ "weight": 4 }))
    }

    #[test]
    fn basic_json_object() {
        let cats_request = SomeField::Cats {
            name: "Leopold".to_string(),
        };
        let mut response = Response::<Error>::new().on(cats_request, resolve_query_root);

        assert_eq!(
            ::futures::executor::block_on(response.resolve()).unwrap(),
            json!({ "data": { "weight": 4 } })
        );
    }

    #[test]
    fn basic_future() {
        let cats_request = SomeField::Cats {
            name: "Leopold".to_string(),
        };
        let mut response = Response::<Error>::new().on(cats_request, |_req, _res| {
            ::futures::future::ok(json!({ "weight": 5 } ))
        });

        let resolved = ::futures::executor::block_on(response.resolve()).unwrap();

        assert_eq!(resolved, json!({ "data": { "weight": 5 } }));
    }

    #[test]
    fn nested_response() {
        let dogs_request = SomeField::Dogs {
            selection: vec![
                Dog::Name,
                Dog::Fluffiness,
                Dog::Skills {
                    selection: vec![DogSkills::CanJump],
                },
            ],
            puppies_only: false,
        };

        let response = Response::<Error>::new().on(dogs_request, |req, res| match req {
            SomeField::Dogs {
                selection,
                puppies_only,
            } => {
                let default_response = json!({
                    "name": "Laika",
                    "age": 2,
                    "unrelated_field": [3, 4, 5],
                });

                let skills_field = selection
                    .iter()
                    .filter_map(|dog| {
                        if let Dog::Skills { .. } = dog {
                            Some(dog)
                        } else {
                            None
                        }
                    })
                    .next();

                res.maybe_on(skills_field, |req, res| {
                    Ok(json!({ "can_jump": true, "can_sit": true }))
                }).merge(default_response)
                    .resolve()
            }
            SomeField::Cats { name } => unimplemented!(),
        });

        let resolved = ::futures::executor::block_on(response.resolve()).unwrap();

        assert_eq!(
            resolved,
            json!({
                "data": {
                    "dogs": {
                        "name": "Laika",
                        "age": 2,
                        "unrelated_field": [3, 4, 5],
                        "skills": { "can_jump": true, "can_sit": true },
                    }
                }
            })
        );
    }
}
