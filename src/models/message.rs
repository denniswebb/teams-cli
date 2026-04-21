use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub content: String,
    #[serde(default, alias = "imdisplayname")]
    pub im_display_name: String,
    #[serde(default)]
    pub from: String,
    #[serde(default)]
    pub compose_time: String,
    #[serde(default)]
    pub original_arrival_time: String,
    #[serde(default, alias = "messagetype")]
    pub message_type: String,
    #[serde(default, alias = "contenttype")]
    pub content_type: String,
    #[serde(default, alias = "clientmessageid")]
    pub client_message_id: String,
    #[serde(default)]
    pub properties: Option<MessageProperties>,
    #[serde(default, alias = "conversationid")]
    pub conversation_id: String,
    #[serde(default, alias = "conversationLink")]
    pub conversation_link: String,
    #[serde(default, alias = "type")]
    pub msg_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageProperties {
    #[serde(default)]
    pub emotions: Option<serde_json::Value>,
    #[serde(default)]
    pub delta_emotions: Option<serde_json::Value>,
    #[serde(default)]
    pub mentions: Option<serde_json::Value>,
    #[serde(default)]
    pub edit_time: Option<String>,
    #[serde(default)]
    pub delete_time: Option<String>,
    #[serde(default)]
    pub admin_delete: Option<bool>,
    #[serde(default)]
    pub files: Option<serde_json::Value>,
    #[serde(default)]
    pub cards: Option<serde_json::Value>,
    #[serde(default)]
    pub links: Option<serde_json::Value>,
    #[serde(default)]
    pub importance: Option<String>,
    #[serde(default)]
    pub subject: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagesResponse {
    #[serde(default)]
    pub messages: Vec<ChatMessage>,
    #[serde(rename = "_metadata", default)]
    pub metadata: Option<MessagesMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessagesMetadata {
    #[serde(default)]
    pub sync_state: Option<String>,
    #[serde(default)]
    pub back_ward_link: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageRequest {
    pub content: String,
    #[serde(default = "default_message_type")]
    pub messagetype: String,
    #[serde(default = "default_content_type")]
    pub contenttype: String,
    #[serde(default)]
    pub clientmessageid: String,
    #[serde(default)]
    pub imdisplayname: String,
    #[serde(default)]
    pub properties: Option<SendMessageProperties>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub amsreferences: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageProperties {
    #[serde(default)]
    pub importance: Option<String>,
    #[serde(default)]
    pub subject: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mentions: Option<String>,
}

fn default_message_type() -> String {
    "Text".into()
}

fn default_content_type() -> String {
    "text".into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_message_deserialization_camel_case() {
        let json = r#"{
            "id": "1234",
            "messageType": "Text",
            "content": "Hello world",
            "imDisplayName": "John Doe",
            "composeTime": "2024-01-01T00:00:00Z",
            "conversationId": "19:abc@thread.v2"
        }"#;

        let msg: ChatMessage = serde_json::from_str(json).unwrap();
        assert_eq!(msg.id, "1234");
        assert_eq!(msg.message_type, "Text");
        assert_eq!(msg.content, "Hello world");
        assert_eq!(msg.im_display_name, "John Doe");
        assert_eq!(msg.compose_time, "2024-01-01T00:00:00Z");
        assert_eq!(msg.conversation_id, "19:abc@thread.v2");
    }

    #[test]
    fn chat_message_deserialization_aliases() {
        // The API sometimes uses lowercase field names (aliases)
        let json = r#"{
            "id": "5678",
            "messagetype": "RichText/Html",
            "content": "<p>Hi</p>",
            "imdisplayname": "Jane Smith",
            "composeTime": "2024-06-15T12:30:00Z",
            "conversationid": "19:xyz@thread.v2",
            "contenttype": "text/html",
            "clientmessageid": "msg-001"
        }"#;

        let msg: ChatMessage = serde_json::from_str(json).unwrap();
        assert_eq!(msg.id, "5678");
        assert_eq!(msg.message_type, "RichText/Html");
        assert_eq!(msg.im_display_name, "Jane Smith");
        assert_eq!(msg.conversation_id, "19:xyz@thread.v2");
        assert_eq!(msg.content_type, "text/html");
        assert_eq!(msg.client_message_id, "msg-001");
    }

    #[test]
    fn chat_message_missing_optional_fields_uses_defaults() {
        let json = r#"{}"#;

        let msg: ChatMessage = serde_json::from_str(json).unwrap();
        assert_eq!(msg.id, "");
        assert_eq!(msg.content, "");
        assert_eq!(msg.im_display_name, "");
        assert_eq!(msg.message_type, "");
        assert!(msg.properties.is_none());
    }

    #[test]
    fn chat_message_with_properties() {
        let json = r#"{
            "id": "100",
            "properties": {
                "importance": "high",
                "subject": "Meeting notes"
            }
        }"#;

        let msg: ChatMessage = serde_json::from_str(json).unwrap();
        let props = msg.properties.unwrap();
        assert_eq!(props.importance, Some("high".to_string()));
        assert_eq!(props.subject, Some("Meeting notes".to_string()));
    }

    #[test]
    fn messages_response_deserialization() {
        let json = r#"{
            "messages": [
                {"id": "1", "content": "first"},
                {"id": "2", "content": "second"}
            ],
            "_metadata": {
                "syncState": "some-state",
                "backWardLink": "https://example.com/prev"
            }
        }"#;

        let resp: MessagesResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.messages.len(), 2);
        assert_eq!(resp.messages[0].id, "1");
        assert_eq!(resp.messages[1].content, "second");
        let meta = resp.metadata.unwrap();
        assert_eq!(meta.sync_state, Some("some-state".to_string()));
        assert_eq!(
            meta.back_ward_link,
            Some("https://example.com/prev".to_string())
        );
    }

    #[test]
    fn messages_response_empty() {
        let json = r#"{}"#;

        let resp: MessagesResponse = serde_json::from_str(json).unwrap();
        assert!(resp.messages.is_empty());
        assert!(resp.metadata.is_none());
    }

    #[test]
    fn send_message_request_serialization() {
        let req = SendMessageRequest {
            content: "Hello".to_string(),
            messagetype: "Text".to_string(),
            contenttype: "text".to_string(),
            clientmessageid: "12345".to_string(),
            imdisplayname: "Test User".to_string(),
            properties: Some(SendMessageProperties {
                importance: Some("normal".to_string()),
                subject: None,
                mentions: None,
            }),
            amsreferences: None,
        };

        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["content"], "Hello");
        assert_eq!(json["messagetype"], "Text");
        assert_eq!(json["imdisplayname"], "Test User");
    }

    #[test]
    fn send_message_request_default_types() {
        let json = r#"{"content": "test"}"#;
        let req: SendMessageRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.messagetype, "Text");
        assert_eq!(req.contenttype, "text");
    }
}
