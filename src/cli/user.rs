use clap::{Args, Subcommand};
use std::time::Instant;

use crate::api::mt::MtClient;
use crate::api::HttpClient;
use crate::auth::token::TokenSet;
use crate::error::Result;
use crate::output::{self, OutputFormat};

#[derive(Args)]
pub struct UserArgs {
    #[command(subcommand)]
    pub command: UserCommand,
}

#[derive(Subcommand)]
pub enum UserCommand {
    /// Get current user profile
    Me,
    /// Lookup user by email
    Get {
        /// User email address
        email: String,
    },
    /// Search users by MRI identifiers
    Search {
        /// Comma-separated MRI identifiers
        mris: String,
    },
}

pub async fn handle(
    args: &UserArgs,
    tokens: &TokenSet,
    http: &HttpClient,
    region: &str,
    format: OutputFormat,
) -> Result<()> {
    let mt = MtClient::new(http, tokens, region);

    match &args.command {
        UserCommand::Me => {
            let start = Instant::now();
            let user = mt.get_me().await?;
            output::print_output(format, user, start.elapsed().as_millis() as u64);
        }
        UserCommand::Get { email } => {
            let start = Instant::now();
            let user = mt.get_user(email).await?;
            output::print_output(format, user, start.elapsed().as_millis() as u64);
        }
        UserCommand::Search { mris } => {
            let start = Instant::now();
            let mri_list: Vec<String> = mris.split(',').map(|s| s.trim().to_string()).collect();
            let users = mt.fetch_short_profiles(&mri_list).await?;
            output::print_output(format, users, start.elapsed().as_millis() as u64);
        }
    }
    Ok(())
}
