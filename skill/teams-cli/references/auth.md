# Authentication

## Check Status

```sh
teams auth status
```

Exit code 0 = authenticated. Exit code 3 = need to log in.

## Login

```sh
# Interactive webview (needs display)
teams auth login

# Specific Azure AD tenant
teams auth login --tenant <tenant-id-or-domain>
```

The webview acquires 4 tokens in sequence: Teams, Skype, ChatSvcAgg, Outlook.
Tokens are stored at `~/.config/teams-cli/tokens/<profile>.json` (file mode 0600).
If a command fails with exit code 3 (AUTH_FAILED / AUTH_TOKEN_EXPIRED), the user
needs to re-authenticate.

## Auto-login

If tokens are missing or expired, the CLI attempts authentication automatically
before running any command. Disable with `--no-auto-login`.

## Logout

```sh
teams auth logout          # Current profile
teams auth logout --all    # All profiles
```

## Token Export

Print raw tokens for scripting or debugging:

```sh
teams auth token            # Skype token (default)
teams auth token teams      # Teams JWT
teams auth token skype      # Skype token
teams auth token chatsvcagg # ChatSvcAgg token
teams auth token outlook    # Outlook token
```

## Environment Variable Override

Set all three core tokens together to bypass file-based auth entirely:

```sh
export TEAMS_CLI_TEAMS_TOKEN="..."
export TEAMS_CLI_SKYPE_TOKEN="..."
export TEAMS_CLI_CHATSVCAGG_TOKEN="..."
```

All three must be set; setting only one or two is not sufficient.
Optionally also set `TEAMS_CLI_OUTLOOK_TOKEN` for mail/calendar commands.
