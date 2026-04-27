use clap::{Args, Subcommand};
use std::io::{BufRead, Write};
use std::time::Instant;

use crate::api::copilot::CopilotClient;
use crate::auth::token::TokenSet;
use crate::error::{Result, TeamsError};
use crate::output::{self, OutputFormat};

#[derive(Args)]
pub struct CopilotArgs {
    #[command(subcommand)]
    pub command: CopilotCommand,
}

#[derive(Subcommand)]
pub enum CopilotCommand {
    /// Ask Copilot a question (one-shot)
    Ask {
        /// The question to ask
        question: String,
        /// Continue an existing conversation
        #[arg(long)]
        conversation: Option<String>,
    },
    /// Start an interactive chat session with Copilot
    Chat,
}

pub async fn handle(args: &CopilotArgs, tokens: &TokenSet, format: OutputFormat) -> Result<()> {
    let client = CopilotClient::new(tokens);

    match &args.command {
        CopilotCommand::Ask {
            question,
            conversation,
        } => {
            let start = Instant::now();
            let response = client.ask(question, conversation.as_deref()).await?;

            let display = serde_json::json!({
                "conversation_id": response.conversation_id,
                "response": response.message,
                "citations": response.citations,
            });

            output::print_output(format, display, start.elapsed().as_millis() as u64);
        }

        CopilotCommand::Chat => {
            eprintln!("Starting interactive Copilot chat (Ctrl+D to exit)");
            eprintln!();

            let mut conversation_id: Option<String> = None;
            let stdin = std::io::stdin();

            loop {
                eprint!("You> ");
                std::io::stderr().flush().ok();

                let mut line = String::new();
                match stdin.lock().read_line(&mut line) {
                    Ok(0) => break, // EOF
                    Ok(_) => {}
                    Err(e) => {
                        return Err(TeamsError::Other(anyhow::anyhow!("stdin read error: {e}")));
                    }
                }

                let question = line.trim();
                if question.is_empty() {
                    continue;
                }

                let response = client.ask(question, conversation_id.as_deref()).await?;

                conversation_id = Some(response.conversation_id.clone());

                eprintln!();
                eprintln!("Copilot> {}", response.message);
                eprintln!();
            }

            eprintln!("Chat ended.");
        }
    }

    Ok(())
}
