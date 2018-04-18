use graphql_parser;
use graphql_parser::query::*;
use graphql_parser::schema;
use json;
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct ValidationContext {
    fragment_definitions: Vec<FragmentDefinition>,
    variable_definitions: Vec<VariableDefinition>,
    variables: json::Map<String, json::Value>,
}

impl ValidationContext {
    pub fn new(variables: json::Map<String, json::Value>) -> ValidationContext {
        let fragment_definitions = Vec::new();
        let variable_definitions = Vec::new();
        ValidationContext {
            fragment_definitions,
            variable_definitions,
            variables,
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
    #[fail(display = "The following variable was not provided: {}", name)]
    MissingVariable { name: String },
    #[fail(
        display = "Other error (if you see this it is a bug, a report would be very appreciated)"
    )]
    Other,
    #[fail(display = "Variable mismatch")]
    VariableMismatch,
}

pub fn validate_query(
    query: &graphql_parser::query::Document,
    variables: json::Map<String, json::Value>,
    schema: &graphql_parser::schema::Document,
) -> Result<ValidationContext, QueryValidationError> {
    let mut context = ValidationContext::new(variables);

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
                        validate_variables(&mut context.variables, &q.variable_definitions, &schema)?;
                        find_by_name(&schema.definitions, name)?
                            .validate_selection_set(&q.selection_set, &schema, &context)?;
                    }
                    None => {
                        return Err(QueryValidationError::InvalidOperation { operation: "query" })
                    }
                },
                OperationDefinition::Mutation(ref m) => match &schema_definition.mutation {
                    Some(name) => {
                        validate_variables(&mut context.variables, &m.variable_definitions, &schema)?;
                        find_by_name(&schema.definitions, name)?
                            .validate_selection_set(&m.selection_set, &schema, &context)?;
                    }
                    None => {
                        return Err(QueryValidationError::InvalidOperation {
                            operation: "mutation",
                        })
                    }
                },
                OperationDefinition::Subscription(s) => match &schema_definition.subscription {
                    Some(name) => {
                        validate_variables(&mut context.variables, &s.variable_definitions, &schema)?;
                        find_by_name(&schema.definitions, name)?
                            .validate_selection_set(&s.selection_set, &schema, &context)?;
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
        context: &ValidationContext,
    ) -> Result<(), QueryValidationError>;
}

impl Selectable for schema::TypeDefinition {
    fn validate_selection_set(
        &self,
        set: &SelectionSet,
        schema: &graphql_parser::schema::Document,
        context: &ValidationContext,
    ) -> Result<(), QueryValidationError> {
        match self {
            schema::TypeDefinition::Object(obj) => obj.validate_selection_set(set, &schema, &context),
            _ => unimplemented!(),
        }
    }
}

impl Selectable for schema::ObjectType {
    fn validate_selection_set(
        &self,
        set: &SelectionSet,
        schema: &graphql_parser::schema::Document,
        context: &ValidationContext,
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

                    validate_argument_types(&field.arguments, &schema_field.arguments, &context)?;

                    let inner_name = ::shared::extract_inner_name(&schema_field.field_type);
                    let field_type = find_by_name(&schema.definitions, inner_name).ok();
                    if let Some(field_type) = field_type {
                        field_type.validate_selection_set(&field.selection_set, &schema, &context)?;
                    }
                }
                _ => return Err(QueryValidationError::InvalidSelectionSet(set.clone())),
            }
        }

        Ok(())
    }
}

fn validate_argument_types(
    field_arguments: &[(String, graphql_parser::query::Value)],
    schema_arguments: &[graphql_parser::schema::InputValue],
    context: &ValidationContext,
) -> Result<(), QueryValidationError> {
    use graphql_parser::query::Value;
    use graphql_parser::schema::Type;

    for (name, value) in field_arguments {
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
            Value::Object(_obj) => true,
            Value::Variable(variable_name) => {
                context.variables.contains_key(variable_name)
                // TODO: Validate that the variable is the right type.
            },
            Value::Enum(_) => unimplemented!("Enum validation"),
            Value::Null | Value::List(_) => true,
        };

        if !valid {
            return Err(QueryValidationError::InvalidFieldArguments);
        }
    }
    Ok(())
}

