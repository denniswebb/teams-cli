mod api;
mod auth;
mod cli;
mod config;
mod error;
mod models;
mod output;

use clap::Parser;
use tracing_subscriber::EnvFilter;

use cli::{Cli, Commands};
use output::OutputFormat;

fn main() {
    let cli = Cli::parse();

    // Set up tracing
    let filter = match cli.verbose {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(filter)),
        )
        .with_writer(std::io::stderr)
        .init();

    let format = OutputFormat::detect(cli.output.as_deref());

    // Webview login must run on the main thread (tao/wry requirement).
    if let Commands::Auth(ref auth_args) = cli.command {
        if let cli::auth::AuthCommand::Login {
            device_code: false,
            ref tenant,
        } = auth_args.command
        {
            auth::webview::webview_login(tenant, &cli.profile);
        }
    }

    let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
    rt.block_on(async {
        if let Err(e) = run(cli, format).await {
            output::print_error(format, e.error_code(), &e.to_string(), 0);
            std::process::exit(e.exit_code());
        }
    });
}

async fn run(cli: Cli, format: OutputFormat) -> error::Result<()> {
    let cfg = config::Config::load()?;

    let profile = &cli.profile;
    let region = &cli.region;

    let network = config::NetworkConfig {
        timeout: cli.timeout.unwrap_or(cfg.network.timeout),
        max_retries: cli.retry.unwrap_or(cfg.network.max_retries),
        retry_backoff_base: cfg.network.retry_backoff_base,
    };

    match &cli.command {
        Commands::Auth(args) => cli::auth::handle(args, profile, format).await,
        Commands::Config(args) => cli::config_cmd::handle(args, format),
        Commands::Completions { shell } => {
            let mut cmd = <Cli as clap::CommandFactory>::command();
            clap_complete::generate(*shell, &mut cmd, "teams", &mut std::io::stdout());
            Ok(())
        }

        // All other commands need auth + authz token exchange
        cmd => {
            let tenant = cfg.profile(profile).tenant_id.clone();
            let tokens = auth::get_or_login(profile, &tenant, cli.auto_login).await?;
            let http = api::HttpClient::new(&network);

            // Exchange OAuth token for messaging skype token + discover region
            let authz = api::authz::exchange_token(&http, &tokens).await?;
            let chat_service_url = &authz.region_gtms.chat_service;
            let mt_url = &authz.region_gtms.middle_tier;
            let messaging_token = &authz.tokens.skype_token;

            match cmd {
                Commands::User(args) => {
                    cli::user::handle(args, &tokens, &http, mt_url, format).await
                }
                Commands::Team(args) => cli::team::handle(args, &tokens, &http, format).await,
                Commands::Channel(args) => cli::channel::handle(args, &tokens, &http, format).await,
                Commands::Chat(args) => cli::chat::handle(args, &tokens, &http, format).await,
                Commands::Message(args) => {
                    cli::message::handle(
                        args,
                        &tokens,
                        messaging_token,
                        &http,
                        chat_service_url,
                        format,
                    )
                    .await
                }
                Commands::Tenant(args) => {
                    cli::tenant::handle(args, &tokens, &http, region, format).await
                }
                _ => unreachable!(),
            }
        }
    }
}
