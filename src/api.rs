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

/// Build an authenticated Progenitor client using the stored API token.
#[allow(dead_code)]
pub fn authenticated_client() -> Result<Client, crate::auth::AuthError> {
    let token = crate::auth::get_token()?;
    client_with_token(&token, BASE_URL).map_err(|_| {
        crate::auth::AuthError::Keyring(keyring::Error::PlatformFailure(
            "failed to build HTTP client".into(),
        ))
    })
}
