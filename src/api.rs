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

/// Maximum number of files per `POST /api/v3/files` request.
///
/// The Shortcut API documents the multipart fields `file0` through `file3`,
/// so uploads with more files are split into batches of this size.
pub const MAX_FILES_PER_UPLOAD: usize = 4;

/// One file to send in a multipart upload request.
#[derive(Debug, Clone)]
pub struct UploadPart {
    /// File name sent in the part's `Content-Disposition` header.
    pub filename: String,
    /// MIME type sent as the part's `Content-Type`.
    pub content_type: String,
    /// Raw file contents.
    pub bytes: Vec<u8>,
}

/// Upload up to [`MAX_FILES_PER_UPLOAD`] files in a single `POST /api/v3/files`
/// request, optionally attaching them to a story.
///
/// Progenitor cannot generate multipart endpoints, so this request is built by
/// hand on the generated client's `reqwest::Client` (which already carries the
/// `Shortcut-Token` header) and base URL.
///
/// Returns the raw `UploadedFile` JSON objects so new or missing fields in the
/// API response never break an upload that already succeeded server-side.
pub async fn upload_files(
    client: &Client,
    story_id: Option<i64>,
    parts: Vec<UploadPart>,
) -> Result<Vec<serde_json::Value>, String> {
    if parts.is_empty() {
        return Err("at least one file is required".to_string());
    }
    if parts.len() > MAX_FILES_PER_UPLOAD {
        return Err(format!(
            "at most {MAX_FILES_PER_UPLOAD} files can be uploaded per request"
        ));
    }

    let mut form = reqwest::multipart::Form::new();
    if let Some(id) = story_id {
        form = form.text("story_id", id.to_string());
    }
    for (i, part) in parts.into_iter().enumerate() {
        let p = reqwest::multipart::Part::bytes(part.bytes)
            .file_name(part.filename)
            .mime_str(&part.content_type)
            .map_err(|e| format!("invalid content type: {e}"))?;
        form = form.part(format!("file{i}"), p);
    }

    let url = format!("{}/api/v3/files", ClientInfo::baseurl(client));
    let resp = ClientInfo::client(client)
        .post(url)
        .multipart(form)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let status = resp.status();
    let body = resp.text().await.map_err(|e| e.to_string())?;
    if !status.is_success() {
        return Err(format_http_error(status, &body));
    }

    serde_json::from_str::<Vec<serde_json::Value>>(&body)
        .map_err(|e| format!("unexpected response from Shortcut: {e}"))
}

/// Format a non-success HTTP response from a hand-written request the same way
/// [`format_api_error`] formats Progenitor errors: `<status>: <message>`.
///
/// Falls back to the raw body (or nothing) when it has no `message` field.
pub fn format_http_error(status: reqwest::StatusCode, body: &str) -> String {
    let message = serde_json::from_str::<serde_json::Value>(body)
        .ok()
        .and_then(|v| v.get("message").and_then(|m| m.as_str()).map(String::from))
        .unwrap_or_else(|| body.trim().to_string());
    if message.is_empty() {
        status.to_string()
    } else {
        format!("{status}: {message}")
    }
}
