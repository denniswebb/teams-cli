---
name: teams-cli
description: >
  Use the teams-cli command-line tool to interact with Microsoft Teams and Outlook: send messages
  to channels and chats, list teams/channels/chats, read messages, look up users, manage
  authentication, read/send email, and manage calendar events. Use this skill whenever the user
  wants to send a Teams message, read Teams messages, find a Teams channel or chat, look up a Teams
  user, list their Teams/channels, read or send email, check their calendar, create meetings, or
  do anything involving Microsoft Teams or Outlook from the command line. Also trigger when the user
  mentions "teams", "Teams channel", "Teams chat", "Teams message", "email", "inbox", "calendar",
  "meeting", "outlook", or asks to post/notify/message someone on Teams, even if they don't
  explicitly say "teams-cli".
---

# teams-cli

CLI for Microsoft Teams and Outlook via internal Microsoft APIs.

## Before anything else

1. Verify the binary exists: `command -v teams`
   - If missing, read `references/install.md`
2. **Do NOT pre-check auth.** The CLI has auto-login — it refreshes expired tokens automatically before running any command. Just run the command directly. Only read `references/auth.md` if a command fails with exit code 3.

## Task routing

Read **only** the reference file(s) needed for the task at hand:

| User wants to... | Read |
|---|---|
| Send a message, read messages, reply to someone | `references/messaging.md` |
| Find a team, channel, or chat by name | `references/discovery.md` |
| Look up a user or list tenants/domains | `references/users-and-tenants.md` |
| Read email, send email, search inbox | `references/outlook-mail.md` |
| List calendar events, create meetings | `references/outlook-calendar.md` |
| Change config, set defaults, shell completions | `references/configuration.md` |
| Fix errors, understand exit codes | `references/troubleshooting.md` |
| Install the CLI or this skill | `references/install.md` |
| Log in, fix auth, export tokens | `references/auth.md` |

For tasks that need a target **and** an action (e.g. "send a message to the General channel"),
read `references/discovery.md` first to resolve the ID, then the action-specific reference.

## Key pattern: lazy cache lookups

For ANY operation needing a team, channel, or chat ID, use the cache script.
**Do NOT call `populate` upfront.** The cache is lazy — on a miss, it fetches
from the API, caches the result, and returns it automatically.

```sh
# Just look up by name (fuzzy, case-insensitive). Fetches on miss.
bash ${CLAUDE_SKILL_DIR}/scripts/cache.sh lookup-channel "General"
bash ${CLAUDE_SKILL_DIR}/scripts/cache.sh lookup-chat "John Smith"
bash ${CLAUDE_SKILL_DIR}/scripts/cache.sh lookup-team "Engineering"
bash ${CLAUDE_SKILL_DIR}/scripts/cache.sh lookup-user "chines@thig.com"

# Manually cache a value you discovered
bash ${CLAUDE_SKILL_DIR}/scripts/cache.sh set chats "My Chat" "19:abc@thread.v2"
```

Full cache docs are in `references/discovery.md`.

## Output

Always use `--output json` when parsing output programmatically.
All commands support `--output json|human|plain`.
