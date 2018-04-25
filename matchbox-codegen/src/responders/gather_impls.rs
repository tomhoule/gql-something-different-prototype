use super::traits::*;
use context::DeriveContext;
use quote;

pub fn gather_impls(context: &DeriveContext) -> Vec<quote::Tokens> {
    let mut result = Vec::new();

    for object_type in context.object_types.iter() {
        result.push(object_type.impl_responder(context))
    }

    for interface_type in context.interface_types.values() {
        result.push(interface_type.impl_responder(context));
    }

    for union_type in context.union_types.values() {
        result.push(union_type.impl_responder(context));
    }

    // if let Some(schema) = context.get_schema() {
    //     result.push(schema.impl_responder(&context));
    // }

    result
}
