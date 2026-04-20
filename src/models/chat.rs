use serde::{Deserialize, Serialize};

use super::message::ChatMessage;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Chat {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub chat_type: String,
    #[serde(default)]
    pub hidden: bool,
    #[serde(default)]
    pub is_sticky: bool,
    #[serde(default)]
    pub last_message: Option<ChatMessage>,
    #[serde(default)]
    pub members: Vec<ChatMember>,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub last_join_at: String,
    #[serde(default)]
    pub last_leave_at: String,
    #[serde(default)]
    pub version: i64,
    #[serde(default)]
    pub thread_version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMember {
    #[serde(default)]
    pub mri: String,
    #[serde(default)]
    pub role: String,
    #[serde(default)]
    pub friendly_name: String,
    #[serde(default)]
    pub is_muted: bool,
}
