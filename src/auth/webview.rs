use std::sync::{Arc, Mutex};

use base64::Engine;
use sha2::{Digest, Sha256};
use tao::event::{Event, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tao::platform::run_return::EventLoopExtRunReturn;
use tao::window::WindowBuilder;
use wry::WebViewBuilder;

use super::token::{
    extract_tenant_id, TokenInfo, TokenSet, TokenType, CHATSVCAGG_RESOURCE, COPILOT_APP_ID,
    COPILOT_RESOURCE, MICROSOFT_TENANT_ID, OUTLOOK_RESOURCE, REDIRECT_URI, SKYPE_RESOURCE,
    TEAMS_APP_ID,
};
use crate::error::TeamsError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AuthPhase {
    Teams,
    Skype,
    ChatSvcAgg,
    Outlook,
    Copilot,
    Done,
}

struct AuthState {
    phase: AuthPhase,
    teams_token: Option<String>,
    skype_token: Option<String>,
    chatsvcagg_token: Option<String>,
    outlook_token: Option<String>,
    copilot_token: Option<String>,
    tenant_id: String,
    redirect_count: u32,
    expected_state: Option<String>,
    code_verifier: Option<String>,
}

fn build_auth_url(
    tenant: &str,
    response_type: &str,
    resource: Option<&str>,
    client_id: &str,
) -> (String, String) {
    let state = uuid::Uuid::new_v4().to_string();
    let nonce = uuid::Uuid::new_v4().to_string();
    let client_request_id = uuid::Uuid::new_v4().to_string();

    let mut url = format!(
        "https://login.microsoftonline.com/{tenant}/oauth2/authorize\
         ?response_type={response_type}\
         &client_id={client_id}\
         &redirect_uri={REDIRECT_URI}\
         &state={state}\
         &client-request-id={client_request_id}\
         &nonce={nonce}\
         &x-client-SKU=Js\
         &x-client-Ver=1.0.9"
    );

    if let Some(res) = resource {
        url.push_str(&format!("&resource={res}"));
    }

    (url, state)
}

// The Copilot app ID doesn't have the Teams redirect URI registered.
// Use the standard native client redirect URI instead.
const PKCE_REDIRECT_URI: &str = "https://login.microsoftonline.com/common/oauth2/nativeclient";

/// Build an OAuth2 authorization code URL with PKCE.
/// Returns (url, state, code_verifier).
fn build_pkce_auth_url(tenant: &str, resource: &str, client_id: &str) -> (String, String, String) {
    let state = uuid::Uuid::new_v4().to_string();
    let nonce = uuid::Uuid::new_v4().to_string();
    let client_request_id = uuid::Uuid::new_v4().to_string();

    // Generate PKCE code_verifier (43-128 chars, unreserved URI chars)
    let code_verifier = uuid::Uuid::new_v4().to_string().replace('-', "")
        + &uuid::Uuid::new_v4().to_string().replace('-', "");

    // code_challenge = BASE64URL(SHA256(code_verifier))
    let mut hasher = Sha256::new();
    hasher.update(code_verifier.as_bytes());
    let hash = hasher.finalize();
    let code_challenge = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(hash);

    let encoded_redirect = urlencoding::encode(PKCE_REDIRECT_URI);

    let url = format!(
        "https://login.microsoftonline.com/{tenant}/oauth2/authorize\
         ?response_type=code\
         &client_id={client_id}\
         &redirect_uri={encoded_redirect}\
         &state={state}\
         &client-request-id={client_request_id}\
         &nonce={nonce}\
         &resource={resource}\
         &code_challenge={code_challenge}\
         &code_challenge_method=S256\
         &x-client-SKU=Js\
         &x-client-Ver=1.0.9"
    );

    (url, state, code_verifier)
}

/// Exchange an authorization code for an access token using PKCE.
fn exchange_code_for_token(
    tenant: &str,
    code: &str,
    code_verifier: &str,
    client_id: &str,
    resource: &str,
) -> std::result::Result<String, TeamsError> {
    let token_url = format!("https://login.microsoftonline.com/{tenant}/oauth2/token");

    let client = reqwest::blocking::Client::new();
    let resp = client
        .post(&token_url)
        .form(&[
            ("grant_type", "authorization_code"),
            ("client_id", client_id),
            ("code", code),
            ("redirect_uri", PKCE_REDIRECT_URI),
            ("code_verifier", code_verifier),
            ("resource", resource),
        ])
        .send()
        .map_err(|e| TeamsError::AuthError(format!("token exchange request failed: {e}")))?;

    if !resp.status().is_success() {
        let body = resp.text().unwrap_or_default();
        return Err(TeamsError::AuthError(format!(
            "token exchange failed: {body}"
        )));
    }

    let body: serde_json::Value = resp
        .json()
        .map_err(|e| TeamsError::AuthError(format!("token exchange parse failed: {e}")))?;

    body.get("access_token")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| TeamsError::AuthError("token exchange: no access_token in response".into()))
}

