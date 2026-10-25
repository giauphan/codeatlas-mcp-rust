use crate::entities::brain::{DreamMemory, ImmuneGene};
use async_trait::async_trait;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CloudSyncError {
    #[error("Network connection error: {0}")]
    Network(String),
    #[error("Authentication failed: {0}")]
    AuthFailed(String),
    #[error("API error ({status}): {message}")]
    Api { status: u16, message: String },
    #[error("Offline mode enabled")]
    Offline,
}

#[async_trait]
pub trait CloudSyncPort: Send + Sync {
    async fn save_dream(&self, dream: &DreamMemory) -> Result<(), CloudSyncError>;
    async fn query_dreams(
        &self,
        query: &str,
        project: Option<&str>,
        limit: usize,
    ) -> Result<Vec<DreamMemory>, CloudSyncError>;
    async fn search_genome(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<ImmuneGene>, CloudSyncError>;
    async fn scan_immune_genes(&self, problem: &str) -> Result<Vec<ImmuneGene>, CloudSyncError>;
    async fn save_immune_gene(&self, gene: &ImmuneGene) -> Result<(), CloudSyncError>;
    async fn check_auth(&self) -> Result<bool, CloudSyncError>;
}
