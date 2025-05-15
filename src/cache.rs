//! Token caching utility.
//!
//! This stores OAuth tokens securely using the system keyring (by default) or
//! to a file if the `GCLOUD_IDENTITY_TOKEN_PATH` environment variable is set.
//!
//! The keyring entry is namespaced under the service `gcloud-identity-token`
//! and the keyring "username" is extracted from the ID token's email field.

use crate::config::SavedToken;
use anyhow::Result;
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use keyring::Entry;
use serde::Deserialize;
use std::{fs, path::PathBuf};

const SERVICE: &str = env!("CARGO_PKG_NAME");

/// Claims in a Google ID token. Used to extract the email address.
#[derive(Debug, Deserialize)]
struct IdTokenClaims {
    email: String,
}

/// Extracts the email address from a Google-provided ID token.
///
/// Returns `None` if the token is malformed or does not include `email`.
fn extract_email_from_id_token(id_token: &str) -> Option<String> {
    let parts: Vec<&str> = id_token.split('.').collect();
    if parts.len() != 3 {
        return None;
    }

    let payload = URL_SAFE_NO_PAD.decode(parts[1]).ok()?;
    let claims: IdTokenClaims = serde_json::from_slice(&payload).ok()?;
    Some(claims.email)
}

/// Loads a cached token from either a file or the system keyring.
///
/// - If `GCLOUD_IDENTITY_TOKEN_PATH` is set, the token will be loaded from the specified file.
/// - Otherwise, it attempts to read from the OS keyring using a fixed service name
///   and the default user identifier `"default"`.
///
/// # Returns
///
/// An optional `SavedToken` if the token was found and deserialized.
pub fn load_cached_token() -> Option<SavedToken> {
    if let Ok(env_path) = std::env::var("GCLOUD_IDENTITY_TOKEN_PATH") {
        let path = PathBuf::from(env_path);
        let data = fs::read_to_string(path).ok()?;
        return serde_json::from_str(&data).ok();
    }

    let user = fs::read_to_string(email_hint_path()).unwrap_or_else(|_| "default".to_string());
    let entry = Entry::new(SERVICE, &user).ok()?;
    let json = entry.get_password().ok()?;
    serde_json::from_str(&json).ok()
}

/// Saves a token to either a file or the system keyring.
///
/// - If `GCLOUD_IDENTITY_TOKEN_PATH` is set, the token will be saved to that file path.
/// - Otherwise, it saves to the keyring using the `email` field in the ID token as the user ID.
///   If the email cannot be extracted, it falls back to `"default"`.
///
/// # Errors
///
/// Returns an error if the token cannot be serialized or stored.
pub fn save_token(token: &SavedToken) -> Result<()> {
    if let Ok(env_path) = std::env::var("GCLOUD_IDENTITY_TOKEN_PATH") {
        let path = PathBuf::from(env_path);
        fs::create_dir_all(path.parent().unwrap())?;
        fs::write(path, serde_json::to_string_pretty(token)?)?;
        return Ok(());
    }

    let user =
        extract_email_from_id_token(&token.id_token).unwrap_or_else(|| "default".to_string());
    fs::write(email_hint_path(), &user)?;

    let json = serde_json::to_string(token)?;
    let entry = Entry::new(SERVICE, &user)?;
    entry.set_password(&json)?;
    Ok(())
}

/// Deletes a token from the system keyring.
///
/// Only applies if you're using the default `"default"` user ID. For more dynamic handling,
/// you'd want to extract the appropriate email-based user name from the current context.
pub fn delete_token() -> Result<()> {
    let user = fs::read_to_string(email_hint_path()).unwrap_or_else(|_| "default".to_string());
    let entry = Entry::new(SERVICE, &user)?;
    entry.delete_password()?;
    Ok(())
}

fn email_hint_path() -> PathBuf {
    dirs::home_dir()
        .expect("no home dir")
        .join(".cache")
        .join(format!("{}.email", env!("CARGO_PKG_NAME")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_save_with_file_cache() {
        unsafe {
            std::env::set_var("GCLOUD_IDENTITY_TOKEN_PATH", "/tmp/test_token.json");
        }

        let token = SavedToken {
            refresh_token: "r".into(),
            access_token: "a".into(),
            id_token: encode_dummy_id_token_with_email("test@example.com"),
            token_expiry: "2025-01-01T00:00:00Z".into(),
        };

        save_token(&token).unwrap();
        let loaded = load_cached_token().unwrap();
        assert_eq!(loaded.refresh_token, "r");
        assert_eq!(loaded.id_token, token.id_token);
    }

    /// Creates a fake but structurally valid JWT with an email field in the payload.
    fn encode_dummy_id_token_with_email(email: &str) -> String {
        let header = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(r#"{"alg":"none"}"#);
        let payload = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .encode(format!(r#"{{"email":"{}"}}"#, email));
        let signature = "";
        format!("{}.{}.{}", header, payload, signature)
    }
}
