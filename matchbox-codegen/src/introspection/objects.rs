use super::traits::Introspectable;
use graphql_parser::schema;
use quote;

impl Introspectable for schema::ObjectType {
    fn introspect(&self) -> quote::Tokens {
        let name_lit = &self.name;
        let description = match &self.description {
            Some(lit) => quote!(Some(#lit)),
            None => quote!(None),
        };
        let fields = self.fields.iter().map(|field| field.introspect());

        quote! {
            ::tokio_gql::introspection::Type {
                kind: ::tokio_gql::introspection::TypeKind::Object,
                name: Some(#name_lit),
                description: #description,
                fields: &[#(#fields),*]
            }
        }
    }
}
