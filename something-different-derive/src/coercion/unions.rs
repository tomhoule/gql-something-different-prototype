use coercion::traits::*;
use context::DeriveContext;
use graphql_parser::schema::*;
use proc_macro2::{Span, Term};
use quote;

impl ImplCoerce for UnionType {
    fn impl_coerce(&self, _context: &DeriveContext) -> quote::Tokens {
        let name_term: Term = Term::new(&self.name, Span::call_site());

        let field_matchers = self.types.iter().map(|ty| {
            let variant_term = Term::new(&format!("On{}", ty), Span::call_site());
            let ty_term = Term::new(ty, Span::call_site());
            quote! {
                if let Some(::tokio_gql::graphql_parser::query::TypeCondition::On(ref name)) = fragment.type_condition {
                    if name == #ty {
                        let coerced_inner = <#ty_term as ::tokio_gql::coercion::CoerceSelection>::coerce(&fragment.selection_set, &context);
                        results.push(#name_term::#variant_term(coerced_inner?));
                    }
                }
            }
        });

        quote! {
            impl ::tokio_gql::coercion::CoerceSelection for #name_term {
                fn coerce(
                    query: &::tokio_gql::graphql_parser::query::SelectionSet,
                    context: &::tokio_gql::query_validation::ValidationContext,
                ) -> Result<Vec<#name_term>, ::tokio_gql::coercion::CoercionError> {
                    let mut results = Vec::<#name_term>::new();
                    for selection in query.items.iter() {
                        match selection {
                            ::tokio_gql::graphql_parser::query::Selection::Field(_) => unreachable!("field on union"),
                            ::tokio_gql::graphql_parser::query::Selection::FragmentSpread(_) => unimplemented!("fragment spread on union"),
                            ::tokio_gql::graphql_parser::query::Selection::InlineFragment(fragment) => {
                                #(#field_matchers)*
                            },

                        }
                    }
                    Ok(results)
                }
            }
        }
    }
}
