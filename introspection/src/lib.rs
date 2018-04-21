#[macro_use]
extern crate tokio_gql;
extern crate futures;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

mod introspection;

pub use introspection::resolver::IntrospectionResolver;
