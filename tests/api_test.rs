use sc::api;

#[test]
fn client_with_token_builds_successfully() {
    let client = api::client_with_token("test-token", "http://localhost:1234");
    assert!(client.is_ok());
}