fn extract_token_from_fragment(url_str: &str) -> Option<(String, bool, Option<String>)> {
    let fragment = url_str.split('#').nth(1)?;
    let params = url::form_urlencoded::parse(fragment.as_bytes());
    let mut token = None;
    let mut is_id_token = false;
    let mut state_param = None;

    for (key, value) in params {
        match key.as_ref() {
            "id_token" => {
                token = Some(value.to_string());
                is_id_token = true;
            }
            "access_token" => {
                if token.is_none() {
                    token = Some(value.to_string());
                }
            }
            "state" => {
                state_param = Some(value.to_string());
            }
            _ => {}
        }
    }

    token.map(|t| (t, is_id_token, state_param))
}

/// Extract authorization code from redirect URL query params.
fn extract_code_from_query(url_str: &str) -> Option<(String, Option<String>)> {
    let parsed = url::Url::parse(url_str).ok()?;
    let mut code = None;
    let mut state_param = None;

    for (key, value) in parsed.query_pairs() {
        match key.as_ref() {
            "code" => code = Some(value.to_string()),
            "state" => state_param = Some(value.to_string()),
            _ => {}
        }
    }

    code.map(|c| (c, state_param))
}

fn is_allowed_domain(url: &str) -> bool {
    let allowed = [
        "login.microsoftonline.com",
        "login.microsoft.com",
        "login.live.com",
        "teams.microsoft.com",
        "aadcdn.msauth.net",
        "aadcdn.msftauth.net",
    ];
    if let Ok(parsed) = url::Url::parse(url) {
        if let Some(host) = parsed.host_str() {
            return allowed
                .iter()
                .any(|d| host == *d || host.ends_with(&format!(".{d}")));
        }
    }
    false
}

