# teams-cli

CLI for Microsoft Teams using the internal Skype/CSA APIs (not the Microsoft Graph API).

## Install

```sh
cargo install --path .
```

## Authentication

Teams CLI acquires three OAuth2 tokens via a native webview that opens the
Microsoft login page. After authentication, a token exchange via the authz
endpoint provides a messaging-capable Skype token and auto-discovers the
correct regional API endpoints.

```sh
# Interactive webview login (default)
teams auth login

# Device code login (for headless/SSH environments)
teams auth login --device-code

# Check token status
teams auth status

# Export a token for scripting
teams auth token skype

# Clear credentials
teams auth logout
```

Tokens are stored as JSON files at `~/.config/teams-cli/tokens/<profile>.json`
(mode 0600). The CLI also reads tokens from `~/.config/fossteams/` for backward
compatibility with [fossteams/teams-token](https://github.com/fossteams/teams-token).

### Auto Re-authentication

If a command fails due to expired tokens, the CLI automatically triggers
re-authentication and retries the command. Disable with `--auto-login false`.

## Usage

### Teams

```sh
teams team list
teams team get <team-id>
```

### Channels

```sh
teams channel list <team-id>
teams channel get <team-id> <channel-id>
teams channel pinned
```

### Chats

```sh
teams chat list
teams chat list --all   # include hidden chats
teams chat get <chat-id>
```

### Messages

```sh
teams message list <conversation-id> --limit 50
teams message send <conversation-id> --body "Hello from the CLI"
echo "piped message" | teams message send <conversation-id> --stdin
teams message get <conversation-id> <message-id>
```

### Users

```sh
teams user me
teams user get user@example.com
teams user search "8:orgid:mri-1,8:orgid:mri-2"
```

### Tenants

```sh
teams tenant list
teams tenant domains
```

### Configuration

```sh
teams config init
teams config show
teams config set default.region amer
teams config path
```

### Shell Completions

```sh
teams completions bash > ~/.bash_completion.d/teams
teams completions zsh > ~/.zfunc/_teams
teams completions fish > ~/.config/fish/completions/teams.fish
```

## Output Formats

```sh
# Auto-detect (human for TTY, JSON for pipes)
teams team list

# Force JSON envelope
teams team list --output json

# Plain text (tab-separated, for scripting)
teams team list --output plain
```

## Global Options

| Flag | Description |
|------|-------------|
| `--output <format>` | json, human, plain (auto-detect) |
| `--region <region>` | API region: emea, amer, apac (auto-detected via authz) |
| `--profile <name>` | Named credential profile |
| `--timeout <secs>` | Request timeout |
| `--retry <count>` | Max retry attempts |
| `--auto-login` | Re-auth on expiry (default: true) |
| `-v` / `-vv` / `-vvv` | Verbosity: info / debug / trace |
| `-q` / `--quiet` | Suppress non-essential output |

## Environment Variables

| Variable | Description |
|----------|-------------|
| `TEAMS_CLI_TEAMS_TOKEN` | Override Teams JWT |
| `TEAMS_CLI_SKYPE_TOKEN` | Override Skype JWT |
| `TEAMS_CLI_CHATSVCAGG_TOKEN` | Override ChatSvcAgg JWT |
| `RUST_LOG` | Tracing filter (e.g. `debug`) |

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Invalid input |
| 3 | Auth failed / token expired |
| 4 | Permission denied |
| 5 | Not found |
| 6 | Rate limited |
| 7 | Network error |
| 8 | Server error |
| 10 | Config error |

## API Services

The CLI communicates with three Microsoft Teams backend services, discovered
dynamically via the authz token exchange:

- **Chat Service Aggregator (CSA)** -- teams, channels, chats listing
- **Messages Service** -- message read/write (uses the authz-exchanged Skype token)
- **MiddleTier (MT)** -- user profiles, tenants, domains

These are the same internal APIs used by the official Teams client, not the
public Microsoft Graph API.

## License

MIT
