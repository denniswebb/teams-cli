use clap::{Args, Subcommand};
use std::io::Read;
use std::time::Instant;

use crate::api::messages::MessagesClient;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_html_paragraph() {
        assert_eq!(strip_html("<p>hello</p>"), "hello");
    }

    #[test]
    fn strip_html_no_tags() {
        assert_eq!(strip_html("no tags"), "no tags");
    }

    #[test]
    fn strip_html_empty_string() {
        assert_eq!(strip_html(""), "");
    }

    #[test]
    fn strip_html_bold_and_italic() {
        assert_eq!(
            strip_html("<b>bold</b> and <i>italic</i>"),
            "bold and italic"
        );
    }

    #[test]
    fn strip_html_nested_tags() {
        assert_eq!(
            strip_html("<div>nested<span>tags</span></div>"),
            "nestedtags"
        );
    }

    #[test]
    fn strip_html_self_closing_tag() {
        assert_eq!(strip_html("<br/>"), "");
    }

    #[test]
    fn strip_html_multiple_self_closing() {
        assert_eq!(strip_html("a<br/>b<hr/>c"), "abc");
    }

    #[test]
    fn strip_html_angle_brackets_in_text() {
        // "a < b > c" - the '<' starts a "tag", ' b ' is treated as tag content,
        // '>' ends the "tag", then ' c' is text. So we lose ' b '.
        // The result after trim is "a  c" (with the space before '<' and after '>').
        let result = strip_html("a < b > c");
        assert_eq!(result, "a  c");
    }

    #[test]
    fn strip_html_unclosed_tag() {
        // "unclosed <div tag" - '<' starts a tag, everything after is inside the tag
        // because there's no closing '>'. So only "unclosed " is kept, trimmed to "unclosed".
        let result = strip_html("unclosed <div tag");
        assert_eq!(result, "unclosed");
    }

    #[test]
    fn strip_html_complex_html() {
        let html = "<html><body><h1>Title</h1><p>Some <b>bold</b> text</p></body></html>";
        assert_eq!(strip_html(html), "TitleSome bold text");
    }

    #[test]
    fn strip_html_with_attributes() {
        assert_eq!(
            strip_html(r#"<a href="https://example.com">link</a>"#),
            "link"
        );
    }

    #[test]
    fn strip_html_whitespace_only_content() {
        // After stripping tags, only whitespace remains -> trimmed to empty
        assert_eq!(strip_html("<p>  </p>"), "");
    }

    #[test]
    fn strip_html_preserves_inner_whitespace() {
        assert_eq!(strip_html("<p>hello   world</p>"), "hello   world");
    }
}
