use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fmt, fs, io};

const DIR_NAME: &str = ".shortcut";

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
                "No project registered for this directory. Run `shortcut init` first."
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
    shortcut_dir: PathBuf,
}

impl ProjectRoot {
    pub fn token_path(&self) -> PathBuf {
        self.shortcut_dir.join("token")
    }

    pub fn cache_dir(&self) -> PathBuf {
        self.shortcut_dir.join("cache")
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

fn git_output(path: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(path)
        .args(args)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_COMMON_DIR")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8(output.stdout).ok()
}

/// Return the current and main checkout roots when `path` is in a linked worktree.
fn linked_worktree_roots(path: &Path) -> Option<(PathBuf, PathBuf)> {
    let current = PathBuf::from(
        git_output(path, &["rev-parse", "--show-toplevel"])?.trim_end_matches(['\n', '\r']),
    )
    .canonicalize()
    .ok()?;
    let listing = git_output(path, &["worktree", "list", "--porcelain"])?;
    let mut lines = listing.lines();
    let main = lines.next()?.strip_prefix("worktree ")?;
    if lines.next()? == "bare" {
        return None;
    }
    let main = PathBuf::from(main).canonicalize().ok()?;
    if current == main || !path.starts_with(&current) {
        return None;
    }
    Some((current, main))
}

fn registered_ancestor(
    projects_base: &Path,
    start: &Path,
    stop_at: Option<&Path>,
) -> Option<ProjectRoot> {
    for dir in start.ancestors() {
        let hash = fnv1a_hex(dir.as_os_str().as_encoded_bytes());
        let shortcut_dir = projects_base.join(hash);
        if shortcut_dir.is_dir() {
            return Some(ProjectRoot { shortcut_dir });
        }
        if Some(dir) == stop_at {
            break;
        }
    }
    None
}

/// Locate the project directory under `~/.shortcut/projects/<hash>/` for the current working directory.
pub fn discover() -> Result<ProjectRoot, ProjectError> {
    discover_in(&home_dir()?, &cwd()?)
}

/// Create `~/.shortcut/projects/<hash>/` directory structure for the current working directory.
pub fn init() -> Result<(ProjectRoot, PathBuf), ProjectError> {
    init_in(&home_dir()?, &cwd()?)
}

/// Locate a project directory inside the given `home` for `project_path`.
/// Walks up from `project_path` through its ancestors until a registered project is found.
pub fn discover_in(home: &Path, project_path: &Path) -> Result<ProjectRoot, ProjectError> {
    let canonical = project_path.canonicalize()?;
    let projects_base = home.join(DIR_NAME).join("projects");

    if let Some((worktree, main)) = linked_worktree_roots(&canonical) {
        if let Some(root) = registered_ancestor(&projects_base, &canonical, Some(&worktree)) {
            return Ok(root);
        }
        if let Some(root) = registered_ancestor(&projects_base, &main, None) {
            return Ok(root);
        }
        if let Some(parent) = worktree.parent()
            && let Some(root) = registered_ancestor(&projects_base, parent, None)
        {
            return Ok(root);
        }
    } else if let Some(root) = registered_ancestor(&projects_base, &canonical, None) {
        return Ok(root);
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
    let canonical = project_path.canonicalize()?;
    let project_root = linked_worktree_roots(&canonical)
        .map(|(_, main)| main)
        .unwrap_or(canonical);
    let shortcut_dir = project_dir(home, &project_root)?;
    if shortcut_dir.exists() {
        return Err(ProjectError::AlreadyExists(shortcut_dir));
    }

    let cache_dir = shortcut_dir.join("cache");
    fs::create_dir_all(&cache_dir)?;

    Ok((ProjectRoot { shortcut_dir }, project_root))
}
