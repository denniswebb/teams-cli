use crate::auth::token::TokenSet;
use crate::error::Result;
use crate::models::{ConversationResponse, PinnedChannelsResponse};

use super::HttpClient;

const CSA_BASE: &str = "https://teams.microsoft.com/api/csa/api/v1";

pub struct CsaClient<'a> {
    http: &'a HttpClient,
    tokens: &'a TokenSet,
}

impl<'a> CsaClient<'a> {
    pub fn new(http: &'a HttpClient, tokens: &'a TokenSet) -> Self {
        Self { http, tokens }
    }

    pub async fn get_conversations(&self) -> Result<ConversationResponse> {
        let url =
            format!("{CSA_BASE}/teams/users/me?isPrefetch=false&enableMembershipSummary=true");
        let bearer = self.tokens.chatsvcagg_bearer();

        let resp = self
            .http
            .execute_with_retry(|| self.http.client.get(&url).header("Authorization", &bearer))
            .await?;

        resp.json::<ConversationResponse>()
            .await
            .map_err(|e| crate::error::TeamsError::ApiError {
                status: 0,
                message: format!("failed to parse conversations: {e}"),
            })
    }

    pub async fn get_pinned_channels(&self) -> Result<PinnedChannelsResponse> {
        let url = format!("{CSA_BASE}/teams/users/me/pinnedChannels");
        let bearer = self.tokens.chatsvcagg_bearer();

        let resp = self
            .http
            .execute_with_retry(|| self.http.client.get(&url).header("Authorization", &bearer))
            .await?;

        resp.json::<PinnedChannelsResponse>().await.map_err(|e| {
            crate::error::TeamsError::ApiError {
                status: 0,
                message: format!("failed to parse pinned channels: {e}"),
            }
        })
    }
}
