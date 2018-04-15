use coercion::CoerceQueryDocument;
use errors::GqlError;
use futures::prelude::*;
use graphql_parser;
use hyper::{self, server::{NewService, Service}};
use json;
use query_validation::ValidationContext;
use response::Response;
use service::GqlService;
use std::fmt::Debug;
use std::sync::Arc;

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
        Box::new(
            body_to_response(self.0.clone(), req.body())
                .and_then(|body| {
                    let mut res = hyper::Response::new();
                    res.set_body(json::to_string(&body).expect("the response is valid json"));
                    Ok(res)
                })
                .or_else(|err| {
                    let mut res = hyper::Response::new();
                    res.set_body("heh, error");
                    Ok(res)
                }),
        )

        // let req.body()
        //     .collect()
        //     .and_then(|req_body| String::from_utf8(request_string))
        //     .and_then(|schema| graphql_parser::parse_schem(&schema))
        //     .and_then(|schema| {
        //         self.resolver
        //             .handle(schema, Response::new())
        //             .and_then(move |json| res.write(json))
        //     })
        //     .or_else(move |err| {
        //         let debug_error = format!("{:?}", err);
        //         res.write(json!({ "errors": debug_error }))
        //     })
    }
}

fn body_to_response<
    Schema: Debug + CoerceQueryDocument + 'static,
    Error: Debug + PartialEq + 'static,
    Resolver: GqlService<Schema = Schema, Error = Error> + 'static,
>(
    server: Arc<StandaloneServer<Schema, Error, Resolver>>,
    body: hyper::Body,
) -> impl Future<Item = json::Value, Error = GqlError> {
    body.map_err(|_| GqlError::InternalError)
        .fold(Vec::new(), |mut acc, item| {
            acc.extend(item);
            Ok(acc) as Result<Vec<u8>, GqlError>
        })
        .and_then(|req_body| String::from_utf8(req_body).map_err(|_| GqlError::InvalidRequest))
        .and_then(|request_string| {
            let parsed_query = graphql_parser::parse_query(&request_string)?;
            let validation_context = ValidationContext::new();
            let query =
                <Schema as CoerceQueryDocument>::coerce(&parsed_query, &validation_context)?;
            Ok(query)
        })
        .and_then(move |query| {
            server
                .resolver
                .resolve(query, Response::new())
                .map_err(|err| GqlError::ResolverError)
        })
        .map(|_| json!({}))
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
        hyper::server::Http::new().bind(&"8000".parse().unwrap(), move || {
            Ok(ServerWrapper(new_service.clone()))
        });
        Ok(())
    }
}
