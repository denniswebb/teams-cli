pub mod device_code;
pub mod keyring;
pub mod token;
pub mod webview;

use crate::error::{Result, TeamsError};
use token::TokenSet;

/// Resolve tokens for the given profile.
/// Priority: env vars > keyring (if valid) > keyring (expired)
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

    // Check keyring (this is where webview login stores tokens)
    keyring::get_tokens(profile)
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
