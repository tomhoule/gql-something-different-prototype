extern crate futures;
#[macro_use]
extern crate tokio_gql;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;

use futures::prelude::*;
use serde_json as json;
use tokio_gql::resolver::*;
use tokio_gql::response::Response;

mod star_wars {
    #[allow(dead_code)]
    #[derive(SomethingCompletelyDifferent)]
    #[SomethingCompletelyDifferent(path = "tests/star_wars_schema.graphql")]
    struct ComplexSchema;
}

struct StarWarsResolver;

impl Resolver for StarWarsResolver {
    type Schema = star_wars::Operation;
    type Responder = star_wars::RootResponder;

    fn resolve(&self, request: Self::Schema, responder: Self::Responder) -> ResolverFuture {
        unimplemented!();
    }
}

fn test_response(req: &star_wars::Operation, expected_response: json::Value) {}

#[test]
fn basic_sync_field() {
    let field = star_wars::Human::HomePlanet {
        respond: ::std::default::Default::default(),
    };

    if let star_wars::Human::HomePlanet { respond } = field {
        match respond.with(Some("Vosges".into())) {
            Response::Async(_) => unreachable!(),
            Response::Immediate(result) => assert_eq!(
                result,
                ("homePlanet", ::serde_json::Value::String("Vosges".into()))
            ),
        }
    } else {
        unreachable!();
    }
}

#[test]
fn basic_async_object() {
    let field = star_wars::Human::FriendsConnection {
        respond: Default::default(),
        after: Some("eh".to_string()),
        first: Some(5),
        selection: vec![star_wars::FriendsConnection::PageInfo {
            respond: Default::default(),
            selection: vec![
                star_wars::PageInfo::HasNextPage {
                    respond: Default::default(),
                },
                star_wars::PageInfo::EndCursor {
                    respond: Default::default(),
                },
            ],
        }],
    };

    fn resolve_page_info(
        selection: &[star_wars::PageInfo],
    ) -> impl Future<
        Item = impl Fn(star_wars::PageInfo) -> Response,
        Error = tokio_gql::errors::ResolverError,
    > {
        ::futures::future::ok(("to_alpha", "to_omega", true)).map(|(start, end, has_next_page)| {
            move |field| {
                unimplemented!();
                match field {
                    star_wars::PageInfo::StartCursor { respond } => {
                        respond.with(Some(start.to_string().into()))
                    }
                    star_wars::PageInfo::EndCursor { respond } => {
                        respond.with(Some(end.to_string()))
                    }
                    star_wars::PageInfo::HasNextPage { respond } => respond.with(has_next_page),
                }
            }
        })
    }

    fn load_friends_connection(
        selection: &[star_wars::FriendsConnection],
    ) -> impl Future<Item = (), Error = tokio_gql::errors::ResolverError> {
        ::futures::future::ok(())
    }

    fn resolve_friends_connection(data: &(), field: star_wars::FriendsConnection) -> Response {
        match field {
            star_wars::FriendsConnection::PageInfo { selection, respond } => {
                respond.to(selection, resolve_page_info)
            }
            _ => unimplemented!(),
        }
    }

    if let star_wars::Human::FriendsConnection {
        respond, selection, ..
    } = field
    {
        let fut = { respond.to(selection, resolve_friends_connection) };

        match fut.wait() {
            Ok(i) => assert_eq!(i, json!({ "hasNextPage": true, "endCursor": "to_omega" })),
            _ => unreachable!(),
        }
    } else {
        unreachable!();
    }
}
