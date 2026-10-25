use async_trait::async_trait;
use codeatlas_domain::entities::adr::ADR;
use codeatlas_domain::ports::adr_storage::{AdrRepositoryPort, AdrStorageError};
use std::path::PathBuf;

fn safe_segment(s: &str) -> Result<&str, AdrStorageError> {
    let trimmed = s.trim();
    if trimmed.is_empty() || trimmed == "." || trimmed == ".." {
        return Err(AdrStorageError::Validation(format!(
            "Invalid path segment: {}",
            s
        )));
    }
    if !trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
    {
        return Err(AdrStorageError::Validation(format!(
            "Path segment contains invalid characters: {}",
            s
        )));
    }
    Ok(trimmed)
}

pub struct AdrFileSystemAdapter {
    base_dir: PathBuf,
}

impl AdrFileSystemAdapter {
    pub fn new() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let base_dir = PathBuf::from(home).join(".codeatlas").join("adr");
        Self { base_dir }
    }

    pub fn with_dir(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }
}

impl Default for AdrFileSystemAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AdrRepositoryPort for AdrFileSystemAdapter {
    async fn list_adrs(&self, project: Option<&str>) -> Result<Vec<ADR>, AdrStorageError> {
        let dir = match project {
            Some(p) => self.base_dir.join(safe_segment(p)?),
            None => self.base_dir.clone(),
        };

        if !dir.exists() {
            return Ok(Vec::new());
        }

        let mut adrs = Vec::new();
        let mut entries = tokio::fs::read_dir(&dir)
            .await
            .map_err(|e| AdrStorageError::Io(e.to_string()))?;

        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                if let Ok(content) = tokio::fs::read_to_string(&path).await {
                    if let Ok(adr) = serde_json::from_str::<ADR>(&content) {
                        adrs.push(adr);
                    }
                }
            }
        }

        Ok(adrs)
    }

    async fn get_adr(&self, project: &str, id: &str) -> Result<Option<ADR>, AdrStorageError> {
        let path = self
            .base_dir
            .join(safe_segment(project)?)
            .join(format!("{}.json", safe_segment(id)?));
        if !path.exists() {
            return Ok(None);
        }

        let content = tokio::fs::read_to_string(&path)
            .await
            .map_err(|e| AdrStorageError::Io(e.to_string()))?;

        let adr = serde_json::from_str(&content)
            .map_err(|e| AdrStorageError::Validation(e.to_string()))?;

        Ok(Some(adr))
    }

    async fn save_adr(&self, adr: ADR) -> Result<ADR, AdrStorageError> {
        let safe_proj = safe_segment(&adr.project)?;
        let safe_id = safe_segment(&adr.id)?;

        let dir = self.base_dir.join(safe_proj);
        tokio::fs::create_dir_all(&dir)
            .await
            .map_err(|e| AdrStorageError::Io(e.to_string()))?;

        let file_path = dir.join(format!("{}.json", safe_id));
        let content = serde_json::to_string_pretty(&adr)
            .map_err(|e| AdrStorageError::Validation(e.to_string()))?;

        tokio::fs::write(&file_path, content)
            .await
            .map_err(|e| AdrStorageError::Io(e.to_string()))?;

        Ok(adr)
    }

    async fn delete_adr(&self, project: &str, id: &str) -> Result<bool, AdrStorageError> {
        let path = self
            .base_dir
            .join(safe_segment(project)?)
            .join(format!("{}.json", safe_segment(id)?));
        if !path.exists() {
            return Ok(false);
        }

        tokio::fs::remove_file(path)
            .await
            .map_err(|e| AdrStorageError::Io(e.to_string()))?;

        Ok(true)
    }
}
