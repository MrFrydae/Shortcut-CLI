use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Serialize};

/// A parsed STL template document.
#[derive(Debug, Deserialize)]
pub struct Template {
    pub version: u32,
    pub meta: Option<Meta>,
    pub vars: Option<HashMap<String, serde_yaml::Value>>,
    pub on_error: Option<ErrorHandling>,
    pub operations: Vec<Operation>,
}

/// Informational metadata (not sent to the API).
#[derive(Debug, Deserialize)]
pub struct Meta {
    pub description: Option<String>,
    pub author: Option<String>,
}

/// Error handling strategy.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorHandling {
    Continue,
}

/// A single operation in the template.
#[derive(Debug, Deserialize)]
pub struct Operation {
    pub action: Action,
    pub entity: Entity,
    pub alias: Option<String>,
    pub id: Option<serde_yaml::Value>,
    pub on_error: Option<ErrorHandling>,
    pub fields: Option<serde_yaml::Mapping>,
    pub repeat: Option<Vec<serde_yaml::Mapping>>,
}

/// Write-only action vocabulary.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Create,
    Update,
    Delete,
    Comment,
    Link,
    Unlink,
    Check,
    Uncheck,
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Action::Create => write!(f, "create"),
            Action::Update => write!(f, "update"),
            Action::Delete => write!(f, "delete"),
            Action::Comment => write!(f, "comment"),
            Action::Link => write!(f, "link"),
            Action::Unlink => write!(f, "unlink"),
            Action::Check => write!(f, "check"),
            Action::Uncheck => write!(f, "uncheck"),
        }
    }
}

/// Supported entity types.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Entity {
    Story,
    Epic,
    Iteration,
    Label,
    Objective,
    Milestone,
    Category,
    Group,
    Document,
    Project,
    Task,
    Comment,
    StoryLink,
}

impl fmt::Display for Entity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Entity::Story => write!(f, "story"),
            Entity::Epic => write!(f, "epic"),
            Entity::Iteration => write!(f, "iteration"),
            Entity::Label => write!(f, "label"),
            Entity::Objective => write!(f, "objective"),
            Entity::Milestone => write!(f, "milestone"),
            Entity::Category => write!(f, "category"),
            Entity::Group => write!(f, "group"),
            Entity::Document => write!(f, "document"),
            Entity::Project => write!(f, "project"),
            Entity::Task => write!(f, "task"),
            Entity::Comment => write!(f, "comment"),
            Entity::StoryLink => write!(f, "story_link"),
        }
    }
}

/// Result of executing a single operation.
#[derive(Debug, Clone, Serialize)]
pub struct OperationResult {
    pub index: usize,
    pub action: String,
    pub entity: String,
    pub status: String,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
}

/// Full execution result.
#[derive(Debug, Serialize)]
pub struct ExecutionResult {
    pub operations: Vec<OperationResult>,
    pub summary: ExecutionSummary,
}

#[derive(Debug, Serialize)]
pub struct ExecutionSummary {
    pub total: usize,
    pub succeeded: usize,
    pub failed: usize,
}
