use context::DeriveContext;
use graphql_parser::schema::*;
use heck::CamelCase;
use proc_macro2::{Span, Term};
use quote;

pub trait ImplCoerce {
    fn impl_coerce(&self, context: &DeriveContext) -> quote::Tokens;
}

impl ImplCoerce for SchemaDefinition {
    fn impl_coerce(&self, _context: &DeriveContext) -> quote::Tokens {
        let mut field_values: Vec<Term> = Vec::new();
        let mut field_names: Vec<Term> = Vec::new();

        if let Some(ref name) = self.query {
            let name = Term::new(name.as_str(), Span::call_site());
            field_values.push(name);
            field_names.push(Term::new("query", Span::call_site()));
        }

        if let Some(ref name) = self.mutation {
            let name = Term::new(name.as_str(), Span::call_site());
            field_values.push(name);
            field_names.push(Term::new("mutation", Span::call_site()));
        }

        if let Some(ref name) = self.subscription {
            let name = Term::new(name.as_str(), Span::call_site());
            field_values.push(name);
            field_names.push(Term::new("subscription", Span::call_site()));
        }

        let node_types: Vec<Term> = field_names
            .iter()
            .map(|name| Term::new(&format!("{}", name).to_camel_case(), Span::call_site()))
            .collect();
        let field_names_2 = field_names.clone();
        let field_values_clone = field_values.clone();

        quote! {
            impl ::tokio_gql::coercion::CoerceQueryDocument for Schema {
                fn coerce(
                    document: &::tokio_gql::graphql_parser::query::Document,
                    context: &::tokio_gql::query_validation::ValidationContext
                ) -> Result<Self, ::tokio_gql::coercion::CoercionError> {
                    #(
                        let #field_names: Vec<#field_values> = document.definitions
                            .iter()
                            .filter_map(|op| {
                                if let ::tokio_gql::graphql_parser::query::Definition::Operation(::tokio_gql::graphql_parser::query::OperationDefinition::#node_types(ref definition)) = op {
                                    Some(
                                        <#field_values_clone as ::tokio_gql::coercion::CoerceSelection>::coerce(
                                            &definition.clone().selection_set,
                                            context,
                                        )
                                    )
                                } else {
                                    None
                                }
                            })
                            .next()
                            .ok_or(::tokio_gql::coercion::CoercionError)??;
                    )*

                    Ok(Schema {
                        #(#field_names_2),*
                    })
                }
            }
        }
    }
}
