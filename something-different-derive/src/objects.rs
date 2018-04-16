use context::DeriveContext;
use graphql_parser;
use heck::*;
use proc_macro2::{Literal, Span, Term};
use quote;
use shared;

pub fn gql_type_to_rs(
    object_type: &graphql_parser::schema::ObjectType,
    context: &DeriveContext,
) -> quote::Tokens {
    let name = Term::new(object_type.name.as_str(), Span::call_site());
    // let struct_name_lit: syn::Lit = object_type.name.as_str().into();
    let field_names: Vec<quote::Tokens> = object_type
        .fields
        .iter()
        .map(|f| {
            let ident = Term::new(&f.name.to_camel_case(), Span::call_site());
            let args: Vec<quote::Tokens> = f.arguments
                .iter()
                .map(|arg| {
                    let field_name =
                        Term::new(arg.name.to_mixed_case().as_str(), Span::call_site());
                    let field_type = shared::gql_type_to_json_type(&arg.value_type);
                    quote!( #field_name: #field_type )
                })
                .collect();
            let field_type_name = shared::extract_inner_name(&f.field_type);
            let sub_field_set: Option<Term> =
                if (context.is_scalar(field_type_name) || context.is_enum(field_type_name)) {
                    None
                } else {
                    Some(Term::new(
                        field_type_name.to_camel_case().as_str(),
                        Span::call_site(),
                    ))
                };
            let sub_field_set: Option<quote::Tokens> =
                sub_field_set.map(|set| quote!{ selection: Vec<#set>, });
            if sub_field_set.is_some() || !args.is_empty() {
                quote!{#ident { #sub_field_set #(#args),* }}
            } else {
                quote!(#ident)
            }
        })
        .collect();
    let doc_attr: quote::Tokens = if let Some(ref doc_string) = object_type.description {
        let str_literal = Literal::string(doc_string.as_str());
        quote!(#[doc = #str_literal])
    } else {
        quote!()
    };

    quote!(
        #doc_attr
        #[derive(Debug, PartialEq)]
        pub enum #name {
            #(#field_names),*
        }
    )
}
