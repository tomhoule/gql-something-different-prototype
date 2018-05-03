//! The mental model is the following:
//!
//! Very often a graphql object contains fields that are usually fetched together or trivial to get. We can just always set them in the Response and they will be pruned when the actual JSON response is sent. For other, more intricate fields (those that take an argument or require fetching more data), we want to have special handlers (but still go the simple route for the simple fields).
//!
//! This is achieved with the `on` and `merge` methods on `Response` and judicious usage of Rust's pattern matching and iterators.

use futures::prelude::*;
use serde_json as json;
use std::collections::HashMap;

pub trait PathFragment {
    fn as_path_fragment(&self) -> &'static str;
}

impl<'a, T: PathFragment> PathFragment for &'a T {
    fn as_path_fragment(&self) -> &'static str {
        (*self).as_path_fragment()
    }
}

// pub struct Response {
//     async_fields: Vec<ResponseFut>,
//     resolved_fields: HashMap<String, json::Value>,
//     path: Vec<&'static str>,
// }

// type ResponseFut = Box<Future<Item = json::Value, Error = ::errors::ResolverError>>;

// impl Response {
//     pub fn new() -> Response {
//         Response::create(&[])
//     }

//     pub fn create(prefix: &[&'static str]) -> Response {
//         Response {
//             async_fields: Vec::new(),
//             resolved_fields: HashMap::new(),
//             path: prefix.to_owned(),
//         }
//     }

//     pub fn on<T: PathFragment>(
//         fields: Vec<T>,
//         handler: impl Fn(&T, Response) -> ResponseValue,
//     ) -> HashMap<&'static str, ResponseValue> {
//         let mut result_value = HashMap::with_capacity(fields.len());

//         for field in fields {
//             result_value.insert(field.as_path_fragment(), handler(&field, Response::new()));
//         }

//         result_value
//     }
// }

// impl IntoFuture for Response {
//     type Item = json::Value;
//     type Error = Error;
//     type Future = Box<Future<Item = Self::Item, Error = Self::Error>>;

//     fn into_future(self) -> Self::Future {
//         let Response {
//             async_fields,
//             mut resolved_fields,
//             ..
//         } = self;

//         Box::new(
//             ::futures::future::join_all(async_fields).and_then(move |deferred_fields| {
//                 for field in deferred_fields.into_iter() {
//                     if let json::Value::Object(map) = field {
//                         resolved_fields.extend(map)
//                     }
//                 }

//                 Ok(json::to_value(&resolved_fields).expect("the response map is valid JSON"))
//             }),
//         )
//     }
// }

