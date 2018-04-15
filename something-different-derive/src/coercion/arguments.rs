use coercion::traits::*;
use context::DeriveContext;
use graphql_parser;
use graphql_parser::schema::*;
use heck::*;
use proc_macro2::{Literal, Span, Term};
use quote;
use shared;

pub struct AbstractField {
    name: String,
    field_type: Type,
    arguments: Vec<InputValue>,
}

impl From<InputValue> for AbstractField {
    fn from(input: InputValue) -> Self {
        AbstractField {
            name: input.name,
            field_type: input.value_type,
            arguments: vec![],
        }
    }
}

impl From<Field> for AbstractField {
    fn from(field: Field) -> Self {
        AbstractField {
            name: field.name,
            field_type: field.field_type,
            arguments: field.arguments,
        }
    }
}

/// This is evaluated in the context of the ObjectType coercion impl. Please refer to that to understand where things come from.
pub struct ArgumentsContext {
    pub fields: Vec<AbstractField>,
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
                let argument_type = shared::value_variant_for_type(&arg.value_type, &context);
                let term = Term::new(&arg.name.to_mixed_case(), Span::call_site());
                let literal = Literal::string(&arg.name);

                let coercion_target = resolve_coercion_target(&arg.value_type);
                let coercion_target_type_name = &coercion_target.type_name;

                if !coercion_target.optional {
                    quote! {
                        let #term = field
                            .arguments
                            .iter()
                            .find(|(name, _)| name == #literal)
                            .and_then(|(_, value)| {
                                if let #argument_type(_) = value {
                                    <#coercion_target_type_name as ::tokio_gql::coercion::CoerceScalar>::coerce(value).ok()
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
                            .map(|(_, value)| {
                                if let #argument_type(_) = value {
                                    <#coercion_target_type_name as ::tokio_gql::coercion::CoerceScalar>::coerce(value).expect("Should be propagated as a CoercionError")
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
    if argument_idents.is_empty()
        && (context.is_scalar(field_type_name) || context.is_enum(field_type_name))
    {
        quote!(#field_name::#variant_name)
    } else if !argument_idents.is_empty()
        && !(context.is_scalar(field_type_name) || context.is_enum(field_type_name))
    {
        let field_type = Term::new(field_type_name, Span::call_site());
        quote!(#field_name::#variant_name { selection: <#field_type as ::tokio_gql::coercion::CoerceSelection>::coerce(&field.selection_set, context).unwrap(), #(#argument_idents_clone),* })
    } else if argument_idents.is_empty() {
        let field_type = Term::new(field_type_name, Span::call_site());
        quote!(#field_name::#variant_name { selection: <#field_type as ::tokio_gql::coercion::CoerceSelection>::coerce(&field.selection_set, context).unwrap() })
    } else {
        quote!(#field_name::#variant_name { #(#argument_idents_clone),* })
    }
}

/// The rust type the field should resolve to
#[derive(Debug, PartialEq)]
struct CoercionTarget {
    /// Whether this is optional *at the top level*. This is used when implementing the extractor.
    optional: bool,
    /// The *rust* type name of the target.
    type_name: quote::Tokens,
}

/// Given a schema argument, resolve what it should coerce to
fn resolve_coercion_target(arg: &graphql_parser::query::Type) -> CoercionTarget {
    resolve_coercion_target_inner(arg, true)
}

fn resolve_coercion_target_inner(
    arg: &graphql_parser::query::Type,
    optional: bool,
) -> CoercionTarget {
    use graphql_parser::query::Type;

    match arg {
        Type::ListType(inner) => {
            let inner_target = resolve_coercion_target_inner(inner, true).type_name;
            CoercionTarget {
                optional: true,
                type_name: if optional {
                    quote!(Option<Vec<#inner_target>>)
                } else {
                    quote!(Vec<#inner_target>)
                },
            }
        }
        Type::NonNullType(inner) => {
            let CoercionTarget { type_name, .. } = resolve_coercion_target_inner(inner, false);
            CoercionTarget {
                optional: false,
                type_name,
            }
        }
        Type::NamedType(inner) => {
            let term_inner = Term::new(shared::correspondant_type(inner), Span::call_site());
            CoercionTarget {
                // This is always ignored
                optional,
                type_name: if optional {
                    quote!(Option<#term_inner>)
                } else {
                    quote!(#term_inner)
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_coercion_target_works() {
        use graphql_parser::query::Type;

        macro_rules! test {
            ($input:expr => $expected:expr) => {{
                let expectation = $expected;
                let actual = resolve_coercion_target(&$input);
                assert_eq!(expectation, actual)
            }};
        }

        test!(
            Type::NamedType("Cat".to_string())
            =>
            CoercionTarget {
                optional: true,
                type_name: quote!(Option<Cat>),
            }
        );

        test!(
            Type::NonNullType(Box::new(Type::NamedType("Cat".to_string())))
            =>
            CoercionTarget {
                optional: false,
                type_name: quote!(Cat),
            }
        );

        test!(
            Type::ListType(Box::new(Type::NonNullType(Box::new(Type::NamedType("Cat".to_string())))))
            =>
            CoercionTarget {
                optional: true,
                type_name: quote!(Option<Vec<Cat>>),
            }
        );

        test!(
            Type::NonNullType(Box::new(Type::ListType(Box::new(Type::NonNullType(Box::new(Type::NamedType("Cat".to_string())))))))
            =>
            CoercionTarget {
                optional: false,
                type_name: quote!(Vec<Cat>),
            }
        );

        test!(
            Type::ListType(Box::new(Type::NamedType("Int".to_string())))
            =>
            CoercionTarget {
                optional: true,
                type_name: quote!(Option<Vec<Option<i32> >>)
            }
        );
    }
}
