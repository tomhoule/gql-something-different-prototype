use coercion::arguments::ArgumentsContext;
use coercion::traits::*;
use context::DeriveContext;
use graphql_parser::schema::*;
use proc_macro2::{Span, Term};
use quote;
use shared;

impl ImplCoerce for InputObjectType {
    fn impl_coerce(&self, context: &DeriveContext) -> quote::Tokens {
        let name = Term::new(&self.name, Span::call_site());
        let field_name_literals: Vec<String> =
            self.fields.iter().map(|i| i.name.to_string()).collect();

        let field_name_terms: Vec<Term> = self.fields
            .iter()
            .map(|i| Term::new(&i.name, Span::call_site()))
            .collect();

        let field_types: Vec<_> = self.fields
            .iter()
            .map(|i| shared::gql_type_to_json_type(&i.value_type))
            .collect();

        let extractors: Vec<quote::Tokens> = self.fields
            .iter()
            .enumerate()
            .map(|(idx, f)| {
                let value_variant = shared::value_variant_for_type(&f.value_type, context);
                let field_type = &field_types[idx];
                let field_name_term = &field_name_terms[idx];
                let field_name_literal = &field_name_literals[idx];
                if shared::type_is_optional(&f.value_type) {
                    quote!{
                        let #field_name_term = obj.get(#field_name_literal)
                            .and_then(|value| {
                                if let #value_variant(_) = value {
                                    <#field_type as ::tokio_gql::coercion::CoerceScalar>::coerce(value).expect("should be fine")
                                } else {
                                    None
                                }
                            });
                    }
                } else {
                    quote!{
                        let #field_name_term = obj.get(#field_name_literal)
                            .and_then(|value| {
                                if let #value_variant(_) = value {
                                    <#field_type as ::tokio_gql::coercion::CoerceScalar>::coerce(value).ok()
                                } else {
                                    None
                                }
                            }).ok_or(::tokio_gql::coercion::CoercionError)?;
                    }
                }
            })
            .collect();
        let extractors = quote!(#(#extractors)*);

        let object_constructor = {
            let name = name.clone();
            let fields = field_name_terms.clone();
            quote!(#name { #(#fields),* })
        };

        let field_name_literals_clone = field_name_literals.clone();

        quote! {
            impl ::tokio_gql::coercion::CoerceScalar for #name {
                fn coerce(
                    query: &::tokio_gql::graphql_parser::query::Value,
                ) -> Result<#name, ::tokio_gql::coercion::CoercionError> {
                    if let ::tokio_gql::graphql_parser::query::Value::Object(obj) = query {

                        #extractors

                        Ok(#object_constructor)
                    } else {
                        Err(::tokio_gql::coercion::CoercionError)
                    }
                }
            }
        }
    }
}
