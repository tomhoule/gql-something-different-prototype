use super::input_values::AsField;
use super::traits::Introspectable;
use graphql_parser::schema;
use quote;

impl Introspectable for schema::InputObjectType {
    fn introspect(&self) -> quote::Tokens {
        let name_lit = &self.name;
        let description = match &self.description {
            Some(lit) => quote!(Some(#lit)),
            None => quote!(None),
        };
        let fields = self.fields
            .iter()
            .map(|field| AsField(field.clone()).introspect());

        quote! {
            ::tokio_gql::introspection::Type {
                kind: ::tokio_gql::introspection::TypeKind::InputObject,
                name: Some(#name_lit),
                description: #description,
                fields: &[#(#fields),*],
            }
        }
    }
}
