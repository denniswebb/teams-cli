# AGENTS.md

## Project Overview

teams-cli is a Rust CLI for Microsoft Teams using the internal Skype/CSA APIs
(not the Microsoft Graph API). It uses the same endpoints as the official Teams
client.

## Build and Test

```sh
cargo build
cargo test --all-targets
cargo fmt -- --check
cargo clippy --all-targets -- -D warnings
```

## Architecture

```
src/
  main.rs           -- Entry point. Webview login runs before tokio starts (main thread requirement).
  error.rs          -- TeamsError enum with deterministic exit codes (0-10).
  config.rs         -- TOML config at ~/.config/teams-cli/config.toml
  cli/              -- clap command definitions and handlers
    mod.rs          -- Cli struct, Commands enum, global flags
    auth.rs         -- login (webview/device-code), status, logout, token
    user.rs         -- me, get, search
    team.rs         -- list, get
    channel.rs      -- list, get, pinned
    chat.rs         -- list (--all for hidden), get
    message.rs      -- list, send (--body or --stdin), get
    tenant.rs       -- list, domains
    config_cmd.rs   -- init, show, set, path
  auth/
    mod.rs          -- Token resolution: env vars > file store > fossteams compat
    token.rs        -- TokenSet, TokenInfo, JWT decode, constants
    webview.rs      -- tao/wry webview for 3-token OAuth2 implicit flow (diverges via process::exit)
    device_code.rs  -- OAuth2 device code flow fallback for headless
    keyring.rs      -- File-based token storage at ~/.config/teams-cli/tokens/<profile>.json
  api/
    mod.rs          -- HttpClient with retry/backoff
    authz.rs        -- Token exchange: POST authsvc/v1.0/authz -> skypeToken + regionGtms
    csa.rs          -- Chat Service Aggregator: teams, channels, chats
    messages.rs     -- Messages API: read/write messages (uses authz skypeToken)
    mt.rs           -- MiddleTier: user profiles, tenants (uses OAuth skype Bearer token)
  models/           -- Serde structs for API responses
  output/           -- JSON envelope, table, plain text formatters
```

## Authentication Flow

1. **Webview login** (`teams auth login`): Opens native webview via tao/wry,
   navigates to `login.microsoftonline.com/oauth2/authorize` three times to
   capture Teams id_token, Skype access_token, and ChatSvcAgg access_token via
   OAuth2 implicit flow. Tokens stored to file.

2. **Authz exchange** (on every API command): POST to
   `teams.microsoft.com/api/authsvc/v1.0/authz` with the OAuth Skype token.
   Returns a messaging-capable `skypeToken` (24h TTL) and `regionGtms` with
   correct regional base URLs.

3. **Token usage by service**:
   - CSA: `Authorization: Bearer {chatsvcagg_token}` (OAuth token)
   - Messages: `Authentication: skypetoken={authz_skype_token}` (exchanged token)
   - MiddleTier: `Authorization: Bearer {oauth_skype_token}` (OAuth token)

## Key Constants

- Teams App ID: `5e3ce6c0-2b1f-4285-8d4b-75ee78787346`
- Redirect URI: `https://teams.microsoft.com/go`
- Authz endpoint: `https://teams.microsoft.com/api/authsvc/v1.0/authz`

## API Quirks

- The Messages API uses `Authentication:` header (not `Authorization:`).
- The `messagetype` for plain text is `"Text"`, for HTML is `"RichText/Html"`.
- The `clientmessageid` is a Unix timestamp in milliseconds (as string).
- Region is auto-discovered via authz (e.g., `amer`, `emea`, `apac`).
  The `--region` CLI flag is only used as fallback for tenant commands
  when authz isn't called.
- The CSA API returns large JSON (~4MB for 700+ chats). Models use
  `serde_json::Value` for flexibility since field shapes vary.
- The MT `/users/me` endpoint doesn't exist. `get_me()` extracts the
  UPN from the JWT and calls `get_user(email)`.

## Conversation IDs

- Channel threads: `19:{uuid}@thread.tacv2`
- Chat threads: `19:{uuid}@thread.v2`
- Self-chat: `19:{uuid1}_{uuid2}@unq.gbl.spaces` (often hidden)
- User MRIs: `8:orgid:{azure-ad-object-id}`

## Tracing

`RUST_LOG` env var controls log level. All logs go to stderr; stdout is
reserved for command output.

## Adding New Commands

1. Add clap structs in `src/cli/<command>.rs`
2. Add the variant to `Commands` enum in `src/cli/mod.rs`
3. Add the handler dispatch in `src/main.rs` `run()` match
4. API calls go in `src/api/`. Use `HttpClient::execute_with_retry()`.
5. Run `cargo fmt && cargo clippy --all-targets -- -D warnings`
