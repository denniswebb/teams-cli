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
    #[allow(dead_code)]
    pub expires_in: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RegionGtms {
    #[serde(rename = "chatService")]
    pub chat_service: String,
    #[serde(rename = "middleTier")]
    pub middle_tier: String,
    #[serde(rename = "chatServiceAggregator", default)]
    #[allow(dead_code)]
    pub chat_service_aggregator: String,
    /// AMS endpoint for image/file rendering URLs (e.g. https://us-api.asm.skype.com)
    #[serde(default)]
    pub ams: String,
    /// AMS V2 endpoint for blob upload (e.g. https://us-prod.asyncgw.teams.microsoft.com)
    #[serde(rename = "amsV2", default)]
    pub ams_v2: String,
}

fn validate_service_url(url_str: &str, name: &str) -> Result<()> {
    let parsed = url::Url::parse(url_str)
        .map_err(|_| TeamsError::AuthError(format!("invalid {name} URL from authz")))?;
    if parsed.scheme() != "https" {
        return Err(TeamsError::AuthError(format!("{name} URL must use HTTPS")));
    }
    let host = parsed
        .host_str()
        .ok_or_else(|| TeamsError::AuthError(format!("{name} URL missing host")))?;
    if !host.ends_with(".microsoft.com") && !host.ends_with(".skype.com") {
        return Err(TeamsError::AuthError(format!(
            "{name} URL has untrusted host: {host}"
        )));
    }
    Ok(())
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

    let authz = resp
        .json::<AuthzResponse>()
        .await
        .map_err(|e| TeamsError::ApiError {
            status: 0,
            message: format!("failed to parse authz response: {e}"),
        })?;

    validate_service_url(&authz.region_gtms.chat_service, "chat_service")?;
    validate_service_url(&authz.region_gtms.middle_tier, "middle_tier")?;

    Ok(authz)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_service_url_valid_teams() {
        assert!(
            validate_service_url("https://teams.microsoft.com/api/mt/amer/beta", "test").is_ok()
        );
    }

    #[test]
    fn validate_service_url_valid_msg_subdomain() {
        assert!(validate_service_url("https://amer.ng.msg.teams.microsoft.com", "test").is_ok());
    }

    #[test]
    fn validate_service_url_valid_skype() {
        assert!(validate_service_url("https://something.skype.com/path", "test").is_ok());
    }

    #[test]
    fn validate_service_url_rejects_http() {
        let result = validate_service_url("http://teams.microsoft.com", "test");
        assert!(result.is_err());
        let err_msg = format!("{}", result.unwrap_err());
        assert!(
            err_msg.contains("HTTPS"),
            "expected HTTPS error, got: {err_msg}"
        );
    }

    #[test]
    fn validate_service_url_rejects_untrusted_host() {
        let result = validate_service_url("https://evil.com", "test");
        assert!(result.is_err());
        let err_msg = format!("{}", result.unwrap_err());
        assert!(
            err_msg.contains("untrusted host"),
            "expected untrusted host error, got: {err_msg}"
        );
    }

    #[test]
    fn validate_service_url_rejects_non_url() {
        let result = validate_service_url("not a url", "test");
        assert!(result.is_err());
        let err_msg = format!("{}", result.unwrap_err());
        assert!(
            err_msg.contains("invalid"),
            "expected invalid URL error, got: {err_msg}"
        );
    }

    #[test]
    fn validate_service_url_rejects_missing_host() {
        let result = validate_service_url("https://", "test");
        assert!(result.is_err());
    }

    #[test]
    fn authz_response_deserialization() {
        let json = r#"{
            "tokens": {
                "skypeToken": "fake-skype-token",
                "expiresIn": 86400
            },
            "regionGtms": {
                "chatService": "https://amer.ng.msg.teams.microsoft.com",
                "middleTier": "https://teams.microsoft.com/api/mt/amer/beta",
                "chatServiceAggregator": "https://teams.microsoft.com/api/csa/amer"
            }
        }"#;

        let resp: AuthzResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.tokens.skype_token, "fake-skype-token");
        assert_eq!(resp.tokens.expires_in, 86400);
        assert_eq!(
            resp.region_gtms.chat_service,
            "https://amer.ng.msg.teams.microsoft.com"
        );
        assert_eq!(
            resp.region_gtms.middle_tier,
            "https://teams.microsoft.com/api/mt/amer/beta"
        );
        assert_eq!(
            resp.region_gtms.chat_service_aggregator,
            "https://teams.microsoft.com/api/csa/amer"
        );
    }

    #[test]
    fn authz_response_missing_aggregator_uses_default() {
        let json = r#"{
            "tokens": {
                "skypeToken": "tok",
                "expiresIn": 100
            },
            "regionGtms": {
                "chatService": "https://x.microsoft.com",
                "middleTier": "https://y.microsoft.com"
            }
        }"#;

        let resp: AuthzResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.region_gtms.chat_service_aggregator, "");
    }
}
