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
    pub fn detect(flag: Option<&str>) -> Self {
        match flag {
            Some("json") => OutputFormat::Json,
            Some("human") | Some("table") => OutputFormat::Human,
            Some("plain") | Some("text") => OutputFormat::Plain,
            _ => {
                if std::io::stdout().is_terminal() {
                    OutputFormat::Human
                } else {
                    OutputFormat::Json
                }
            }
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
