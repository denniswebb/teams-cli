pub mod authz;
pub mod csa;
pub mod messages;
pub mod mt;

use std::time::Duration;

use reqwest::{Client, RequestBuilder, Response, StatusCode};

use crate::config::NetworkConfig;
use crate::error::{Result, TeamsError};

pub struct HttpClient {
    pub client: Client,
    pub network: NetworkConfig,
}

impl HttpClient {
    pub fn new(network: &NetworkConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(network.timeout))
            .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) MicrosoftTeams-Insiders/1.5.00.25054 Chrome/91.0.4472.164 Electron/13.6.6 Safari/537.36")
            .build()
            .expect("failed to build HTTP client");

        Self {
            client,
            network: network.clone(),
        }
    }

    pub async fn execute_with_retry(
        &self,
        build_request: impl Fn() -> RequestBuilder,
    ) -> Result<Response> {
        let max_attempts = self.network.max_retries + 1;
        let mut last_error = None;

        for attempt in 0..max_attempts {
            if attempt > 0 {
                let delay = Duration::from_secs(self.network.retry_backoff_base.pow(attempt - 1));
                let delay = delay.min(Duration::from_secs(30));
                tracing::debug!("retry attempt {attempt}, waiting {delay:?}");
                tokio::time::sleep(delay).await;
            }

            let resp = match build_request().send().await {
                Ok(resp) => resp,
                Err(e) => {
                    tracing::warn!("request failed (attempt {attempt}): {e}");
                    last_error = Some(TeamsError::NetworkError(e));
                    continue;
                }
            };

            let status = resp.status();

            match status {
                s if s.is_success() => return Ok(resp),

                StatusCode::UNAUTHORIZED => {
                    return Err(TeamsError::ApiError {
                        status: 401,
                        message: resp.text().await.unwrap_or_default(),
                    });
                }

                StatusCode::FORBIDDEN => {
                    return Err(TeamsError::PermissionDenied(
                        resp.text().await.unwrap_or_default(),
                    ));
                }

                StatusCode::NOT_FOUND => {
                    return Err(TeamsError::NotFound(resp.text().await.unwrap_or_default()));
                }

                StatusCode::TOO_MANY_REQUESTS => {
                    let retry_after = resp
                        .headers()
                        .get("retry-after")
                        .and_then(|v| v.to_str().ok())
                        .and_then(|v| v.parse::<u64>().ok())
                        .unwrap_or(10);

                    if attempt + 1 < max_attempts {
                        tracing::warn!("rate limited, waiting {retry_after}s");
                        tokio::time::sleep(Duration::from_secs(retry_after)).await;
                        continue;
                    }

                    return Err(TeamsError::RateLimited {
                        retry_after_secs: retry_after,
                    });
                }

                s if s.is_server_error() => {
                    let body = resp.text().await.unwrap_or_default();
                    tracing::warn!("server error {s} (attempt {attempt}): {body}");
                    last_error = Some(TeamsError::ServerError {
                        status: s.as_u16(),
                        message: body,
                    });
                    continue;
                }

                s => {
                    return Err(TeamsError::ApiError {
                        status: s.as_u16(),
                        message: resp.text().await.unwrap_or_default(),
                    });
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            TeamsError::Other(anyhow::anyhow!("request failed after all retries"))
        }))
    }
}
