use graphql_parser::query::*;

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
}
