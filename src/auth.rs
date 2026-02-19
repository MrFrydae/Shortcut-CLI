use std::fmt;

const SERVICE: &str = "sc-cli";
const USER: &str = "shortcut-api-token";

#[derive(Debug)]
pub enum AuthError {
    NotFound,
    Keyring(keyring::Error),
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuthError::NotFound => write!(f, "No API token found. Run `sc login` first."),
            AuthError::Keyring(e) => write!(f, "Keychain error: {e}"),
        }
    }
}

impl std::error::Error for AuthError {}

impl From<keyring::Error> for AuthError {
    fn from(e: keyring::Error) -> Self {
        match e {
            keyring::Error::NoEntry => AuthError::NotFound,
            other => AuthError::Keyring(other),
        }
    }
}

/// Trait abstracting token storage, allowing test implementations that
/// avoid the system keychain.
pub trait TokenStore {
    fn store_token(&self, token: &str) -> Result<(), AuthError>;
    fn get_token(&self) -> Result<String, AuthError>;
    fn delete_token(&self) -> Result<(), AuthError>;
}

/// Production implementation backed by the OS keychain.
pub struct KeychainStore;

impl TokenStore for KeychainStore {
    fn store_token(&self, token: &str) -> Result<(), AuthError> {
        let entry = keyring::Entry::new(SERVICE, USER)?;
        entry.set_password(token)?;
        Ok(())
    }

    fn get_token(&self) -> Result<String, AuthError> {
        let entry = keyring::Entry::new(SERVICE, USER)?;
        Ok(entry.get_password()?)
    }

    fn delete_token(&self) -> Result<(), AuthError> {
        let entry = keyring::Entry::new(SERVICE, USER)?;
        entry.delete_credential()?;
        Ok(())
    }
}

// Convenience free functions that delegate to KeychainStore.

pub fn store_token(token: &str) -> Result<(), AuthError> {
    KeychainStore.store_token(token)
}

#[allow(dead_code)]
pub fn get_token() -> Result<String, AuthError> {
    KeychainStore.get_token()
}

#[allow(dead_code)]
pub fn delete_token() -> Result<(), AuthError> {
    KeychainStore.delete_token()
}
