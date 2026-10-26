use crate::value_objects::file_path::SanitizedPath;
use async_trait::async_trait;
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FileSystemError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("File not found: {0}")]
    NotFound(String),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
}

#[async_trait]
pub trait FileSystemPort: Send + Sync {
    async fn discover_files(
        &self,
        root: &Path,
        max_files: usize,
    ) -> Result<Vec<SanitizedPath>, FileSystemError>;
    async fn read_file_string(&self, path: &Path) -> Result<String, FileSystemError>;
    async fn read_file_lines(
        &self,
        path: &Path,
        start_line: u32,
        end_line: u32,
    ) -> Result<String, FileSystemError>;
    async fn file_exists(&self, path: &Path) -> bool;
}
