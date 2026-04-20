use std::sync::{Arc, Mutex};

use tao::event::{Event, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tao::window::WindowBuilder;
use wry::WebViewBuilder;

use super::token::{
    extract_tenant_id, TokenInfo, TokenSet, TokenType, CHATSVCAGG_RESOURCE, MICROSOFT_TENANT_ID,
    REDIRECT_URI, SKYPE_RESOURCE, TEAMS_APP_ID,
};
use crate::error::TeamsError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AuthPhase {
    Teams,
    Skype,
    ChatSvcAgg,
    Done,
}

struct AuthState {
    phase: AuthPhase,
    teams_token: Option<String>,
    skype_token: Option<String>,
    chatsvcagg_token: Option<String>,
    tenant_id: String,
    redirect_count: u32,
}

fn build_auth_url(tenant: &str, response_type: &str, resource: Option<&str>) -> String {
    let state = uuid::Uuid::new_v4().to_string();
    let nonce = uuid::Uuid::new_v4().to_string();
    let client_request_id = uuid::Uuid::new_v4().to_string();

    let mut url = format!(
        "https://login.microsoftonline.com/{tenant}/oauth2/authorize\
         ?response_type={response_type}\
         &client_id={TEAMS_APP_ID}\
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

    url
}

fn extract_token_from_fragment(url_str: &str) -> Option<(String, bool)> {
    let fragment = url_str.split('#').nth(1)?;
    let params = url::form_urlencoded::parse(fragment.as_bytes());
    let mut token = None;
    let mut is_id_token = false;

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
            _ => {}
        }
    }

    token.map(|t| (t, is_id_token))
}

/// Run the webview-based 3-token OAuth2 flow.
/// Must be called from the main thread. Calls process::exit when done.
pub fn webview_login(initial_tenant: &str, profile: &str) -> ! {
    let state = Arc::new(Mutex::new(AuthState {
        phase: AuthPhase::Teams,
        teams_token: None,
        skype_token: None,
        chatsvcagg_token: None,
        tenant_id: initial_tenant.to_string(),
        redirect_count: 0,
    }));

    let profile = profile.to_string();

    // UserEvent carries: "navigate:<url>", "done", or "error:<msg>"
    let event_loop = EventLoopBuilder::<String>::with_user_event().build();
    let proxy = event_loop.create_proxy();

    let window = WindowBuilder::new()
        .with_title("Teams CLI - Sign In")
        .with_inner_size(tao::dpi::LogicalSize::new(500.0, 700.0))
        .with_focused(true)
        .build(&event_loop)
        .expect("failed to create window");

    let initial_url = build_auth_url(initial_tenant, "id_token", None);

    let state_for_nav = state.clone();
    let proxy_for_nav = proxy.clone();

    let webview = WebViewBuilder::new()
        .with_url(&initial_url)
        .with_navigation_handler(move |url: String| {
            if !url.starts_with("https://teams.microsoft.com/go") {
                return true; // Allow all non-redirect navigation
            }

            let Some((token, is_id_token)) = extract_token_from_fragment(&url) else {
                return false; // Block redirect but no token found
            };

            let mut auth = state_for_nav.lock().unwrap();
            auth.redirect_count += 1;

            if auth.redirect_count > 10 {
                let _ = proxy_for_nav.send_event("error:too many redirects".into());
                return false;
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
                        let next_url =
                            build_auth_url(&auth.tenant_id, "token", Some(SKYPE_RESOURCE));
                        let _ = proxy_for_nav.send_event(format!("navigate:{next_url}"));
                    }
                }
                AuthPhase::Skype => {
                    auth.skype_token = Some(token);
                    auth.phase = AuthPhase::ChatSvcAgg;
                    let next_url =
                        build_auth_url(&auth.tenant_id, "token", Some(CHATSVCAGG_RESOURCE));
                    let _ = proxy_for_nav.send_event(format!("navigate:{next_url}"));
                }
                AuthPhase::ChatSvcAgg => {
                    auth.chatsvcagg_token = Some(token);
                    auth.phase = AuthPhase::Done;
                    let _ = proxy_for_nav.send_event("done".into());
                }
                AuthPhase::Done => {}
            }

            false // Always block redirect to teams.microsoft.com/go
        })
        .build(&window)
        .expect("failed to create webview");

    let state_for_loop = state.clone();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::UserEvent(ref msg) => {
                if let Some(url) = msg.strip_prefix("navigate:") {
                    // Navigate the webview to the next auth URL
                    if let Err(e) = webview.load_url(url) {
                        eprintln!("Failed to navigate: {e}");
                        std::process::exit(3);
                    }
                } else if msg == "done" {
                    let auth = state_for_loop.lock().unwrap();
                    match build_token_set(&auth, &profile) {
                        Ok(token_set) => {
                            if let Err(e) = crate::auth::keyring::store_tokens(&profile, &token_set)
                            {
                                eprintln!("Failed to store tokens: {e}");
                                std::process::exit(3);
                            }
                            let username = super::token::extract_username(&token_set.teams.raw)
                                .unwrap_or_else(|_| "unknown".into());
                            eprintln!("Authenticated as {username}");
                            std::process::exit(0);
                        }
                        Err(e) => {
                            eprintln!("Authentication failed: {e}");
                            std::process::exit(3);
                        }
                    }
                } else if let Some(err_msg) = msg.strip_prefix("error:") {
                    eprintln!("Authentication error: {err_msg}");
                    std::process::exit(3);
                }
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                let auth = state_for_loop.lock().unwrap();
                if auth.phase == AuthPhase::Done {
                    match build_token_set(&auth, &profile) {
                        Ok(token_set) => {
                            let _ = crate::auth::keyring::store_tokens(&profile, &token_set);
                            std::process::exit(0);
                        }
                        Err(e) => {
                            eprintln!("Authentication failed: {e}");
                            std::process::exit(3);
                        }
                    }
                } else {
                    eprintln!("Login cancelled by user");
                    std::process::exit(3);
                }
            }
            _ => {}
        }
    })
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

    Ok(TokenSet {
        teams: TokenInfo::from_jwt(teams_raw, TokenType::IdToken)?,
        skype: TokenInfo::from_jwt(skype_raw, TokenType::AccessToken)?,
        chatsvcagg: TokenInfo::from_jwt(chatsvcagg_raw, TokenType::AccessToken)?,
        profile: profile.to_string(),
        tenant_id: auth.tenant_id.clone(),
    })
}
