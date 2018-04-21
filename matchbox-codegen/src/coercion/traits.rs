use context::DeriveContext;
use quote;

pub trait ImplCoerce {
    fn impl_coerce(&self, context: &DeriveContext) -> quote::Tokens;
}
