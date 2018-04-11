use coercion::traits::*;
use context::DeriveContext;
use graphql_parser;
use graphql_parser::schema::*;
use heck::*;
use proc_macro2::{Literal, Span, Term};
use quote;
use shared;

/// This is evaluated in the context of the ObjectType coercion impl. Please refer to that to understand where things come from.
pub struct ArgumentsContext {
    pub fields: Vec<Field>,
    pub object_name: Term,
}

impl ImplCoerce for ArgumentsContext {
    fn impl_coerce(&self, context: &DeriveContext) -> quote::Tokens {
        let matchers = self.fields.iter().map(|field| {
            // we have to split this in two: required and optional arguments

            let variant_name = Term::new(&field.name.to_camel_case(), Span::call_site());
            let variant_name_literal = &field.name;
            let field_type_name = shared::extract_inner_name(&field.field_type);
            let variant_constructor = field_variant_constructor(
                &self.object_name,
                variant_name,
                &field.arguments,
                field_type_name,
                context,
            );

            let arguments_matchers = field.arguments.iter().map(|arg| {
                let argument_type = {
                    let variant = Term::new(
                        shared::extract_inner_name(&arg.value_type),
                        Span::call_site(),
                    );
                    quote!(::tokio_gql::graphql_parser::schema::Value::#variant)
                };
                let term = Term::new(&arg.name.to_mixed_case(), Span::call_site());

                let rust_type = shared::gql_type_to_json_type(&arg.value_type);
                let literal = Literal::string(&arg.name);

                if let graphql_parser::query::Type::NamedType(_) = arg.value_type {
                    quote! {
                        let #term = field
                            .arguments
                            .iter()
                            .find(|(name, _)| name == #literal)
                            .map(|(_, value)| {
                                if let #argument_type(_) = value {
                                    <#rust_type as ::tokio_gql::coercion::CoerceScalar>::coerce(value).expect("Should be propagated as a coercionerror")
                                } else {
                                    None
                                }
                            }).ok_or(::tokio_gql::coercion::CoercionError)?;
                    }
                } else {
                    quote! {
                        let #term = field
                            .arguments
                            .iter()
                            .find(|(name, _)| name == #literal)
                            .and_then(|(_, value)| {
                                if let #argument_type(_) = value {
                                    Some(<#rust_type as ::tokio_gql::coercion::CoerceScalar>::coerce(value).expect("Should be propagated as a coercionerror"))
                                } else {
                                    None
                                }
                            }).ok_or(::tokio_gql::coercion::CoercionError)?;
                    }
                }
            });

            quote! {
                if field.name == #variant_name_literal {
                    #(#arguments_matchers)*
                    result.push(#variant_constructor)
                }
            }
        });
        quote!(#(#matchers)*)
    }
}

fn field_variant_constructor(
    field_name: &Term,
    variant_name: Term,
    argument_idents: &[InputValue],
    field_type_name: &str,
    context: &DeriveContext,
) -> quote::Tokens {
    let argument_idents: Vec<_> = argument_idents
        .iter()
        .map(|arg| Term::new(&arg.name.to_mixed_case(), Span::call_site()))
        .collect();
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
