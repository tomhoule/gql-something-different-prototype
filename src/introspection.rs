//! These types are directly adapted from the introspection schema.
//! See https://github.com/facebook/graphql/blob/master/spec/Section%204%20--%20Introspection.md

pub struct Directive {
    pub name: &'static str,
    pub description: Option<&'static str>,
    pub locations: &'static [DirectiveLocation],
    pub args: &'static [InputValue],
}

pub enum TypeKind {
    Scalar,
    Object,
    Interface,
    Union,
    Enum,
    InputObject,
    List,
    NonNull,
}

pub struct InputValue {
    pub name: &'static str,
    pub description: Option<&'static str>,
    pub type_: &'static str,
    pub default_value: Option<&'static str>,
}

pub struct EnumValue {
    pub name: &'static str,
    pub description: Option<&'static str>,
    pub is_deprecated: bool,
    pub deprecation_reason: Option<&'static str>,
}

pub struct Field {
    pub name: &'static str,
    pub description: Option<&'static str>,
    pub args: &'static [InputValue],
    pub type_: &'static str,
    pub is_deprecated: bool,
    pub deprecation_reason: Option<&'static str>,
}

pub struct Type {
    pub kind: TypeKind,
    pub name: Option<&'static str>,
    pub description: Option<&'static str>,
    pub fields: &'static [Field],
}

pub struct Schema {
    pub types: &'static [Type],
    pub query_type: Option<&'static str>,
    pub mutation_type: Option<&'static str>,
    pub subscription_type: Option<&'static str>,
    pub directives: &'static [Directive],
}

pub enum DirectiveLocation {
    Query,
    Mutation,
    Subscription,
    Field,
    FragmentDefinition,
    FragmentSpread,
    InlineFragment,
    Schema,
    Scalar,
    Object,
    FieldDefinition,
    ArgumentDefinition,
    Interface,
    Union,
    Enum,
    EnumValue,
    InputObject,
    InputFieldDefinition,
}
