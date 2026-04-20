pub mod device_code;
pub mod keyring;
pub mod token;
#[allow(dead_code)]
pub mod webview;

use std::future::Future;

use crate::error::{Result, TeamsError};
use token::TokenSet;

/// Resolve tokens for the given profile.
/// Priority: env vars > keyring (if valid) > fossteams files (if valid) > keyring (expired) > fossteams (expired)
pub fn resolve_tokens(profile: &str) -> Result<Option<TokenSet>> {
    // Check environment variables first
    if let (Ok(teams), Ok(skype), Ok(chatsvcagg)) = (
        std::env::var("TEAMS_CLI_TEAMS_TOKEN"),
        std::env::var("TEAMS_CLI_SKYPE_TOKEN"),
        std::env::var("TEAMS_CLI_CHATSVCAGG_TOKEN"),
    ) {
        let token_set = TokenSet {
            teams: token::TokenInfo::from_jwt(&teams, token::TokenType::IdToken)?,
            skype: token::TokenInfo::from_jwt(&skype, token::TokenType::AccessToken)?,
            chatsvcagg: token::TokenInfo::from_jwt(&chatsvcagg, token::TokenType::AccessToken)?,
            profile: profile.to_string(),
            tenant_id: token::extract_tenant_id(&teams)?.unwrap_or_else(|| "common".to_string()),
        };
        return Ok(Some(token_set));
    }

    // Check keyring first (this is where webview login stores tokens)
    let keyring_tokens = keyring::get_tokens(profile)?;
    if let Some(ref ts) = keyring_tokens {
        if !ts.is_expired() {
            return Ok(keyring_tokens);
        }
    }

    // Check fossteams token directory for compatibility
    let fossteams_tokens = resolve_fossteams_tokens(profile)?;
    if let Some(ref ts) = fossteams_tokens {
        if !ts.is_expired() {
            return Ok(fossteams_tokens);
        }
    }

    // Return whichever expired tokens we have (keyring preferred)
    if keyring_tokens.is_some() {
        return Ok(keyring_tokens);
    }
    Ok(fossteams_tokens)
}

/// Check ~/.config/fossteams/ for tokens (backward compat with fossteams/teams-token)
fn resolve_fossteams_tokens(profile: &str) -> Result<Option<TokenSet>> {
    let config_dir = dirs::home_dir()
        .unwrap_or_default()
        .join(".config")
        .join("fossteams");

    let teams_path = config_dir.join("token-teams.jwt");
    let skype_path = config_dir.join("token-skype.jwt");
    let chatsvcagg_path = config_dir.join("token-chatsvcagg.jwt");

    if !teams_path.exists() || !skype_path.exists() || !chatsvcagg_path.exists() {
        return Ok(None);
    }

    let teams_raw = std::fs::read_to_string(&teams_path)
        .map_err(|e| TeamsError::AuthError(format!("failed to read teams token: {e}")))?
        .trim()
        .to_string();
    let skype_raw = std::fs::read_to_string(&skype_path)
        .map_err(|e| TeamsError::AuthError(format!("failed to read skype token: {e}")))?
        .trim()
        .to_string();
    let chatsvcagg_raw = std::fs::read_to_string(&chatsvcagg_path)
        .map_err(|e| TeamsError::AuthError(format!("failed to read chatsvcagg token: {e}")))?
        .trim()
        .to_string();

    let tenant_id = token::extract_tenant_id(&teams_raw)?.unwrap_or_else(|| "common".to_string());

    Ok(Some(TokenSet {
        teams: token::TokenInfo::from_jwt(&teams_raw, token::TokenType::IdToken)?,
        skype: token::TokenInfo::from_jwt(&skype_raw, token::TokenType::AccessToken)?,
        chatsvcagg: token::TokenInfo::from_jwt(&chatsvcagg_raw, token::TokenType::AccessToken)?,
        profile: profile.to_string(),
        tenant_id,
    }))
}

/// Get tokens, performing auto-login if needed and allowed.
pub async fn get_or_login(profile: &str, tenant: &str, auto_login: bool) -> Result<TokenSet> {
    match resolve_tokens(profile)? {
        Some(ts) if !ts.is_expired() => Ok(ts),
        Some(_) if auto_login => {
            tracing::info!("tokens expired, re-authenticating...");
            do_device_code_login(tenant, profile).await
        }
        Some(_) => Err(TeamsError::TokenExpired),
        None if auto_login => {
            tracing::info!("no tokens found, authenticating...");
            do_device_code_login(tenant, profile).await
        }
        None => Err(TeamsError::AuthError("not logged in".into())),
    }
}

async fn do_device_code_login(tenant: &str, profile: &str) -> Result<TokenSet> {
    let token_set = device_code::device_code_login(tenant, profile).await?;
    keyring::store_tokens(profile, &token_set)?;
    Ok(token_set)
}

/// Execute a function with automatic re-authentication on auth errors.
pub async fn with_auto_reauth<T, F, Fut>(
    profile: &str,
    tenant: &str,
    tokens: &mut TokenSet,
    f: F,
) -> Result<T>
where
    F: Fn(TokenSet) -> Fut,
    Fut: Future<Output = Result<T>>,
{
    match f(tokens.clone()).await {
        Ok(v) => Ok(v),
        Err(e) if e.is_auth_error() => {
            tracing::info!("auth error, attempting re-authentication...");
            let new_tokens = do_device_code_login(tenant, profile).await?;
            *tokens = new_tokens;
            f(tokens.clone()).await
        }
        Err(e) => Err(e),
    }
}
