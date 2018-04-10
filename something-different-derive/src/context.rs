use graphql_parser::schema::{EnumType, InputObjectType, InterfaceType, ObjectType,
                             SchemaDefinition, UnionType};
use std::collections::{HashMap, HashSet};

pub struct DeriveContext {
    pub enum_types: HashMap<String, EnumType>,
    pub input_types: HashMap<String, InputObjectType>,
    pub interface_types: HashMap<String, InterfaceType>,
    pub object_types: Vec<ObjectType>,
    pub scalar_types: HashSet<String>,
    pub union_types: HashMap<String, UnionType>,
    schema_type: Option<SchemaDefinition>,
}

impl DeriveContext {
    pub fn new() -> DeriveContext {
        let mut scalar_types = HashSet::new();

        // See https://graphql.org/learn/schema/#scalar-types
        scalar_types.insert("Int".to_string());
        scalar_types.insert("Float".to_string());
        scalar_types.insert("String".to_string());
        scalar_types.insert("Boolean".to_string());
        scalar_types.insert("ID".to_string());

        let object_types = Vec::new();
        let input_types = HashMap::new();
        let enum_types = HashMap::new();
        let union_types = HashMap::new();
        let interface_types = HashMap::new();

        DeriveContext {
            enum_types,
            input_types,
            interface_types,
            object_types,
            scalar_types,
            schema_type: None,
            union_types,
        }
    }

    pub fn get_schema(&self) -> Option<SchemaDefinition> {
        self.schema_type.clone()
    }

    pub fn set_schema(&mut self, schema: SchemaDefinition) {
        self.schema_type = Some(schema)
    }

    pub fn insert_object(&mut self, object_type: ObjectType) {
        self.object_types.push(object_type);
    }

    pub fn insert_enum(&mut self, enum_type: EnumType) {
        self.enum_types.insert(enum_type.name.clone(), enum_type);
    }

    pub fn insert_input_type(&mut self, input_type: InputObjectType) {
        self.input_types.insert(input_type.name.clone(), input_type);
    }

    pub fn insert_scalar(&mut self, scalar_type: String) {
        self.scalar_types.insert(scalar_type);
    }

    pub fn is_scalar(&self, type_name: &str) -> bool {
        self.scalar_types.contains(type_name)
    }

    pub fn insert_union(&mut self, union_type: UnionType) {
        self.union_types.insert(union_type.name.clone(), union_type);
    }

    pub fn insert_interface(&mut self, interface_type: InterfaceType) {
        self.interface_types
            .insert(interface_type.name.clone(), interface_type);
    }
}