/// Run the webview-based 5-token OAuth2 flow (Teams, Skype, ChatSvcAgg, Outlook, Copilot).
/// The first 4 tokens use implicit grant. The Copilot token uses authorization code + PKCE
/// because the Copilot app ID doesn't support implicit grant.
/// Must be called from the main thread. Returns the token set on success.
pub fn webview_login(initial_tenant: &str, profile: &str) -> crate::error::Result<TokenSet> {
    let (initial_url, initial_state) =
        build_auth_url(initial_tenant, "id_token", None, TEAMS_APP_ID);

    let state = Arc::new(Mutex::new(AuthState {
        phase: AuthPhase::Teams,
        teams_token: None,
        skype_token: None,
        chatsvcagg_token: None,
        outlook_token: None,
        copilot_token: None,
        tenant_id: initial_tenant.to_string(),
        redirect_count: 0,
        expected_state: Some(initial_state),
        code_verifier: None,
    }));

    let profile = profile.to_string();
    let result: Arc<Mutex<Option<crate::error::Result<TokenSet>>>> = Arc::new(Mutex::new(None));

    let mut event_loop = EventLoopBuilder::<String>::with_user_event().build();
    let proxy = event_loop.create_proxy();

    let window = WindowBuilder::new()
        .with_title("Teams CLI - Sign In")
        .with_inner_size(tao::dpi::LogicalSize::new(500.0, 700.0))
        .with_focused(true)
        .build(&event_loop)
        .expect("failed to create window");

    let state_for_nav = state.clone();
    let proxy_for_nav = proxy.clone();

    let webview = WebViewBuilder::new()
        .with_url(&initial_url)
        .with_navigation_handler(move |url: String| {
            let is_teams_redirect = url.starts_with("https://teams.microsoft.com/go");
            let is_native_redirect = url.starts_with(PKCE_REDIRECT_URI);

            if !is_teams_redirect && !is_native_redirect {
                if !is_allowed_domain(&url) {
                    tracing::warn!("blocked navigation to disallowed domain: {url}");
                    return false;
                }
                return true;
            }

            let mut auth = state_for_nav.lock().unwrap_or_else(|e| e.into_inner());
            auth.redirect_count += 1;

            if auth.redirect_count > 12 {
                let _ = proxy_for_nav.send_event("error:too many redirects".into());
                return false;
            }

            // For the Copilot phase (PKCE), extract code from query params
            if auth.phase == AuthPhase::Copilot && is_native_redirect {
                if let Some((code, returned_state)) = extract_code_from_query(&url) {
                    // Validate state
                    if let Some(ref expected) = auth.expected_state {
                        match returned_state {
                            Some(ref rs) if rs != expected => {
                                tracing::warn!(
                                    "OAuth state mismatch: expected={expected}, got={rs}"
                                );
                                return false;
                            }
                            None => {
                                tracing::warn!("OAuth response missing state parameter");
                                return false;
                            }
                            _ => {}
                        }
                    }

                    // Exchange code for token (blocking HTTP POST)
                    let verifier = auth.code_verifier.as_deref().unwrap_or("");
                    match exchange_code_for_token(
                        &auth.tenant_id,
                        &code,
                        verifier,
                        COPILOT_APP_ID,
                        COPILOT_RESOURCE,
                    ) {
                        Ok(access_token) => {
                            auth.copilot_token = Some(access_token);
                            auth.phase = AuthPhase::Done;
                            auth.expected_state = None;
                            let _ = proxy_for_nav.send_event("done".into());
                        }
                        Err(e) => {
                            let _ = proxy_for_nav
                                .send_event(format!("error:copilot token exchange: {e}"));
                        }
                    }
                }
                return false;
            }

            // For all other phases, extract token from fragment (implicit grant)
            let Some((token, is_id_token, returned_state)) = extract_token_from_fragment(&url)
            else {
                return false;
            };

            if let Some(ref expected) = auth.expected_state {
                match returned_state {
                    Some(ref rs) if rs != expected => {
                        tracing::warn!("OAuth state mismatch: expected={expected}, got={rs}");
                        return false;
                    }
                    None => {
                        tracing::warn!("OAuth response missing state parameter");
                        return false;
                    }
                    _ => {}
                }
            }

            match auth.phase {
                AuthPhase::Teams => {
                    if is_id_token {
                        if let Ok(Some(tid)) = extract_tenant_id(&token) {
                            if tid == MICROSOFT_TENANT_ID {
                                auth.tenant_id = "common".to_string();
                            } else {
                                auth.tenant_id = tid;
                            }
                        }
                        auth.teams_token = Some(token);
                        auth.phase = AuthPhase::Skype;
                        let (next_url, next_state) = build_auth_url(
                            &auth.tenant_id,
                            "token",
                            Some(SKYPE_RESOURCE),
                            TEAMS_APP_ID,
                        );
                        auth.expected_state = Some(next_state);
                        let _ = proxy_for_nav.send_event(format!("navigate:{next_url}"));
                    }
                }
                AuthPhase::Skype => {
                    auth.skype_token = Some(token);
                    auth.phase = AuthPhase::ChatSvcAgg;
                    let (next_url, next_state) = build_auth_url(
                        &auth.tenant_id,
                        "token",
                        Some(CHATSVCAGG_RESOURCE),
                        TEAMS_APP_ID,
                    );
                    auth.expected_state = Some(next_state);
                    let _ = proxy_for_nav.send_event(format!("navigate:{next_url}"));
                }
                AuthPhase::ChatSvcAgg => {
                    auth.chatsvcagg_token = Some(token);
                    auth.phase = AuthPhase::Outlook;
                    let (next_url, next_state) = build_auth_url(
                        &auth.tenant_id,
                        "token",
                        Some(OUTLOOK_RESOURCE),
                        TEAMS_APP_ID,
                    );
                    auth.expected_state = Some(next_state);
                    let _ = proxy_for_nav.send_event(format!("navigate:{next_url}"));
                }
                AuthPhase::Outlook => {
                    auth.outlook_token = Some(token);
                    auth.phase = AuthPhase::Copilot;
                    // Copilot uses PKCE authorization code flow with a different app ID
                    let (next_url, next_state, code_verifier) =
                        build_pkce_auth_url(&auth.tenant_id, COPILOT_RESOURCE, COPILOT_APP_ID);
                    auth.expected_state = Some(next_state);
                    auth.code_verifier = Some(code_verifier);
                    let _ = proxy_for_nav.send_event(format!("navigate:{next_url}"));
                }
                AuthPhase::Copilot | AuthPhase::Done => {}
            }

            false // Always block redirect to teams.microsoft.com/go
        })
        .build(&window)
        .expect("failed to create webview");

    let state_for_loop = state.clone();
    let result_for_loop = result.clone();

    event_loop.run_return(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::UserEvent(ref msg) => {
                if let Some(url) = msg.strip_prefix("navigate:") {
                    if let Err(e) = webview.load_url(url) {
                        *result_for_loop.lock().unwrap_or_else(|e| e.into_inner()) = Some(Err(
                            TeamsError::AuthError(format!("failed to navigate: {e}")),
                        ));
                        *control_flow = ControlFlow::Exit;
                    }
                } else if msg == "done" {
                    let auth = state_for_loop.lock().unwrap_or_else(|e| e.into_inner());
                    let outcome = build_token_set(&auth, &profile).and_then(|token_set| {
                        crate::auth::keyring::store_tokens(&profile, &token_set)?;
                        Ok(token_set)
                    });
                    *result_for_loop.lock().unwrap_or_else(|e| e.into_inner()) = Some(outcome);
                    *control_flow = ControlFlow::Exit;
                } else if let Some(err_msg) = msg.strip_prefix("error:") {
                    *result_for_loop.lock().unwrap_or_else(|e| e.into_inner()) =
                        Some(Err(TeamsError::AuthError(err_msg.to_string())));
                    *control_flow = ControlFlow::Exit;
                }
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                let auth = state_for_loop.lock().unwrap_or_else(|e| e.into_inner());
                if auth.phase == AuthPhase::Done {
                    let outcome = build_token_set(&auth, &profile).and_then(|token_set| {
                        crate::auth::keyring::store_tokens(&profile, &token_set)?;
                        Ok(token_set)
                    });
                    *result_for_loop.lock().unwrap_or_else(|e| e.into_inner()) = Some(outcome);
                } else {
                    *result_for_loop.lock().unwrap_or_else(|e| e.into_inner()) =
                        Some(Err(TeamsError::AuthError("login cancelled by user".into())));
                }
                *control_flow = ControlFlow::Exit;
            }
            _ => {}
        }
    });

    // Extract the result from the event loop
    let outcome = result
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .take()
        .unwrap_or_else(|| Err(TeamsError::AuthError("login did not complete".into())));

    outcome
}

