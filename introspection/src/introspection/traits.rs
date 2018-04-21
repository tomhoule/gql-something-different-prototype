pub trait Introspectable {
    fn schema_json() -> &'static str;
}
