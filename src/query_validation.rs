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

    pub fn push_fragment_definition(&mut self, definition: &FragmentDefinition) {
        self.fragment_definitions.push(definition.clone());
    }
}

#[derive(Debug, PartialEq)]
pub enum QueryValidationError {
    InvalidSelectionSet(SelectionSet),
    UnknownDirective(Directive),
    InvalidField,
    InvalidFieldArguments,
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
                    Some(_q) => unimplemented!(),
                    None => return Err(QueryValidationError::InvalidField),
                },
                OperationDefinition::Mutation(ref q) => match &schema_definition.mutation {
                    Some(_m) => unimplemented!(),
                    None => return Err(QueryValidationError::InvalidField),
                },
                OperationDefinition::Subscription(q) => match &schema_definition.subscription {
                    Some(_s) => unimplemented!(),
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
}
