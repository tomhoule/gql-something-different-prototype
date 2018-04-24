use super::traits::*;
use context::DeriveContext;
use quote;

pub fn gather_impls(context: &DeriveContext) -> Vec<quote::Tokens> {
    let mut result = Vec::new();

    if let Some(schema) = context.get_schema() {
        result.push(schema.impl_responder(&context));
    }

    result
}
