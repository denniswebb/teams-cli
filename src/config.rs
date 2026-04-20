use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::{Result, TeamsError};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub default: DefaultConfig,
    #[serde(default)]
    pub output: OutputConfig,
    #[serde(default)]
    pub network: NetworkConfig,
    #[serde(default)]
    pub profiles: HashMap<String, ProfileConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultConfig {
    #[serde(default = "default_profile")]
    pub profile: String,
    #[serde(default = "default_region")]
    pub region: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    #[serde(default = "default_format")]
    pub format: String,
    #[serde(default = "default_true")]
    pub color: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    #[serde(default = "default_timeout")]
    pub timeout: u64,
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
    #[serde(default = "default_backoff_base")]
    pub retry_backoff_base: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileConfig {
    #[serde(default = "default_tenant")]
    pub tenant_id: String,
}

fn default_profile() -> String {
    "default".into()
}
fn default_region() -> String {
    "emea".into()
}
fn default_format() -> String {
    "auto".into()
}
fn default_true() -> bool {
    true
}
fn default_timeout() -> u64 {
    30
}
fn default_max_retries() -> u32 {
    3
}
fn default_backoff_base() -> u64 {
    2
}
fn default_tenant() -> String {
    "common".into()
}

impl Default for DefaultConfig {
    fn default() -> Self {
        Self {
            profile: default_profile(),
            region: default_region(),
        }
    }
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            format: default_format(),
            color: default_true(),
        }
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            timeout: default_timeout(),
            max_retries: default_max_retries(),
            retry_backoff_base: default_backoff_base(),
        }
    }
}

impl Config {
    pub fn config_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("~/.config"))
            .join("teams-cli")
    }

    pub fn config_path() -> PathBuf {
        Self::config_dir().join("config.toml")
    }

    pub fn load() -> Result<Self> {
        let path = Self::config_path();
        if !path.exists() {
            return Ok(Config::default());
        }
        let contents =
            fs::read_to_string(&path).map_err(|e| TeamsError::ConfigError(e.to_string()))?;
        toml::from_str(&contents).map_err(|e| TeamsError::ConfigError(e.to_string()))
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| TeamsError::ConfigError(e.to_string()))?;
        }
        let contents =
            toml::to_string_pretty(self).map_err(|e| TeamsError::ConfigError(e.to_string()))?;
        fs::write(&path, contents).map_err(|e| TeamsError::ConfigError(e.to_string()))
    }

    pub fn profile(&self, name: &str) -> &ProfileConfig {
        static DEFAULT_PROFILE: std::sync::LazyLock<ProfileConfig> =
            std::sync::LazyLock::new(|| ProfileConfig {
                tenant_id: default_tenant(),
            });
        self.profiles.get(name).unwrap_or(&DEFAULT_PROFILE)
    }
}
