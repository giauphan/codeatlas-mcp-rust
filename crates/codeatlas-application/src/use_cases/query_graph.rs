use codeatlas_domain::entities::graph::{AnalysisResult, GraphData, GraphNode};
use codeatlas_domain::ports::graph_storage::GraphRepositoryPort;
use codeatlas_domain::value_objects::project_id::ProjectId;
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum QueryGraphError {
    #[error("Storage error: {0}")]
    Storage(String),
    #[error("Symbol not found: {0}")]
    SymbolNotFound(String),
}

pub struct QueryGraphUseCase {
    graph_repo: Arc<dyn GraphRepositoryPort>,
}

impl QueryGraphUseCase {
    pub fn new(graph_repo: Arc<dyn GraphRepositoryPort>) -> Self {
        Self { graph_repo }
    }

    pub async fn get_callers(
        &self,
        project: &ProjectId,
        symbol: &str,
    ) -> Result<Vec<GraphNode>, QueryGraphError> {
        self.graph_repo
            .find_callers(project, symbol)
            .await
            .map_err(|e| QueryGraphError::Storage(e.to_string()))
    }

    pub async fn get_callees(
        &self,
        project: &ProjectId,
        symbol: &str,
    ) -> Result<Vec<GraphNode>, QueryGraphError> {
        self.graph_repo
            .find_callees(project, symbol)
            .await
            .map_err(|e| QueryGraphError::Storage(e.to_string()))
    }

    pub async fn get_dependencies(
        &self,
        project: &ProjectId,
        symbol: &str,
        depth: u32,
    ) -> Result<GraphData, QueryGraphError> {
        self.graph_repo
            .get_dependencies(project, symbol, depth)
            .await
            .map_err(|e| QueryGraphError::Storage(e.to_string()))
    }

    pub async fn detect_cycles(
        &self,
        project: &ProjectId,
    ) -> Result<Vec<Vec<String>>, QueryGraphError> {
        self.graph_repo
            .detect_cycles(project)
            .await
            .map_err(|e| QueryGraphError::Storage(e.to_string()))
    }

    pub async fn generate_system_flow(
        &self,
        project: &ProjectId,
        max_nodes: usize,
    ) -> Result<String, QueryGraphError> {
        self.graph_repo
            .generate_system_flow(project, max_nodes)
            .await
            .map_err(|e| QueryGraphError::Storage(e.to_string()))
    }

    pub async fn generate_feature_flow(
        &self,
        project: &ProjectId,
        keyword: &str,
        max_depth: u32,
    ) -> Result<String, QueryGraphError> {
        self.graph_repo
            .generate_feature_flow(project, keyword, max_depth)
            .await
            .map_err(|e| QueryGraphError::Storage(e.to_string()))
    }

    pub async fn get_analysis(
        &self,
        project: &ProjectId,
    ) -> Result<Option<AnalysisResult>, QueryGraphError> {
        self.graph_repo
            .get_analysis(project)
            .await
            .map_err(|e| QueryGraphError::Storage(e.to_string()))
    }

    pub async fn list_projects(&self) -> Result<Vec<(ProjectId, String)>, QueryGraphError> {
        self.graph_repo
            .list_projects()
            .await
            .map_err(|e| QueryGraphError::Storage(e.to_string()))
    }
}
