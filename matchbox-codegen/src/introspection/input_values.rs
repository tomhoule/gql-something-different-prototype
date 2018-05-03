use super::traits::Introspectable;
use graphql_parser::schema;
use quote;

impl Introspectable for schema::InputValue {
    fn introspect(&self) -> quote::Tokens {
        let name = &self.name;
        let description = match &self.description {
            Some(lit) => quote!(Some(#lit)),
            None => quote!(None),
        };
        let type_name = ::shared::extract_inner_name(&self.value_type);
        let default_value = match &self.default_value {
            Some(value) => {
                let inner = format!("{}", value);
                quote!(Some(#inner))
            }
            None => quote!(None),
        };
        quote! {
            ::tokio_gql::introspection::InputValue {
                name: #name,
                description: #description,
                type_: #type_name,
                default_value: #default_value,
            }
        }
    }
}

pub(crate) struct AsField(pub schema::InputValue);

impl Introspectable for AsField {
    fn introspect(&self) -> quote::Tokens {
        let name = &self.0.name;
        let description = match &self.0.description {
            Some(lit) => quote!(Some(#lit)),
            None => quote!(None),
        };
        let type_name = ::shared::extract_inner_name(&self.0.value_type);
        quote! {
            ::tokio_gql::introspection::Field {
                name: #name,
                description: #description,
                args: &[],
                type_: #type_name,
                is_deprecated: false,
                deprecation_reason: None,
            }
        }
    }
}
