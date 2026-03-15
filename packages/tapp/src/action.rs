use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    name: String,
    data: HashMap<String, Value>,
}

impl Action {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            data: HashMap::new(),
        }
    }

    pub fn with_data(mut self, key: impl Into<String>, value: impl Serialize) -> Self {
        self.data.insert(
            key.into(),
            serde_json::to_value(value).unwrap_or(Value::Null),
        );
        self
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Result<T, serde_json::Error> {
        let value = self.data.get(key).cloned().unwrap_or(Value::Null);
        serde_json::from_value(value)
    }

    pub fn get_optional<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T> {
        self.data
            .get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    pub fn get_string(&self, key: &str) -> Option<String> {
        self.data.get(key).and_then(|v| v.as_str()).map(|s| s.to_string())
    }

    pub fn get_i64(&self, key: &str) -> Option<i64> {
        self.data.get(key).and_then(|v| v.as_i64())
    }

    pub fn get_f64(&self, key: &str) -> Option<f64> {
        self.data.get(key).and_then(|v| v.as_f64())
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.data.get(key).and_then(|v| v.as_bool())
    }

    pub fn raw_data(&self) -> &HashMap<String, Value> {
        &self.data
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub status: ResponseStatus,
    pub data: Option<Value>,
    pub render: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ResponseStatus {
    Ok,
    Error,
    NotFound,
}

impl Response {
    pub fn ok() -> Self {
        Self {
            status: ResponseStatus::Ok,
            data: None,
            render: false,
        }
    }

    pub fn render() -> Self {
        Self {
            status: ResponseStatus::Ok,
            data: None,
            render: true,
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            status: ResponseStatus::Error,
            data: Some(Value::String(message.into())),
            render: false,
        }
    }

    pub fn not_found() -> Self {
        Self {
            status: ResponseStatus::NotFound,
            data: None,
            render: false,
        }
    }

    pub fn with_data(mut self, data: impl Serialize) -> Self {
        self.data = Some(serde_json::to_value(data).unwrap_or(Value::Null));
        self
    }

    pub fn is_ok(&self) -> bool {
        self.status == ResponseStatus::Ok
    }

    pub fn should_render(&self) -> bool {
        self.render
    }
}
