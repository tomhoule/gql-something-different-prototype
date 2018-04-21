mod objects;

use context::DeriveContext;
use quote;

pub trait ImplPathFragment {
    fn impl_path_fragment(&self, context: &DeriveContext) -> quote::Tokens;
}

pub fn path_fragment_impls(context: &DeriveContext) -> Vec<quote::Tokens> {
    let mut results = Vec::with_capacity(context.object_types.len());

    for object in context.object_types.iter() {
        results.push(object.impl_path_fragment(context))
    }

    results
}
