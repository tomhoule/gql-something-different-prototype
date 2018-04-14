use graphql_parser;
use graphql_parser::query::*;
use graphql_parser::schema;

#[derive(Debug, PartialEq)]
pub struct ValidationContext {
    fragment_definitions: Vec<FragmentDefinition>,
    variable_definitions: Vec<VariableDefinition>,
}

impl ValidationContext {
    pub fn new() -> ValidationContext {
        let fragment_definitions = Vec::new();
        let variable_definitions = Vec::new();
        ValidationContext {
            fragment_definitions,
            variable_definitions,
        }
    }

    pub fn extend_variable_definitions(
        &mut self,
        defs: impl IntoIterator<Item = VariableDefinition>,
    ) {
        self.variable_definitions.extend(defs.into_iter());
    }

    pub fn push_fragment_definition(&mut self, definition: &FragmentDefinition) {
        self.fragment_definitions.push(definition.clone());
    }
}

#[derive(Debug, PartialEq, Fail)]
pub enum QueryValidationError {
    #[fail(display = "Invalid selection set")]
    InvalidSelectionSet(SelectionSet),
    #[fail(display = "Unknown directive {}", directive)]
    UnknownDirective { directive: Directive },
    #[fail(display = "Invalid field")]
    InvalidField,
    #[fail(display = "Invalid field arguments")]
    InvalidFieldArguments,
    #[fail(display = "This operation is not defined for the schema: {}", operation)]
    InvalidOperation { operation: &'static str },
    #[fail(display = "Missing definition")]
    MissingDefinition,
    #[fail(display = "Other error (if you see this it is a bug, a report would be very appreciated)")]
    Other,
}

pub fn validate_query(
    query: &graphql_parser::query::Document,
    schema: &graphql_parser::schema::Document,
) -> Result<ValidationContext, QueryValidationError> {
    let mut context = ValidationContext::new();

    let schema_definition = schema
        .definitions
        .iter()
        .filter_map(|def| {
            if let schema::Definition::SchemaDefinition(sd) = def {
                Some(sd)
            } else {
                None
            }
        })
        .next()
        .ok_or(QueryValidationError::Other)?;

    for definition in query.definitions.iter() {
        match definition {
            Definition::Operation(op) => match op {
                OperationDefinition::Query(ref q) => match &schema_definition.query {
                    Some(name) => {
                        context.extend_variable_definitions(q.variable_definitions.clone());
                        find_by_name(&schema.definitions, name)?
                            .validate_selection_set(&q.selection_set, &schema)?;
                    }
                    None => {
                        return Err(QueryValidationError::InvalidOperation { operation: "query" })
                    }
                },
                OperationDefinition::Mutation(ref m) => match &schema_definition.mutation {
                    Some(name) => {
                        context.extend_variable_definitions(m.variable_definitions.clone());
                        find_by_name(&schema.definitions, name)?
                            .validate_selection_set(&m.selection_set, &schema)?;
                    }
                    None => {
                        return Err(QueryValidationError::InvalidOperation {
                            operation: "mutation",
                        })
                    }
                },
                OperationDefinition::Subscription(s) => match &schema_definition.subscription {
                    Some(name) => {
                        context.extend_variable_definitions(s.variable_definitions.clone());
                        find_by_name(&schema.definitions, name)?
                            .validate_selection_set(&s.selection_set, &schema)?;
                    }
                    None => {
                        return Err(QueryValidationError::InvalidOperation {
                            operation: "subscription",
                        })
                    }
                },
                OperationDefinition::SelectionSet(_) => {
                    return Err(QueryValidationError::InvalidOperation {
                        operation: "selection set",
                    })
                }
            },
            Definition::Fragment(def) => {
                context.push_fragment_definition(def);
            }
        }
    }

    Ok(context)
}

fn find_by_name(
    definitions: &[schema::Definition],
    name: &str,
) -> Result<impl Selectable, QueryValidationError> {
    for definition in definitions.iter() {
        match definition {
            schema::Definition::TypeDefinition(schema::TypeDefinition::Object(def))
                if def.name == name =>
            {
                return Ok(def.clone())
            }
            _ => (),
        }
    }

    Err(QueryValidationError::MissingDefinition)
}

trait Selectable {
    fn validate_selection_set(
        &self,
        set: &SelectionSet,
        schema: &graphql_parser::schema::Document,
    ) -> Result<(), QueryValidationError>;
}

impl Selectable for schema::TypeDefinition {
    fn validate_selection_set(
        &self,
        set: &SelectionSet,
        schema: &graphql_parser::schema::Document,
    ) -> Result<(), QueryValidationError> {
        match self {
            schema::TypeDefinition::Object(obj) => obj.validate_selection_set(set, &schema),
            _ => unimplemented!(),
        }
    }
}

impl Selectable for schema::ObjectType {
    fn validate_selection_set(
        &self,
        set: &SelectionSet,
        schema: &graphql_parser::schema::Document,
    ) -> Result<(), QueryValidationError> {
        for selected in set.items.iter() {
            match selected {
                Selection::Field(field) => {
                    let schema_field = self.fields
                        .iter()
                        .find(|f| f.name == field.name)
                        .ok_or_else(|| QueryValidationError::InvalidSelectionSet(set.clone()))?;

                    let mut required_arguments = schema_field.arguments.iter().filter(|arg| {
                        matches!(arg.value_type, graphql_parser::schema::Type::NonNullType(_))
                    });

                    if required_arguments.any(|arg| {
                        !field.arguments.iter().any(|(name, value)| {
                            name.as_str() == arg.name.as_str()
                                && value != &graphql_parser::query::Value::Null
                        })
                    }) {
                        return Err(QueryValidationError::InvalidFieldArguments);
                    }

                    validate_argument_types(&field.arguments, &schema_field.arguments)?;

                    let inner_name = ::shared::extract_inner_name(&schema_field.field_type);
                    let field_type = find_by_name(&schema.definitions, inner_name).ok();
                    if let Some(field_type) = field_type {
                        field_type.validate_selection_set(&field.selection_set, &schema)?;
                    }
                }
                _ => return Err(QueryValidationError::InvalidSelectionSet(set.clone())),
            }
        }

        Ok(())
    }
}

fn validate_argument_types(
    query_arguments: &[(String, graphql_parser::query::Value)],
    schema_arguments: &[graphql_parser::schema::InputValue],
) -> Result<(), QueryValidationError> {
    use graphql_parser::query::Value;
    use graphql_parser::schema::Type;

    for (name, value) in query_arguments {
        let schema_argument = schema_arguments
            .iter()
            .find(|arg| arg.name.as_str() == name.as_str())
            .ok_or(QueryValidationError::InvalidFieldArguments)?;

        // Validate listness of arguments
        if let Value::List(_) = value {
            if !matches!(schema_argument.value_type, Type::ListType(_)) {
                return Err(QueryValidationError::InvalidFieldArguments);
            }
        }

        let valid = match value {
            Value::Boolean(_) => {
                ::shared::extract_inner_name(&schema_argument.value_type) == "Boolean"
            }
            Value::Float(_) => ::shared::extract_inner_name(&schema_argument.value_type) == "Float",
            Value::Int(_) => ::shared::extract_inner_name(&schema_argument.value_type) == "Int",
            Value::String(_) => {
                ::shared::extract_inner_name(&schema_argument.value_type) == "String"
            }
            // TODO: implement input object literals validation
            Value::Object(obj) => true,
            Value::Variable(_) => unimplemented!("Variable validation"),
            Value::Enum(_) => unimplemented!("Enum validation"),
            Value::Null | Value::List(_) => true,
        };

        if !valid {
            return Err(QueryValidationError::InvalidFieldArguments);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_validates {
        ($query:expr, $schema:expr => $expected:expr) => {
            let query = $query;
            let schema = $schema;
            let expected = $expected;

            let parsed_query = graphql_parser::parse_query(query).unwrap();
            let parsed_schema = graphql_parser::parse_schema(schema).unwrap();

            assert_eq!(validate_query(&parsed_query, &parsed_schema), expected);
        };
    }

    #[test]
    fn inexistent_operation_query() {
        assert_validates! {
            r##"
            mutation {
                changeFruit(name: "Tomato")
            }
            "##,
            r##"
            type FruitQuery {
                color: String
                shape: String
            }

            schema {
                query: FruitQuery
            }
            "## =>
            Err(QueryValidationError::InvalidOperation { operation: "mutation" })
        }
    }

    #[test]
    fn minimal_valid_query() {
        assert_validates! {
            r##"
            query {
                dogs {
                    name
                    age
                    furDensity
                    barks
                }
            }
            "##,
            r##"
            type Dog {
                name: String!
                age: Int
                furDensity: Int
                barks: Boolean
            }

            type Query {
                dogs: [Dog!]!
            }

            schema {
                query: Query
            }
            "## =>
            Ok(ValidationContext::new())
        }
    }

    #[test]
    fn missing_arguments() {
        assert_validates! {
            r##"
            query {
                dogs {
                    name
                    age
                    furDensity
                    barks
                }
            }
            "##,
            r##"
            type Dog {
                name: String!
                age(dogYears: Boolean!): Int
                furDensity: Int
                barks: Boolean
            }

            type Query {
                dogs: [Dog!]!
            }

            schema {
                query: Query
            }
            "## =>
            Err(QueryValidationError::InvalidFieldArguments)
        }
    }

    #[test]
    fn wrong_argument_type() {
        assert_validates! {
            r##"
            query {
                dogs {
                    age(dogYears: 10)
                    furDensity
                }
            }
            "##,
            r##"
            type Dog {
                name: String!
                age(dogYears: Boolean!): Int
                furDensity: Int
                barks: Boolean
            }

            type Query {
                dogs: [Dog!]!
            }

            schema {
                query: Query
            }
            "## =>
            Err(QueryValidationError::InvalidFieldArguments)
        }
    }

    #[test]
    fn wrong_argument_listness() {
        assert_validates! {
            r##"
            query {
                dogs {
                    age(dogYears: [true])
                    furDensity
                }
            }
            "##,
            r##"
            type Dog {
                name: String!
                age(dogYears: Boolean!): Int
                furDensity: Int
                barks: Boolean
            }

            type Query {
                dogs: [Dog!]!
            }

            schema {
                query: Query
            }
            "## =>
            Err(QueryValidationError::InvalidFieldArguments)
        }
    }
}
