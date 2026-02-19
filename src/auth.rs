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

pub fn store_token(token: &str) -> Result<(), AuthError> {
    let entry = keyring::Entry::new(SERVICE, USER)?;
    entry.set_password(token)?;
    Ok(())
}

#[allow(dead_code)]
pub fn get_token() -> Result<String, AuthError> {
    let entry = keyring::Entry::new(SERVICE, USER)?;
    Ok(entry.get_password()?)
}

#[allow(dead_code)]
pub fn delete_token() -> Result<(), AuthError> {
    let entry = keyring::Entry::new(SERVICE, USER)?;
    entry.delete_credential()?;
    Ok(())
}
