use clap::{Args, Subcommand};
use std::time::Instant;

use crate::api::mt::MtClient;
use crate::api::HttpClient;
use crate::auth::token::TokenSet;
use crate::error::Result;
use crate::output::{self, OutputFormat};

#[derive(Args)]
pub struct TenantArgs {
    #[command(subcommand)]
    pub command: TenantCommand,
}

#[derive(Subcommand)]
pub enum TenantCommand {
    /// List tenants
    List,
    /// List verified domains for current tenant
    Domains,
}

pub async fn handle(
    args: &TenantArgs,
    tokens: &TokenSet,
    http: &HttpClient,
    region: &str,
    format: OutputFormat,
) -> Result<()> {
    let mt = MtClient::new(http, tokens, region);

    match &args.command {
        TenantCommand::List => {
            let start = Instant::now();
            let tenants = mt.get_tenants().await?;
            output::print_output(format, tenants, start.elapsed().as_millis() as u64);
        }
        TenantCommand::Domains => {
            let start = Instant::now();
            let domains = mt.get_verified_domains().await?;
            output::print_output(format, domains, start.elapsed().as_millis() as u64);
        }
    }
    Ok(())
}