pub fn type_matches(
    variable: &json::Value,
    type_name: &str,
    schema: &graphql_parser::schema::Document,
) -> Result<(), QueryValidationError> {
    use json::Value;

    match variable {
        Value::Array(_) => Err(QueryValidationError::VariableMismatch), // arrays are already handled in validate_variable
        Value::Bool(b) => if type_name == "Boolean" {
            Ok(())
        } else {
            Err(QueryValidationError::VariableMismatch)
        },
        Value::Null => Ok(()),
        Value::Number(num) => if type_name == "Float" {
            num.as_f64()
                .ok_or(QueryValidationError::VariableMismatch)
                .map(|_| ())
        } else if type_name == "Int" {
            num.as_i64()
                .ok_or(QueryValidationError::VariableMismatch)
                .map(|_| ())
        } else {
            Err(QueryValidationError::VariableMismatch)
        },
        Value::String(s) => if type_name == "String" {
            Ok(())
        } else {
            Err(QueryValidationError::VariableMismatch)
        },
        Value::Object(_) => unimplemented!("object variable validation"),
    }
}

pub fn validate_variable(
    variable: &json::Value,
    expected_type: &graphql_parser::schema::Type,
    schema: &graphql_parser::schema::Document,
) -> Result<(), QueryValidationError> {
    use graphql_parser::schema::Type;

    match expected_type {
        Type::NamedType(name) => type_matches(variable, name, schema),
        Type::NonNullType(inner) => {
            if let json::Value::Null = variable {
                Err(QueryValidationError::MissingVariable {
                    name: "<unavailable>".to_string(),
                })
            } else {
                validate_variable(variable, inner, schema)
            }
        }
        Type::ListType(elem_type) => match variable {
            json::Value::Array(inner) => {
                for value in inner.iter() {
                    let _ = validate_variable(value, elem_type, schema)?;
                }
                Ok(())
            }
            _ => Err(QueryValidationError::VariableMismatch)?,
        },
    }
}

fn validate_variables(
    variables: &mut json::Map<String, json::Value>,
    definitions: &[VariableDefinition],
    schema: &graphql_parser::schema::Document,
) -> Result<(), QueryValidationError> {
    use graphql_parser::schema::Type;

    let mut default_values = HashMap::new();

    for definition in definitions.iter() {
        match (
            &definition.var_type,
            variables.get(&definition.name),
            &definition.default_value,
        ) {
            (_, Some(val), _) => validate_variable(val, &definition.var_type, schema)?,
            (_, None, Some(val)) => {
                default_values.insert(definition.name.to_string(), query_value_to_json(val)?);
            }
            (Type::NonNullType(_), None, None) => Err(QueryValidationError::MissingVariable {
                name: definition.name.to_string(),
            })?,
            (_, None, None) => (),
        }
    }

    variables.extend(default_values);

    Ok(())
}

pub fn query_value_to_json(value: &graphql_parser::query::Value) -> Result<json::Value, QueryValidationError> {
    use graphql_parser::query::Value;

    match value {
        Value::Boolean(b) => Ok(json::Value::Bool(*b)),
        Value::Enum(variant) => Ok(json::Value::String(variant.to_string())),
        Value::Float(n) => Ok(json!(n)),
        Value::Int(n) => { let n = n.as_i64().unwrap(); Ok(json!(n)) },
        Value::String(s) => Ok(json!(s)),
        Value::Variable(_) => unreachable!("variable in variable definition"),
        Value::List(items) => {
            let inner: Result<Vec<json::Value>, _> = items.iter().map(query_value_to_json).collect();
            let inner = inner?;
            Ok(json::Value::Array(inner))
        },
        Value::Object(object) => {
            let map: Result<json::Map<_, _>, _> = object.iter().map(|(k, v)| {
                let json_v = query_value_to_json(v)?;
                Ok((k.to_string(), json_v))
            }).collect();
            Ok(json::Value::Object(map?))
        },
        Value::Null => Ok(json!(null)),
    }
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

            assert_eq!(
                validate_query(&parsed_query, json::Map::new(), &parsed_schema),
                expected
            );
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
            Ok(ValidationContext::new(json::Map::new()))
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

    #[test]
    fn query_value_to_json_works() {
        use graphql_parser::query::Value;

        let cases = vec![
            (Value::Boolean(true), json::Value::Bool(true)),
            (Value::Float(33.4), json!(33.4)),
            (Value::Null, json!(null)),
            (Value::String("Ravelociraptor".to_string()), json!("Ravelociraptor")),
        ];

        for case in cases {
            assert_eq!(query_value_to_json(&case.0).unwrap(), case.1);
        }
    }
}
