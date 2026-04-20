use serde::{Deserialize, Serialize};

/// Raw conversation response from CSA API.
/// We use Value for the inner structures since the API returns many
/// fields that vary and we only need a subset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationResponse {
    #[serde(default)]
    pub teams: Vec<serde_json::Value>,
    #[serde(default)]
    pub chats: Vec<serde_json::Value>,
    #[serde(default)]
    pub users: Vec<serde_json::Value>,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conversation_response_deserialization() {
        let json = r#"{
            "teams": [{"id": "1"}],
            "chats": [],
            "users": [{"mri": "8:orgid:abc"}]
        }"#;

        let resp: ConversationResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.teams.len(), 1);
        assert_eq!(resp.teams[0]["id"], "1");
        assert!(resp.chats.is_empty());
        assert_eq!(resp.users.len(), 1);
        assert_eq!(resp.users[0]["mri"], "8:orgid:abc");
        assert!(resp.metadata.is_none());
    }

    #[test]
    fn conversation_response_missing_fields_default_to_empty() {
        let json = r#"{}"#;

        let resp: ConversationResponse = serde_json::from_str(json).unwrap();
        assert!(resp.teams.is_empty());
        assert!(resp.chats.is_empty());
        assert!(resp.users.is_empty());
        assert!(resp.metadata.is_none());
    }

    #[test]
    fn conversation_response_with_metadata() {
        let json = r#"{
            "teams": [],
            "chats": [],
            "users": [],
            "metadata": {"syncToken": "abc123"}
        }"#;

        let resp: ConversationResponse = serde_json::from_str(json).unwrap();
        let meta = resp.metadata.unwrap();
        assert_eq!(meta["syncToken"], "abc123");
    }

    #[test]
    fn conversation_response_multiple_teams_and_chats() {
        let json = r#"{
            "teams": [
                {"id": "t1", "displayName": "Team A"},
                {"id": "t2", "displayName": "Team B"}
            ],
            "chats": [
                {"id": "c1"},
                {"id": "c2"},
                {"id": "c3"}
            ],
            "users": []
        }"#;

        let resp: ConversationResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.teams.len(), 2);
        assert_eq!(resp.chats.len(), 3);
        assert_eq!(resp.teams[1]["displayName"], "Team B");
        assert_eq!(resp.chats[2]["id"], "c3");
    }
}
