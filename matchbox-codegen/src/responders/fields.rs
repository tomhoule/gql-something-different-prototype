use super::traits::ImplResponder;
use context::DeriveContext;
use graphql_parser::schema;
use heck::*;
use proc_macro2::{Span, Term};
use quote;

impl ImplResponder for schema::Field {
    fn impl_responder(&self, context: &DeriveContext) -> quote::Tokens {
        let responder_name = Term::new(
            &::shared::schema_name_to_responder_name(&self.name),
            Span::call_site(),
        );

        let expanded = field_impl_inner(
            &responder_name,
            &self.name,
            &self.field_type,
            context,
            false,
        );

        quote!{ #expanded }
    }
}

fn field_impl_inner(
    responder_name: &Term,
    field_name: &str,
    ty: &schema::Type,
    context: &DeriveContext,
    non_nullable: bool,
) -> quote::Tokens {
    match ty {
        schema::Type::NonNullType(inner) => {
            field_impl_inner(responder_name, field_name, inner, context, true)
        }
        schema::Type::ListType(inner) => list_responder_impl(responder_name, inner),
        schema::Type::NamedType(name) => {
            if context.is_scalar(&name) {
                scalar_responder_impl(responder_name, field_name, name, context, non_nullable)
            } else if context.is_enum(&name) {
                unimplemented!("enum field responders")
            } else {
                // object
                unimplemented!("object field responders")
            }
        }
    }
}

fn list_responder_impl(responder_name: &Term, inner_type: &schema::Type) -> quote::Tokens {
    quote! {
        pub struct #responder_name;

        impl #responder_name {
            fn to_each<Reponder, ResponderFuture>(selection: Vec<Never>, responder: Responder) -> Never
            where
                Responder: Fn(Never) -> ResponderFuture
                ResponderFuture: Future<Item = ::tokio_gql::response::Response, Error = ::tokio_gql::errors::ResolverError> {
                    unimplemented!("list field responder")
                }
        }
    }
}

fn scalar_responder_impl(
    responder_name: &Term,
    field_name: &str,
    ty: &str,
    context: &DeriveContext,
    non_nullable: bool,
) -> quote::Tokens {
    let param_type = if non_nullable {
        schema::Type::NonNullType(Box::new(schema::Type::NamedType(ty.to_string())))
    } else {
        schema::Type::NamedType(ty.to_string())
    };
    let rust_ty = ::shared::graphql_type_to_response_type(&param_type, context);
    quote! {
        pub struct #responder_name;

        impl #responder_name {
            fn with(value: #rust_ty) -> ::tokio_gql::response::Response {
                ::tokio_gql::response::Response::Immediate((#field_name , value))
            }
        }
    }
}
