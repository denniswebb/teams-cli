# Finding Teams, Channels, and Chats

The Teams API works with opaque IDs, not names. The cache script eliminates
the overhead of resolving names to IDs on every request.

## Cache System

**Location**: `~/.config/teams-cli/cache.json`
**TTL**: 1 hour (override with `TEAMS_CLI_CACHE_TTL` env var, in seconds)
**Dependency**: `jq`

### Populate the cache

Run once per session or when results look stale:

```sh
bash ${CLAUDE_SKILL_DIR}/scripts/cache.sh populate
```

This fetches all teams, their channels, and all chats in one pass and stores
name-to-ID mappings locally.

### Look up by name

**Channels** — returns `team_id|channel_id` (pipe-separated):

```sh
result=$(bash ${CLAUDE_SKILL_DIR}/scripts/cache.sh lookup-channel "Engineering/General")
team_id="${result%%|*}"
channel_id="${result##*|}"
```

You can also search by bare channel name (without team prefix):

```sh
result=$(bash ${CLAUDE_SKILL_DIR}/scripts/cache.sh lookup-channel "General")
```

**Chats** — returns the chat thread ID:

```sh
chat_id=$(bash ${CLAUDE_SKILL_DIR}/scripts/cache.sh lookup-chat "John Smith")
```

**Teams** — returns the team ID:

```sh
team_id=$(bash ${CLAUDE_SKILL_DIR}/scripts/cache.sh lookup-team "Engineering")
```

All lookups are fuzzy and case-insensitive. "general" matches "General",
"eng" matches "Engineering". On cache miss, the script auto-refreshes and retries.

### Cache management

```sh
bash ${CLAUDE_SKILL_DIR}/scripts/cache.sh show     # Print the cache
bash ${CLAUDE_SKILL_DIR}/scripts/cache.sh clear    # Delete and start fresh
bash ${CLAUDE_SKILL_DIR}/scripts/cache.sh age <section> <key>  # Seconds since last update
```

### Workflow: always try cache first

For ANY operation that needs a team, channel, or chat ID:

1. Run the cache lookup command
2. If hit, use the ID directly (0 extra API calls)
3. If miss, the lookup auto-refreshes and retries
4. If still not found, fall back to direct API commands below

## Direct API Commands

Use these when the cache is insufficient or you need full details:

### Teams

```sh
teams team list --output json          # All joined teams
teams team get <team-id> --output json # Team details
```

### Channels

```sh
teams channel list <team-id> --output json       # Channels in a team
teams channel get <team-id> <channel-id>         # Channel details
teams channel pinned --output json               # Pinned channels only
```

### Chats

```sh
teams chat list --output json          # Chats (excludes hidden)
teams chat list --all --output json    # Include hidden chats
teams chat get <chat-id> --output json # Chat details
```

### Listing all teams and their channels

```sh
teams team list --output json | jq -r '.data[] | "\(.displayName) (\(.id))"'
```
