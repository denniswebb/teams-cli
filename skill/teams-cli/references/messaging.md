# Sending and Reading Messages

## Output Format

All commands support `--output <format>`:
- `json` — structured JSON envelope (best for parsing programmatically)
- `human` or `table` — formatted tables
- `plain` or `text` — tab-separated values (for scripting)
- Auto-detects if not specified (human for TTY, JSON for pipes)

Always use `--output json` when you need to parse the result.

## Commands

```sh
# List recent messages (default limit: 50)
teams message list <conversation-id> --limit 20 --output json

# Send a message
teams message send <conversation-id> --body "Hello from CLI"

# Send via stdin (multi-line or piped content)
echo "Message content" | teams message send <conversation-id> --stdin

# Get a specific message
teams message get <conversation-id> <message-id> --output json
```

## Conversation ID Formats

The `<conversation-id>` is a channel thread ID or chat thread ID:
- **Channel threads**: `19:{uuid}@thread.tacv2`
- **Chat threads**: `19:{uuid}@thread.v2`
- **Self-chat**: `19:{uuid1}_{uuid2}@unq.gbl.spaces`

To find the right conversation ID by name, see `references/discovery.md`.

## Common Workflows

### Send to a named channel

```sh
# 1. Resolve channel (cache-first, 0 API calls if cached)
result=$(bash ${CLAUDE_SKILL_DIR}/scripts/cache.sh lookup-channel "Engineering/General")
team_id="${result%%|*}"
channel_id="${result##*|}"

# 2. Send (1 API call)
teams message send "$channel_id" --body "Deployment complete."
```

### Send to a chat

```sh
chat_id=$(bash ${CLAUDE_SKILL_DIR}/scripts/cache.sh lookup-chat "John Smith")
teams message send "$chat_id" --body "Hey, PR is ready for review."
```

### Read recent messages from a channel

```sh
result=$(bash ${CLAUDE_SKILL_DIR}/scripts/cache.sh lookup-channel "General")
channel_id="${result##*|}"
teams message list "$channel_id" --limit 10 --output json
```

### @Mentioning users

To @mention someone in a Teams message, use HTML `<at>` tags with their MRI.
Use the cache to resolve names to MRIs first (see `references/discovery.md`).

```sh
# 1. Resolve user
result=$(bash ${CLAUDE_SKILL_DIR}/scripts/cache.sh lookup-user "Colin Hines")
email="${result%%|*}"
mri="${result##*|}"

# 2. Send with @mention (must use --stdin for HTML)
cat <<MSG | teams message send "$chat_id" --stdin
<at id="$mri">Colin Hines</at> the deploy is done.
MSG
```

The `<at id="MRI">Display Name</at>` syntax is the input format. The CLI
automatically converts these to the `<span>` mention tags and metadata that
the Teams API requires. The MRI format is `8:orgid:{azure-ad-object-id}`.
Multiple mentions work in the same message. Content containing `<at>` tags
is auto-detected as HTML — no need to pass `--html` explicitly.

### Multi-line messages

```sh
cat <<'MSG' | teams message send "$channel_id" --stdin
Build #1234 completed successfully.

Changes:
- Fixed auth timeout bug
- Updated dependency versions
MSG
```
