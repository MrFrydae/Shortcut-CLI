#[allow(clippy::all, unused)]
mod inner {
    include!(concat!(env!("OUT_DIR"), "/shortcut_api.rs"));
}

#[allow(unused_imports)]
pub use inner::*;

pub const BASE_URL: &str = "https://api.app.shortcut.com";

/// Build an authenticated Progenitor client using the stored API token.
#[allow(dead_code)]
pub fn authenticated_client() -> Result<Client, crate::auth::AuthError> {
    let token = crate::auth::get_token()?;

    let http = reqwest::Client::builder()
        .default_headers({
            let mut headers = reqwest::header::HeaderMap::new();
            headers.insert(
                "Shortcut-Token",
                reqwest::header::HeaderValue::from_str(&token)
                    .expect("stored token contains invalid header characters"),
            );
            headers
        })
        .build()
        .expect("failed to build HTTP client");

    Ok(Client::new_with_client(BASE_URL, http))
}
