/// Represents a type that has an identifier [in the GraphQL sense of the term](https://graphql.org/learn/schema/#scalar-types)
///
/// It is automatically implemented for types derived with SomethingCompletelyDifferent.
pub trait Identifiable {
    fn get_id(&self) -> &str;
}
