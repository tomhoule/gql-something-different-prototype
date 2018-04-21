use quote;

/// This is meant to be implemented by schema items. The `introspect` trait method produces a literal value (struct or enum) suitable for introspection. The constructors are in `tokio_gql::introspection`.
pub trait Introspectable {
    fn introspect(&self) -> quote::Tokens;
}
