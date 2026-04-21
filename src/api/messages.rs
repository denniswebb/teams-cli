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
        let encoded_id = urlencoding::encode(conversation_id);
        let url = format!(
            "{}/v1/users/ME/conversations/{}/messages?view=msnp24Equivalent|supportsMessageProperties&pageSize={}&startTime=1",
            self.chat_service_url, encoded_id, page_size,
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
        is_html: bool,
        mentions_json: Option<&str>,
        amsreferences: Option<Vec<String>>,
    ) -> Result<serde_json::Value> {
        let encoded_id = urlencoding::encode(conversation_id);
        let url = format!(
            "{}/v1/users/ME/conversations/{}/messages",
            self.chat_service_url, encoded_id,
        );
        let auth = self.auth_header();

        let messagetype = if is_html { "RichText/Html" } else { "Text" };
        let body = SendMessageRequest {
            content: content.to_string(),
            messagetype: messagetype.to_string(),
            contenttype: "text".to_string(),
            clientmessageid: chrono::Utc::now().timestamp_millis().to_string(),
            imdisplayname: display_name.to_string(),
            properties: Some(SendMessageProperties {
                importance: Some(String::new()),
                subject: None,
                mentions: mentions_json.map(|s| s.to_string()),
            }),
            amsreferences,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::NetworkConfig;

    #[test]
    fn auth_header_format() {
        let http = HttpClient::new(&NetworkConfig::default());
        let client = MessagesClient::new(&http, "test-token-123", "https://chat.example.com");
        assert_eq!(client.auth_header(), "skypetoken=test-token-123");
    }

    #[test]
    fn conversation_id_is_url_encoded_in_get_messages_url() {
        // Verify the URL encoding logic by checking the encoded form of a typical conversation ID
        let conv_id = "19:abc@thread.v2";
        let encoded = urlencoding::encode(conv_id);
        assert_eq!(encoded, "19%3Aabc%40thread.v2");
    }

    #[test]
    fn conversation_id_encoding_preserves_safe_chars() {
        let conv_id = "simple-id-123";
        let encoded = urlencoding::encode(conv_id);
        assert_eq!(encoded, "simple-id-123");
    }

    #[test]
    fn messages_client_stores_chat_service_url() {
        let http = HttpClient::new(&NetworkConfig::default());
        let client = MessagesClient::new(&http, "tok", "https://amer.ng.msg.teams.microsoft.com");
        assert_eq!(
            client.chat_service_url,
            "https://amer.ng.msg.teams.microsoft.com"
        );
    }
}
