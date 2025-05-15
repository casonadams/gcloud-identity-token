//! # gcloud-identity-token
//!
//! A Rust library for obtaining and caching Google OAuth2 tokens for use with
//! Google Cloud APIs.
//!
//! This crate handles the entire OAuth 2.0 Authorization Code flow:
//! - Launching a browser to log in
//! - Receiving and exchanging authorization codes
//! - Saving access, refresh, and ID tokens
//! - Securely caching them using the system keyring or a local file
//!
//! ## Features
//! - Secure keyring-backed token storage (per Google user)
//! - File-based cache via `GCLOUD_IDENTITY_TOKEN_PATH` override
//! - Fully async support with `reqwest` + `tokio`
//! - Intelligent refresh with expiry tracking
//!
//! ## Example
//!
//! ```rust,no_run
//! use anyhow::Result;
//! use gcloud_identity_token::{auth::get_token, config::load_creds};
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let creds = load_creds()?;
//!     let token = get_token(&creds).await?;
//!     println!("Access Token: {}", token.access_token);
//!     println!("ID Token: {}", token.id_token);
//!     Ok(())
//! }
//! ```
//!
//! ## Environment Variables
//! - `GCLOUD_IDENTITY_TOKEN_PATH` — path to file-based token cache
//! - `DISPLAY` / `WAYLAND_DISPLAY` — if unset, triggers headless login flow
//!
//! ## Modules

/// Authorization flow and token refresh logic.
pub mod auth;

/// Browser launching and redirect capture logic.
pub mod browser;

/// Token cache handling (keyring and file-based).
pub mod cache;

/// Configuration structures and token types.
pub mod config;

/// Shared utilities like port picking.
pub mod shared;
