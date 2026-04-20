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
