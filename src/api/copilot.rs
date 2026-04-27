use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::Message;

use crate::auth::token::TokenSet;
use crate::error::{Result, TeamsError};
use crate::models::copilot::{CopilotResponse, SignalRMessage, SIGNALR_RECORD_SEPARATOR};

const COPILOT_HUB_BASE: &str = "wss://substrate.office.com/m365Copilot/Chathub";

pub struct CopilotClient<'a> {
    tokens: &'a TokenSet,
}

impl<'a> CopilotClient<'a> {
    pub fn new(tokens: &'a TokenSet) -> Self {
        Self { tokens }
    }

    fn build_ws_url(&self, conversation_id: &str, session_id: &str) -> Result<String> {
        let token = self.tokens.copilot_token()?;

        // Extract the user's Azure AD object ID from the copilot JWT 'oid' claim.
        // The hub path format is {oid}@{tenant_id}.
        let copilot_claims = crate::auth::token::decode_jwt_claims(token)?;
        let oid = copilot_claims
            .oid
            .ok_or_else(|| TeamsError::AuthError("copilot token missing 'oid' claim".into()))?;

        let encoded_token = urlencoding::encode(token);

        Ok(format!(
            "{COPILOT_HUB_BASE}/{}@{}?ConversationId={}&ClientRequestId={}&X-SessionId={}&access_token={}&agent=work&product=Office&scenario=officeweb&licenseType=Premium&source=%22officeweb%22",
            urlencoding::encode(&oid),
            urlencoding::encode(&self.tokens.tenant_id),
            urlencoding::encode(conversation_id),
            uuid::Uuid::new_v4(),
            urlencoding::encode(session_id),
            encoded_token,
        ))
    }

    pub async fn ask(
        &self,
        question: &str,
        conversation_id: Option<&str>,
    ) -> Result<CopilotResponse> {
        let conv_id = conversation_id
            .map(|s| s.to_string())
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let session_id = uuid::Uuid::new_v4().to_string();

        let ws_url = self.build_ws_url(&conv_id, &session_id)?;

        let (ws_stream, _response) = tokio_tungstenite::connect_async(&ws_url)
            .await
            .map_err(|e| TeamsError::Other(anyhow::anyhow!("websocket connect failed: {e}")))?;

        let (mut write, mut read) = ws_stream.split();

        // SignalR handshake
        let handshake = format!(
            "{{\"protocol\":\"json\",\"version\":1}}{}",
            SIGNALR_RECORD_SEPARATOR
        );
        write.send(Message::Text(handshake)).await.map_err(|e| {
            TeamsError::Other(anyhow::anyhow!("signalr handshake send failed: {e}"))
        })?;

        // Wait for handshake response
        let handshake_resp = read
            .next()
            .await
            .ok_or_else(|| {
                TeamsError::AuthError("signalr connection closed during handshake".into())
            })?
            .map_err(|e| {
                TeamsError::Other(anyhow::anyhow!("signalr handshake read failed: {e}"))
            })?;

        tracing::debug!("signalr handshake response: {:?}", handshake_resp);

        // Send the chat message invocation
        // NOTE: The exact invocation format needs to be captured from the web client.
        // This is a best-guess based on SignalR conventions. The target method name
        // and argument structure will need adjustment once we capture a real request.
        let invocation = serde_json::json!({
            "type": 1,
            "target": "chat",
            "arguments": [{
                "message": question,
                "conversationId": conv_id,
                "from": "user",
            }],
            "invocationId": "0",
        });
        let invocation_msg = format!(
            "{}{}",
            serde_json::to_string(&invocation).map_err(|e| TeamsError::Other(e.into()))?,
            SIGNALR_RECORD_SEPARATOR
        );
        write
            .send(Message::Text(invocation_msg))
            .await
            .map_err(|e| TeamsError::Other(anyhow::anyhow!("signalr send failed: {e}")))?;

        // Collect response
        let mut response_text = String::new();
        let timeout = tokio::time::Duration::from_secs(120);
        let deadline = tokio::time::Instant::now() + timeout;

        loop {
            let msg = tokio::time::timeout_at(deadline, read.next())
                .await
                .map_err(|_| {
                    TeamsError::Other(anyhow::anyhow!("copilot response timed out after 120s"))
                })?
                .ok_or_else(|| {
                    TeamsError::Other(anyhow::anyhow!("copilot connection closed unexpectedly"))
                })?
                .map_err(|e| TeamsError::Other(anyhow::anyhow!("websocket read error: {e}")))?;

            let text = match msg {
                Message::Text(t) => t,
                Message::Ping(_) => {
                    write.send(Message::Pong(vec![])).await.ok();
                    continue;
                }
                Message::Close(_) => break,
                _ => continue,
            };

            // SignalR sends multiple JSON messages separated by \x1e
            for part in text.split(SIGNALR_RECORD_SEPARATOR) {
                let part = part.trim();
                if part.is_empty() {
                    continue;
                }

                tracing::debug!("signalr recv: {}", &part[..part.len().min(500)]);

                let signalr_msg: SignalRMessage = match serde_json::from_str(part) {
                    Ok(m) => m,
                    Err(e) => {
                        tracing::debug!("failed to parse signalr message: {e}, raw: {part}");
                        continue;
                    }
                };

                match signalr_msg.msg_type {
                    6 => {
                        // Ping — respond with pong
                        let pong = format!("{{\"type\":6}}{}", SIGNALR_RECORD_SEPARATOR);
                        write.send(Message::Text(pong)).await.ok();
                    }
                    1 => {
                        // Invocation — Copilot sends response chunks this way
                        if let Some(args) = &signalr_msg.arguments {
                            for arg in args {
                                if let Some(t) = arg.get("text").and_then(|v| v.as_str()) {
                                    response_text = t.to_string();
                                }
                                if let Some(t) = arg.get("message").and_then(|v| v.as_str()) {
                                    response_text = t.to_string();
                                }
                                if let Some(delta) = arg.get("delta").and_then(|v| v.as_str()) {
                                    response_text.push_str(delta);
                                }
                            }
                        }
                    }
                    2 => {
                        // StreamItem
                        if let Some(item) = &signalr_msg.item {
                            if let Some(t) = item.get("text").and_then(|v| v.as_str()) {
                                response_text = t.to_string();
                            }
                        }
                    }
                    3 => {
                        // Completion — may contain result or error
                        if let Some(err) = &signalr_msg.error {
                            return Err(TeamsError::ApiError {
                                status: 0,
                                message: format!("copilot error: {err}"),
                            });
                        }
                        if let Some(result) = &signalr_msg.result {
                            if let Some(t) = result.get("text").and_then(|v| v.as_str()) {
                                response_text = t.to_string();
                            }
                        }
                        break;
                    }
                    7 => {
                        // Close
                        if let Some(err) = &signalr_msg.error {
                            return Err(TeamsError::ApiError {
                                status: 0,
                                message: format!("copilot connection closed with error: {err}"),
                            });
                        }
                        break;
                    }
                    _ => {
                        tracing::debug!(
                            "unhandled signalr message type {}: {part}",
                            signalr_msg.msg_type
                        );
                    }
                }
            }
        }

        write.close().await.ok();

        Ok(CopilotResponse {
            conversation_id: conv_id,
            message: response_text,
            citations: vec![],
        })
    }
}
