#!/usr/bin/env bash
# teams-cli cache manager
# Stores team/channel/chat name-to-ID mappings to avoid repeated API lookups.
# Cache lives at ~/.config/teams-cli/cache.json with a configurable TTL.

set -euo pipefail

CACHE_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/teams-cli"
CACHE_FILE="$CACHE_DIR/cache.json"
CACHE_TTL="${TEAMS_CLI_CACHE_TTL:-3600}" # seconds, default 1 hour

mkdir -p "$CACHE_DIR"

# Initialize empty cache if missing
if [[ ! -f "$CACHE_FILE" ]]; then
  echo '{"teams":{},"channels":{},"chats":{},"updated_at":{}}' > "$CACHE_FILE"
fi

usage() {
  cat <<EOF
Usage: cache.sh <command> [args...]

Commands:
  get <section> <key>         Get cached value (returns empty + exit 1 if miss/stale)
  set <section> <key> <value> Store a value
  populate                    Refresh all teams, channels, and chats
  lookup-channel <name>       Find channel by name (cache-first, then API)
  lookup-chat <name>          Find chat by display name (cache-first, then API)
  lookup-team <name>          Find team by name (cache-first, then API)
  clear                       Delete the cache file
  show                        Print the cache
  age <section> <key>         Print seconds since last update (empty if never)
EOF
}

now() { date +%s; }

# All JSON operations use jq (required dependency)
require_jq() {
  if ! command -v jq &>/dev/null; then
    echo "Error: jq is required for cache operations. Install it with: brew install jq (macOS) or apt install jq (Linux)" >&2
    exit 1
  fi
}

cmd_get() {
  require_jq
  local section="$1" key="$2"
  local value updated_at age

  value=$(jq -r --arg s "$section" --arg k "$key" '.[$s][$k] // empty' "$CACHE_FILE" 2>/dev/null)
  [[ -z "$value" ]] && return 1

  updated_at=$(jq -r --arg s "$section" --arg k "$key" '.updated_at[$s][$k] // 0' "$CACHE_FILE" 2>/dev/null)
  age=$(( $(now) - updated_at ))
  if (( age > CACHE_TTL )); then
    return 1
  fi

  echo "$value"
}

cmd_set() {
  require_jq
  local section="$1" key="$2" value="$3"
  local tmp
  tmp=$(mktemp)
  jq --arg s "$section" --arg k "$key" --arg v "$value" --argjson t "$(now)" \
    '.[$s][$k] = $v | .updated_at[$s][$k] = $t' "$CACHE_FILE" > "$tmp" \
    && mv "$tmp" "$CACHE_FILE"
}

cmd_age() {
  require_jq
  local section="$1" key="$2"
  local updated_at

  updated_at=$(jq -r --arg s "$section" --arg k "$key" '.updated_at[$s][$k] // 0' "$CACHE_FILE" 2>/dev/null)

  if [[ "$updated_at" == "0" ]]; then
    echo ""
    return 1
  fi
  echo $(( $(now) - updated_at ))
}

# Populate cache from teams-cli output (JSON mode for reliable parsing)
cmd_populate() {
  require_jq
  echo "Populating teams cache..." >&2

  # Teams
  local teams_json
  teams_json=$(teams team list --output json 2>/dev/null) || { echo "Failed to list teams" >&2; return 1; }
  echo "$teams_json" | jq -r '.data[] | "\(.displayName)\t\(.id)"' 2>/dev/null | while IFS=$'\t' read -r name id; do
    cmd_set "teams" "$name" "$id"
  done

  # Channels per team
  echo "$teams_json" | jq -r '.data[] | "\(.displayName)\t\(.id)"' 2>/dev/null | while IFS=$'\t' read -r team_name team_id; do
    local channels_json
    channels_json=$(teams channel list "$team_id" --output json 2>/dev/null) || continue
    echo "$channels_json" | jq -r '.data[] | "\(.displayName)\t\(.id)"' 2>/dev/null | while IFS=$'\t' read -r ch_name ch_id; do
      # Store as "team_name/channel_name" -> "team_id|channel_id"
      cmd_set "channels" "${team_name}/${ch_name}" "${team_id}|${ch_id}"
      # Also store bare channel name (last match wins if ambiguous)
      cmd_set "channels" "$ch_name" "${team_id}|${ch_id}"
    done
  done

  # Chats
  local chats_json
  chats_json=$(teams chat list --output json 2>/dev/null) || { echo "Failed to list chats" >&2; return 1; }
  # Chat model uses "title" field; for 1:1 chats with no title, join member friendlyNames
  echo "$chats_json" | jq -r '.data[] | "\(if .title != "" then .title else ([.members[]?.friendlyName // empty] | join(", ")) end)\t\(.id)"' 2>/dev/null | while IFS=$'\t' read -r name id; do
    [[ -n "$name" && "$name" != "null" ]] && cmd_set "chats" "$name" "$id"
  done

  echo "Cache populated." >&2
}

