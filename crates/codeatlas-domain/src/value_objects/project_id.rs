use serde::{Deserialize, Serialize};
use std::path::Path;

/// Strongly-typed Project Identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProjectId(String);

impl ProjectId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn from_path(path: &Path) -> Self {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unnamed_project");
        Self(name.to_string())
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ProjectId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for ProjectId {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for ProjectId {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}
