# gcloud-identity-token

A Rust crate for seamless, secure Google Cloud OAuth authentication.

This library handles the OAuth2 authorization code flow (with browser-based login)
to obtain:

- **Access tokens** (for calling Google APIs)
- **ID tokens** (for verifying user identity)
- **Refresh tokens** (to renew tokens silently)

It securely caches credentials using the OS-native keyring or a file-based
fallback — making it ideal for long-lived CLI tools, automation, and server
integrations.

---

## Features

- **Secure credential caching**
  - Defaults to OS keyring (`keyring` crate)
  - Optional file-based cache via `GCLOUD_IDENTITY_TOKEN_PATH`
- **Smart refresh logic**
  - Automatically reuses tokens until they expire
  - Refreshes silently using stored refresh token
- **Headless & browser login support**
  - Opens browser for login when possible
  - Falls back to manual URL copy if needed
- **Email-based keyring separation**
  - Keyring entries are scoped to your Google email (from ID token)

---

## Usage

Obtain application-default credentials (Required)

```sh
gcloud auth application-default login
```

Add to your `Cargo.toml`:

```toml
[dependencies]
gcloud-identity-token = "0.1"
```

---

## Example

```rs
use anyhow::Result;
use gcloud_identity_token::auth::get_token;
use gcloud_identity_token::config::load_creds;

#[tokio::main]
async fn main() -> Result<()> {
    let creds = load_creds()?;
    let token = get_token(&creds).await?;

    println!("Access token: {}", token.access_token);
    println!("ID token: {}", token.id_token);
    println!("Expires at:  {}", token.token_expiry);

    Ok(())
}
```
