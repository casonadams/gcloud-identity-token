//! OAuth authentication logic for obtaining and refreshing Google tokens.

use crate::browser::{build_auth_url, capture_auth_code, open_browser_or_print};
use crate::cache::{load_cached_token, save_token};
use crate::config::{Creds, SavedToken, TokenOutput, TokenResponse};
use crate::shared::get_or_init_port;
use anyhow::Result;
use chrono::{Duration, Utc};
use reqwest::Client;

/// Obtain a fresh or cached Google access token and ID token.
///
/// Handles refresh, browser login, and local secure caching.
pub async fn get_token(creds: &Creds) -> Result<TokenOutput<'static>> {
    // Try cache first
    if let Some(saved) = load_cached_token() {
        if saved.token_expiry > Utc::now() + Duration::seconds(60) {
            return Ok(token_output_from_saved(saved));
        }

        // Expired — attempt refresh
        return refresh_token(creds, &saved).await;
    }

    // No cached token — full auth flow
    perform_login(creds).await
}

/// Refresh an expired token using the stored refresh token.
async fn refresh_token(creds: &Creds, saved: &SavedToken) -> Result<TokenOutput<'static>> {
    let client = Client::new();
    let res = client
        .post("https://oauth2.googleapis.com/token")
        .form(&[
            ("client_id", &creds.client_id),
            ("client_secret", &creds.client_secret),
            ("refresh_token", &saved.refresh_token),
            ("grant_type", &"refresh_token".to_string()),
        ])
        .send()
        .await?
        .json::<TokenResponse>()
        .await?;

    let expires_at = Utc::now() + Duration::seconds(res.expires_in);
    let refresh_token = res
        .refresh_token
        .clone()
        .unwrap_or_else(|| saved.refresh_token.clone());

    let updated = SavedToken {
        refresh_token,
        access_token: res.access_token.clone(),
        id_token: res.id_token.clone(),
        token_expiry: expires_at,
    };

    save_token(&updated)?;
    Ok(token_output_from_saved(updated))
}

/// Perform full browser-based OAuth flow.
async fn perform_login(creds: &Creds) -> Result<TokenOutput<'static>> {
    let port = get_or_init_port();
    let redirect_uri = format!("http://localhost:{port}");
    let auth_url = build_auth_url(&creds.client_id, &redirect_uri);
    open_browser_or_print(&auth_url);
    let code = capture_auth_code()?;

    let client = Client::new();
    let res = client
        .post("https://oauth2.googleapis.com/token")
        .form(&[
            ("code", &code),
            ("client_id", &creds.client_id),
            ("client_secret", &creds.client_secret),
            ("redirect_uri", &redirect_uri),
            ("grant_type", &"authorization_code".to_string()),
        ])
        .send()
        .await?
        .json::<TokenResponse>()
        .await?;

    let expires_at = Utc::now() + Duration::seconds(res.expires_in);

    if let Some(refresh_token) = &res.refresh_token {
        let saved = SavedToken {
            refresh_token: refresh_token.clone(),
            access_token: res.access_token.clone(),
            id_token: res.id_token.clone(),
            token_expiry: expires_at,
        };
        save_token(&saved)?;
    }

    Ok(TokenOutput {
        access_token: Box::leak(res.access_token.into_boxed_str()),
        id_token: Box::leak(res.id_token.into_boxed_str()),
        token_expiry: expires_at,
    })
}

/// Construct a `TokenOutput` from a `SavedToken`.
fn token_output_from_saved(saved: SavedToken) -> TokenOutput<'static> {
    TokenOutput {
        access_token: Box::leak(saved.access_token.into_boxed_str()),
        id_token: Box::leak(saved.id_token.into_boxed_str()),
        token_expiry: saved.token_expiry,
    }
}
