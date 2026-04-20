# Configuration

## Config Commands

```sh
teams config init                       # Create default config file
teams config show                       # Display current config
teams config set <key> <value>          # Set a value
teams config path                       # Print config file location
```

Config lives at `~/.config/teams-cli/config.toml` (Linux/macOS).

## Settable Keys

| Key | Values | Description |
|-----|--------|-------------|
| `default.profile` | string | Default credential profile name |
| `default.region` | `emea`, `amer`, `apac` | Default API region |
| `output.format` | `auto`, `json`, `human`, `plain` | Default output format |
| `output.color` | `true`, `false` | Enable ANSI colors |
| `network.timeout` | integer (seconds) | Request timeout |
| `network.max_retries` | integer | Max retry attempts |

## Global Flags

Available on all commands:

| Flag | Default | Description |
|------|---------|-------------|
| `--output <fmt>` | auto | json, human, plain |
| `--region <r>` | emea | emea, amer, apac (fallback; auto-discovered via authz) |
| `--profile <name>` | default | Named credential profile |
| `--timeout <secs>` | 30 | Request timeout |
| `--retry <count>` | 3 | Max retry attempts |
| `--no-auto-login` | false | Skip auto-auth when tokens expired |
| `--no-color` | false | Disable ANSI colors (also respects NO_COLOR env var) |
| `-v` / `-vv` / `-vvv` | — | Verbosity: info / debug / trace |
| `-q` / `--quiet` | false | Suppress non-essential output |

## Shell Completions

```sh
teams completions bash > ~/.bash_completion.d/teams
teams completions zsh > ~/.zfunc/_teams
teams completions fish > ~/.config/fish/completions/teams.fish
```

## Environment Variables

| Variable | Description |
|----------|-------------|
| `TEAMS_CLI_TEAMS_TOKEN` | Override Teams JWT |
| `TEAMS_CLI_SKYPE_TOKEN` | Override Skype JWT |
| `TEAMS_CLI_CHATSVCAGG_TOKEN` | Override ChatSvcAgg JWT |
| `NO_COLOR` | Disable ANSI color output |

All three token env vars must be set together to override file-based auth.
