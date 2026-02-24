use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Persisted state for `sc template sync`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncState {
    pub version: u32,
    pub created_at: String,
    pub updated_at: String,
    pub resources: HashMap<String, ResourceState>,
    #[serde(default)]
    pub applied: Vec<String>,
}

impl Default for SyncState {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncState {
    pub fn new() -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            version: 1,
            created_at: now.clone(),
            updated_at: now,
            resources: HashMap::new(),
            applied: Vec::new(),
        }
    }

    pub fn touch(&mut self) {
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }
}

/// State for a single aliased resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ResourceState {
    /// A single create operation (no repeat).
    #[serde(rename = "single")]
    Single {
        entity: String,
        id: serde_json::Value,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        tasks: Option<HashMap<String, TaskEntry>>,
    },
    /// A repeat create operation keyed by entry `key`.
    #[serde(rename = "repeat")]
    Repeat {
        entity: String,
        entries: HashMap<String, EntryState>,
    },
}

impl ResourceState {
    pub fn entity(&self) -> &str {
        match self {
            ResourceState::Single { entity, .. } => entity,
            ResourceState::Repeat { entity, .. } => entity,
        }
    }
}

/// State for a single repeat entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryState {
    pub id: serde_json::Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tasks: Option<HashMap<String, TaskEntry>>,
}

/// State for an inline task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskEntry {
    pub id: serde_json::Value,
}

/// Load state from disk. Returns `None` if the file doesn't exist.
pub fn load_state(path: &Path) -> Result<Option<SyncState>, Box<dyn std::error::Error>> {
    if !path.exists() {
        return Ok(None);
    }
    let contents = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read state file '{}': {e}", path.display()))?;
    let state: SyncState = serde_json::from_str(&contents)
        .map_err(|e| format!("Failed to parse state file '{}': {e}", path.display()))?;
    if state.version != 1 {
        return Err(format!(
            "Unsupported state file version {} in '{}'",
            state.version,
            path.display()
        )
        .into());
    }
    Ok(Some(state))
}

/// Save state to disk atomically (write to .tmp, then rename).
pub fn save_state(state: &SyncState, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let tmp_path = path.with_extension("state.json.tmp");
    let contents = serde_json::to_string_pretty(state)?;
    std::fs::write(&tmp_path, &contents)
        .map_err(|e| format!("Failed to write state file '{}': {e}", tmp_path.display()))?;
    std::fs::rename(&tmp_path, path)
        .map_err(|e| format!("Failed to rename state file '{}': {e}", path.display()))?;
    Ok(())
}

/// Compute the default state file path for a template file.
pub fn default_state_path(template_path: &str) -> PathBuf {
    PathBuf::from(format!("{template_path}.state.json"))
}
