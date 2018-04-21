use super::traits::Introspectable;
use graphql_parser::schema;
use quote;

impl Introspectable for schema::Field {
    fn introspect(&self) -> quote::Tokens {
        let name_lit = &self.name;
        let description = match &self.description {
            Some(lit) => quote!(Some(#lit)),
            None => quote!(None),
        };
        let args = self.arguments.iter().map(|arg| arg.introspect());
        let type_name = ::shared::extract_inner_name(&self.field_type);

        quote! {
            ::tokio_gql::introspection::Field {
                name: #name_lit,
                description: #description,
                args: &[#(#args),*],
                type_: #type_name,
                is_deprecated: false,
                deprecation_reason: None,
            }
        }
    }
}
