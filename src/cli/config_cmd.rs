use clap::{Args, Subcommand};

use crate::config::Config;
use crate::error::Result;
use crate::output::{self, OutputFormat};

#[derive(Args)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub command: ConfigCommand,
}

#[derive(Subcommand)]
pub enum ConfigCommand {
    /// Create default configuration file
    Init,
    /// Display current configuration
    Show,
    /// Set a configuration value
    Set {
        /// Config key (e.g., default.region, network.timeout)
        key: String,
        /// Value to set
        value: String,
    },
    /// Print config file location
    Path,
}

pub fn handle(args: &ConfigArgs, format: OutputFormat) -> Result<()> {
    match &args.command {
        ConfigCommand::Init => {
            let config = Config::default();
            config.save()?;
            eprintln!("Created config at {}", Config::config_path().display());
            Ok(())
        }
        ConfigCommand::Show => {
            let config = Config::load()?;
            output::print_output(format, &config, 0);
            Ok(())
        }
        ConfigCommand::Set { key, value } => {
            let mut config = Config::load()?;
            match key.as_str() {
                "default.profile" => config.default.profile = value.clone(),
                "default.region" => config.default.region = value.clone(),
                "output.format" => config.output.format = value.clone(),
                "output.color" => {
                    config.output.color = value.parse().map_err(|_| {
                        crate::error::TeamsError::InvalidInput("expected true/false".into())
                    })?;
                }
                "network.timeout" => {
                    config.network.timeout = value.parse().map_err(|_| {
                        crate::error::TeamsError::InvalidInput("expected number".into())
                    })?;
                }
                "network.max_retries" => {
                    config.network.max_retries = value.parse().map_err(|_| {
                        crate::error::TeamsError::InvalidInput("expected number".into())
                    })?;
                }
                other => {
                    return Err(crate::error::TeamsError::InvalidInput(format!(
                        "unknown config key: {other}"
                    )));
                }
            }
            config.save()?;
            eprintln!("Set {key} = {value}");
            Ok(())
        }
        ConfigCommand::Path => {
            println!("{}", Config::config_path().display());
            Ok(())
        }
    }
}