pub enum Response {
    Async(Box<Future<Item = (&'static str, json::Value), Error = ::errors::ResolverError>>),
    /// Produced by attaching to a dataloader
    // Deferred(::futures::sync::oneshot::Receiver<json::Value>),
    Immediate((&'static str, json::Value)),
}

// impl IntoFuture for ResponseValue {
//     type Item = json::Value;
//     type Error = ::errors::ResolverError;
//     type Future = Box<Future<Item = Self::Item, Error = Self::Error>>;

//     fn into_future(self) -> Self::Future {
//         match self {
//             ResponseValue::Node(fut) => fut,
//             ResponseValue::Immediate(val) => Box::new(::futures::future::ok(val)),
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(dead_code)]
    #[derive(Debug, PartialEq)]
    enum DogSkills {
        CanJump(#[must_use] CanJumpResponder),
        CanPlayDead(CanPlayDeadResponder),
        CanSwim(CanSwimResponder),
        CanSit(CanSitResponder),
    }

    #[derive(Debug, PartialEq)]
    struct CanJumpResponder;

    impl CanJumpResponder {
        fn with(&self, val: bool) -> (&'static str, json::Value) {
            ("can_jump", json::Value::Bool(val))
        }
    }

    #[derive(Debug, PartialEq)]
    struct CanPlayDeadResponder;

    impl CanPlayDeadResponder {
        fn with(&self, val: bool) -> (&'static str, json::Value) {
            ("can_play_dead", json::Value::Bool(val))
        }
    }

    #[derive(Debug, PartialEq)]
    struct CanSwimResponder;

    impl CanSwimResponder {
        fn with(&self, val: bool) -> (&'static str, json::Value) {
            ("can_swim", json::Value::Bool(val))
        }
    }

    #[derive(Debug, PartialEq)]
    struct CanSitResponder;

    impl CanSitResponder {
        fn with(&self, val: bool) -> (&'static str, json::Value) {
            ("can_sit", json::Value::Bool(val))
        }

        fn with_async<Resolver, ResolverFuture>(
            &self,
            resolver: Resolver,
        ) -> Box<Future<Item = ::serde_json::Value, Error = ::errors::ResolverError>>
        where
            Resolver: Fn() -> ResolverFuture,
            ResolverFuture: Future<Item = bool, Error = ::failure::Error> + Sized,
        {
            unimplemented!();
        }
    }

    #[allow(dead_code)]
    #[derive(Debug, PartialEq)]
    enum Dog {
        Name,
        Age,
        Fluffiness,
        FetchBall,
        Skills {
            selection: Vec<DogSkills>,
            res: DogSkillsResponder,
        },
    }

    struct DogResponder;

    #[derive(Debug, PartialEq)]
    struct DogSkillsResponder;

    type ObjectFuture =
        Box<Future<Item = HashMap<&'static str, json::Value>, Error = ::errors::ResolverError>>;

    impl DogSkillsResponder {
        fn on<Resolver, ResolverFuture>(
            &self,
            selection: Vec<DogSkills>,
            resolver: Resolver,
        ) -> Box<Future<Item = HashMap<&'static str, json::Value>, Error = ::errors::ResolverError>>
        where
            Resolver: Fn(DogSkills) -> ResolverFuture,
            ResolverFuture:
                IntoFuture<Item = HashMap<&'static str, json::Value>, Error = ::failure::Error>,
        {
            unimplemented!();
        }

        fn on_sync<Resolver>(
            &self,
            selection: Vec<DogSkills>,
            resolver: Resolver,
        ) -> Box<Future<Item = HashMap<&'static str, json::Value>, Error = ::errors::ResolverError>>
        where
            Resolver: Fn(DogSkills) -> (&'static str, json::Value),
        {
            unimplemented!();
        }
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

    #[derive(Debug, PartialEq)]
    struct Error;

    fn resolve_query_root(_res: Response) -> Result<json::Value, Error> {
        Ok(json!({ "weight": 4 }))
    }

    #[test]
    fn multi_field_nested_response() {
        let dogs_request = vec![SomeField::Dogs {
            selection: vec![
                Dog::Name,
                Dog::Fluffiness,
                Dog::FetchBall,
                Dog::Skills {
                    selection: vec![DogSkills::CanJump(CanJumpResponder)],
                    res: DogSkillsResponder,
                },
            ],
            puppies_only: false,
        }];

        fn resolve_skills(res: DogSkillsResponder, skills: Vec<DogSkills>) -> ObjectFuture {
            res.on_sync(skills, |field| match field {
                DogSkills::CanSit(respond) => respond.with(true),
                DogSkills::CanSwim(respond) => respond.with(false),
                DogSkills::CanJump(respond) => respond.with(true),
                DogSkills::CanPlayDead(respond) => respond.with(false),
            })
        }

        // let response = Response::new().on(dogs_request, |req, res| match req {
        //     SomeField::Dogs { selection, .. } => res.on(selection, |req, res| match req {
        //         Dog::Skills { selection } => leaf(req, resolve_skills(res, selection)),
        //         Dog::FetchBall => leaf(req, Ok(json!("doing a bamboozle"))),
        //         _ => unimplemented!(),
        //     }),
        //     SomeField::Cats { .. } => unreachable!(),
        // });

        // let resolved = response.into_future().wait().unwrap();

        // assert_eq!(
        //     resolved,
        //     json!({
        //         "skills": { "can_jump": true },
        //         "fetch_ball": "doing a bamboozle"
        //     })
        // );
    }
}
