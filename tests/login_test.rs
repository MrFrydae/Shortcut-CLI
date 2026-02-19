mod support;

use sc::{auth::TokenStore, commands::login};
use support::{MockTokenStore, member_info_json};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn make_args(token: Option<&str>) -> login::LoginArgs {
    login::LoginArgs {
        token: token.map(String::from),
    }
}

#[tokio::test]
async fn login_with_token_arg_stores_on_success() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/member"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(&member_info_json("Test User", "testuser")),
        )
        .expect(1)
        .mount(&server)
        .await;

    let store = MockTokenStore::new();
    let args = make_args(Some("tok_valid"));
    let result = login::run(&args, &server.uri(), &store, || {
        panic!("prompt should not be called")
    })
    .await;

    assert!(result.is_ok());
    assert_eq!(store.get_token().unwrap(), "tok_valid");
}

#[tokio::test]
async fn login_without_token_calls_prompt() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/member"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(&member_info_json("Prompted User", "prompted")),
        )
        .expect(1)
        .mount(&server)
        .await;

    let store = MockTokenStore::new();
    let args = make_args(None);
    let result = login::run(&args, &server.uri(), &store, || Ok("tok_prompted".into())).await;

    assert!(result.is_ok());
    assert_eq!(store.get_token().unwrap(), "tok_prompted");
}

#[tokio::test]
async fn login_api_failure_does_not_store() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/member"))
        .respond_with(ResponseTemplate::new(400))
        .expect(1)
        .mount(&server)
        .await;

    let store = MockTokenStore::new();
    let args = make_args(Some("tok_bad"));
    let result = login::run(&args, &server.uri(), &store, || {
        panic!("prompt should not be called")
    })
    .await;

    assert!(result.is_err());
    assert!(store.get_token().is_err());
}

#[tokio::test]
async fn login_prompt_error_propagates() {
    let server = MockServer::start().await;
    // No mock needed — we never reach the HTTP call.

    let store = MockTokenStore::new();
    let args = make_args(None);
    let result = login::run(&args, &server.uri(), &store, || {
        Err("terminal not available".into())
    })
    .await;

    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("terminal not available")
    );
}
