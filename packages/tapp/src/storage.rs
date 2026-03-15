use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::error::Result;

#[allow(unused_imports)]
use crate::error::Error;

pub struct Storage {
    session: HashMap<String, Value>,
}

impl Storage {
    pub fn new() -> Self {
        Self {
            session: HashMap::new(),
        }
    }

    pub fn session_get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T> {
        self.session
            .get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    pub fn session_set<T: Serialize>(&mut self, key: &str, value: T) {
        if let Ok(v) = serde_json::to_value(value) {
            self.session.insert(key.to_string(), v);
        }
    }

    pub fn session_delete(&mut self, key: &str) -> bool {
        self.session.remove(key).is_some()
    }

    pub fn session_clear(&mut self) {
        self.session.clear();
    }

    #[cfg(target_arch = "wasm32")]
    pub fn json_get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Result<Option<T>> {
        Err(Error::Storage("Not implemented in WASM".to_string()))
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn json_get<T: for<'de> Deserialize<'de>>(&self, _key: &str) -> Result<Option<T>> {
        Ok(None)
    }

    #[cfg(target_arch = "wasm32")]
    pub fn json_set<T: Serialize>(&self, key: &str, value: T) -> Result<()> {
        Err(Error::Storage("Not implemented in WASM".to_string()))
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn json_set<T: Serialize>(&self, _key: &str, _value: T) -> Result<()> {
        Ok(())
    }

    #[cfg(target_arch = "wasm32")]
    pub fn json_delete(&self, key: &str) -> Result<bool> {
        Err(Error::Storage("Not implemented in WASM".to_string()))
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn json_delete(&self, _key: &str) -> Result<bool> {
        Ok(false)
    }

    #[cfg(target_arch = "wasm32")]
    pub fn sql_execute(&self, sql: &str, params: &[Value]) -> Result<u64> {
        Err(Error::Storage("Not implemented in WASM".to_string()))
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn sql_execute(&self, _sql: &str, _params: &[Value]) -> Result<u64> {
        Ok(0)
    }

    #[cfg(target_arch = "wasm32")]
    pub fn sql_query(&self, sql: &str, params: &[Value]) -> Result<Vec<HashMap<String, Value>>> {
        Err(Error::Storage("Not implemented in WASM".to_string()))
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn sql_query(&self, _sql: &str, _params: &[Value]) -> Result<Vec<HashMap<String, Value>>> {
        Ok(Vec::new())
    }
}

impl Default for Storage {
    fn default() -> Self {
        Self::new()
    }
}
