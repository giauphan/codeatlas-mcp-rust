use crate::entities::adr::ADR;
use async_trait::async_trait;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AdrStorageError {
    #[error("ADR not found: {0}")]
    NotFound(String),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("I/O error: {0}")]
    Io(String),
}

#[async_trait]
pub trait AdrRepositoryPort: Send + Sync {
    async fn list_adrs(&self, project: Option<&str>) -> Result<Vec<ADR>, AdrStorageError>;
    async fn get_adr(&self, project: &str, id: &str) -> Result<Option<ADR>, AdrStorageError>;
    async fn save_adr(&self, adr: ADR) -> Result<ADR, AdrStorageError>;
    async fn delete_adr(&self, project: &str, id: &str) -> Result<bool, AdrStorageError>;
}
