//! These types are directly adapted from the introspection schema.
//! See https://gist.github.com/craigbeck/b90915d49fda19d5b2b17ead14dcd6da

pub struct Directive {
    pub name: &'static str,
    pub description: Option<&'static str>,
    pub locations: Vec<DirectiveLocation>,
    pub args: Vec<InputValue>,
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
    pub type_: Type,
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
    pub description: &'static str,
    pub args: Vec<InputValue>,
    pub type_: Type,
    pub is_deprecated: bool,
    pub deprecation_reason: Option<&'static str>,
}

pub struct Type {
    pub kind: TypeKind,
    pub name: Option<&'static str>,
    pub description: Option<&'static str>,
    pub fields: Vec<Field>,
}

pub struct Schema {
    pub types: Vec<Type>,
    pub query_type: Option<Type>,
    pub mutation_type: Option<Type>,
    pub subscription_type: Option<Type>,
    pub directives: Vec<Directive>,
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
