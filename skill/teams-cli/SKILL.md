---
name: teams-cli
description: >
  Use the teams-cli command-line tool to interact with Microsoft Teams: send messages to channels
  and chats, list teams/channels/chats, read messages, look up users, and manage authentication.
  Use this skill whenever the user wants to send a Teams message, read Teams messages, find a Teams
  channel or chat, look up a Teams user, list their Teams/channels, or do anything involving
  Microsoft Teams from the command line. Also trigger when the user mentions "teams", "Teams channel",
  "Teams chat", "Teams message", or asks to post/notify/message someone on Teams, even if they
  don't explicitly say "teams-cli".
---

# teams-cli

CLI for Microsoft Teams via internal Skype/CSA APIs.

## Before anything else

1. Verify the binary exists: `command -v teams`
   - If missing, read `references/install.md`
2. Check auth: `teams auth status`
   - If exit code 3, read `references/auth.md`

## Task routing

Read **only** the reference file(s) needed for the task at hand:

| User wants to... | Read |
|---|---|
| Send a message, read messages, reply to someone | `references/messaging.md` |
| Find a team, channel, or chat by name | `references/discovery.md` |
| Look up a user or list tenants/domains | `references/users-and-tenants.md` |
| Change config, set defaults, shell completions | `references/configuration.md` |
| Fix errors, understand exit codes | `references/troubleshooting.md` |
| Install the CLI or this skill | `references/install.md` |
| Log in, fix auth, export tokens | `references/auth.md` |

For tasks that need a target **and** an action (e.g. "send a message to the General channel"),
read `references/discovery.md` first to resolve the ID, then the action-specific reference.

## Key pattern: cache-first lookups

For ANY operation needing a team, channel, or chat ID, use the cache script
instead of manually listing and searching. This avoids repeated API calls:

```sh
# Populate once per session
bash ${CLAUDE_SKILL_DIR}/scripts/cache.sh populate

# Then look up by name (fuzzy, case-insensitive)
bash ${CLAUDE_SKILL_DIR}/scripts/cache.sh lookup-channel "General"
bash ${CLAUDE_SKILL_DIR}/scripts/cache.sh lookup-chat "John Smith"
bash ${CLAUDE_SKILL_DIR}/scripts/cache.sh lookup-team "Engineering"
bash ${CLAUDE_SKILL_DIR}/scripts/cache.sh lookup-user "chines@thig.com"
```

Full cache docs are in `references/discovery.md`.

## Output

Always use `--output json` when parsing output programmatically.
All commands support `--output json|human|plain`.
