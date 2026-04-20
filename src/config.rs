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
            .or_else(|| dirs::home_dir().map(|h| h.join(".config")))
            .unwrap_or_else(|| PathBuf::from("/tmp"))
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
            Self::create_config_dir(parent)?;
        }
        let contents =
            toml::to_string_pretty(self).map_err(|e| TeamsError::ConfigError(e.to_string()))?;

        #[cfg(unix)]
        {
            use std::io::Write;
            use std::os::unix::fs::OpenOptionsExt;
            let mut opts = std::fs::OpenOptions::new();
            opts.write(true).create(true).truncate(true).mode(0o600);
            let mut file = opts
                .open(&path)
                .map_err(|e| TeamsError::ConfigError(e.to_string()))?;
            file.write_all(contents.as_bytes())
                .map_err(|e| TeamsError::ConfigError(e.to_string()))?;
        }
        #[cfg(not(unix))]
        {
            fs::write(&path, contents).map_err(|e| TeamsError::ConfigError(e.to_string()))?;
        }

        Ok(())
    }

    #[cfg(unix)]
    fn create_config_dir(dir: &std::path::Path) -> Result<()> {
        use std::os::unix::fs::DirBuilderExt;
        std::fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(dir)
            .map_err(|e| TeamsError::ConfigError(e.to_string()))
    }

    #[cfg(not(unix))]
    fn create_config_dir(dir: &std::path::Path) -> Result<()> {
        fs::create_dir_all(dir).map_err(|e| TeamsError::ConfigError(e.to_string()))
    }

    pub fn profile(&self, name: &str) -> &ProfileConfig {
        static DEFAULT_PROFILE: std::sync::LazyLock<ProfileConfig> =
            std::sync::LazyLock::new(|| ProfileConfig {
                tenant_id: default_tenant(),
            });
        self.profiles.get(name).unwrap_or(&DEFAULT_PROFILE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Config::default() ---

    #[test]
    fn default_profile_value() {
        let cfg = Config::default();
        assert_eq!(cfg.default.profile, "default");
    }

    #[test]
    fn default_region_value() {
        let cfg = Config::default();
        assert_eq!(cfg.default.region, "emea");
    }

    #[test]
    fn default_output_format() {
        let cfg = Config::default();
        assert_eq!(cfg.output.format, "auto");
    }

    #[test]
    fn default_output_color() {
        let cfg = Config::default();
        assert!(cfg.output.color);
    }

    #[test]
    fn default_network_timeout() {
        let cfg = Config::default();
        assert_eq!(cfg.network.timeout, 30);
    }

    #[test]
    fn default_network_max_retries() {
        let cfg = Config::default();
        assert_eq!(cfg.network.max_retries, 3);
    }

    #[test]
    fn default_network_retry_backoff_base() {
        let cfg = Config::default();
        assert_eq!(cfg.network.retry_backoff_base, 2);
    }

    #[test]
    fn default_profiles_empty() {
        let cfg = Config::default();
        assert!(cfg.profiles.is_empty());
    }

    // --- TOML round-trip ---

    #[test]
    fn toml_round_trip() {
        let cfg = Config::default();
        let toml_str = toml::to_string_pretty(&cfg).unwrap();
        let parsed: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(cfg.default.profile, parsed.default.profile);
        assert_eq!(cfg.default.region, parsed.default.region);
        assert_eq!(cfg.output.format, parsed.output.format);
        assert_eq!(cfg.output.color, parsed.output.color);
        assert_eq!(cfg.network.timeout, parsed.network.timeout);
        assert_eq!(cfg.network.max_retries, parsed.network.max_retries);
        assert_eq!(
            cfg.network.retry_backoff_base,
            parsed.network.retry_backoff_base
        );
    }

    // --- TOML deserialization with missing fields ---

    #[test]
    fn toml_partial_uses_defaults() {
        let cfg: Config = toml::from_str("[default]\nprofile = \"myprofile\"\n").unwrap();
        assert_eq!(cfg.default.profile, "myprofile");
        assert_eq!(cfg.default.region, "emea");
        assert_eq!(cfg.network.timeout, 30);
        assert_eq!(cfg.network.max_retries, 3);
        assert_eq!(cfg.output.format, "auto");
        assert!(cfg.output.color);
    }

    #[test]
    fn toml_empty_string_uses_all_defaults() {
        let cfg: Config = toml::from_str("").unwrap();
        assert_eq!(cfg.default.profile, "default");
        assert_eq!(cfg.default.region, "emea");
    }

    #[test]
    fn toml_with_custom_network() {
        let toml_str = r#"
[network]
timeout = 60
max_retries = 5
retry_backoff_base = 3
"#;
        let cfg: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(cfg.network.timeout, 60);
        assert_eq!(cfg.network.max_retries, 5);
        assert_eq!(cfg.network.retry_backoff_base, 3);
        // other sections should still be defaults
        assert_eq!(cfg.default.profile, "default");
    }

    // --- Config::profile() ---

    #[test]
    fn profile_existing_returns_it() {
        let mut cfg = Config::default();
        cfg.profiles.insert(
            "work".into(),
            ProfileConfig {
                tenant_id: "my-tenant".into(),
            },
        );
        let p = cfg.profile("work");
        assert_eq!(p.tenant_id, "my-tenant");
    }

    #[test]
    fn profile_missing_returns_default() {
        let cfg = Config::default();
        let p = cfg.profile("nonexistent");
        assert_eq!(p.tenant_id, "common");
    }

    // --- config_dir() ---

    #[test]
    fn config_dir_ends_with_teams_cli() {
        let dir = Config::config_dir();
        assert!(
            dir.ends_with("teams-cli"),
            "config_dir should end with 'teams-cli', got: {:?}",
            dir
        );
    }

    #[test]
    fn config_path_ends_with_config_toml() {
        let path = Config::config_path();
        assert!(
            path.ends_with("teams-cli/config.toml"),
            "config_path should end with 'teams-cli/config.toml', got: {:?}",
            path
        );
    }

    // --- TOML with profiles ---

    #[test]
    fn toml_with_profiles() {
        let toml_str = r#"
[default]
profile = "work"

[profiles.work]
tenant_id = "abc-123"

[profiles.personal]
tenant_id = "def-456"
"#;
        let cfg: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(cfg.default.profile, "work");
        assert_eq!(cfg.profiles.len(), 2);
        assert_eq!(cfg.profiles["work"].tenant_id, "abc-123");
        assert_eq!(cfg.profiles["personal"].tenant_id, "def-456");
    }
}
