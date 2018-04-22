use super::traits::ImplResponder;
use context::DeriveContext;
use graphql_parser::schema;
use proc_macro2::{Span, Term};
use quote;

impl ImplResponder for schema::ObjectType {
    fn impl_responder(&self, context: &DeriveContext) -> quote::Tokens {
        let response_name = ::shared::schema_name_to_response_name(&self.name);
        let field_names: Vec<_> = self.fields
            .iter()
            .map(|field| Term::new(&field.name, Span::call_site()))
            .collect();
        let field_types = self.fields
            .iter()
            .map(|field| ::shared::graphql_type_to_response_type(&field.field_type, context));

        quote! {
            #[derive(Serialize)]
            struct #response_name {
                #(
                    #[serde(serialize_with = "serialize_response_field")]
                    #field_names: Option<#field_types>,
                )*
            }
        }
    }
}
