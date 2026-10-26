use crate::entities::graph::{AnalysisResult, GraphData, GraphNode};
use crate::value_objects::project_id::ProjectId;
use async_trait::async_trait;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GraphStorageError {
    #[error("Project not found: {0}")]
    NotFound(String),
    #[error("Internal storage failure: {0}")]
    Internal(String),
}

#[async_trait]
pub trait GraphRepositoryPort: Send + Sync {
    async fn store_analysis(
        &self,
        project: ProjectId,
        analysis: AnalysisResult,
    ) -> Result<(), GraphStorageError>;
    async fn get_analysis(
        &self,
        project: &ProjectId,
    ) -> Result<Option<AnalysisResult>, GraphStorageError>;
    async fn list_projects(&self) -> Result<Vec<(ProjectId, String)>, GraphStorageError>;
    async fn find_callers(
        &self,
        project: &ProjectId,
        symbol: &str,
    ) -> Result<Vec<GraphNode>, GraphStorageError>;
    async fn find_callees(
        &self,
        project: &ProjectId,
        symbol: &str,
    ) -> Result<Vec<GraphNode>, GraphStorageError>;
    async fn get_dependencies(
        &self,
        project: &ProjectId,
        symbol: &str,
        depth: u32,
    ) -> Result<GraphData, GraphStorageError>;
    async fn detect_cycles(
        &self,
        project: &ProjectId,
    ) -> Result<Vec<Vec<String>>, GraphStorageError>;
    async fn generate_system_flow(
        &self,
        project: &ProjectId,
        max_nodes: usize,
    ) -> Result<String, GraphStorageError>;
    async fn generate_feature_flow(
        &self,
        project: &ProjectId,
        keyword: &str,
        max_depth: u32,
    ) -> Result<String, GraphStorageError>;
}
