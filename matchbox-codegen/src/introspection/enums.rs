use super::traits::Introspectable;
use graphql_parser::schema;
use quote;

impl Introspectable for schema::EnumType {
    fn introspect(&self) -> quote::Tokens {
        let name_lit = &self.name;
        let description = match &self.description {
            Some(lit) => quote!(Some(#lit)),
            None => quote!(None),
        };

        quote!{
            ::tokio_gql::introspection::Type {
                kind: ::tokio_gql::introspection::TypeKind::Enum,
                name: Some(#name_lit),
                description: #description,
                fields: &[],
            }
        }
    }
}
