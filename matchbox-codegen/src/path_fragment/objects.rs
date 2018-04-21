use super::ImplPathFragment;
use context::DeriveContext;
use graphql_parser;
use heck::*;
use proc_macro2::{Span, Term};
use quote;
use shared;

impl ImplPathFragment for graphql_parser::schema::ObjectType {
    fn impl_path_fragment(&self, context: &DeriveContext) -> quote::Tokens {
        let object_name = Term::new(&self.name, Span::call_site());
        let variant_matchers = self.fields.iter().map(|field| {
            let term = Term::new(&field.name.to_camel_case(), Span::call_site());
            let inner_type = shared::extract_inner_name(&field.field_type);

            if field.arguments.is_empty()
                && (context.is_scalar(inner_type) || context.is_enum(inner_type))
            {
                quote!(#object_name::#term)
            } else {
                quote!(#object_name::#term { .. })
            }
        });
        let name_literals = self.fields.iter().map(|field| &field.name);

        quote! {
            impl ::tokio_gql::response::PathFragment for #object_name {
                fn as_path_fragment(&self) -> &'static str {
                    match self {
                        #(#variant_matchers => #name_literals),*
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use graphql_parser::schema;

    #[test]
    fn normal_object() {
        let object = r#"
        type FriendsEdge {
            cursor: ID!
            node: Character
        }
        "#;
        let parsed = schema::parse_schema(object).unwrap();
        let parsed = if let schema::Definition::TypeDefinition(schema::TypeDefinition::Object(
            ref obj,
        )) = parsed.definitions[0]
        {
            Some(obj)
        } else {
            None
        }.unwrap();
        let context = DeriveContext::new();
        let expected = quote! {
            impl ::tokio_gql::response::PathFragment for FriendsEdge {
                fn as_path_fragment(&self) -> &'static str {
                    match self {
                        FriendsEdge::Cursor => "cursor",
                        FriendsEdge::Node { .. } => "node"
                    }
                }
            }
        };
        assert_eq!(parsed.impl_path_fragment(&context), expected);
    }
}
