use serde::{Deserialize, Serialize};

use crate::error::{Result, TeamsError};

use super::HttpClient;

/// Event received from the long-poll subscription.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventMessage {
    #[serde(default)]
    pub id: u64,
    #[serde(default, rename = "type")]
    pub event_type: String,
    #[serde(default)]
    pub resource_type: String,
    #[serde(default)]
    pub time: String,
    #[serde(default, rename = "resourceLink")]
    pub resource_link: String,
    #[serde(default)]
    pub resource: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PollResponse {
    #[serde(default)]
    pub event_messages: Vec<EventMessage>,
    #[serde(default, rename = "errorCode")]
    pub error_code: Option<u32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SubscriptionRequest {
    channel_type: String,
    #[serde(rename = "interestedResources")]
    interested_resources: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct EndpointRegistration {
    id: String,
    #[serde(rename = "endpointFeatures")]
    endpoint_features: String,
}

pub struct SubscriptionClient<'a> {
    http: &'a HttpClient,
    skype_token: &'a str,
    chat_service_url: String,
    endpoint_id: String,
}

impl<'a> SubscriptionClient<'a> {
    pub fn new(http: &'a HttpClient, skype_token: &'a str, chat_service_url: &str) -> Self {
        let endpoint_id = uuid::Uuid::new_v4().to_string();
        Self {
            http,
            skype_token,
            chat_service_url: chat_service_url.to_string(),
            endpoint_id,
        }
    }

    fn auth_header(&self) -> String {
        format!("skypetoken={}", self.skype_token)
    }

    /// Register an endpoint with the chat service. Must be called before subscribing.
    pub async fn register_endpoint(&self) -> Result<()> {
        let url = format!("{}/v1/users/ME/endpoints", self.chat_service_url,);
        let auth = self.auth_header();

        let body = EndpointRegistration {
            id: self.endpoint_id.clone(),
            endpoint_features: "Agent".to_string(),
        };

        self.http
            .execute_with_retry(|| {
                self.http
                    .client
                    .post(&url)
                    .header("Authentication", &auth)
                    .json(&body)
            })
            .await?;

        tracing::debug!("registered endpoint: {}", self.endpoint_id);
        Ok(())
    }

    /// Create a subscription for incoming events. If `conversation_id` is None,
    /// subscribes to all conversations. Returns the subscription ID.
    pub async fn create_subscription(&self, conversation_id: Option<&str>) -> Result<()> {
        let url = format!(
            "{}/v1/users/ME/endpoints/{}/subscriptions",
            self.chat_service_url,
            urlencoding::encode(&self.endpoint_id),
        );
        let auth = self.auth_header();

        let interested_resources = match conversation_id {
            Some(conv_id) => {
                let encoded = urlencoding::encode(conv_id);
                vec![
                    format!("/v1/threads/{encoded}"),
                    format!("/v1/users/ME/conversations/{encoded}/messages"),
                ]
            }
            None => vec![
                "/v1/users/ME/conversations/ALL/messages".to_string(),
                "/v1/users/ME/conversations/ALL/properties".to_string(),
            ],
        };

        let body = SubscriptionRequest {
            channel_type: "httpLongPoll".to_string(),
            interested_resources,
        };

        self.http
            .execute_with_retry(|| {
                self.http
                    .client
                    .post(&url)
                    .header("Authentication", &auth)
                    .json(&body)
            })
            .await?;

        tracing::debug!("subscription created for endpoint {}", self.endpoint_id);
        Ok(())
    }

    /// Long-poll for events. Blocks until events arrive or server timeout (~30s).
    /// Returns the batch of events received.
    pub async fn poll(&self) -> Result<Vec<EventMessage>> {
        let url = format!(
            "{}/v1/users/ME/endpoints/{}/subscriptions/0/poll",
            self.chat_service_url,
            urlencoding::encode(&self.endpoint_id),
        );
        let auth = self.auth_header();

        // Use a longer timeout for long-polling (server typically holds ~30s)
        let resp = self
            .http
            .client
            .get(&url)
            .header("Authentication", &auth)
            .timeout(std::time::Duration::from_secs(120))
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    // Timeout is normal for long-poll — return empty
                    tracing::debug!("poll timeout (normal)");
                    return TeamsError::Other(anyhow::anyhow!("poll_timeout"));
                }
                TeamsError::NetworkError(e)
            })?;

        let status = resp.status();
        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(TeamsError::ApiError {
                status: 404,
                message: "endpoint or subscription not found — re-register required".to_string(),
            });
        }
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(TeamsError::ApiError {
                status: status.as_u16(),
                message: format!("poll failed: {body}"),
            });
        }

        let poll_resp = resp
            .json::<PollResponse>()
            .await
            .map_err(|e| TeamsError::ApiError {
                status: 0,
                message: format!("failed to parse poll response: {e}"),
            })?;

        Ok(poll_resp.event_messages)
    }

    pub fn endpoint_id(&self) -> &str {
        &self.endpoint_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::NetworkConfig;

    #[test]
    fn subscription_client_generates_endpoint_id() {
        let http = HttpClient::new(&NetworkConfig::default());
        let client = SubscriptionClient::new(&http, "token", "https://chat.example.com");
        assert!(!client.endpoint_id().is_empty());
        // UUID format: 8-4-4-4-12
        assert_eq!(client.endpoint_id().len(), 36);
    }

    #[test]
    fn auth_header_format() {
        let http = HttpClient::new(&NetworkConfig::default());
        let client = SubscriptionClient::new(&http, "my-token", "https://chat.example.com");
        assert_eq!(client.auth_header(), "skypetoken=my-token");
    }

    #[test]
    fn subscription_request_serialization_all() {
        let body = SubscriptionRequest {
            channel_type: "httpLongPoll".to_string(),
            interested_resources: vec![
                "/v1/users/ME/conversations/ALL/messages".to_string(),
                "/v1/users/ME/conversations/ALL/properties".to_string(),
            ],
        };
        let json = serde_json::to_value(&body).unwrap();
        assert_eq!(json["channelType"], "httpLongPoll");
        let resources = json["interestedResources"].as_array().unwrap();
        assert_eq!(resources.len(), 2);
        assert!(resources[0].as_str().unwrap().contains("ALL/messages"));
    }

    #[test]
    fn subscription_request_serialization_specific() {
        let conv_id = "19:abc@thread.v2";
        let encoded = urlencoding::encode(conv_id);
        let body = SubscriptionRequest {
            channel_type: "httpLongPoll".to_string(),
            interested_resources: vec![
                format!("/v1/threads/{encoded}"),
                format!("/v1/users/ME/conversations/{encoded}/messages"),
            ],
        };
        let json = serde_json::to_value(&body).unwrap();
        let resources = json["interestedResources"].as_array().unwrap();
        assert_eq!(resources.len(), 2);
        assert!(resources[0]
            .as_str()
            .unwrap()
            .contains("19%3Aabc%40thread.v2"));
    }

    #[test]
    fn endpoint_registration_serialization() {
        let body = EndpointRegistration {
            id: "test-uuid".to_string(),
            endpoint_features: "Agent".to_string(),
        };
        let json = serde_json::to_value(&body).unwrap();
        assert_eq!(json["id"], "test-uuid");
        assert_eq!(json["endpointFeatures"], "Agent");
    }

    #[test]
    fn poll_response_deserialization() {
        let json = r#"{
            "eventMessages": [
                {
                    "id": 42,
                    "type": "EventMessage",
                    "resourceType": "NewMessage",
                    "time": "2024-01-01T00:00:00Z",
                    "resourceLink": "/v1/users/ME/conversations/19:abc@thread.v2/messages/1234",
                    "resource": {"id": "1234", "content": "hello", "messagetype": "Text"}
                }
            ]
        }"#;
        let resp: PollResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.event_messages.len(), 1);
        assert_eq!(resp.event_messages[0].id, 42);
        assert_eq!(resp.event_messages[0].resource_type, "NewMessage");
        assert_eq!(resp.event_messages[0].resource["content"], "hello");
    }

    #[test]
    fn poll_response_empty() {
        let json = r#"{"eventMessages": []}"#;
        let resp: PollResponse = serde_json::from_str(json).unwrap();
        assert!(resp.event_messages.is_empty());
    }

    #[test]
    fn poll_response_missing_events_defaults_empty() {
        let json = r#"{}"#;
        let resp: PollResponse = serde_json::from_str(json).unwrap();
        assert!(resp.event_messages.is_empty());
    }

    #[test]
    fn event_message_deserialization_minimal() {
        let json = r#"{"id": 1, "resource": {}}"#;
        let event: EventMessage = serde_json::from_str(json).unwrap();
        assert_eq!(event.id, 1);
        assert_eq!(event.event_type, "");
        assert_eq!(event.resource_type, "");
    }

    #[test]
    fn unique_endpoint_ids() {
        let http = HttpClient::new(&NetworkConfig::default());
        let c1 = SubscriptionClient::new(&http, "t", "https://x.microsoft.com");
        let c2 = SubscriptionClient::new(&http, "t", "https://x.microsoft.com");
        assert_ne!(c1.endpoint_id(), c2.endpoint_id());
    }
}
