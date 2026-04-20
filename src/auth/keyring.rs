use std::fs;
use std::path::PathBuf;

use crate::error::{Result, TeamsError};

use super::token::TokenSet;

fn tokens_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("teams-cli")
        .join("tokens")
}

fn token_path(profile: &str) -> PathBuf {
    tokens_dir().join(format!("{profile}.json"))
}

pub fn store_tokens(profile: &str, token_set: &TokenSet) -> Result<()> {
    let dir = tokens_dir();
    fs::create_dir_all(&dir).map_err(|e| TeamsError::KeyringError(e.to_string()))?;

    let json = serde_json::to_string_pretty(token_set)
        .map_err(|e| TeamsError::KeyringError(format!("failed to serialize tokens: {e}")))?;

    let path = token_path(profile);
    fs::write(&path, &json).map_err(|e| TeamsError::KeyringError(e.to_string()))?;

    // Restrict permissions on the file (unix only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o600));
    }

    Ok(())
}

pub fn get_tokens(profile: &str) -> Result<Option<TokenSet>> {
    let path = token_path(profile);
    if !path.exists() {
        return Ok(None);
    }

    let json = fs::read_to_string(&path).map_err(|e| TeamsError::KeyringError(e.to_string()))?;

    let token_set: TokenSet = serde_json::from_str(&json)
        .map_err(|e| TeamsError::KeyringError(format!("failed to parse tokens: {e}")))?;

    Ok(Some(token_set))
}

pub fn delete_tokens(profile: &str) -> Result<()> {
    let path = token_path(profile);
    if path.exists() {
        fs::remove_file(&path).map_err(|e| TeamsError::KeyringError(e.to_string()))?;
    }
    Ok(())
}

pub fn list_profiles() -> Result<Vec<String>> {
    let dir = tokens_dir();
    if !dir.exists() {
        return Ok(vec![]);
    }

    let mut profiles = vec![];
    let entries = fs::read_dir(&dir).map_err(|e| TeamsError::KeyringError(e.to_string()))?;
    for entry in entries.flatten() {
        if let Some(name) = entry.path().file_stem() {
            profiles.push(name.to_string_lossy().to_string());
        }
    }
    Ok(profiles)
}
