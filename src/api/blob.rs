use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;

use crate::error::{Result, TeamsError};

use super::HttpClient;

const AMS_CLIENT_VERSION: &str = "1415/1.0.0.2020050625";
const AMS_USER_AGENT: &str = "Teams-CLI/0.2.0";

#[derive(Debug, Deserialize)]
pub struct CreateObjectResponse {
    pub id: String,
}

pub struct BlobClient<'a> {
    http: &'a HttpClient,
    skype_token: &'a str,
    ams_v2_url: String,
    ams_url: String,
}

impl<'a> BlobClient<'a> {
    pub fn new(
        http: &'a HttpClient,
        skype_token: &'a str,
        ams_v2_url: &str,
        ams_url: &str,
    ) -> Self {
        Self {
            http,
            skype_token,
            ams_v2_url: ams_v2_url.to_string(),
            ams_url: ams_url.to_string(),
        }
    }

    fn auth_header(&self) -> String {
        format!("skype_token {}", self.skype_token)
    }

    pub async fn create_object(&self, conversation_id: &str, filename: &str) -> Result<String> {
        let url = format!("{}/v1/objects", self.ams_v2_url);
        let auth = self.auth_header();

        let mut permissions = HashMap::new();
        permissions.insert(conversation_id.to_string(), vec!["read"]);

        let body = serde_json::json!({
            "type": "pish/image",
            "permissions": permissions,
            "filename": filename,
        });

        let resp = self
            .http
            .execute_with_retry(|| {
                self.http
                    .client
                    .post(&url)
                    .header("Authorization", &auth)
                    .header("User-Agent", AMS_USER_AGENT)
                    .header("x-ms-client-version", AMS_CLIENT_VERSION)
                    .json(&body)
            })
            .await?;

        let obj = resp
            .json::<CreateObjectResponse>()
            .await
            .map_err(|e| TeamsError::ApiError {
                status: 0,
                message: format!("failed to parse create object response: {e}"),
            })?;

        Ok(obj.id)
    }

    pub async fn upload_content(&self, blob_id: &str, data: Vec<u8>) -> Result<()> {
        let url = format!("{}/v1/objects/{}/content/imgpsh", self.ams_v2_url, blob_id);
        let auth = self.auth_header();

        self.http
            .execute_with_retry(|| {
                self.http
                    .client
                    .put(&url)
                    .header("Authorization", &auth)
                    .header("User-Agent", AMS_USER_AGENT)
                    .header("x-ms-client-version", AMS_CLIENT_VERSION)
                    .header("Content-Type", "application/octet-stream")
                    .body(data.clone())
            })
            .await?;

        Ok(())
    }

    pub fn image_url(&self, blob_id: &str) -> String {
        format!("{}/v1/objects/{}/views/imgo", self.ams_url, blob_id)
    }

    pub fn build_image_html(&self, blob_id: &str) -> String {
        let src = self.image_url(blob_id);
        format!(
            "<span contenteditable='false'>\
             <img src='{src}' id='{blob_id}' \
             target-src='{src}' \
             itemscope='' itemtype='http://schema.skype.com/AMSImage'>\
             </span>"
        )
    }

    pub async fn upload_image(&self, conversation_id: &str, file_path: &Path) -> Result<String> {
        let filename = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("image.png");

        let data = std::fs::read(file_path).map_err(|e| {
            TeamsError::InvalidInput(format!("failed to read file {}: {e}", file_path.display()))
        })?;

        let blob_id = self.create_object(conversation_id, filename).await?;
        self.upload_content(&blob_id, data).await?;

        Ok(blob_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::NetworkConfig;

    #[test]
    fn auth_header_format() {
        let http = HttpClient::new(&NetworkConfig::default());
        let client = BlobClient::new(
            &http,
            "test-token",
            "https://us-prod.asyncgw.teams.microsoft.com",
            "https://us-api.asm.skype.com",
        );
        assert_eq!(client.auth_header(), "skype_token test-token");
    }

    #[test]
    fn image_url_format() {
        let http = HttpClient::new(&NetworkConfig::default());
        let client = BlobClient::new(
            &http,
            "tok",
            "https://us-prod.asyncgw.teams.microsoft.com",
            "https://us-api.asm.skype.com",
        );
        assert_eq!(
            client.image_url("0-us-d3-abc123"),
            "https://us-api.asm.skype.com/v1/objects/0-us-d3-abc123/views/imgo"
        );
    }

    #[test]
    fn build_image_html_contains_required_attrs() {
        let http = HttpClient::new(&NetworkConfig::default());
        let client = BlobClient::new(
            &http,
            "tok",
            "https://us-prod.asyncgw.teams.microsoft.com",
            "https://us-api.asm.skype.com",
        );
        let html = client.build_image_html("0-us-d3-abc123");
        assert!(html.contains("itemtype='http://schema.skype.com/AMSImage'"));
        assert!(html.contains("id='0-us-d3-abc123'"));
        assert!(html.contains("us-api.asm.skype.com/v1/objects/0-us-d3-abc123/views/imgo"));
        assert!(html.contains("<span contenteditable='false'>"));
    }
}
