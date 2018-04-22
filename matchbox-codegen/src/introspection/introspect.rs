use super::traits::Introspectable;
use context::DeriveContext;
use quote;

pub fn introspect_context(context: &DeriveContext) -> quote::Tokens {
    let mut types = Vec::new();

    for object in context.object_types.iter() {
        types.push(object.introspect());
    }

    for enum_type in context.enum_types.values() {
        types.push(enum_type.introspect());
    }

    for input in context.input_types.values() {
        types.push(input.introspect());
    }

    let schema = match context.get_schema() {
        Some(schema) => {
            let query_type = match schema.query {
                Some(name) => quote!(Some(#name)),
                None => quote!(None),
            };
            let mutation_type = match schema.mutation {
                Some(name) => quote!(Some(#name)),
                None => quote!(None),
            };
            let subscription_type = match schema.subscription {
                Some(name) => quote!(Some(#name)),
                None => quote!(None),
            };

            quote! {
                ::tokio_gql::introspection::Schema {
                    types: INTROSPECTION_TYPES,
                    query_type: #query_type,
                    mutation_type: #mutation_type,
                    subscription_type: #subscription_type,
                    directives: &[],
                }
            }
        }
        None => quote! {
            ::tokio_gql::introspection::Schema {
                types: INTROSPECTION_TYPES,
                query_type: None,
                mutation_type: None,
                subscription_type: None,
                directives: &[],
            }
        },
    };

    quote! {
        #[allow(dead_code)]
        const INTROSPECTION_TYPES: &'static [::tokio_gql::introspection::Type] = &[
            #(#types),*
        ];

        #[allow(dead_code)]
        const INTROSPECTION_SCHEMA: ::tokio_gql::introspection::Schema = #schema;
    }
}
