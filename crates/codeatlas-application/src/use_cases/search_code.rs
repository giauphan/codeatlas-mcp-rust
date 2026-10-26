use codeatlas_domain::entities::graph::GraphNode;
use codeatlas_domain::ports::file_system::FileSystemPort;
use codeatlas_domain::ports::graph_storage::GraphRepositoryPort;
use codeatlas_domain::value_objects::project_id::ProjectId;
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SearchCodeError {
    #[error("Storage error: {0}")]
    Storage(String),
    #[error("I/O error: {0}")]
    Io(String),
}

pub struct SearchCodeUseCase {
    graph_repo: Arc<dyn GraphRepositoryPort>,
    fs: Arc<dyn FileSystemPort>,
}

impl SearchCodeUseCase {
    pub fn new(graph_repo: Arc<dyn GraphRepositoryPort>, fs: Arc<dyn FileSystemPort>) -> Self {
        Self { graph_repo, fs }
    }

    pub async fn search_entities(
        &self,
        project: &ProjectId,
        query: &str,
    ) -> Result<Vec<GraphNode>, SearchCodeError> {
        let analysis_opt = self
            .graph_repo
            .get_analysis(project)
            .await
            .map_err(|e| SearchCodeError::Storage(e.to_string()))?;

        let analysis = match analysis_opt {
            Some(a) => a,
            None => return Ok(Vec::new()),
        };

        let q_lower = query.to_lowercase();
        let matches = analysis
            .graph
            .nodes
            .into_iter()
            .filter(|n| n.label.to_lowercase().contains(&q_lower))
            .collect();

        Ok(matches)
    }

    pub async fn get_code_snippet(
        &self,
        project: &ProjectId,
        symbol: &str,
    ) -> Result<Option<String>, SearchCodeError> {
        let analysis_opt = self
            .graph_repo
            .get_analysis(project)
            .await
            .map_err(|e| SearchCodeError::Storage(e.to_string()))?;

        let analysis = match analysis_opt {
            Some(a) => a,
            None => return Ok(None),
        };

        let node_opt = analysis.graph.nodes.iter().find(|n| n.label == symbol);
        let node = match node_opt {
            Some(n) => n,
            None => return Ok(None),
        };

        let file_path = match &node.file_path {
            Some(fp) => std::path::Path::new(fp),
            None => return Ok(None),
        };

        let line = node.line.unwrap_or(1);
        let start = line.saturating_sub(5);
        let end = line + 25;

        let content = self
            .fs
            .read_file_lines(file_path, start, end)
            .await
            .map_err(|e| SearchCodeError::Io(e.to_string()))?;

        Ok(Some(content))
    }
}
