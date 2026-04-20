use clap::{Args, Subcommand};
use std::io::Read;
use std::time::Instant;

use crate::api::messages::MessagesClient;
use crate::api::mt::MtClient;
use crate::api::HttpClient;
use crate::auth::token::TokenSet;
use crate::error::Result;
use crate::output::{self, OutputFormat};

#[derive(Args)]
pub struct MessageArgs {
    #[command(subcommand)]
    pub command: MessageCommand,
}

#[derive(Subcommand)]
pub enum MessageCommand {
    /// List messages in a conversation
    List {
        /// Conversation ID (channel or chat thread ID)
        conversation_id: String,
        /// Maximum number of messages to fetch
        #[arg(long, default_value = "50")]
        limit: u32,
    },
    /// Send a message to a conversation
    Send {
        /// Conversation ID (channel or chat thread ID)
        conversation_id: String,
        /// Message body text
        #[arg(long)]
        body: Option<String>,
        /// Read message body from stdin
        #[arg(long)]
        stdin: bool,
    },
    /// Get a specific message
    Get {
        /// Conversation ID
        conversation_id: String,
        /// Message ID
        message_id: String,
    },
}

pub async fn handle(
    args: &MessageArgs,
    tokens: &TokenSet,
    messaging_token: &str,
    http: &HttpClient,
    chat_service_url: &str,
    format: OutputFormat,
) -> Result<()> {
    let msg_client = MessagesClient::new(http, messaging_token, chat_service_url);

    match &args.command {
        MessageCommand::List {
            conversation_id,
            limit,
        } => {
            let start = Instant::now();
            let messages = msg_client.get_messages(conversation_id, *limit).await?;
            let display: Vec<serde_json::Value> = messages
                .iter()
                .filter(|m| m.message_type == "RichText/Html" || m.message_type == "Text")
                .map(|m| {
                    serde_json::json!({
                        "id": m.id,
                        "from": m.im_display_name,
                        "content": strip_html(&m.content),
                        "time": m.compose_time,
                        "type": m.message_type,
                    })
                })
                .collect();
            output::print_output(format, display, start.elapsed().as_millis() as u64);
        }
        MessageCommand::Send {
            conversation_id,
            body,
            stdin,
        } => {
            let content = if *stdin {
                let mut buf = String::new();
                std::io::stdin()
                    .read_to_string(&mut buf)
                    .map_err(|e| crate::error::TeamsError::InvalidInput(format!("stdin: {e}")))?;
                buf
            } else {
                body.clone().ok_or_else(|| {
                    crate::error::TeamsError::InvalidInput("provide --body or --stdin".into())
                })?
            };

            // Get display name
            let display_name = crate::auth::token::extract_username(&tokens.teams.raw)
                .unwrap_or_else(|_| "Unknown".into());

            let start = Instant::now();
            let result = msg_client
                .send_message(conversation_id, &content, &display_name)
                .await?;
            output::print_output(format, result, start.elapsed().as_millis() as u64);
        }
        MessageCommand::Get {
            conversation_id,
            message_id,
        } => {
            let start = Instant::now();
            let messages = msg_client.get_messages(conversation_id, 200).await?;
            let message = messages
                .iter()
                .find(|m| m.id == *message_id)
                .ok_or_else(|| {
                    crate::error::TeamsError::NotFound(format!("message {message_id}"))
                })?;
            output::print_output(format, message, start.elapsed().as_millis() as u64);
        }
    }
    Ok(())
}

fn strip_html(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }
    result.trim().to_string()
}
