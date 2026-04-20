pub mod json;
pub mod plain;
pub mod table;

use std::io::IsTerminal;

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Json,
    Human,
    Plain,
}

impl OutputFormat {
    pub fn detect(flag: Option<&str>) -> std::result::Result<Self, String> {
        match flag {
            Some("json") => Ok(OutputFormat::Json),
            Some("human") | Some("table") => Ok(OutputFormat::Human),
            Some("plain") | Some("text") => Ok(OutputFormat::Plain),
            Some(other) => Err(format!(
                "unknown output format '{}', expected: json, human, table, plain, text",
                other
            )),
            None => Ok(if std::io::stdout().is_terminal() {
                OutputFormat::Human
            } else {
                OutputFormat::Json
            }),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Envelope<T: Serialize> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorBody>,
    pub metadata: Metadata,
}

#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct Metadata {
    pub request_id: String,
    pub timestamp: String,
    pub duration_ms: u64,
}

impl<T: Serialize> Envelope<T> {
    pub fn success(data: T, duration_ms: u64) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            metadata: Metadata {
                request_id: uuid::Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
                duration_ms,
            },
        }
    }
}

impl Envelope<()> {
    pub fn error(code: &str, message: &str, duration_ms: u64) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(ErrorBody {
                code: code.to_string(),
                message: message.to_string(),
            }),
            metadata: Metadata {
                request_id: uuid::Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
                duration_ms,
            },
        }
    }
}

pub fn print_output<T: Serialize>(format: OutputFormat, data: T, duration_ms: u64) {
    match format {
        OutputFormat::Json => {
            let envelope = Envelope::success(&data, duration_ms);
            json::print_json(&envelope);
        }
        OutputFormat::Human => {
            let pretty = serde_json::to_string_pretty(&data).unwrap_or_else(|_| "{}".into());
            println!("{pretty}");
        }
        OutputFormat::Plain => {
            plain::print_plain(&data);
        }
    }
}

pub fn print_error(format: OutputFormat, code: &str, message: &str, duration_ms: u64) {
    match format {
        OutputFormat::Json => {
            let envelope = Envelope::<()>::error(code, message, duration_ms);
            json::print_json(&envelope);
        }
        OutputFormat::Human | OutputFormat::Plain => {
            eprintln!("Error [{code}]: {message}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- OutputFormat::detect() ---

    #[test]
    fn detect_json() {
        assert_eq!(
            OutputFormat::detect(Some("json")).unwrap(),
            OutputFormat::Json
        );
    }

    #[test]
    fn detect_human() {
        assert_eq!(
            OutputFormat::detect(Some("human")).unwrap(),
            OutputFormat::Human
        );
    }

    #[test]
    fn detect_table_maps_to_human() {
        assert_eq!(
            OutputFormat::detect(Some("table")).unwrap(),
            OutputFormat::Human
        );
    }

    #[test]
    fn detect_plain() {
        assert_eq!(
            OutputFormat::detect(Some("plain")).unwrap(),
            OutputFormat::Plain
        );
    }

    #[test]
    fn detect_text_maps_to_plain() {
        assert_eq!(
            OutputFormat::detect(Some("text")).unwrap(),
            OutputFormat::Plain
        );
    }

    #[test]
    fn detect_invalid_returns_err() {
        let result = OutputFormat::detect(Some("invalid"));
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(msg.contains("invalid"));
        assert!(msg.contains("expected"));
    }

    #[test]
    fn detect_none_returns_ok() {
        // None should succeed regardless of TTY state
        let result = OutputFormat::detect(None);
        assert!(result.is_ok());
    }

    // --- Envelope::success() ---

    #[test]
    fn envelope_success_structure() {
        let env = Envelope::success("hello", 42);
        assert!(env.success);
        assert_eq!(env.data, Some("hello"));
        assert!(env.error.is_none());
        assert_eq!(env.metadata.duration_ms, 42);
        assert!(!env.metadata.request_id.is_empty());
        assert!(!env.metadata.timestamp.is_empty());
    }

    #[test]
    fn envelope_success_request_id_is_uuid() {
        let env = Envelope::success(123, 0);
        // UUID v4 format: 8-4-4-4-12 hex chars
        let id = &env.metadata.request_id;
        assert_eq!(id.len(), 36);
        assert_eq!(id.chars().filter(|c| *c == '-').count(), 4);
    }

    #[test]
    fn envelope_success_timestamp_is_rfc3339() {
        let env = Envelope::success("data", 0);
        // Should parse as a valid datetime
        let ts = &env.metadata.timestamp;
        assert!(
            chrono::DateTime::parse_from_rfc3339(ts).is_ok(),
            "timestamp should be valid RFC3339, got: {}",
            ts
        );
    }

    // --- Envelope::error() ---

    #[test]
    fn envelope_error_structure() {
        let env = Envelope::<()>::error("AUTH_FAILED", "bad token", 99);
        assert!(!env.success);
        assert!(env.data.is_none());
        let err = env.error.as_ref().unwrap();
        assert_eq!(err.code, "AUTH_FAILED");
        assert_eq!(err.message, "bad token");
        assert_eq!(env.metadata.duration_ms, 99);
        assert!(!env.metadata.request_id.is_empty());
        assert!(!env.metadata.timestamp.is_empty());
    }

    #[test]
    fn envelope_error_serialization_skips_none_data() {
        let env = Envelope::<()>::error("ERR", "msg", 0);
        let json = serde_json::to_value(&env).unwrap();
        assert!(
            json.get("data").is_none(),
            "data should be skipped when None"
        );
        assert!(json.get("error").is_some());
    }

    #[test]
    fn envelope_success_serialization_skips_none_error() {
        let env = Envelope::success("val", 0);
        let json = serde_json::to_value(&env).unwrap();
        assert!(
            json.get("error").is_none(),
            "error should be skipped when None"
        );
        assert!(json.get("data").is_some());
    }
}
