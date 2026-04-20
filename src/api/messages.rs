use crate::error::Result;
use crate::models::{ChatMessage, MessagesResponse, SendMessageProperties, SendMessageRequest};

use super::HttpClient;

pub struct MessagesClient<'a> {
    http: &'a HttpClient,
    skype_token: &'a str,
    chat_service_url: String,
}

impl<'a> MessagesClient<'a> {
    pub fn new(http: &'a HttpClient, skype_token: &'a str, chat_service_url: &str) -> Self {
        Self {
            http,
            skype_token,
            chat_service_url: chat_service_url.to_string(),
        }
    }

    fn auth_header(&self) -> String {
        format!("skypetoken={}", self.skype_token)
    }

    pub async fn get_messages(
        &self,
        conversation_id: &str,
        page_size: u32,
    ) -> Result<Vec<ChatMessage>> {
        let url = format!(
            "{}/v1/users/ME/conversations/{}/messages?view=msnp24Equivalent|supportsMessageProperties&pageSize={}&startTime=1",
            self.chat_service_url, conversation_id, page_size,
        );
        let auth = self.auth_header();

        let resp = self
            .http
            .execute_with_retry(|| self.http.client.get(&url).header("Authentication", &auth))
            .await?;

        let messages_resp = resp.json::<MessagesResponse>().await.map_err(|e| {
            crate::error::TeamsError::ApiError {
                status: 0,
                message: format!("failed to parse messages: {e}"),
            }
        })?;

        Ok(messages_resp.messages)
    }

    pub async fn send_message(
        &self,
        conversation_id: &str,
        content: &str,
        display_name: &str,
    ) -> Result<serde_json::Value> {
        let url = format!(
            "{}/v1/users/ME/conversations/{}/messages",
            self.chat_service_url, conversation_id,
        );
        let auth = self.auth_header();

        let body = SendMessageRequest {
            content: content.to_string(),
            messagetype: "Text".to_string(),
            contenttype: "text".to_string(),
            clientmessageid: chrono::Utc::now().timestamp_millis().to_string(),
            imdisplayname: display_name.to_string(),
            properties: Some(SendMessageProperties {
                importance: Some(String::new()),
                subject: None,
            }),
        };

        let resp = self
            .http
            .execute_with_retry(|| {
                self.http
                    .client
                    .post(&url)
                    .header("Authentication", &auth)
                    .json(&body)
            })
            .await?;

        resp.json::<serde_json::Value>()
            .await
            .map_err(|e| crate::error::TeamsError::ApiError {
                status: 0,
                message: format!("failed to parse send response: {e}"),
            })
    }
}
