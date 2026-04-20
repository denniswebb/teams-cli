#![allow(dead_code)]

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csa_team_deserialization() {
        let json = r#"{
            "id": "team-123",
            "displayName": "Engineering",
            "description": "The engineering team",
            "channels": [
                {
                    "id": "19:general@thread.v2",
                    "displayName": "General",
                    "isGeneral": true
                }
            ],
            "isArchived": false,
            "isFavorite": true,
            "isFollowed": false,
            "isDeleted": false,
            "teamType": "standard",
            "teamSiteUrl": "https://contoso.sharepoint.com/sites/eng"
        }"#;

        let team: CsaTeam = serde_json::from_str(json).unwrap();
        assert_eq!(team.id, "team-123");
        assert_eq!(team.display_name, "Engineering");
        assert_eq!(team.description, "The engineering team");
        assert_eq!(team.channels.len(), 1);
        assert!(!team.is_archived);
        assert!(team.is_favorite);
        assert_eq!(team.team_type, "standard");
        assert_eq!(
            team.team_site_url,
            "https://contoso.sharepoint.com/sites/eng"
        );
    }

    #[test]
    fn csa_team_missing_optional_fields() {
        let json = r#"{}"#;

        let team: CsaTeam = serde_json::from_str(json).unwrap();
        assert_eq!(team.id, "");
        assert_eq!(team.display_name, "");
        assert!(team.channels.is_empty());
        assert!(!team.is_archived);
        assert!(!team.is_favorite);
        assert!(team.membership_summary.is_none());
    }

    #[test]
    fn channel_deserialization() {
        let json = r#"{
            "id": "19:channel@thread.v2",
            "displayName": "General",
            "description": "The general channel",
            "isGeneral": true,
            "isFavorite": true,
            "isPinned": false,
            "isArchived": false,
            "isDeleted": false,
            "parentTeamId": "team-123",
            "membershipType": "standard"
        }"#;

        let channel: Channel = serde_json::from_str(json).unwrap();
        assert_eq!(channel.id, "19:channel@thread.v2");
        assert_eq!(channel.display_name, "General");
        assert!(channel.is_general);
        assert!(channel.is_favorite);
        assert!(!channel.is_pinned);
        assert_eq!(channel.parent_team_id, "team-123");
        assert_eq!(channel.membership_type, "standard");
    }

    #[test]
    fn channel_missing_optional_fields() {
        let json = r#"{}"#;

        let channel: Channel = serde_json::from_str(json).unwrap();
        assert_eq!(channel.id, "");
        assert!(!channel.is_general);
        assert!(channel.last_message.is_none());
    }

    #[test]
    fn membership_summary_deserialization() {
        let json = r#"{
            "id": "team-1",
            "membershipSummary": {
                "totalMemberCount": 50,
                "adminRoleCount": 3,
                "userRoleCount": 45,
                "guestRoleCount": 2,
                "botCount": 0
            }
        }"#;

        let team: CsaTeam = serde_json::from_str(json).unwrap();
        let summary = team.membership_summary.unwrap();
        assert_eq!(summary.total_member_count, 50);
        assert_eq!(summary.admin_role_count, 3);
        assert_eq!(summary.user_role_count, 45);
        assert_eq!(summary.guest_role_count, 2);
        assert_eq!(summary.bot_count, 0);
    }

    #[test]
    fn pinned_channels_response_deserialization() {
        let json = r#"{
            "orderVersion": 5,
            "pinChannelOrder": [
                {"channelId": "ch-1", "teamId": "team-1"},
                {"channelId": "ch-2", "teamId": "team-1"}
            ]
        }"#;

        let resp: PinnedChannelsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.order_version, 5);
        assert_eq!(resp.pin_channel_order.len(), 2);
        assert_eq!(resp.pin_channel_order[0].channel_id, "ch-1");
        assert_eq!(resp.pin_channel_order[1].team_id, "team-1");
    }
}
