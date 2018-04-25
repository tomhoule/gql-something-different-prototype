use serde_json;

pub trait IntoJson {
    fn into_json(self) -> serde_json::Value;
}

impl IntoJson for String {
    fn into_json(self) -> serde_json::Value {
        serde_json::Value::String(self)
    }
}
impl IntoJson for bool {
    fn into_json(self) -> serde_json::Value {
        serde_json::Value::Bool(self)
    }
}

impl IntoJson for i32 {
    fn into_json(self) -> serde_json::Value {
        serde_json::Value::Number(self.into())
    }
}

impl IntoJson for f64 {
    fn into_json(self) -> serde_json::Value {
        serde_json::Value::Number(serde_json::Number::from_f64(self).unwrap_or(0.into()))
    }
}

impl<T: IntoJson> IntoJson for Option<T> {
    fn into_json(self) -> serde_json::Value {
        match self {
            Some(value) => value.into_json(),
            None => serde_json::Value::Null,
        }
    }
}
