use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;

use super::error::{TappError, TappResult};
use super::registry::get_app_data_path;

pub struct AppStorage {
    app_id: String,
    json_path: PathBuf,
    db_path: PathBuf,
    session: HashMap<String, Value>,
}

impl AppStorage {
    pub fn new(app_id: &str) -> TappResult<Self> {
        let data_dir = get_app_data_path(app_id)?;
        Ok(Self {
            app_id: app_id.to_string(),
            json_path: data_dir.join("store.json"),
            db_path: data_dir.join("app.db"),
            session: HashMap::new(),
        })
    }

    /// Creates a storage instance with no persistent backend.
    /// Session (in-memory) storage still works, but json_* and sql_* ops
    /// return permission errors. Used when the app lacks storage permissions.
    pub fn empty(app_id: &str) -> Self {
        Self {
            app_id: app_id.to_string(),
            json_path: PathBuf::new(),
            db_path: PathBuf::new(),
            session: HashMap::new(),
        }
    }

    fn require_persistent(&self) -> TappResult<()> {
        if self.db_path.as_os_str().is_empty() {
            Err(TappError::PermissionDenied(format!(
                "App '{}' does not have storage permissions",
                self.app_id
            )))
        } else {
            Ok(())
        }
    }

    pub fn session_get(&self, key: &str) -> Option<Value> {
        self.session.get(key).cloned()
    }

    const MAX_SESSION_ENTRIES: usize = 1000;

    pub fn session_set(&mut self, key: &str, value: Value) -> TappResult<()> {
        if self.session.len() >= Self::MAX_SESSION_ENTRIES && !self.session.contains_key(key) {
            return Err(TappError::PermissionDenied(format!(
                "Session storage limit reached ({} entries)",
                Self::MAX_SESSION_ENTRIES
            )));
        }
        self.session.insert(key.to_string(), value);
        Ok(())
    }

    pub fn session_delete(&mut self, key: &str) -> bool {
        self.session.remove(key).is_some()
    }

    pub fn session_clear(&mut self) {
        self.session.clear();
    }

    pub fn json_get(&self, key: &str) -> TappResult<Option<Value>> {
        self.require_persistent()?;
        let store = self.load_json_store()?;
        Ok(store.get(key).cloned())
    }

    pub fn json_set(&self, key: &str, value: Value) -> TappResult<()> {
        self.require_persistent()?;
        let mut store = self.load_json_store()?;
        store.insert(key.to_string(), value);
        self.save_json_store(&store)
    }

    pub fn json_delete(&self, key: &str) -> TappResult<bool> {
        self.require_persistent()?;
        let mut store = self.load_json_store()?;
        let existed = store.remove(key).is_some();
        self.save_json_store(&store)?;
        Ok(existed)
    }

    fn load_json_store(&self) -> TappResult<HashMap<String, Value>> {
        if self.json_path.exists() {
            let content = std::fs::read_to_string(&self.json_path)?;
            Ok(serde_json::from_str(&content)?)
        } else {
            Ok(HashMap::new())
        }
    }

    fn save_json_store(&self, store: &HashMap<String, Value>) -> TappResult<()> {
        let content = serde_json::to_string_pretty(store)?;
        // Atomic write: temp file + rename to prevent corruption on crash
        let tmp = self.json_path.with_extension("json.tmp");
        std::fs::write(&tmp, content)?;
        std::fs::rename(&tmp, &self.json_path)?;
        Ok(())
    }

    pub fn sql_execute(&self, sql: &str, params: &[Value]) -> TappResult<u64> {
        self.require_persistent()?;
        validate_sql(sql)?;
        let conn = rusqlite::Connection::open(&self.db_path)?;
        conn.busy_timeout(std::time::Duration::from_secs(5))?;
        let params: Vec<_> = params.iter().map(|v| json_to_sql_value(v)).collect();
        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p as &dyn rusqlite::ToSql).collect();
        let affected = conn.execute(sql, param_refs.as_slice())?;
        Ok(affected as u64)
    }

    pub fn sql_query(&self, sql: &str, params: &[Value]) -> TappResult<Vec<HashMap<String, Value>>> {
        self.require_persistent()?;
        validate_sql(sql)?;
        let conn = rusqlite::Connection::open(&self.db_path)?;
        conn.busy_timeout(std::time::Duration::from_secs(5))?;
        let params: Vec<_> = params.iter().map(|v| json_to_sql_value(v)).collect();
        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p as &dyn rusqlite::ToSql).collect();

        let mut stmt = conn.prepare(sql)?;
        let column_names: Vec<String> = stmt
            .column_names()
            .iter()
            .map(|s| s.to_string())
            .collect();

        let rows = stmt.query_map(param_refs.as_slice(), |row| {
            let mut map = HashMap::new();
            for (i, name) in column_names.iter().enumerate() {
                let value: rusqlite::types::Value = row.get(i)?;
                map.insert(name.clone(), sql_to_json_value(value));
            }
            Ok(map)
        })?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }
}