fuzzy_search() {
  require_jq
  local section="$1" query="$2"
  local lquery
  lquery=$(echo "$query" | tr '[:upper:]' '[:lower:]')
  jq -r --arg q "$lquery" --arg s "$section" \
    '.[$s] | to_entries[] | select((.key | ascii_downcase) | contains($q)) | .value' \
    "$CACHE_FILE" 2>/dev/null | head -1
}

cmd_lookup_channel() {
  local query="$1"
  local cached

  # Try exact match
  cached=$(cmd_get "channels" "$query" 2>/dev/null) && { echo "$cached"; return 0; }

  # Try fuzzy match in cache
  cached=$(fuzzy_search "channels" "$query")
  [[ -n "$cached" ]] && { echo "$cached"; return 0; }

  # Cache miss: refresh and retry
  echo "Cache miss for channel '$query', refreshing..." >&2
  cmd_populate
  cached=$(cmd_get "channels" "$query" 2>/dev/null) && { echo "$cached"; return 0; }

  cached=$(fuzzy_search "channels" "$query")
  [[ -n "$cached" ]] && { echo "$cached"; return 0; }

  echo "Channel '$query' not found" >&2
  return 1
}

cmd_lookup_chat() {
  local query="$1"
  local cached

  cached=$(cmd_get "chats" "$query" 2>/dev/null) && { echo "$cached"; return 0; }

  cached=$(fuzzy_search "chats" "$query")
  [[ -n "$cached" ]] && { echo "$cached"; return 0; }

  echo "Cache miss for chat '$query', refreshing..." >&2
  cmd_populate
  cached=$(cmd_get "chats" "$query" 2>/dev/null) && { echo "$cached"; return 0; }

  cached=$(fuzzy_search "chats" "$query")
  [[ -n "$cached" ]] && { echo "$cached"; return 0; }

  echo "Chat '$query' not found" >&2
  return 1
}

cmd_lookup_team() {
  local query="$1"
  local cached

  cached=$(cmd_get "teams" "$query" 2>/dev/null) && { echo "$cached"; return 0; }

  cached=$(fuzzy_search "teams" "$query")
  [[ -n "$cached" ]] && { echo "$cached"; return 0; }

  echo "Cache miss for team '$query', refreshing..." >&2
  cmd_populate
  cached=$(cmd_get "teams" "$query" 2>/dev/null) && { echo "$cached"; return 0; }

  cached=$(fuzzy_search "teams" "$query")
  [[ -n "$cached" ]] && { echo "$cached"; return 0; }

  echo "Team '$query' not found" >&2
  return 1
}

# Main dispatch
case "${1:-}" in
  get)       shift; cmd_get "$@" ;;
  set)       shift; cmd_set "$@" ;;
  age)       shift; cmd_age "$@" ;;
  populate)  cmd_populate ;;
  lookup-channel) shift; cmd_lookup_channel "$@" ;;
  lookup-chat)    shift; cmd_lookup_chat "$@" ;;
  lookup-team)    shift; cmd_lookup_team "$@" ;;
  clear)     rm -f "$CACHE_FILE"; echo "Cache cleared." ;;
  show)      cat "$CACHE_FILE" 2>/dev/null || echo "No cache." ;;
  *)         usage; exit 1 ;;
esac
