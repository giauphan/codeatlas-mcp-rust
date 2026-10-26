use async_trait::async_trait;
use codeatlas_domain::ports::file_system::{FileSystemError, FileSystemPort};
use codeatlas_domain::value_objects::file_path::SanitizedPath;
use std::path::Path;

pub struct LocalFileSystemAdapter;

impl LocalFileSystemAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LocalFileSystemAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FileSystemPort for LocalFileSystemAdapter {
    async fn discover_files(
        &self,
        root: &Path,
        max_files: usize,
    ) -> Result<Vec<SanitizedPath>, FileSystemError> {
        let mut files = Vec::new();
        let mut dirs = vec![root.to_path_buf()];

        while let Some(current_dir) = dirs.pop() {
            if files.len() >= max_files {
                break;
            }

            let mut entries = match tokio::fs::read_dir(&current_dir).await {
                Ok(e) => e,
                Err(_) => continue,
            };

            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                let file_name = entry.file_name();
                let name_str = file_name.to_string_lossy();

                if name_str.starts_with('.')
                    || name_str == "node_modules"
                    || name_str == "dist"
                    || name_str == "target"
                {
                    continue;
                }

                if let Ok(file_type) = entry.file_type().await {
                    if file_type.is_dir() {
                        dirs.push(path);
                    } else if file_type.is_file() {
                        if let Ok(sanitized) = SanitizedPath::within_root(&path, root) {
                            files.push(sanitized);
                            if files.len() >= max_files {
                                break;
                            }
                        }
                    }
                }
            }
        }

        Ok(files)
    }

    async fn read_file_string(&self, path: &Path) -> Result<String, FileSystemError> {
        tokio::fs::read_to_string(path)
            .await
            .map_err(FileSystemError::from)
    }

    async fn read_file_lines(
        &self,
        path: &Path,
        start_line: u32,
        end_line: u32,
    ) -> Result<String, FileSystemError> {
        let content = self.read_file_string(path).await?;
        let lines: Vec<&str> = content.lines().collect();

        let start_idx = (start_line.saturating_sub(1) as usize).min(lines.len());
        let end_idx = (end_line as usize).min(lines.len());

        if start_idx >= lines.len() {
            return Ok(String::new());
        }

        Ok(lines[start_idx..end_idx].join("\n"))
    }

    async fn file_exists(&self, path: &Path) -> bool {
        tokio::fs::metadata(path).await.is_ok()
    }
}
