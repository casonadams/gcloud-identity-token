use crate::shared::get_or_init_port;
use anyhow::Result;
use std::collections::HashMap;
use tiny_http::{Response, Server};
use url::Url;

pub fn is_headless_env() -> bool {
    std::env::var("DISPLAY").is_err() && std::env::var("WAYLAND_DISPLAY").is_err()
}

pub fn build_auth_url(client_id: &str, redirect_uri: &str) -> Url {
    let mut url = Url::parse("https://accounts.google.com/o/oauth2/v2/auth").unwrap();
    url.query_pairs_mut()
        .append_pair("client_id", client_id)
        .append_pair("response_type", "code")
        .append_pair("scope", "openid email")
        .append_pair("redirect_uri", redirect_uri)
        .append_pair("access_type", "offline")
        .append_pair("include_granted_scopes", "true")
        .append_pair("prompt", "consent");
    url
}

pub fn open_browser_or_print(url: &Url) {
    if is_headless_env() {
        println!("\nOpen this URL in your browser:\n\n{}\n", url);
    } else {
        open::that(url.as_str()).unwrap_or_else(|_| {
            println!(
                "\nCouldn't open browser. Please open this URL manually:\n\n{}\n",
                url
            );
        });
    }
}

pub fn capture_auth_code() -> Result<String> {
    let port = get_or_init_port();
    let server = Server::http(("127.0.0.1", port)).unwrap();
    let request = server.recv()?;
    let query = request.url().split('?').nth(1).unwrap_or("");
    let params: HashMap<_, _> = url::form_urlencoded::parse(query.as_bytes())
        .into_owned()
        .collect();
    let code = params
        .get("code")
        .ok_or("Missing ?code= param")
        .map_err(|_| anyhow::anyhow!("Error capturing auth code"))?
        .clone();
    request.respond(Response::from_string(
        "You may now return to the application.",
    ))?;
    Ok(code)
}
