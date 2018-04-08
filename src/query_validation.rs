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
    #[fail(display = "Unknown directive")]
    UnknownDirective(Directive),
    #[fail(display = "Invalid field")]
    InvalidField,
    #[fail(display = "Invalid field arguments")]
    InvalidFieldArguments,
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
                        find_by_name(&schema.definitions, name)
                            .ok_or(QueryValidationError::InvalidField)?
                            .validate_selection_set(&q.selection_set)?;
                    }
                    None => return Err(QueryValidationError::InvalidField),
                },
                OperationDefinition::Mutation(ref m) => match &schema_definition.mutation {
                    Some(name) => {
                        context.extend_variable_definitions(m.variable_definitions.clone());
                        find_by_name(&schema.definitions, name)
                            .ok_or(QueryValidationError::InvalidField)?
                            .validate_selection_set(&m.selection_set)?;
                    }
                    None => return Err(QueryValidationError::InvalidField),
                },
                OperationDefinition::Subscription(s) => match &schema_definition.subscription {
                    Some(name) => {
                        context.extend_variable_definitions(s.variable_definitions.clone());
                        find_by_name(&schema.definitions, name)
                            .ok_or(QueryValidationError::InvalidField)?
                            .validate_selection_set(&s.selection_set)?;
                    }
                    None => return Err(QueryValidationError::InvalidField),
                },
                OperationDefinition::SelectionSet(q) => unimplemented!(),
            },
            Definition::Fragment(def) => {
                context.push_fragment_definition(def);
            }
        }
    }

    Ok(context)
}

fn find_by_name(definitions: &[schema::Definition], name: &str) -> Option<impl Selectable> {
    for definition in definitions.iter() {
        match definition {
            schema::Definition::TypeDefinition(schema::TypeDefinition::Object(def))
                if def.name == name =>
            {
                return Some(def.clone())
            }
            _ => (),
        }
    }

    None
}

trait Selectable {
    fn validate_selection_set(&self, set: &SelectionSet) -> Result<(), QueryValidationError>;
}

impl Selectable for schema::TypeDefinition {
    fn validate_selection_set(&self, set: &SelectionSet) -> Result<(), QueryValidationError> {
        match self {
            schema::TypeDefinition::Object(obj) => obj.validate_selection_set(set),
            _ => unimplemented!(),
        }
    }
}

impl Selectable for schema::ObjectType {
    fn validate_selection_set(&self, set: &SelectionSet) -> Result<(), QueryValidationError> {
        println!(
            "trying to validate selection set {:?}\n\n on \n{:?}\n\n",
            set, self
        );
        for selected in set.items.iter() {
            match selected {
                Selection::Field(field) => {
                    self.fields
                        .iter()
                        .find(|f| f.name == field.name)
                        .ok_or_else(|| QueryValidationError::InvalidSelectionSet(set.clone()))?;
                }
                _ => return Err(QueryValidationError::InvalidSelectionSet(set.clone())),
            }
        }

        Ok(())
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
            Err(QueryValidationError::InvalidField)
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
}
