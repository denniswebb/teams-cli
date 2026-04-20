use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::{Result, TeamsError};

pub const TEAMS_APP_ID: &str = "5e3ce6c0-2b1f-4285-8d4b-75ee78787346";
pub const SKYPE_RESOURCE: &str = "https://api.spaces.skype.com";
pub const CHATSVCAGG_RESOURCE: &str = "https://chatsvcagg.teams.microsoft.com";
#[allow(dead_code)]
pub const REDIRECT_URI: &str = "https://teams.microsoft.com/go";
#[allow(dead_code)]
pub const MICROSOFT_TENANT_ID: &str = "f8cdef31-a31e-4b4a-93e4-5f571e91255a";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenSet {
    pub teams: TokenInfo,
    pub skype: TokenInfo,
    pub chatsvcagg: TokenInfo,
    pub profile: String,
    pub tenant_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    pub raw: String,
    pub token_type: TokenType,
    pub expires_at: Option<DateTime<Utc>>,
    pub audience: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TokenType {
    IdToken,
    AccessToken,
}

impl TokenInfo {
    pub fn from_jwt(raw: &str, token_type: TokenType) -> Result<Self> {
        let claims = decode_jwt_claims(raw)?;
        let expires_at = claims
            .exp
            .map(|exp| DateTime::<Utc>::from_timestamp(exp, 0).unwrap_or_else(Utc::now));
        let audience = claims.aud.unwrap_or_default();

        Ok(Self {
            raw: raw.to_string(),
            token_type,
            expires_at,
            audience,
        })
    }

    pub fn is_expired(&self) -> bool {
        match self.expires_at {
            Some(exp) => Utc::now() >= exp,
            None => false,
        }
    }
}

impl TokenSet {
    pub fn is_expired(&self) -> bool {
        self.teams.is_expired() || self.skype.is_expired() || self.chatsvcagg.is_expired()
    }

    pub fn skype_header(&self) -> String {
        format!("skypetoken={}", self.skype.raw)
    }

    pub fn chatsvcagg_bearer(&self) -> String {
        format!("Bearer {}", self.chatsvcagg.raw)
    }

    pub fn teams_bearer(&self) -> String {
        format!("Bearer {}", self.teams.raw)
    }

    pub fn skype_bearer(&self) -> String {
        format!("Bearer {}", self.skype.raw)
    }
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub(crate) struct JwtClaims {
    #[serde(default)]
    exp: Option<i64>,
    #[serde(default)]
    aud: Option<String>,
    #[serde(default)]
    pub tid: Option<String>,
    #[serde(default)]
    pub upn: Option<String>,
    #[serde(default)]
    pub preferred_username: Option<String>,
    #[serde(default)]
    pub unique_name: Option<String>,
}

pub(crate) fn decode_jwt_claims(token: &str) -> Result<JwtClaims> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() < 2 {
        return Err(TeamsError::AuthError("invalid JWT format".into()));
    }

    let payload = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(parts[1])
        .or_else(|_| base64::engine::general_purpose::STANDARD.decode(parts[1]))
        .map_err(|e| TeamsError::AuthError(format!("failed to decode JWT payload: {e}")))?;

    serde_json::from_slice(&payload)
        .map_err(|e| TeamsError::AuthError(format!("failed to parse JWT claims: {e}")))
}

pub fn extract_tenant_id(token: &str) -> Result<Option<String>> {
    let claims = decode_jwt_claims(token)?;
    Ok(claims.tid)
}

pub fn extract_username(token: &str) -> Result<String> {
    let claims = decode_jwt_claims(token)?;
    claims
        .upn
        .or(claims.preferred_username)
        .or(claims.unique_name)
        .ok_or_else(|| TeamsError::AuthError("no username claim in token".into()))
}

use base64::Engine;
