extern crate futures;
extern crate hyper;
#[macro_use]
extern crate serde_json;
extern crate tokio_gql;

use futures::prelude::*;
use hyper::server::{NewService, Service};
use serde_json as json;
use std::fmt::Debug;
use std::sync::Arc;
use tokio_gql::coercion::CoerceQueryDocument;
use tokio_gql::errors::GqlError;
use tokio_gql::graphql_parser;
use tokio_gql::query_validation::ValidationContext;
use tokio_gql::response::Response;
use tokio_gql::service::GqlService;

struct ServerWrapper<Server>(Arc<Server>);

impl<Server> NewService for ServerWrapper<Server>
where
    ServerWrapper<Server>: Service,
{
    type Request = <ServerWrapper<Server> as Service>::Request;
    type Response = <ServerWrapper<Server> as Service>::Response;
    type Error = <ServerWrapper<Server> as Service>::Error;
    type Instance = Self;

    fn new_service(&self) -> Result<Self::Instance, ::std::io::Error> {
        Ok(ServerWrapper(self.0.clone()))
    }
}

impl<Schema, Error, Resolver> Service for ServerWrapper<StandaloneServer<Schema, Error, Resolver>>
where
    Error: Debug + PartialEq + 'static,
    Schema: Debug + CoerceQueryDocument + 'static,
    Resolver: GqlService<Schema = Schema, Error = Error> + 'static,
{
    type Request = hyper::Request<hyper::Body>;
    type Response = hyper::Response<hyper::Body>;
    type Error = hyper::Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        let cloned = self.0.clone();

        match (req.uri().path(), req.method()) {
            ("/graphql", hyper::Method::Get) => Box::new(playground().into_future()),
            ("/graphql", hyper::Method::Post) => Box::new(
                body_to_response(self.0.clone(), req.body())
                    .and_then(|body| {
                        let mut res = hyper::Response::new();
                        res.set_body(json::to_string(&body).expect("the response is valid json"));
                        Ok(res)
                    })
                    .or_else(move |err| {
                        let mut res = hyper::Response::new();
                        let resolver: &Resolver = &cloned.resolver;
                        let body_string = json::to_string(&resolver.handle_errors(err)).unwrap();
                        res.set_body(body_string);
                        Ok(res)
                    }),
            ),
            (_, _) => Box::new(redirect().into_future()),
        }
    }
}

fn redirect() -> Result<hyper::Response, hyper::Error> {
    let mut res = hyper::Response::new();
    res.set_status(hyper::StatusCode::SeeOther);
    {
        let headers = res.headers_mut();
        headers.set_raw("Location", "/graphql");
    }
    Ok(res)
}

fn body_to_response<
    Schema: Debug + CoerceQueryDocument + 'static,
    Error: Debug + PartialEq + 'static,
    Resolver: GqlService<Schema = Schema, Error = Error> + 'static,
>(
    server: Arc<StandaloneServer<Schema, Error, Resolver>>,
    body: hyper::Body,
) -> impl Future<Item = json::Value, Error = GqlError<Error>> {
    body.map_err(|_| GqlError::InternalError)
        .fold(Vec::new(), |mut acc, item| {
            acc.extend(item);
            Ok(acc) as Result<Vec<u8>, GqlError<Error>>
        })
        .and_then(|req_body| String::from_utf8(req_body).map_err(|_| GqlError::InvalidRequest))
        .and_then(|request_string| {
            let parsed_query = graphql_parser::parse_query(&request_string)?;
            let parsed_variables = json!({});
            let parsed_variables = json::Map::new();
            let validation_context = ValidationContext::new(parsed_variables);
            let query =
                <Schema as CoerceQueryDocument>::coerce(&parsed_query, &validation_context)?;
            Ok(query)
        })
        .and_then(move |query| {
            server
                .resolver
                .resolve(query, Response::new())
                .map_err(|err| GqlError::ResolverError(err))
        })
        .map(|_| json!({}))
}

fn playground() -> Result<hyper::Response, hyper::Error> {
    let template = include_str!("graphql_playground.html");
    let mut res = hyper::Response::new();
    res.set_body(template);
    {
        let headers = res.headers_mut();
        headers.set_raw("Content-Type", "text/html");
    }
    Ok(res)
}

pub struct StandaloneServer<Schema, Error, Resolver>
where
    Schema: CoerceQueryDocument,
    Error: Debug + PartialEq,
    Resolver: GqlService<Schema = Schema, Error = Error>,
{
    resolver: Resolver,
}

impl<Schema, Error, Resolver> StandaloneServer<Schema, Error, Resolver>
where
    Schema: Debug + CoerceQueryDocument + 'static,
    Error: Debug + PartialEq + 'static,
    Resolver: GqlService<Schema = Schema, Error = Error> + 'static,
{
    pub fn new(resolver: Resolver) -> Self {
        StandaloneServer { resolver }
    }

    pub fn start(self) -> Result<(), ()> {
        let new_service = ::std::sync::Arc::new(self);
        hyper::server::Http::new()
            .bind(&"127.0.0.1:8000".parse().unwrap(), move || {
                Ok(ServerWrapper(new_service.clone()))
            })
            .expect("bound to localhost:8000")
            .run()
            .unwrap();
        Ok(())
    }
}
