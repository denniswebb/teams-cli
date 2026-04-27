# Outlook Email

## List inbox messages

```sh
teams mail list
teams mail list --limit 10
teams mail list --since 1h          # messages from last hour
teams mail list --since 24h --unread
teams mail list --folder "Sent Items" --limit 5
```

`--since` accepts `Nh` (hours), `Nm` (minutes), `Nd` (days).

## Read a specific email

```sh
teams mail read <message-id>
```

The message ID comes from `mail list` output (the `id` field in JSON mode).

## Send email

```sh
teams mail send --to user@example.com --subject "Hello" --body "Message body"
teams mail send --to user@example.com --cc other@example.com --subject "FYI" --body "Details"
echo "piped body" | teams mail send --to user@example.com --subject "Test" --stdin
teams mail send --to user@example.com --subject "HTML" --body "<h1>Rich</h1>" --html
```

## Search emails

```sh
teams mail search "quarterly report"
teams mail search "from:someone@example.com" --limit 20
```

## Notes

- All commands use `--output json` for programmatic parsing.
- The Outlook token is acquired during `teams auth login` (4th phase).
- If the Outlook token is missing, re-run `teams auth login`.
