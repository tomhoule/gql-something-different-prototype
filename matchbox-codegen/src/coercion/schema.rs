use coercion::traits::*;
use context::DeriveContext;
use graphql_parser::schema::*;
use heck::CamelCase;
use proc_macro2::{Span, Term};
use quote;

impl ImplCoerce for SchemaDefinition {
    fn impl_coerce(&self, _context: &DeriveContext) -> quote::Tokens {
        let mut selection_types: Vec<Term> = Vec::new();
        let mut variant_names: Vec<Term> = Vec::new();
        let mut operations: Vec<Term> = Vec::new();

        if let Some(ref name) = self.query {
            let name = Term::new(name.as_str(), Span::call_site());
            selection_types.push(name);
            variant_names.push(Term::new("query", Span::call_site()));
            operations.push(Term::new("Query", Span::call_site()));
        }

        if let Some(ref name) = self.mutation {
            let name = Term::new(name.as_str(), Span::call_site());
            selection_types.push(name);
            variant_names.push(Term::new("mutation", Span::call_site()));
            operations.push(Term::new("Mutation", Span::call_site()));
        }

        if let Some(ref name) = self.subscription {
            let name = Term::new(name.as_str(), Span::call_site());
            selection_types.push(name);
            variant_names.push(Term::new("subscription", Span::call_site()));
            operations.push(Term::new("Subscription", Span::call_site()));
        }

        let node_types: Vec<Term> = variant_names
            .iter()
            .map(|name| Term::new(&format!("{}", name).to_camel_case(), Span::call_site()))
            .collect();
        let variant_names_2 = variant_names.clone();
        let variant_names_3 = variant_names.clone();
        let variant_names_4 = variant_names.clone();
        let selection_types_clone = selection_types.clone();

        quote! {
            impl ::tokio_gql::coercion::CoerceQueryDocument for Operation {
                fn coerce(
                    document: &::tokio_gql::graphql_parser::query::Document,
                    context: &::tokio_gql::query_validation::ValidationContext
                ) -> Result<Vec<Self>, ::tokio_gql::coercion::CoercionError> {
                    #(
                        let #variant_names: Result<Vec<#selection_types>, _> = document.definitions
                            .iter()
                            .filter_map(|op| {
                                if let ::tokio_gql::graphql_parser::query::Definition::Operation(::tokio_gql::graphql_parser::query::OperationDefinition::#node_types(ref definition)) = op {
                                    Some(
                                        <#selection_types_clone as ::tokio_gql::coercion::CoerceSelection>::coerce(
                                            &definition.clone().selection_set,
                                            context,
                                        )
                                    )
                                } else {
                                    None
                                }
                            })
                            .next()
                            .unwrap_or(Ok(Vec::new()));
                        let #variant_names_2 = #variant_names_3?;
                    )*

                    Ok(
                        vec![
                            #(
                                Operation::#operations { selection: #variant_names_4 },
                            )*
                        ]
                    )
                }
            }
        }
    }
}