/// Validates that a SQL statement is a safe DML operation.
/// Rejects DDL (CREATE, DROP, ALTER), ATTACH, PRAGMA, and multiple statements.
fn validate_sql(sql: &str) -> TappResult<()> {
    // Strip SQL comments before validation to prevent bypass
    let stripped = strip_sql_comments(sql);
    let normalized = stripped.trim().to_uppercase();

    // Reject multiple statements (semicolons not at the end)
    let trimmed = stripped.trim().trim_end_matches(';').trim();
    if trimmed.contains(';') {
        return Err(TappError::PermissionDenied(
            "Multiple SQL statements are not allowed".to_string(),
        ));
    }

    // Only allow SELECT, INSERT, UPDATE, DELETE
    const ALLOWED_PREFIXES: &[&str] = &["SELECT", "INSERT", "UPDATE", "DELETE"];
    let is_allowed = ALLOWED_PREFIXES
        .iter()
        .any(|prefix| normalized.starts_with(prefix));

    if !is_allowed {
        return Err(TappError::PermissionDenied(format!(
            "SQL statement not allowed. Only SELECT, INSERT, UPDATE, DELETE are permitted. Got: {}",
            &sql[..sql.len().min(40)]
        )));
    }

    // Reject dangerous keywords anywhere in the comment-stripped statement
    const BLOCKED_KEYWORDS: &[&str] = &[
        "ATTACH", "DETACH", "PRAGMA", "CREATE", "DROP", "ALTER",
        "VACUUM", "REINDEX", "LOAD_EXTENSION",
    ];
    for keyword in BLOCKED_KEYWORDS {
        if let Some(pos) = normalized.find(keyword) {
            let before_ok = pos == 0 || !normalized.as_bytes()[pos - 1].is_ascii_alphanumeric();
            let after_pos = pos + keyword.len();
            let after_ok = after_pos >= normalized.len()
                || !normalized.as_bytes()[after_pos].is_ascii_alphanumeric();
            if before_ok && after_ok {
                return Err(TappError::PermissionDenied(format!(
                    "SQL keyword '{}' is not allowed",
                    keyword
                )));
            }
        }
    }

    Ok(())
}

/// Strip SQL comments (both `--` line comments and `/* */` block comments)
/// to prevent keyword check bypass.
fn strip_sql_comments(sql: &str) -> String {
    let mut result = String::with_capacity(sql.len());
    let bytes = sql.as_bytes();
    let mut i = 0;
    let mut in_single_quote = false;

    while i < bytes.len() {
        if in_single_quote {
            result.push(bytes[i] as char);
            if bytes[i] == b'\'' {
                // Handle escaped quotes ''
                if i + 1 < bytes.len() && bytes[i + 1] == b'\'' {
                    result.push('\'');
                    i += 2;
                } else {
                    in_single_quote = false;
                    i += 1;
                }
            } else {
                i += 1;
            }
        } else if bytes[i] == b'\'' {
            in_single_quote = true;
            result.push('\'');
            i += 1;
        } else if i + 1 < bytes.len() && bytes[i] == b'-' && bytes[i + 1] == b'-' {
            // Line comment: skip to end of line
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            result.push(' ');
        } else if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'*' {
            // Block comment: skip to */
            i += 2;
            while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                i += 1;
            }
            if i + 1 < bytes.len() {
                i += 2; // skip */
            }
            result.push(' ');
        } else {
            result.push(bytes[i] as char);
            i += 1;
        }
    }
    result
}

#[derive(Debug)]
enum SqlValue {
    Null,
    Integer(i64),
    Real(f64),
    Text(String),
    Blob(Vec<u8>),
}

impl rusqlite::ToSql for SqlValue {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        match self {
            SqlValue::Null => Ok(rusqlite::types::ToSqlOutput::Owned(rusqlite::types::Value::Null)),
            SqlValue::Integer(i) => Ok(rusqlite::types::ToSqlOutput::Owned(rusqlite::types::Value::Integer(*i))),
            SqlValue::Real(f) => Ok(rusqlite::types::ToSqlOutput::Owned(rusqlite::types::Value::Real(*f))),
            SqlValue::Text(s) => Ok(rusqlite::types::ToSqlOutput::Owned(rusqlite::types::Value::Text(s.clone()))),
            SqlValue::Blob(b) => Ok(rusqlite::types::ToSqlOutput::Owned(rusqlite::types::Value::Blob(b.clone()))),
        }
    }
}

fn json_to_sql_value(v: &Value) -> SqlValue {
    match v {
        Value::Null => SqlValue::Null,
        Value::Bool(b) => SqlValue::Integer(if *b { 1 } else { 0 }),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                SqlValue::Integer(i)
            } else if let Some(f) = n.as_f64() {
                SqlValue::Real(f)
            } else {
                SqlValue::Text(n.to_string())
            }
        }
        Value::String(s) => SqlValue::Text(s.clone()),
        Value::Array(_) | Value::Object(_) => SqlValue::Text(v.to_string()),
    }
}

fn sql_to_json_value(v: rusqlite::types::Value) -> Value {
    match v {
        rusqlite::types::Value::Null => Value::Null,
        rusqlite::types::Value::Integer(i) => Value::Number(i.into()),
        rusqlite::types::Value::Real(f) => {
            serde_json::Number::from_f64(f)
                .map(Value::Number)
                .unwrap_or(Value::Null)
        }
        rusqlite::types::Value::Text(s) => Value::String(s),
        rusqlite::types::Value::Blob(b) => {
            Value::String(base64_encode(&b))
        }
    }
}

fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    for chunk in data.chunks(3) {
        let b0 = chunk.get(0).copied().unwrap_or(0) as usize;
        let b1 = chunk.get(1).copied().unwrap_or(0) as usize;
        let b2 = chunk.get(2).copied().unwrap_or(0) as usize;

        result.push(CHARS[b0 >> 2] as char);
        result.push(CHARS[((b0 & 3) << 4) | (b1 >> 4)] as char);

        if chunk.len() > 1 {
            result.push(CHARS[((b1 & 15) << 2) | (b2 >> 6)] as char);
        } else {
            result.push('=');
        }

        if chunk.len() > 2 {
            result.push(CHARS[b2 & 63] as char);
        } else {
            result.push('=');
        }
    }
    result
}
