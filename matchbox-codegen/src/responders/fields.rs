use super::traits::ImplResponder;
use context::DeriveContext;
use graphql_parser::schema;
use heck::*;
use proc_macro2::{Span, Term};
use quote;

// impl ImplResponder for schema::Field {
//     fn impl_responder(&self, context: &DeriveContext) -> quote::Tokens {
//         let responder_name = Term::new(
//             &::shared::schema_name_to_responder_name(&self.name.to_camel_case()),
//             Span::call_site(),
//         );

//         let expanded = field_impl_inner(
//             &responder_name,
//             &self.name,
//             &self.field_type,
//             context,
//             false,
//         );

//         quote!{ #expanded }
//     }
// }

pub(crate) fn impl_field(
    responder_name: &str,
    field_name: &str,
    ty: &schema::Type,
    context: &DeriveContext,
) -> quote::Tokens {
    let responder_name = Term::new(responder_name, Span::call_site());
    field_impl_inner(&responder_name, field_name, ty, context, false)
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
                quote! {
                    #[derive(Debug, PartialEq)]
                    pub struct #responder_name;
                    trivial_default_impl!(#responder_name, #responder_name);
                }
            } else {
                let object_responder_name = Term::new(
                    &::shared::schema_name_to_responder_name(name),
                    Span::call_site(),
                );
                quote!{
                    #[derive(Debug, PartialEq)]
                    pub struct #responder_name;
                    trivial_default_impl!(#responder_name, #responder_name);

                    impl #responder_name {
                        pub fn to(selection: ..., resolver: ...) -> impl ::futures::Future<Item = (), Error = ::tokio_gql::errors::ResolverError> {
                            #object_responder_name::default().to(selection, resolver).and_then(|json| {
                                (..., json)
                            })
                        }
                    }
                    // #object_responder_name;
                }
                // let object_type = context
                //     .object_types
                //     .iter()
                //     .find(|ty| ty.name == name.as_str());
                // let interface_type = context
                //     .interface_types
                //     .values()
                //     .find(|ty| ty.name == name.as_str());
                // let union_type = context
                //     .union_types
                //     .values()
                //     .find(|ty| ty.name == name.as_str());
                // if object_type.is_none() && interface_type.is_none() && union_type.is_none() {
                //     panic!("No declaration found for field type {}", name);
                // }

                // if let Some(object_type) = object_type {
                //     return object_type.impl_responder(context);
                // }

                // if let Some(interface_type) = interface_type {
                //     return interface_type.impl_responder(context);
                // }

                // if let Some(union_type) = union_type {
                //     return union_type.impl_responder(context);
                // }

                // unreachable!();
            }
        }
    }
}

fn list_responder_impl(responder_name: &Term, inner_type: &schema::Type) -> quote::Tokens {
    quote! {
        #[derive(Debug, PartialEq)]
        pub struct #responder_name;
        trivial_default_impl!(#responder_name, #responder_name);

        // impl #responder_name {
        //     fn to_each<Reponder, ResponderFuture>(selection: Vec<Never>, responder: Responder) -> Never
        //     where
        //         Responder: Fn(Never) -> ResponderFuture
        //         ResponderFuture: Future<Item = ::tokio_gql::response::Response, Error = ::tokio_gql::errors::ResolverError> {
        //             unimplemented!("list field responder")
        //         }
        // }
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
        #[derive(Debug, PartialEq)]
        pub struct #responder_name;
        trivial_default_impl!(#responder_name, #responder_name);

        impl #responder_name {
            pub fn with(&self, value: #rust_ty) -> ::tokio_gql::response::Response {
                ::tokio_gql::response::Response::Immediate((#field_name , <#rust_ty as ::tokio_gql::traits::IntoJson>::into_json(value)))
            }
        }
    }
}
