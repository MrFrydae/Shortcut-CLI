use shortcut_cli::api;
use shortcut_cli::auth::{AuthError, TokenStore};
use std::cell::RefCell;

struct TestTokenStore {
    token: RefCell<Option<String>>,
}

impl TestTokenStore {
    fn with_token(token: Option<&str>) -> Self {
        Self {
            token: RefCell::new(token.map(str::to_string)),
        }
    }
}

impl TokenStore for TestTokenStore {
    fn store_token(&self, token: &str) -> Result<(), AuthError> {
        self.token.replace(Some(token.to_string()));
        Ok(())
    }

    fn get_token(&self) -> Result<String, AuthError> {
        self.token.borrow().clone().ok_or(AuthError::NotFound)
    }

    fn delete_token(&self) -> Result<(), AuthError> {
        self.token.replace(None);
        Ok(())
    }
}

#[test]
fn client_with_token_builds_successfully() {
    let client = api::client_with_token("test-token", "http://localhost:1234");
    assert!(client.is_ok());
}

#[test]
fn format_api_error_extracts_message() {
    let rv = progenitor_client::ResponseValue::new(
        api::types::ApiError {
            message: "Name is required".to_string(),
        },
        reqwest::StatusCode::UNPROCESSABLE_ENTITY,
        reqwest::header::HeaderMap::new(),
    );
    let err = progenitor_client::Error::ErrorResponse(rv);
    assert_eq!(
        api::format_api_error(&err),
        "422 Unprocessable Entity: Name is required"
    );
}

#[test]
fn format_api_error_falls_through_for_other_variants() {
    let err: progenitor_client::Error<api::types::ApiError> =
        progenitor_client::Error::InvalidRequest("bad request".to_string());
    let formatted = api::format_api_error(&err);
    assert!(
        formatted.contains("bad request"),
        "expected 'bad request' in: {formatted}"
    );
}

#[test]
fn select_auth_token_prefers_env_token() {
    let store = TestTokenStore::with_token(Some("stored-token"));
    let token = api::select_auth_token(Some("env-token".to_string()), &store)
        .expect("token should resolve");
    assert_eq!(token, "env-token");
}

#[test]
fn select_auth_token_falls_back_to_store() {
    let store = TestTokenStore::with_token(Some("stored-token"));
    let token = api::select_auth_token(None, &store).expect("token should resolve");
    assert_eq!(token, "stored-token");
}
