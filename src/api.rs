#[allow(clippy::all, unused, rustdoc::bare_urls)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_api_error_extracts_message() {
        let rv = progenitor_client::ResponseValue::new(
            types::ApiError {
                message: "Name is required".to_string(),
            },
            reqwest::StatusCode::UNPROCESSABLE_ENTITY,
            reqwest::header::HeaderMap::new(),
        );
        let err = progenitor_client::Error::ErrorResponse(rv);
        assert_eq!(
            format_api_error(&err),
            "422 Unprocessable Entity: Name is required"
        );
    }

    #[test]
    fn format_api_error_falls_through_for_other_variants() {
        let err: progenitor_client::Error<types::ApiError> =
            progenitor_client::Error::InvalidRequest("bad request".to_string());
        let formatted = format_api_error(&err);
        assert!(
            formatted.contains("bad request"),
            "expected 'bad request' in: {formatted}"
        );
    }
}
