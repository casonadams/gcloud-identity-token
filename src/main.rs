use anyhow::Result;
use gcloud_identity_token::{auth::get_token, config::load_creds};

#[tokio::main]
async fn main() -> Result<()> {
    let creds = load_creds()?;
    let token = get_token(&creds).await?;
    println!("{}", serde_json::to_string_pretty(&token)?);
    Ok(())
}
