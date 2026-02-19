use sc::auth::{FileTokenStore, TokenStore};

#[test]
fn roundtrip_store_and_get() {
    let tmp = tempfile::tempdir().unwrap();
    let store = FileTokenStore {
        path: tmp.path().join("token"),
    };

    assert!(store.get_token().is_err());

    store.store_token("tok_abc").unwrap();
    assert_eq!(store.get_token().unwrap(), "tok_abc");

    store.delete_token().unwrap();
    assert!(store.get_token().is_err());
}

#[test]
fn overwrite_token() {
    let tmp = tempfile::tempdir().unwrap();
    let store = FileTokenStore {
        path: tmp.path().join("token"),
    };

    store.store_token("first").unwrap();
    store.store_token("second").unwrap();
    assert_eq!(store.get_token().unwrap(), "second");
}

#[test]
fn trims_whitespace() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("token");
    std::fs::write(&path, "  tok_spaces  \n").unwrap();

    let store = FileTokenStore { path };
    assert_eq!(store.get_token().unwrap(), "tok_spaces");
}

#[test]
fn empty_file_returns_not_found() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("token");
    std::fs::write(&path, "  \n").unwrap();

    let store = FileTokenStore { path };
    assert!(store.get_token().is_err());
}

#[test]
fn delete_nonexistent_is_ok() {
    let tmp = tempfile::tempdir().unwrap();
    let store = FileTokenStore {
        path: tmp.path().join("token"),
    };
    assert!(store.delete_token().is_ok());
}

#[cfg(unix)]
#[test]
fn sets_unix_permissions() {
    use std::os::unix::fs::PermissionsExt;

    let tmp = tempfile::tempdir().unwrap();
    let store = FileTokenStore {
        path: tmp.path().join("token"),
    };

    store.store_token("secret").unwrap();
    let meta = std::fs::metadata(&store.path).unwrap();
    let mode = meta.permissions().mode() & 0o777;
    assert_eq!(mode, 0o600, "expected 0600, got {mode:o}");
}
