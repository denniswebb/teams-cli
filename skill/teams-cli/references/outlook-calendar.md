# Outlook Calendar

## List upcoming events

```sh
teams calendar list                                    # next 7 days (default)
teams calendar list --from today --to +3d
teams calendar list --from 2026-05-01 --to 2026-05-31
```

`--from` and `--to` accept: ISO 8601 datetime, date-only (`2026-05-01`), `today`, `tomorrow`, `now`, relative offsets (`+3d`, `+1w`).

## Get event details

```sh
teams calendar get <event-id>
```

The event ID comes from `calendar list` output (the `id` field in JSON mode).

## Create a meeting

```sh
teams calendar create --subject "Standup" --start 2026-04-28T09:00:00 --end 2026-04-28T09:30:00

teams calendar create --subject "Review" \
  --start 2026-04-28T14:00:00 --end 2026-04-28T15:00:00 \
  --attendees alice@example.com --attendees bob@example.com \
  --online --location "Room 1"
```

- `--online` adds a Teams meeting link.
- `--attendees` can be repeated for multiple attendees.
- `--location` sets the meeting location.

## Notes

- All commands use `--output json` for programmatic parsing.
- The Outlook token is acquired during `teams auth login` (4th phase).
- If the Outlook token is missing, re-run `teams auth login`.
