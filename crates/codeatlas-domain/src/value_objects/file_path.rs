use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum PathValidationError {
    #[error("Empty path provided")]
    EmptyPath,
    #[error("Path traversal detected: {0}")]
    TraversalAttempt(String),
    #[error("Path exceeds maximum length of 4096 characters")]
    PathTooLong,
    #[error("Target path does not exist: {0}")]
    NotFound(String),
}

/// A validated, canonicalized path safely contained within a root
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SanitizedPath(PathBuf);

impl SanitizedPath {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, PathValidationError> {
        let pb = path.into();
        let path_str = pb.to_string_lossy();
        if path_str.trim().is_empty() {
            return Err(PathValidationError::EmptyPath);
        }
        if path_str.len() > 4096 {
            return Err(PathValidationError::PathTooLong);
        }
        // Reject path traversal patterns
        for component in pb.components() {
            if component == std::path::Component::ParentDir {
                return Err(PathValidationError::TraversalAttempt(path_str.to_string()));
            }
        }
        Ok(Self(pb))
    }

    pub fn within_root(target: &Path, root: &Path) -> Result<Self, PathValidationError> {
        let canonical_root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
        let full_target = if target.is_absolute() || target.starts_with(root) {
            target.to_path_buf()
        } else {
            root.join(target)
        };

        let canonical_target = full_target.canonicalize().unwrap_or(full_target);

        if !canonical_target.starts_with(&canonical_root) {
            return Err(PathValidationError::TraversalAttempt(
                canonical_target.to_string_lossy().to_string(),
            ));
        }

        Ok(Self(canonical_target))
    }

    #[inline]
    pub fn as_path(&self) -> &Path {
        &self.0
    }

    #[inline]
    pub fn into_path_buf(self) -> PathBuf {
        self.0
    }
}

impl std::fmt::Display for SanitizedPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.display())
    }
}
