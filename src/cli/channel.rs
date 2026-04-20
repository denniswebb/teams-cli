use clap::{Args, Subcommand};
use std::time::Instant;

use crate::api::csa::CsaClient;
use crate::api::HttpClient;
use crate::auth::token::TokenSet;
use crate::error::Result;
use crate::output::{self, OutputFormat};

#[derive(Args)]
pub struct ChannelArgs {
    #[command(subcommand)]
    pub command: ChannelCommand,
}

#[derive(Subcommand)]
pub enum ChannelCommand {
    /// List channels in a team
    List {
        /// Team ID
        team_id: String,
    },
    /// Get channel details
    Get {
        /// Team ID
        team_id: String,
        /// Channel ID
        channel_id: String,
    },
    /// List pinned channels
    Pinned,
}

pub async fn handle(
    args: &ChannelArgs,
    tokens: &TokenSet,
    http: &HttpClient,
    format: OutputFormat,
) -> Result<()> {
    let csa = CsaClient::new(http, tokens);

    match &args.command {
        ChannelCommand::List { team_id } => {
            let start = Instant::now();
            let conversations = csa.get_conversations().await?;
            let team = conversations
                .teams
                .iter()
                .find(|t| t.get("id").and_then(|v| v.as_str()) == Some(team_id.as_str()))
                .ok_or_else(|| crate::error::TeamsError::NotFound(format!("team {team_id}")))?;

            let empty = vec![];
            let channels: Vec<serde_json::Value> = team
                .get("channels")
                .and_then(|v| v.as_array())
                .unwrap_or(&empty)
                .iter()
                .map(|c| {
                    serde_json::json!({
                        "id": c.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                        "display_name": c.get("displayName").and_then(|v| v.as_str()).unwrap_or(""),
                        "is_general": c.get("isGeneral").and_then(|v| v.as_bool()).unwrap_or(false),
                        "is_favorite": c.get("isFavorite").and_then(|v| v.as_bool()).unwrap_or(false),
                        "is_pinned": c.get("isPinned").and_then(|v| v.as_bool()).unwrap_or(false),
                    })
                })
                .collect();
            output::print_output(format, channels, start.elapsed().as_millis() as u64);
        }
        ChannelCommand::Get {
            team_id,
            channel_id,
        } => {
            let start = Instant::now();
            let conversations = csa.get_conversations().await?;
            let team = conversations
                .teams
                .iter()
                .find(|t| t.get("id").and_then(|v| v.as_str()) == Some(team_id.as_str()))
                .ok_or_else(|| crate::error::TeamsError::NotFound(format!("team {team_id}")))?;
            let empty2 = vec![];
            let channel = team
                .get("channels")
                .and_then(|v| v.as_array())
                .unwrap_or(&empty2)
                .iter()
                .find(|c| c.get("id").and_then(|v| v.as_str()) == Some(channel_id.as_str()))
                .ok_or_else(|| {
                    crate::error::TeamsError::NotFound(format!("channel {channel_id}"))
                })?;
            output::print_output(format, channel, start.elapsed().as_millis() as u64);
        }
        ChannelCommand::Pinned => {
            let start = Instant::now();
            let pinned = csa.get_pinned_channels().await?;
            output::print_output(
                format,
                pinned.pin_channel_order,
                start.elapsed().as_millis() as u64,
            );
        }
    }
    Ok(())
}
