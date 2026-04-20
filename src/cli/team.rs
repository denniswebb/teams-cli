use clap::{Args, Subcommand};
use std::time::Instant;

use crate::api::csa::CsaClient;
use crate::api::HttpClient;
use crate::auth::token::TokenSet;
use crate::error::Result;
use crate::output::{self, OutputFormat};

#[derive(Args)]
pub struct TeamArgs {
    #[command(subcommand)]
    pub command: TeamCommand,
}

#[derive(Subcommand)]
pub enum TeamCommand {
    /// List joined teams
    List,
    /// Get team details
    Get {
        /// Team ID
        id: String,
    },
}

pub async fn handle(
    args: &TeamArgs,
    tokens: &TokenSet,
    http: &HttpClient,
    format: OutputFormat,
) -> Result<()> {
    let csa = CsaClient::new(http, tokens);

    match &args.command {
        TeamCommand::List => {
            let start = Instant::now();
            let conversations = csa.get_conversations().await?;
            let teams: Vec<serde_json::Value> = conversations
                .teams
                .iter()
                .map(|t| {
                    serde_json::json!({
                        "id": t.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                        "display_name": t.get("displayName").and_then(|v| v.as_str()).unwrap_or(""),
                        "description": t.get("description").and_then(|v| v.as_str()).unwrap_or(""),
                        "is_archived": t.get("isArchived").and_then(|v| v.as_bool()).unwrap_or(false),
                        "is_favorite": t.get("isFavorite").and_then(|v| v.as_bool()).unwrap_or(false),
                        "channels": t.get("channels").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0),
                    })
                })
                .collect();
            output::print_output(format, teams, start.elapsed().as_millis() as u64);
        }
        TeamCommand::Get { id } => {
            let start = Instant::now();
            let conversations = csa.get_conversations().await?;
            let team = conversations
                .teams
                .iter()
                .find(|t| t.get("id").and_then(|v| v.as_str()) == Some(id.as_str()))
                .ok_or_else(|| crate::error::TeamsError::NotFound(format!("team {id}")))?;
            output::print_output(format, team, start.elapsed().as_millis() as u64);
        }
    }
    Ok(())
}
