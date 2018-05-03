#[macro_use]
extern crate tokio_gql;
#[macro_use]
extern crate serde_json;
extern crate futures;
extern crate standalone_server;

use futures::prelude::*;
use serde_json as json;
use tokio_gql::response::{leaf, rest, Response};

pub mod schema {
    #[derive(SomethingCompletelyDifferent)]
    #[SomethingCompletelyDifferent(path = "src/schema.graphql")]
    struct EchoSchema;
}

struct EchoResolver {
    archive: Vec<String>,
}

#[derive(PartialEq, Debug)]
struct Error;

impl tokio_gql::service::GqlService for EchoResolver {
    type Schema = schema::Operation;
    type Error = Error;

    fn resolve(
        &self,
        request: Self::Schema,
        response: Response<Error>,
    ) -> Box<Future<Item = json::Value, Error = Self::Error>> {
        let query_response = response.on(request.query, |req, _res| match req {
            schema::EchoQuery::PastEchoes => leaf(req, Ok(json!(self.archive))),
        });

        let mutation_response =
            tokio_gql::response::Response::new().on(request.mutation, |req, res| match req {
                schema::EchoMutation::Echo { message } => {
                    rest(Ok(json::Value::String(match message {
                        Some(msg) => msg.to_string(),
                        None => "this probably should reply with an empty string".to_string(),
                    })))
                }
            });

        Box::new(
            query_response
                .into_future()
                .join(mutation_response)
                .map(|(q, m)| {
                    json!({
                        "data": {
                            "query": q,
                            "mutation": m,
                        }
                    })
                }),
        )
    }
}

fn main() {
    standalone_server::StandaloneServer::new(EchoResolver {
        archive: Vec::new(),
    }).start()
        .unwrap()
}
