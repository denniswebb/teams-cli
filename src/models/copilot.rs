use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopilotResponse {
    pub conversation_id: String,
    pub message: String,
    #[serde(default)]
    pub citations: Vec<Citation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Citation {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub url: String,
}

/// SignalR JSON protocol message envelope.
/// Type 1 = Invocation, Type 2 = StreamItem, Type 3 = Completion,
/// Type 6 = Ping, Type 7 = Close
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalRMessage {
    #[serde(rename = "type", default)]
    pub msg_type: u8,
    #[serde(default)]
    pub target: Option<String>,
    #[serde(default)]
    pub arguments: Option<Vec<serde_json::Value>>,
    #[serde(rename = "invocationId", default)]
    pub invocation_id: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub result: Option<serde_json::Value>,
    #[serde(default)]
    pub item: Option<serde_json::Value>,
}

pub const SIGNALR_RECORD_SEPARATOR: char = '\x1e';

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_signalr_ping() {
        let json = r#"{"type":6}"#;
        let msg: SignalRMessage = serde_json::from_str(json).unwrap();
        assert_eq!(msg.msg_type, 6);
        assert!(msg.target.is_none());
    }

    #[test]
    fn deserialize_signalr_invocation() {
        let json = r#"{"type":1,"target":"send","arguments":["hello"],"invocationId":"1"}"#;
        let msg: SignalRMessage = serde_json::from_str(json).unwrap();
        assert_eq!(msg.msg_type, 1);
        assert_eq!(msg.target.as_deref(), Some("send"));
        assert_eq!(msg.invocation_id.as_deref(), Some("1"));
    }

    #[test]
    fn deserialize_signalr_completion() {
        let json = r#"{"type":3,"invocationId":"1","result":{"text":"done"}}"#;
        let msg: SignalRMessage = serde_json::from_str(json).unwrap();
        assert_eq!(msg.msg_type, 3);
        assert!(msg.result.is_some());
    }

    #[test]
    fn copilot_response_round_trip() {
        let resp = CopilotResponse {
            conversation_id: "conv-123".to_string(),
            message: "The answer is 4.".to_string(),
            citations: vec![Citation {
                title: "Math".to_string(),
                url: "https://example.com".to_string(),
            }],
        };
        let json = serde_json::to_string(&resp).unwrap();
        let parsed: CopilotResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.conversation_id, "conv-123");
        assert_eq!(parsed.citations.len(), 1);
    }
}
