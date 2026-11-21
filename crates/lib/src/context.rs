use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::Result;

pub struct Parts {
    pub entry: String,
    pub event: String,
    pub payload: Value,
    pub config: Value,
    pub state: Value,
    pub data_dir: PathBuf,
}

pub struct Context {
    pub entry: String,
    pub event: String,
    pub payload: Value,
    pub config: Value,
    pub state: Value,
    pub data_dir: PathBuf,
    pub now_ms: i64,
    translations: Value,
}

impl Context {
    #[must_use]
    pub fn new(parts: Parts, translations: Value) -> Self {
        Self {
            entry: parts.entry,
            event: parts.event,
            payload: parts.payload,
            config: parts.config,
            state: parts.state,
            data_dir: parts.data_dir,
            now_ms: now_ms(),
            translations,
        }
    }

    #[inline]
    #[must_use]
    pub fn config_str(&self, key: &str) -> Option<String> {
        match self.config.get(key)? {
            Value::String(value) => Some(value.clone()),
            Value::Bool(value) => Some(value.to_string()),
            Value::Number(value) => Some(value.to_string()),
            Value::Array(_) | Value::Object(_) | Value::Null => None,
        }
    }

    #[inline]
    #[must_use]
    pub fn config_bool(&self, key: &str, default: bool) -> bool {
        self.config
            .get(key)
            .and_then(Value::as_bool)
            .unwrap_or(default)
    }

    #[inline]
    #[must_use]
    pub fn payload_str(&self, key: &str) -> Option<String> {
        match self.payload.get(key)? {
            Value::String(value) => Some(value.clone()),
            Value::Number(value) => Some(value.to_string()),
            Value::Bool(_) | Value::Array(_) | Value::Object(_) | Value::Null => None,
        }
    }

    #[inline]
    #[must_use]
    pub fn payload_arg(&self, index: usize) -> Option<&Value> {
        self.payload.get("args")?.get(index)
    }

    pub fn state_value<T>(&self, key: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let value = self.state.get(key).cloned().unwrap_or_default();
        Ok(serde_json::from_value(value)?)
    }

    #[must_use]
    pub fn tr(&self, key: &str, params: &[(&str, String)]) -> String {
        let mut result = lookup(&self.translations, key)
            .and_then(Value::as_str)
            .unwrap_or(key)
            .to_owned();

        for (name, value) in params {
            result = result.replace(&format!("{{{name}}}"), value);
        }

        result
    }
}

fn now_ms() -> i64 {
    let elapsed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    i64::try_from(elapsed.as_millis()).unwrap_or(i64::MAX)
}

fn lookup<'a>(root: &'a Value, path: &str) -> Option<&'a Value> {
    path.split('.').try_fold(root, |value, part| value.get(part))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_context() -> Context {
        let parts = Parts {
            entry: "service".to_owned(),
            event: "update".to_owned(),
            payload: json!({ "args": ["10m"] }),
            config: json!({ "enabled": true, "name": "Timed" }),
            state: json!({ "timers": [1, 2, 3] }),
            data_dir: PathBuf::from("/tmp/noctalia"),
        };
        let translations = json!({
            "notify": { "title": "Timer {kind} done" }
        });
        Context::new(parts, translations)
    }

    #[test]
    fn test_config_str_reads_string_and_bool_values() {
        let context = make_context();

        assert_eq!(context.config_str("name"), Some("Timed".to_owned()));
        assert_eq!(context.config_str("enabled"), Some("true".to_owned()));
        assert_eq!(context.config_str("missing"), None);
    }

    #[test]
    fn test_config_bool_falls_back_to_default() {
        let context = make_context();

        assert!(context.config_bool("enabled", false));
        assert!(!context.config_bool("missing", false));
    }

    #[test]
    fn test_payload_arg_returns_nested_argument() {
        let context = make_context();

        assert_eq!(context.payload_arg(0), Some(&json!("10m")));
        assert_eq!(context.payload_arg(1), None);
    }

    #[test]
    fn test_state_value_deserializes_timers() {
        let context = make_context();

        let timers: Vec<u32> = context.state_value("timers").expect("valid timers");

        assert_eq!(timers, vec![1, 2, 3]);
    }

    #[test]
    fn test_tr_interpolates_dotted_keys() {
        let context = make_context();
        let params = [("kind", "countdown".to_owned())];

        assert_eq!(context.tr("notify.title", &params), "Timer countdown done");
        assert_eq!(context.tr("missing.key", &[]), "missing.key");
    }
}
