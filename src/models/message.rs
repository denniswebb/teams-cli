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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageProperties {
    #[serde(default)]
    pub importance: Option<String>,
    #[serde(default)]
    pub subject: Option<String>,
}

fn default_message_type() -> String {
    "Text".into()
}

fn default_content_type() -> String {
    "text".into()
}
