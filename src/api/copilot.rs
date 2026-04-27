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
            "{COPILOT_HUB_BASE}/{}@{}?ConversationId={}&ClientRequestId={}&X-SessionId={}&access_token={}&agent=work&product=Office&agentHost=Bizchat.FullScreen&scenario=officeweb&licenseType=Premium&source=%22officeweb%22",
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

        // Send the chat message as a SignalR StreamInvocation (type 4).
        // Format captured from the M365 Copilot web client.
        // The web client uses the same ID for traceId and message.requestId.
        let request_id = uuid::Uuid::new_v4().to_string();
        let invocation = serde_json::json!({
            "type": 4,
            "target": "chat",
            "invocationId": "0",
            "arguments": [{
                "source": "officeweb",
                "clientCorrelationId": &session_id,
                "sessionId": &session_id,
                "traceId": &request_id,
                "isStartOfSession": conversation_id.is_none(),
                "streamingMode": "ConciseWithPadding",
                "spokenTextMode": "None",
                "tone": "Magic",
                "isSbsSupported": false,
                "renderReferencesBehindEOS": true,
                "options": {},
                "extraExtensionParameters": {},
                "threadLevelGptId": {},
                "plugins": [],
                "optionsSets": [
                    "enterprise_flux_work",
                    "enterprise_flux_web",
                    "enterprise_flux_image",
                    "enterprise_toolbox_with_skdsstore",
                    "enterprise_pagination_support",
                    "enterprise_flux_work_code_interpreter",
                    "enterprise_code_interpreter_citation_fix",
                    "at_mention_plugins_enable",
                    "enable_confirmation_interstitial",
                    "enable_plugin_auth_interstitial",
                    "enable_request_response_interstitials",
                    "enable_response_action_processing",
                    "enable_batch_token_processing",
                    "enable_selective_url_redaction",
                    "enable_gg_gpt",
                    "enable_inferred_memory_read",
                    "search_result_progress_messages_with_search_queries",
                    "flux_v3_references",
                    "flux_v3_references_entities",
                    "flux_v3_gptv_enable_upload_multi_image_in_turn_wo_ch",
                    "flux_v3_image_gen_enable_dimensions",
                    "gptvnorm2048",
                    "update_memory_plugin",
                    "add_custom_instructions",
                    "agent_recommendations",
                    "disable_cea_message_listener",
                    "code_interpreter_interactive_charts",
                    "cwc_code_interpreter_citation_fix",
                    "cwc_code_interpreter_interactive_charts_inline_image",
                    "code_interpreter_matplotlib_patching",
                    "flux_v3_image_gen_enable_icon_dimensions",
                    "flux_v3_image_gen_enable_system_text_with_params",
                    "flux_v3_image_gen_enable_designer_dimensions_meta_prompting_in_system_prompts",
                ],
                "allowedMessageTypes": [
                    "Chat",
                    "Suggestion",
                    "InternalSearchQuery",
                    "Disengaged",
                    "InternalLoaderMessage",
                    "Progress",
                    "GeneratedCode",
                    "RenderCardRequest",
                    "AdsQuery",
                    "SemanticSerp",
                    "GenerateContentQuery",
                    "SearchQuery",
                    "ConfirmationCard",
                    "AuthError",
                    "DeveloperLogs",
                    "TriggerPlugin",
                    "HintInvocation",
                    "MemoryUpdate",
                    "EndOfRequest",
                    "TriggerConfirmation",
                    "ResumeInvokeAction",
                    "ResumeUserInputRequest",
                    "TriggerUserInputRequest",
                    "EscapeHatch",
                    "TriggerPluginAuth",
                    "ResumePluginAuth",
                    "SideBySide",
                    "ReferencesListComplete",
                    "SwitchRespondingEndpoint",
                    "GenerateGraphicArt",
                ],
                "clientInfo": {
                    "clientPlatform": "teams-cli",
                    "clientAppName": "Office",
                    "clientEntrypoint": "teams-cli",
                    "clientSessionId": &session_id,
                    "clientAppType": "CLI",
                    "deviceOS": std::env::consts::OS,
                    "deviceType": "Desktop",
                },
                "message": {
                    "author": "user",
                    "inputMethod": "Keyboard",
                    "text": question,
                    "messageType": "Chat",
                    "requestId": &request_id,
                    "locale": "en-us",
                    "experienceType": "Default",
                    "locationInfo": {
                        "timeZoneOffset": chrono::Local::now().offset().local_minus_utc() / 60,
                        "timeZone": "UTC",
                    },
                    "adaptiveCards": [],
                    "entityAnnotationTypes": ["People", "File", "Event", "Email"],
                },
            }],
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
        let mut conv_id_from_server = None;
        let timeout = tokio::time::Duration::from_secs(120);
        let deadline = tokio::time::Instant::now() + timeout;
        let mut done = false;

        while !done {
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
                        // Server-to-client invocation (update, etc.) — extract any text
                        if let Some(args) = &signalr_msg.arguments {
                            for arg in args {
                                if let Some(t) = arg.get("text").and_then(|v| v.as_str()) {
                                    response_text = t.to_string();
                                }
                            }
                        }
                    }
                    2 => {
                        // StreamItem — main response delivery
                        if let Some(item) = &signalr_msg.item {
                            // Capture conversation ID from server
                            if let Some(cid) = item.get("conversationId").and_then(|v| v.as_str()) {
                                conv_id_from_server = Some(cid.to_string());
                            }

                            // Response text can be in multiple locations:
                            // - item.messages[last].text (streaming chunks)
                            // - item.result.message (final/error)
                            if let Some(messages) = item.get("messages").and_then(|v| v.as_array())
                            {
                                // Get the last bot message text
                                for m in messages.iter().rev() {
                                    if m.get("author").and_then(|v| v.as_str()) == Some("bot") {
                                        if let Some(t) = m.get("text").and_then(|v| v.as_str()) {
                                            response_text = t.to_string();
                                            break;
                                        }
                                    }
                                }
                            }

                            // Also check result.message for error/final messages
                            if let Some(result) = item.get("result") {
                                if let Some(t) = result.get("message").and_then(|v| v.as_str()) {
                                    if response_text.is_empty() {
                                        response_text = t.to_string();
                                    }
                                }
                            }
                        }
                    }
                    3 => {
                        // Completion — stream is done
                        if let Some(err) = &signalr_msg.error {
                            return Err(TeamsError::ApiError {
                                status: 0,
                                message: format!("copilot error: {err}"),
                            });
                        }
                        done = true;
                    }
                    7 => {
                        // Close
                        if let Some(err) = &signalr_msg.error {
                            return Err(TeamsError::ApiError {
                                status: 0,
                                message: format!("copilot connection closed with error: {err}"),
                            });
                        }
                        done = true;
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
            conversation_id: conv_id_from_server.unwrap_or(conv_id),
            message: response_text,
            citations: vec![],
        })
    }
}
