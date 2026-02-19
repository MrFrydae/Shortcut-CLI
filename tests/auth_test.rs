mod support;

use sc::auth::TokenStore;
use support::MockTokenStore;

#[test]
fn mock_store_roundtrip() {
    let store = MockTokenStore::new();
    assert!(store.get_token().is_err());

    store.store_token("tok_abc").unwrap();
    assert_eq!(store.get_token().unwrap(), "tok_abc");

    store.delete_token().unwrap();
    assert!(store.get_token().is_err());
}

#[test]
fn mock_store_overwrites() {
    let store = MockTokenStore::new();
    store.store_token("first").unwrap();
    store.store_token("second").unwrap();
    assert_eq!(store.get_token().unwrap(), "second");
}
