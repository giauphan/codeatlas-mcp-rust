use codeatlas_domain::entities::adr::ADR;
use codeatlas_domain::ports::adr_storage::AdrRepositoryPort;
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ManageAdrError {
    #[error("Storage error: {0}")]
    Storage(String),
    #[error("Validation error: {0}")]
    Validation(String),
}

pub struct ManageAdrUseCase {
    adr_repo: Arc<dyn AdrRepositoryPort>,
}

impl ManageAdrUseCase {
    pub fn new(adr_repo: Arc<dyn AdrRepositoryPort>) -> Self {
        Self { adr_repo }
    }

    pub async fn list(&self, project: Option<&str>) -> Result<Vec<ADR>, ManageAdrError> {
        self.adr_repo
            .list_adrs(project)
            .await
            .map_err(|e| ManageAdrError::Storage(e.to_string()))
    }

    pub async fn get(&self, project: &str, id: &str) -> Result<Option<ADR>, ManageAdrError> {
        self.adr_repo
            .get_adr(project, id)
            .await
            .map_err(|e| ManageAdrError::Storage(e.to_string()))
    }

    pub async fn save(&self, adr: ADR) -> Result<ADR, ManageAdrError> {
        self.adr_repo
            .save_adr(adr)
            .await
            .map_err(|e| ManageAdrError::Storage(e.to_string()))
    }

    pub async fn delete(&self, project: &str, id: &str) -> Result<bool, ManageAdrError> {
        self.adr_repo
            .delete_adr(project, id)
            .await
            .map_err(|e| ManageAdrError::Storage(e.to_string()))
    }
}
