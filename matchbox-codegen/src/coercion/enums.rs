use coercion::traits::*;
use context::DeriveContext;
use graphql_parser::schema::*;
use heck::*;
use proc_macro2::{Span, Term};
use quote;

impl ImplCoerce for EnumType {
    fn impl_coerce(&self, _context: &DeriveContext) -> quote::Tokens {
        let name_term = Term::new(&self.name, Span::call_site());
        let matchers = self.values.iter().map(|value| {
            let value_term = Term::new(&value.name.to_camel_case(), Span::call_site());
            let value_lit = &value.name;
            quote! {
                if value == #value_lit {
                    return Ok(#name_term::#value_term)
                };
            }
        });

        quote! {
            impl ::tokio_gql::coercion::CoerceScalar for #name_term {
                fn coerce(
                    query: &::tokio_gql::graphql_parser::query::Value,
                ) -> Result<#name_term, ::tokio_gql::coercion::CoercionError> {
                    if let ::tokio_gql::graphql_parser::query::Value::Enum(value) = query {
                        #(#matchers)*
                    }

                    Err(::tokio_gql::coercion::CoercionError)
                }
            }
        }
    }
}