fn build_token_set(auth: &AuthState, profile: &str) -> crate::error::Result<TokenSet> {
    let teams_raw = auth
        .teams_token
        .as_ref()
        .ok_or_else(|| TeamsError::AuthError("missing teams token".into()))?;
    let skype_raw = auth
        .skype_token
        .as_ref()
        .ok_or_else(|| TeamsError::AuthError("missing skype token".into()))?;
    let chatsvcagg_raw = auth
        .chatsvcagg_token
        .as_ref()
        .ok_or_else(|| TeamsError::AuthError("missing chatsvcagg token".into()))?;

    let outlook = auth
        .outlook_token
        .as_ref()
        .map(|raw| TokenInfo::from_jwt(raw, TokenType::AccessToken))
        .transpose()?;

    let copilot = auth
        .copilot_token
        .as_ref()
        .map(|raw| TokenInfo::from_jwt(raw, TokenType::AccessToken))
        .transpose()?;

    Ok(TokenSet {
        teams: TokenInfo::from_jwt(teams_raw, TokenType::IdToken)?,
        skype: TokenInfo::from_jwt(skype_raw, TokenType::AccessToken)?,
        chatsvcagg: TokenInfo::from_jwt(chatsvcagg_raw, TokenType::AccessToken)?,
        outlook,
        copilot,
        profile: profile.to_string(),
        tenant_id: auth.tenant_id.clone(),
    })
}
