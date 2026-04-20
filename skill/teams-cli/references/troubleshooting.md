# Troubleshooting

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General/API error |
| 2 | Invalid input |
| 3 | Auth failure — run `teams auth login` |
| 4 | Permission denied |
| 5 | Not found |
| 6 | Rate limited — wait and retry (built-in exponential backoff) |
| 7 | Network error |
| 8 | Server error (5xx) |
| 10 | Config/keyring error |

## Common Problems

| Symptom | Fix |
|---------|-----|
| Exit code 3 on any command | Run `teams auth login` (see `references/auth.md`) |
| "command not found: teams" | Run `cargo install --path .` (see `references/install.md`) |
| Cache lookups return nothing | Run `bash ${CLAUDE_SKILL_DIR}/scripts/cache.sh populate` |
| Stale cache results | Run `bash ${CLAUDE_SKILL_DIR}/scripts/cache.sh clear` then `populate` |
| "jq: command not found" | `brew install jq` (macOS) or `apt install jq` (Linux) |
| Rate limited (exit code 6) | Wait and retry; the CLI has built-in exponential backoff |
| Wrong region / slow responses | `teams config set default.region amer` (or `emea`/`apac`) |
