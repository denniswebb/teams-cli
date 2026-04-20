use serde::{Deserialize, Serialize};

use super::message::ChatMessage;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CsaTeam {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub channels: Vec<Channel>,
    #[serde(default)]
    pub is_archived: bool,
    #[serde(default)]
    pub is_favorite: bool,
    #[serde(default)]
    pub is_followed: bool,
    #[serde(default)]
    pub is_deleted: bool,
    #[serde(default)]
    pub membership_summary: Option<MembershipSummary>,
    #[serde(default)]
    pub team_type: String,
    #[serde(default)]
    pub team_site_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Channel {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub is_general: bool,
    #[serde(default)]
    pub is_favorite: bool,
    #[serde(default)]
    pub is_pinned: bool,
    #[serde(default)]
    pub is_archived: bool,
    #[serde(default)]
    pub is_deleted: bool,
    #[serde(default)]
    pub last_message: Option<ChatMessage>,
    #[serde(default)]
    pub parent_team_id: String,
    #[serde(default)]
    pub membership_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MembershipSummary {
    #[serde(default)]
    pub total_member_count: u64,
    #[serde(default)]
    pub admin_role_count: u64,
    #[serde(default)]
    pub user_role_count: u64,
    #[serde(default)]
    pub guest_role_count: u64,
    #[serde(default)]
    pub bot_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PinnedChannelsResponse {
    #[serde(default)]
    pub order_version: u64,
    #[serde(default)]
    pub pin_channel_order: Vec<PinnedChannel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PinnedChannel {
    #[serde(default)]
    pub channel_id: String,
    #[serde(default)]
    pub team_id: String,
}
