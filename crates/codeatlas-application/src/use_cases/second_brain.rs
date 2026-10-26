use codeatlas_domain::entities::brain::{BrainContext, DreamMemory, ImmuneGene};
use codeatlas_domain::ports::cloud_sync::CloudSyncPort;
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SecondBrainError {
    #[error("Cloud sync error: {0}")]
    Cloud(String),
}

pub struct SecondBrainUseCase {
    cloud_sync: Arc<dyn CloudSyncPort>,
}

impl SecondBrainUseCase {
    pub fn new(cloud_sync: Arc<dyn CloudSyncPort>) -> Self {
        Self { cloud_sync }
    }

    pub async fn save_dream(&self, dream: &DreamMemory) -> Result<(), SecondBrainError> {
        self.cloud_sync
            .save_dream(dream)
            .await
            .map_err(|e| SecondBrainError::Cloud(e.to_string()))
    }

    pub async fn query_dreams(
        &self,
        query: &str,
        project: Option<&str>,
        limit: usize,
    ) -> Result<Vec<DreamMemory>, SecondBrainError> {
        self.cloud_sync
            .query_dreams(query, project, limit)
            .await
            .map_err(|e| SecondBrainError::Cloud(e.to_string()))
    }

    pub async fn search_genome(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<ImmuneGene>, SecondBrainError> {
        self.cloud_sync
            .search_genome(query, limit)
            .await
            .map_err(|e| SecondBrainError::Cloud(e.to_string()))
    }

    pub async fn scan_immune_genes(
        &self,
        problem: &str,
    ) -> Result<Vec<ImmuneGene>, SecondBrainError> {
        self.cloud_sync
            .scan_immune_genes(problem)
            .await
            .map_err(|e| SecondBrainError::Cloud(e.to_string()))
    }

    pub async fn save_immune_gene(&self, gene: &ImmuneGene) -> Result<(), SecondBrainError> {
        self.cloud_sync
            .save_immune_gene(gene)
            .await
            .map_err(|e| SecondBrainError::Cloud(e.to_string()))
    }

    pub async fn load_brain_context(
        &self,
        task: &str,
        project: Option<&str>,
    ) -> Result<BrainContext, SecondBrainError> {
        let dreams = self
            .cloud_sync
            .query_dreams(task, project, 5)
            .await
            .unwrap_or_default();
        let genes = self
            .cloud_sync
            .scan_immune_genes(task)
            .await
            .unwrap_or_default();

        Ok(BrainContext {
            project: project.unwrap_or("default").to_string(),
            dream_memories: dreams,
            immune_genes: genes,
            system_memory: None,
        })
    }
}
