use serde::Deserialize;

use crate::auth::token::TokenSet;
use crate::error::{Result, TeamsError};

use super::HttpClient;

const AUTHZ_URL: &str = "https://teams.microsoft.com/api/authsvc/v1.0/authz";

#[derive(Debug, Deserialize)]
pub struct AuthzResponse {
    pub tokens: AuthzTokens,
    #[serde(rename = "regionGtms")]
    pub region_gtms: RegionGtms,
}

#[derive(Debug, Deserialize)]
pub struct AuthzTokens {
    #[serde(rename = "skypeToken")]
    pub skype_token: String,
    #[serde(rename = "expiresIn")]
    pub expires_in: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RegionGtms {
    #[serde(rename = "chatService")]
    pub chat_service: String,
    #[serde(rename = "middleTier")]
    pub middle_tier: String,
    #[serde(rename = "chatServiceAggregator", default)]
    pub chat_service_aggregator: String,
}

pub async fn exchange_token(http: &HttpClient, tokens: &TokenSet) -> Result<AuthzResponse> {
    let bearer = tokens.skype_bearer();

    let resp = http
        .execute_with_retry(|| {
            http.client
                .post(AUTHZ_URL)
                .header("Authorization", &bearer)
                .header("Content-Length", "0")
        })
        .await?;

    resp.json::<AuthzResponse>()
        .await
        .map_err(|e| TeamsError::ApiError {
            status: 0,
            message: format!("failed to parse authz response: {e}"),
        })
}
