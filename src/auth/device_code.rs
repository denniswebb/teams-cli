use std::time::Duration;

use reqwest::Client;
use serde::Deserialize;

use super::token::{
    TokenInfo, TokenSet, TokenType, CHATSVCAGG_RESOURCE, SKYPE_RESOURCE, TEAMS_APP_ID,
};
use crate::error::{Result, TeamsError};

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct DeviceCodeResponse {
    device_code: String,
    user_code: String,
    verification_uri: String,
    #[serde(default)]
    expires_in: u64,
    interval: u64,
    message: String,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    #[serde(default)]
    access_token: Option<String>,
    #[serde(default)]
    id_token: Option<String>,
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    error_description: Option<String>,
}

pub async fn device_code_login(tenant: &str, profile: &str) -> Result<TokenSet> {
    let client = Client::new();

    // Step 1: Get Teams id_token via device code
    eprintln!("Requesting Teams authorization...");
    let teams_token =
        device_code_flow(&client, tenant, TEAMS_APP_ID, None, "openid profile", true).await?;

    let tid = super::token::extract_tenant_id(&teams_token)?.unwrap_or_else(|| tenant.to_string());

    // Step 2: Get Skype access_token
    eprintln!("Requesting Skype authorization...");
    let skype_token =
        device_code_flow(&client, &tid, TEAMS_APP_ID, Some(SKYPE_RESOURCE), "", false).await?;

    // Step 3: Get ChatSvcAgg access_token
    eprintln!("Requesting ChatSvcAgg authorization...");
    let chatsvcagg_token = device_code_flow(
        &client,
        &tid,
        TEAMS_APP_ID,
        Some(CHATSVCAGG_RESOURCE),
        "",
        false,
    )
    .await?;

    Ok(TokenSet {
        teams: TokenInfo::from_jwt(&teams_token, TokenType::IdToken)?,
        skype: TokenInfo::from_jwt(&skype_token, TokenType::AccessToken)?,
        chatsvcagg: TokenInfo::from_jwt(&chatsvcagg_token, TokenType::AccessToken)?,
        profile: profile.to_string(),
        tenant_id: tid,
    })
}

async fn device_code_flow(
    client: &Client,
    tenant: &str,
    client_id: &str,
    resource: Option<&str>,
    scope: &str,
    want_id_token: bool,
) -> Result<String> {
    let device_code_url = format!("https://login.microsoftonline.com/{tenant}/oauth2/devicecode");

    let mut params = vec![("client_id", client_id.to_string())];

    if let Some(res) = resource {
        params.push(("resource", res.to_string()));
    }
    if !scope.is_empty() {
        params.push(("scope", scope.to_string()));
    }

    let resp = client
        .post(&device_code_url)
        .form(&params)
        .send()
        .await
        .map_err(|e| TeamsError::AuthError(format!("device code request failed: {e}")))?;

    let dc: DeviceCodeResponse = resp
        .json()
        .await
        .map_err(|e| TeamsError::AuthError(format!("failed to parse device code response: {e}")))?;

    eprintln!("{}", dc.message);

    // Poll for token
    let token_url = format!("https://login.microsoftonline.com/{tenant}/oauth2/token");
    let interval = Duration::from_secs(dc.interval.max(5));
    let deadline = tokio::time::Instant::now() + Duration::from_secs(dc.expires_in.max(300));

    loop {
        tokio::time::sleep(interval).await;

        if tokio::time::Instant::now() > deadline {
            return Err(TeamsError::AuthError("device code flow timed out".into()));
        }

        let token_params = vec![
            ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
            ("client_id", client_id),
            ("code", &dc.device_code),
        ];

        let resp = client
            .post(&token_url)
            .form(&token_params)
            .send()
            .await
            .map_err(|e| TeamsError::AuthError(format!("token poll failed: {e}")))?;

        let token_resp: TokenResponse = resp
            .json()
            .await
            .map_err(|e| TeamsError::AuthError(format!("failed to parse token response: {e}")))?;

        if let Some(err) = &token_resp.error {
            match err.as_str() {
                "authorization_pending" => continue,
                "slow_down" => {
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    continue;
                }
                "expired_token" => {
                    return Err(TeamsError::AuthError("device code expired".into()));
                }
                other => {
                    let desc = token_resp.error_description.unwrap_or_default();
                    return Err(TeamsError::AuthError(format!("{other}: {desc}")));
                }
            }
        }

        if want_id_token {
            if let Some(id_token) = token_resp.id_token {
                return Ok(id_token);
            }
        }
        if let Some(access_token) = token_resp.access_token {
            return Ok(access_token);
        }

        return Err(TeamsError::AuthError("no token in response".into()));
    }
}
