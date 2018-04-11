use coercion::traits::*;
use context::DeriveContext;
use graphql_parser::schema::*;
use heck::*;
use proc_macro2::{Literal, Span, Term};
use quote;
use shared;

impl ImplCoerce for ObjectType {
    fn impl_coerce(&self, context: &DeriveContext) -> quote::Tokens {
        let name = Term::new(&self.name, Span::call_site());
        let field_matchers = self.fields.iter().map(|field| {
            // we have to split this in two: required and optional arguments

            let variant_name = Term::new(&field.name.to_camel_case(), Span::call_site());
            let variant_name_literal = &field.name;
            let argument_idents: Vec<Term> = field
                .arguments
                .iter()
                .map(|arg| Term::new(&arg.name.to_mixed_case(), Span::call_site()))
                .collect();
            let argument_literals: Vec<Literal> = field
                .arguments
                .iter()
                .map(|arg| Literal::string(&arg.name))
                .collect();
            let argument_types: Vec<quote::Tokens> = field
                .arguments
                .iter()
                .map(|arg| {
                    let variant = Term::new(
                        shared::extract_inner_name(&arg.value_type),
                        Span::call_site(),
                    );
                    quote! {
                        ::tokio_gql::graphql_parser::schema::Value::#variant
                    }
                })
                .collect();
            let argument_rust_types: Vec<_> = field
                .arguments
                .iter()
                .map(|arg| shared::gql_type_to_json_type(&arg.value_type))
                .collect();
            let field_type_name = shared::extract_inner_name(&field.field_type);
            let variant_constructor = field_variant_constructor(
                name,
                variant_name,
                &argument_idents,
                field_type_name,
                context,
            );

            quote! {
                if field.name == #variant_name_literal {
                    #(
                        let #argument_idents = field
                            .arguments
                            .iter()
                            .find(|(name, _)| name == #argument_literals)
                            .and_then(|(_, value)| {
                                if let #argument_types(_) = value {
                                    Some(<#argument_rust_types as ::tokio_gql::coercion::CoerceScalar>::coerce(value).expect("Should be propagated as a coercionerror"))
                                } else {
                                    None
                                }
                            }).ok_or(::tokio_gql::coercion::CoercionError)?;
                    )*
                    result.push(#variant_constructor)
                }
            }
        });

        quote! {
            impl ::tokio_gql::coercion::CoerceSelection for #name {
                fn coerce(
                    query: &::tokio_gql::graphql_parser::query::SelectionSet,
                    context: &::tokio_gql::query_validation::ValidationContext,
                ) -> Result<Vec<#name>, ::tokio_gql::coercion::CoercionError> {
                    let mut result: Vec<#name> = Vec::new();

                    for item in query.items.iter() {
                        match item {
                            ::tokio_gql::graphql_parser::query::Selection::Field(ref field) => {
                                #(#field_matchers)*
                            }
                            ::tokio_gql::graphql_parser::query::Selection::FragmentSpread(_) => unimplemented!(),
                            ::tokio_gql::graphql_parser::query::Selection::InlineFragment(_) => unimplemented!(),

                        }
                    }

                    Ok(result)
                }
            }
        }
    }
}

fn field_variant_constructor(
    field_name: Term,
    variant_name: Term,
    argument_idents: &[Term],
    field_type_name: &str,
    context: &DeriveContext,
) -> quote::Tokens {
    let argument_idents_clone = argument_idents.clone();
    if argument_idents.is_empty() && context.is_scalar(field_type_name) {
        quote!(#field_name::#variant_name)
    } else if !argument_idents.is_empty() && !context.is_scalar(field_type_name) {
        let field_type = Term::new(field_type_name, Span::call_site());
        quote!(#field_name::#variant_name { selection: <::tokio_gql::graphql_parser::schema::Value::#field_type as ::tokio_gql::coercion::CoerceSelection>::coerce(query, context), #(#argument_idents_clone),* })
    } else if argument_idents.is_empty() {
        let field_type = Term::new(field_type_name, Span::call_site());
        quote!(#field_name::#variant_name { selection: <::tokio_gql::graphql_parser::schema::Value::#field_type as tokio_gql::coercion::CoerceSelection>::coerce(query, context) })
    } else {
        quote!(#field_name::#variant_name { #(#argument_idents_clone),* })
    }
}
