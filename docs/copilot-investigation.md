# Copilot Integration - Investigation Notes

Status: **Partially working** - auth, WebSocket, SignalR all work. Backend LLM ("deep-leo") rejects requests with a server error.

## What Works

1. **Auth (5th webview phase)**: PKCE authorization code flow with the Copilot app ID (`c0ab8ce9-e9a0-42e7-b064-33d422df41f1`) and `nativeclient` redirect URI. Token has correct audience (`https://substrate.office.com/sydney`), correct app ID, and full `CopilotPlatform*` scopes.

2. **WebSocket connection**: Connects to `wss://substrate.office.com/m365Copilot/Chathub/{oid}@{tenantId}` with the token as a query parameter. No 403.

3. **SignalR handshake**: Sends `{"protocol":"json","version":1}\x1e`, receives `{}\x1e` back.

4. **Invocation delivery**: Type 4 (StreamInvocation) targeting `chat` method is accepted by the hub. Server responds with type 1 (update with throttling info), type 2 (StreamItem with result), and type 3 (Completion).

5. **Response parsing**: Extracts text from `item.messages[].text` (bot messages) and `item.result.message` (error/final). Handles conversation IDs from server.

## What Fails

The StreamItem (type 2) response always contains:
```json
{
  "result": {
    "value": "InternalError",
    "message": "Sorry, I wasn't able to respond to that...",
    "error": "Error calling extension deep-leo"
  }
}
```

The "deep-leo" extension is the backend LLM orchestration layer. It errors consistently from the CLI while the browser works fine with the same question at the same time.

## Root Cause Hypothesis

The most likely cause is that the **token's authentication context differs** from what the browser produces. Specifically:

### Token comparison

| Property | Browser token | CLI token (PKCE) |
|----------|--------------|-----------------|
| `appid` | `c0ab8ce9-e9a0-42e7-b064-33d422df41f1` | `c0ab8ce9-e9a0-42e7-b064-33d422df41f1` (same) |
| `aud` | `https://substrate.office.com/sydney` | `https://substrate.office.com/sydney` (same) |
| `scp` | Full CopilotPlatform scopes | Full CopilotPlatform scopes (same) |
| redirect_uri used | `https://m365.cloud.microsoft/...` (SPA) | `https://login.microsoftonline.com/common/oauth2/nativeclient` |
| Grant type | Likely auth code + PKCE via MSAL.js | Auth code + PKCE (our webview) |
| `acr` claim | Unknown | `"1"` |

The deep-leo backend may check the token's `redirect_uri` context or require a specific auth method (`amr` claim). The `nativeclient` redirect URI is a generic public client URI; the browser uses the actual Copilot web app's registered redirect URI.

## Things to Try Next

### 1. Use the Copilot web app's redirect URI (most promising)
The Copilot SPA at `m365.cloud.microsoft` likely has redirect URIs like:
- `https://m365.cloud.microsoft/`
- `https://m365.cloud.microsoft/chat`

Try these in `build_pkce_auth_url` instead of `nativeclient`. The webview navigation handler would need to intercept redirects to `m365.cloud.microsoft` for the Copilot phase.

To discover the actual redirect URI: open browser DevTools on `m365.cloud.microsoft/chat`, look for the MSAL configuration in the page source or network requests to `login.microsoftonline.com` during initial page load.

### 2. Check if a `negotiate` endpoint is needed
Some SignalR hubs require a negotiate step before the WebSocket connection:
```
POST https://substrate.office.com/m365Copilot/Chathub/negotiate
```
This returns the actual WebSocket URL with a connection token. The browser might do this transparently. Check browser network requests for a `negotiate` call during page load.

### 3. Match the browser's WS URL `variants` parameter
The browser sends a long `variants=...` query string on the WebSocket URL with feature flags. Our CLI omits this. While the connection works without it, the backend routing/model selection might depend on these flags. Capture the full variants string from browser DevTools (Network tab > WS > Headers > Request URL).

### 4. Compare full invocation payloads
Use browser DevTools (Network tab > WS > Messages) to capture the exact JSON the browser sends as the type 4 invocation. Compare field-by-field with what `src/api/copilot.rs` sends. Key areas to check:
- `clientInfo.clientPlatform` (we send "teams-cli", browser sends "mcmcopilot-web")
- Any fields present in the browser but missing from our payload
- The `participants` field (browser may include it)

### 5. Try the Teams app ID token as a fallback
We already know the Teams app ID (`5e3ce6c0`) can get a token for `substrate.office.com/sydney` via implicit grant. That token got a 403 on the WebSocket connection. But it's worth trying with the `agentHost=Bizchat.FullScreen` param we added later.

## Key Files

- `src/auth/webview.rs` - 5-phase auth, PKCE for Copilot phase (lines ~230-250)
- `src/auth/token.rs` - `COPILOT_RESOURCE`, `COPILOT_APP_ID`, `TokenSet.copilot`
- `src/api/copilot.rs` - SignalR WebSocket client, invocation format, response parsing
- `src/models/copilot.rs` - SignalR message types, CopilotResponse
- `src/cli/copilot.rs` - `ask` and `chat` CLI commands

## API Reference

- **WebSocket endpoint**: `wss://substrate.office.com/m365Copilot/Chathub/{oid}@{tenantId}`
- **Token audience**: `https://substrate.office.com/sydney`
- **Copilot app ID**: `c0ab8ce9-e9a0-42e7-b064-33d422df41f1`
- **PKCE redirect URI currently used**: `https://login.microsoftonline.com/common/oauth2/nativeclient`
- **SignalR protocol**: JSON, record separator `\x1e`
- **Invocation**: type 4 (StreamInvocation), target "chat"
- **Response flow**: type 1 (update) -> type 2 (StreamItem, possibly multiple) -> type 3 (Completion)

## Debugging Commands

```sh
# Test with verbose SignalR logging
teams -vv copilot ask "What is 2+2?"

# Check token claims
teams auth token copilot 2>/dev/null | python3 -c "
import sys, json, base64
t = sys.stdin.read().strip().split('.')[1] + '=='
print(json.dumps(json.loads(base64.urlsafe_b64decode(t)), indent=2))
"

# Verify auth status
teams auth status
```
