use serde::{Deserialize, Serialize};

/// Event emitted by the subscribe poll loop when a new message arrives.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEvent {
    pub event_type: String,
    pub conversation_id: String,
    pub message_id: String,
    pub from: String,
    pub content: String,
    pub message_type: String,
    pub time: String,
    pub resource: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_event_serialization() {
        let event = MessageEvent {
            event_type: "NewMessage".to_string(),
            conversation_id: "19:abc@thread.v2".to_string(),
            message_id: "1234".to_string(),
            from: "Dennis Webb".to_string(),
            content: "Hello".to_string(),
            message_type: "Text".to_string(),
            time: "2024-01-01T00:00:00Z".to_string(),
            resource: serde_json::json!({"id": "1234"}),
        };
        let json = serde_json::to_string(&event).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["event_type"], "NewMessage");
        assert_eq!(parsed["from"], "Dennis Webb");
        assert_eq!(parsed["conversation_id"], "19:abc@thread.v2");
    }

    #[test]
    fn message_event_round_trip() {
        let event = MessageEvent {
            event_type: "NewMessage".to_string(),
            conversation_id: "conv".to_string(),
            message_id: "msg".to_string(),
            from: "user".to_string(),
            content: "text".to_string(),
            message_type: "Text".to_string(),
            time: "now".to_string(),
            resource: serde_json::json!({}),
        };
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: MessageEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.event_type, event.event_type);
        assert_eq!(deserialized.message_id, event.message_id);
    }
}
