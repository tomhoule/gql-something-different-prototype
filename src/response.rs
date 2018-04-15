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
    deferred_fields: Vec<Box<Future<Item = json::Value, Error = Error>>>,
    resolved_fields: HashMap<String, json::Value>,
    path: Vec<&'static str>,
}

impl<Error: Debug + PartialEq + 'static> Response<Error> {
    pub fn new() -> Response<Error> {
        Response::create(&["data"])
    }

    pub fn create(prefix: &[&'static str]) -> Response<Error> {
        Response {
            _err: ::std::marker::PhantomData,
            deferred_fields: Vec::new(),
            resolved_fields: HashMap::new(),
            path: prefix.to_owned(),
        }
    }

    /// Handle one field asynchronously. You can pass your enum variants directly to this method and the `PathFragment` impls will take care of getting the paths right.
    ///
    /// TODO: example
    pub fn on<T: PathFragment, F: IntoFuture<Item = json::Value, Error = Error> + 'static>(
        mut self,
        field: Option<T>,
        handler: impl Fn(&T, Response<Error>) -> F,
    ) -> Self {
        if let Some(field) = field {
            let path_suffix = field.as_path_fragment();
            let mut full_path = self.path.clone();
            full_path.push(path_suffix);
            self.deferred_fields.push(Box::new(
                handler(&field, Response::create(&full_path))
                    .into_future()
                    .map(move |res| json!({ path_suffix: res })),
            ));
        }
        self
    }

    /// Passing all fields to `on_each` should be avoided unless all of them are asynchronous, since this allocates (to store the resulting future). Values that are not futures should go through `merge`.
    pub fn on_each<T: PathFragment, F: IntoFuture<Item = json::Value, Error = Error> + 'static>(
        mut self,
        fields: impl IntoIterator<Item = T>,
        handler: impl Fn(&T, Response<Error>) -> Option<F>,
    ) -> Self {
        for field in fields {
            let path_suffix = field.as_path_fragment();
            let mut full_path = self.path.clone();
            full_path.push(path_suffix);
            let fut = handler(&field, Response::create(&full_path));

            if let Some(fut) = fut {
                self.deferred_fields.push(Box::new(
                    fut.into_future()
                        .map(move |res| json!({ path_suffix: res })),
                ));
            }
        }
        self
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

    /// Combines all the resolved and deferred fields and returns a future, so this can be used as a member field higher up in the response tree.
    ///
    /// This can become an `IntoFuture` impl once the impl Trait in traits feature is implemented and stabilized.
    pub fn resolve(self) -> impl Future<Item = json::Value, Error = Error> {
        let Response {
            deferred_fields,
            mut resolved_fields,
            ..
        } = self;

        ::futures::future::join_all(deferred_fields).and_then(move |deferred_fields| {
            for field in deferred_fields.into_iter() {
                if let json::Value::Object(map) = field {
                    resolved_fields.extend(map)
                }
            }

            Ok(json::to_value(&resolved_fields).expect("the response map is valid JSON"))
        })
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

    fn resolve_query_root(_q: &SomeField, _res: Response<Error>) -> Result<json::Value, Error> {
        Ok(json!({ "weight": 4 }))
    }

    #[test]
    fn basic_json_object() {
        let cats_request = SomeField::Cats {
            name: "Leopold".to_string(),
        };
        let response = Response::<Error>::new().on(Some(cats_request), resolve_query_root);

        assert_eq!(
            response.resolve().wait().unwrap(),
            json!({ "cats": { "weight": 4 } })
        );
    }

    #[test]
    fn basic_future() {
        let cats_request = SomeField::Cats {
            name: "Leopold".to_string(),
        };
        let response = Response::<Error>::new().on(Some(cats_request), |_req, _res| {
            ::futures::future::ok(json!({ "weight": 5 } ))
        });

        let resolved = response.resolve().wait().unwrap();

        assert_eq!(resolved, json!({ "cats": { "weight": 5 } }));
    }

    #[test]
    fn response_set_works() {
        let cats_request = SomeField::Cats {
            name: "Leopold".to_string(),
        };
        let response = Response::<Error>::new().on(Some(cats_request), |_req, res| {
            res.set("whiskers", json!(9000)).resolve()
        });

        let resolved = response.resolve().wait().unwrap();

        assert_eq!(resolved, json!({ "cats": { "whiskers": 9000 } }));
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

        let response = Response::<Error>::new().on(Some(dogs_request), |req, res| match req {
            SomeField::Dogs { selection, .. } => {
                let default_response = json!({
                    "name": "Laika",
                    "age": 2,
                    "unrelated_field": [3, 4, 5],
                });

                let skills_field = selection
                    .iter()
                    .filter(|dog| matches!(dog, Dog::Skills { .. }))
                    .next();

                res.on(skills_field, |_req, _res| {
                    Ok(json!({ "can_jump": true, "can_sit": true }))
                }).merge(default_response)
                    .resolve()
            }
            SomeField::Cats { .. } => unreachable!(),
        });

        let resolved = response.resolve().wait().unwrap();

        assert_eq!(
            resolved,
            json!({
                "dogs": {
                    "name": "Laika",
                    "age": 2,
                    "unrelated_field": [3, 4, 5],
                    "skills": { "can_jump": true, "can_sit": true },
                }
            })
        );
    }

    #[test]
    fn multi_field_nested_response() {
        let dogs_request = SomeField::Dogs {
            selection: vec![
                Dog::Name,
                Dog::Fluffiness,
                Dog::FetchBall,
                Dog::Skills {
                    selection: vec![DogSkills::CanJump],
                },
            ],
            puppies_only: false,
        };

        let response = Response::<Error>::new().on(Some(dogs_request), |req, res| match req {
            SomeField::Dogs { selection, .. } => res.on_each(selection, |req, _res| match req {
                Dog::Skills { .. } => Some(Ok(json!({ "can_sit": true }))),
                Dog::FetchBall => Some(Ok(json!("doing a bamboozle"))),
                _ => None,
            }).resolve(),
            SomeField::Cats { .. } => unreachable!(),
        });

        let resolved = response.resolve().wait().unwrap();

        assert_eq!(
            resolved,
            json!({
                "dogs": {
                    "skills": { "can_sit": true },
                    "fetch_ball": "doing a bamboozle"
                }
            })
        );
    }
}
