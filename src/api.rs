#[allow(clippy::all, unused)]
mod inner {
    include!(concat!(env!("OUT_DIR"), "/shortcut_api.rs"));
}

#[allow(unused_imports)]
pub use inner::*;

pub const BASE_URL: &str = "https://api.app.shortcut.com";

/// Build a Progenitor client pointed at `base_url` using the given API token.
pub fn client_with_token(
    token: &str,
    base_url: &str,
) -> Result<Client, Box<dyn std::error::Error>> {
    let http = reqwest::Client::builder()
        .default_headers({
            let mut headers = reqwest::header::HeaderMap::new();
            headers.insert(
                "Shortcut-Token",
                reqwest::header::HeaderValue::from_str(token)?,
            );
            headers
        })
        .build()?;

    Ok(Client::new_with_client(base_url, http))
}

/// Build an authenticated Progenitor client using the given token store.
pub fn authenticated_client(
    token_store: &dyn crate::auth::TokenStore,
) -> Result<Client, crate::auth::AuthError> {
    let token = token_store.get_token()?;
    client_with_token(&token, BASE_URL).map_err(|e| {
        crate::auth::AuthError::Io(std::io::Error::other(format!(
            "failed to build HTTP client: {e}"
        )))
    })
}
