use context::DeriveContext;
use quote;

pub trait ImplResponder {
    fn impl_responder(&self, context: &DeriveContext) -> quote::Tokens;
}
