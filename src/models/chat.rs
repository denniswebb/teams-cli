#![allow(dead_code)]

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_deserialization() {
        let json = r#"{
            "id": "19:chat-id@unq.gbl.spaces",
            "title": "Project Chat",
            "chatType": "oneOnOne",
            "hidden": false,
            "isSticky": true,
            "members": [
                {
                    "mri": "8:orgid:user-1",
                    "role": "Admin",
                    "friendlyName": "Alice",
                    "isMuted": false
                },
                {
                    "mri": "8:orgid:user-2",
                    "role": "User",
                    "friendlyName": "Bob",
                    "isMuted": true
                }
            ],
            "createdAt": "2024-01-01T00:00:00Z",
            "lastJoinAt": "2024-01-01T00:00:00Z",
            "lastLeaveAt": "",
            "version": 100,
            "threadVersion": 50
        }"#;

        let chat: Chat = serde_json::from_str(json).unwrap();
        assert_eq!(chat.id, "19:chat-id@unq.gbl.spaces");
        assert_eq!(chat.title, "Project Chat");
        assert_eq!(chat.chat_type, "oneOnOne");
        assert!(!chat.hidden);
        assert!(chat.is_sticky);
        assert_eq!(chat.members.len(), 2);
        assert_eq!(chat.members[0].friendly_name, "Alice");
        assert_eq!(chat.members[1].role, "User");
        assert!(chat.members[1].is_muted);
        assert_eq!(chat.version, 100);
        assert_eq!(chat.thread_version, 50);
    }

    #[test]
    fn chat_missing_optional_fields() {
        let json = r#"{}"#;

        let chat: Chat = serde_json::from_str(json).unwrap();
        assert_eq!(chat.id, "");
        assert_eq!(chat.title, "");
        assert_eq!(chat.chat_type, "");
        assert!(!chat.hidden);
        assert!(!chat.is_sticky);
        assert!(chat.last_message.is_none());
        assert!(chat.members.is_empty());
        assert_eq!(chat.version, 0);
        assert_eq!(chat.thread_version, 0);
    }

    #[test]
    fn chat_with_last_message() {
        let json = r#"{
            "id": "19:chat@thread",
            "lastMessage": {
                "id": "msg-1",
                "content": "Last message content",
                "imDisplayName": "Sender"
            }
        }"#;

        let chat: Chat = serde_json::from_str(json).unwrap();
        let msg = chat.last_message.unwrap();
        assert_eq!(msg.id, "msg-1");
        assert_eq!(msg.content, "Last message content");
        assert_eq!(msg.im_display_name, "Sender");
    }

    #[test]
    fn chat_member_deserialization() {
        let json = r#"{
            "mri": "8:orgid:abc-123",
            "role": "Admin",
            "friendlyName": "Test User",
            "isMuted": false
        }"#;

        let member: ChatMember = serde_json::from_str(json).unwrap();
        assert_eq!(member.mri, "8:orgid:abc-123");
        assert_eq!(member.role, "Admin");
        assert_eq!(member.friendly_name, "Test User");
        assert!(!member.is_muted);
    }

    #[test]
    fn chat_member_defaults() {
        let json = r#"{}"#;

        let member: ChatMember = serde_json::from_str(json).unwrap();
        assert_eq!(member.mri, "");
        assert_eq!(member.role, "");
        assert_eq!(member.friendly_name, "");
        assert!(!member.is_muted);
    }
}
