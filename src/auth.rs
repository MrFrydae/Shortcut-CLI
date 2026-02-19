use std::path::PathBuf;
use std::{fmt, fs, io};

#[derive(Debug)]
pub enum AuthError {
    NotFound,
    Io(io::Error),
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuthError::NotFound => write!(f, "No API token found. Run `sc login` first."),
            AuthError::Io(e) => write!(f, "Token storage error: {e}"),
        }
    }
}

impl std::error::Error for AuthError {}

impl From<io::Error> for AuthError {
    fn from(e: io::Error) -> Self {
        AuthError::Io(e)
    }
}

/// Trait abstracting token storage, allowing test implementations that
/// avoid the filesystem.
pub trait TokenStore {
    fn store_token(&self, token: &str) -> Result<(), AuthError>;
    fn get_token(&self) -> Result<String, AuthError>;
    fn delete_token(&self) -> Result<(), AuthError>;
}

/// File-backed token store. The token is stored as plain text at `path`.
pub struct FileTokenStore {
    pub path: PathBuf,
}

impl TokenStore for FileTokenStore {
    fn store_token(&self, token: &str) -> Result<(), AuthError> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&self.path, token)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&self.path, fs::Permissions::from_mode(0o600))?;
        }

        Ok(())
    }

    fn get_token(&self) -> Result<String, AuthError> {
        let data = match fs::read_to_string(&self.path) {
            Ok(s) => s,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Err(AuthError::NotFound),
            Err(e) => return Err(AuthError::Io(e)),
        };
        let trimmed = data.trim().to_string();
        if trimmed.is_empty() {
            return Err(AuthError::NotFound);
        }
        Ok(trimmed)
    }

    fn delete_token(&self) -> Result<(), AuthError> {
        match fs::remove_file(&self.path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(AuthError::Io(e)),
        }
    }
}
