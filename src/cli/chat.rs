use clap::{Args, Subcommand};
use std::time::Instant;

use crate::api::csa::CsaClient;
use crate::api::HttpClient;
use crate::auth::token::TokenSet;
use crate::error::Result;
use crate::output::{self, OutputFormat};

#[derive(Args)]
pub struct ChatArgs {
    #[command(subcommand)]
    pub command: ChatCommand,
}

#[derive(Subcommand)]
pub enum ChatCommand {
    /// List direct and group chats
    List {
        /// Show hidden chats too
        #[arg(long)]
        all: bool,
    },
    /// Get chat details
    Get {
        /// Chat ID
        chat_id: String,
    },
}

pub async fn handle(
    args: &ChatArgs,
    tokens: &TokenSet,
    http: &HttpClient,
    format: OutputFormat,
) -> Result<()> {
    let csa = CsaClient::new(http, tokens);

    match &args.command {
        ChatCommand::List { all } => {
            let start = Instant::now();
            let conversations = csa.get_conversations().await?;
            let chats: Vec<serde_json::Value> = conversations
                .chats
                .iter()
                .filter(|c| {
                    *all || !c
                        .get("hidden")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false)
                })
                .map(|c| {
                    let title = c
                        .get("title")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    serde_json::json!({
                        "id": c.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                        "title": title,
                        "chat_type": c.get("chatType").and_then(|v| v.as_str()).unwrap_or(""),
                        "members": c.get("members").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0),
                    })
                })
                .collect();
            output::print_output(format, chats, start.elapsed().as_millis() as u64);
        }
        ChatCommand::Get { chat_id } => {
            let start = Instant::now();
            let conversations = csa.get_conversations().await?;
            let chat = conversations
                .chats
                .iter()
                .find(|c| c.get("id").and_then(|v| v.as_str()) == Some(chat_id.as_str()))
                .ok_or_else(|| crate::error::TeamsError::NotFound(format!("chat {chat_id}")))?;
            output::print_output(format, chat, start.elapsed().as_millis() as u64);
        }
    }
    Ok(())
}
