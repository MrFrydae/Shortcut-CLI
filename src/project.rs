use std::path::{Path, PathBuf};
use std::{env, fmt, fs, io};

const DIR_NAME: &str = ".sc";

#[derive(Debug)]
pub enum ProjectError {
    NotFound,
    AlreadyExists(PathBuf),
    Io(io::Error),
}

impl fmt::Display for ProjectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProjectError::NotFound => write!(
                f,
                "No project registered for this directory. Run `sc init` first."
            ),
            ProjectError::AlreadyExists(p) => {
                write!(f, "Project already initialized at {}", p.display())
            }
            ProjectError::Io(e) => write!(f, "I/O error: {e}"),
        }
    }
}

impl std::error::Error for ProjectError {}

impl From<io::Error> for ProjectError {
    fn from(e: io::Error) -> Self {
        ProjectError::Io(e)
    }
}

#[derive(Debug, Clone)]
pub struct ProjectRoot {
    sc_dir: PathBuf,
}

impl ProjectRoot {
    pub fn token_path(&self) -> PathBuf {
        self.sc_dir.join("token")
    }

    pub fn cache_dir(&self) -> PathBuf {
        self.sc_dir.join("cache")
    }
}

fn fnv1a_hex(data: &[u8]) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for &byte in data {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x00000100000001B3);
    }
    format!("{hash:016x}")
}

fn project_dir(home: &Path, project_path: &Path) -> Result<PathBuf, ProjectError> {
    let canonical = project_path.canonicalize()?;
    let hash = fnv1a_hex(canonical.as_os_str().as_encoded_bytes());
    Ok(home.join(DIR_NAME).join("projects").join(hash))
}

fn home_dir() -> Result<PathBuf, ProjectError> {
    env::var("HOME")
        .map(PathBuf::from)
        .map_err(|_| ProjectError::Io(io::Error::new(io::ErrorKind::NotFound, "HOME not set")))
}

fn cwd() -> Result<PathBuf, ProjectError> {
    env::current_dir().map_err(ProjectError::Io)
}

/// Locate the project directory under `~/.sc/projects/<hash>/` for the current working directory.
pub fn discover() -> Result<ProjectRoot, ProjectError> {
    discover_in(&home_dir()?, &cwd()?)
}

/// Create `~/.sc/projects/<hash>/` directory structure for the current working directory.
pub fn init() -> Result<(ProjectRoot, PathBuf), ProjectError> {
    init_in(&home_dir()?, &cwd()?)
}

/// Locate a project directory inside the given `home` for `project_path`.
/// Walks up from `project_path` through its ancestors until a registered project is found.
pub fn discover_in(home: &Path, project_path: &Path) -> Result<ProjectRoot, ProjectError> {
    let canonical = project_path.canonicalize()?;
    let projects_base = home.join(DIR_NAME).join("projects");

    let mut current = Some(canonical.as_path());
    while let Some(dir) = current {
        let hash = fnv1a_hex(dir.as_os_str().as_encoded_bytes());
        let sc_dir = projects_base.join(hash);
        if sc_dir.is_dir() {
            return Ok(ProjectRoot { sc_dir });
        }
        current = dir.parent();
    }

    Err(ProjectError::NotFound)
}

/// Try ancestor-walking discovery first; if no project is found, init for CWD.
pub fn discover_or_init() -> Result<ProjectRoot, ProjectError> {
    discover_or_init_in(&home_dir()?, &cwd()?)
}

/// Try ancestor-walking discovery first; if no project is found, init for `project_path`.
pub fn discover_or_init_in(home: &Path, project_path: &Path) -> Result<ProjectRoot, ProjectError> {
    match discover_in(home, project_path) {
        Ok(root) => Ok(root),
        Err(ProjectError::NotFound) => {
            let (root, _) = init_in(home, project_path)?;
            Ok(root)
        }
        Err(e) => Err(e),
    }
}

/// Create project directory structure inside the given `home` for `project_path`. Useful for testing.
pub fn init_in(home: &Path, project_path: &Path) -> Result<(ProjectRoot, PathBuf), ProjectError> {
    let sc_dir = project_dir(home, project_path)?;
    if sc_dir.exists() {
        return Err(ProjectError::AlreadyExists(sc_dir));
    }

    let cache_dir = sc_dir.join("cache");
    fs::create_dir_all(&cache_dir)?;

    let canonical = project_path.canonicalize()?;
    Ok((ProjectRoot { sc_dir }, canonical))
}
