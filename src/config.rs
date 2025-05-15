//! Configuration types and helpers for working with Google OAuth credentials.
//!
//! This module defines the key data structures used during OAuth flows and
//! provides a helper to load credentials from the user's local environment.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents OAuth client credentials used to initiate the authorization flow.
///
/// These credentials are typically loaded from a JSON file located at:
/// `~/.config/gcloud/application_default_credentials.json`
#[derive(Deserialize)]
pub struct Creds {
    /// OAuth 2.0 client ID
    pub client_id: String,
    /// OAuth 2.0 client secret
    pub client_secret: String,
}

/// A token response received from Google's OAuth token endpoint.
///
/// This includes the access token, ID token, optional refresh token,
/// and the token expiration duration (in seconds).
#[derive(Deserialize)]
pub struct TokenResponse {
    /// OAuth 2.0 access token used for Google APIs
    pub access_token: String,
    /// OpenID Connect ID token (JWT) containing user identity
    pub id_token: String,
    /// Refresh token (only returned during first login)
    #[serde(default)]
    pub refresh_token: Option<String>,
    /// Time until expiration in seconds
    pub expires_in: i64,
}

/// Output returned by the library to the user after successful authentication.
///
/// This structure is printed as JSON and includes only the fields necessary
/// for downstream use (access, identity, and expiration).
#[derive(Serialize)]
pub struct TokenOutput<'a> {
    /// OAuth 2.0 access token
    pub access_token: &'a str,
    /// ID token (JWT) identifying the user
    pub id_token: &'a str,
    /// UTC expiry timestamp
    pub token_expiry: DateTime<Utc>,
}

/// A saved token cached on disk for future reuse.
///
/// This includes the refresh token, current access and ID tokens,
/// and their expiration timestamp.
#[derive(Serialize, Deserialize)]
pub struct SavedToken {
    /// Long-lived refresh token for future access
    pub refresh_token: String,
    /// Most recently issued access token
    pub access_token: String,
    /// Most recently issued ID token
    pub id_token: String,
    /// Expiration timestamp of the token
    pub token_expiry: DateTime<Utc>,
}

/// Loads the user's OAuth 2.0 credentials from the default gcloud location.
///
/// This typically reads the file:
/// `~/.config/gcloud/application_default_credentials.json`
///
/// # Errors
///
/// Returns an error if the file is missing, unreadable, or invalid JSON.
pub fn load_creds() -> Result<Creds> {
    let path = dirs::home_dir()
        .ok_or("Could not determine home directory")
        .map_err(|_| anyhow::anyhow!("Home directory not found"))?
        .join(".config/gcloud/application_default_credentials.json");

    let creds = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&creds)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_creds() {
        let json = r#"{
            "client_id": "abc123",
            "client_secret": "secret"
        }"#;
        let creds: Creds = serde_json::from_str(json).unwrap();
        assert_eq!(creds.client_id, "abc123");
    }

    #[test]
    fn test_missing_field_fails() {
        let json = r#"{
            "client_id": "abc123"
        }"#;
        let result: Result<Creds, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }
}
