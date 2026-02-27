#[allow(clippy::all, unused, rustdoc::bare_urls)]
mod inner {
    include!(concat!(env!("OUT_DIR"), "/shortcut_api.rs"));
}

#[allow(unused_imports)]
pub use inner::*;

pub const BASE_URL: &str = "https://api.app.shortcut.com";
pub const SHORTCUT_API_TOKEN_ENV: &str = "SHORTCUT_API_TOKEN";

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
    let token = select_auth_token(read_token_from_env(), token_store)?;

    client_with_token(&token, BASE_URL).map_err(|e| {
        crate::auth::AuthError::Io(std::io::Error::other(format!(
            "failed to build HTTP client: {e}"
        )))
    })
}

fn read_token_from_env() -> Option<String> {
    std::env::var(SHORTCUT_API_TOKEN_ENV)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

#[doc(hidden)]
pub fn select_auth_token(
    env_token: Option<String>,
    token_store: &dyn crate::auth::TokenStore,
) -> Result<String, crate::auth::AuthError> {
    env_token.map(Ok).unwrap_or_else(|| token_store.get_token())
}

/// Format a Progenitor API error, extracting the server's `message` field
/// from error responses instead of showing raw debug output.
///
/// Produces output like: `422 Unprocessable Entity: Name is required`
pub fn format_api_error(err: &progenitor_client::Error<types::ApiError>) -> String {
    match err {
        progenitor_client::Error::ErrorResponse(rv) => {
            format!("{}: {}", rv.status(), rv.message)
        }
        other => other.to_string(),
    }
}
