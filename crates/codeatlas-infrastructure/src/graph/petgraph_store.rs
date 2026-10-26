use async_trait::async_trait;
use codeatlas_domain::entities::graph::{AnalysisResult, GraphData, GraphNode};
use codeatlas_domain::ports::graph_storage::{GraphRepositoryPort, GraphStorageError};
use codeatlas_domain::value_objects::project_id::ProjectId;
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::Arc;

use super::traversals::{
    bfs_reachable, detect_cycles_in_graph, generate_mermaid_feature_flow,
    generate_mermaid_system_flow,
};

pub struct PetgraphStore {
    projects: Arc<DashMap<String, AnalysisResult>>,
}

impl PetgraphStore {
    pub fn new() -> Self {
        Self {
            projects: Arc::new(DashMap::new()),
        }
    }
}

impl Default for PetgraphStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl GraphRepositoryPort for PetgraphStore {
    async fn store_analysis(
        &self,
        project: ProjectId,
        analysis: AnalysisResult,
    ) -> Result<(), GraphStorageError> {
        self.projects.insert(project.as_str().to_string(), analysis);
        Ok(())
    }

    async fn get_analysis(
        &self,
        project: &ProjectId,
    ) -> Result<Option<AnalysisResult>, GraphStorageError> {
        Ok(self.projects.get(project.as_str()).map(|r| r.clone()))
    }

    async fn list_projects(&self) -> Result<Vec<(ProjectId, String)>, GraphStorageError> {
        let list = self
            .projects
            .iter()
            .map(|entry| (ProjectId::new(entry.key().clone()), entry.key().clone()))
            .collect();
        Ok(list)
    }

    async fn find_callers(
        &self,
        project: &ProjectId,
        symbol: &str,
    ) -> Result<Vec<GraphNode>, GraphStorageError> {
        let analysis = self
            .projects
            .get(project.as_str())
            .ok_or_else(|| GraphStorageError::NotFound(project.to_string()))?;

        let caller_names: Vec<String> = analysis
            .graph
            .links
            .iter()
            .filter(|l| (l.link_type == "call" || l.link_type == "import") && l.target == symbol)
            .map(|l| l.source.clone())
            .collect();

        let callers = analysis
            .graph
            .nodes
            .iter()
            .filter(|n| caller_names.contains(&n.label))
            .cloned()
            .collect();

        Ok(callers)
    }

    async fn find_callees(
        &self,
        project: &ProjectId,
        symbol: &str,
    ) -> Result<Vec<GraphNode>, GraphStorageError> {
        let analysis = self
            .projects
            .get(project.as_str())
            .ok_or_else(|| GraphStorageError::NotFound(project.to_string()))?;

        let callee_names: Vec<String> = analysis
            .graph
            .links
            .iter()
            .filter(|l| (l.link_type == "call" || l.link_type == "import") && l.source == symbol)
            .map(|l| l.target.clone())
            .collect();

        let callees = analysis
            .graph
            .nodes
            .iter()
            .filter(|n| callee_names.contains(&n.label))
            .cloned()
            .collect();

        Ok(callees)
    }

    async fn get_dependencies(
        &self,
        project: &ProjectId,
        symbol: &str,
        depth: u32,
    ) -> Result<GraphData, GraphStorageError> {
        let analysis = self
            .projects
            .get(project.as_str())
            .ok_or_else(|| GraphStorageError::NotFound(project.to_string()))?;

        let mut adj: HashMap<String, Vec<String>> = HashMap::new();
        for link in &analysis.graph.links {
            adj.entry(link.source.clone())
                .or_default()
                .push(link.target.clone());
        }

        let reachable = bfs_reachable(&adj, symbol, depth, 500);

        let filtered_nodes = analysis
            .graph
            .nodes
            .iter()
            .filter(|n| reachable.contains(&n.label))
            .cloned()
            .collect();

        let filtered_links = analysis
            .graph
            .links
            .iter()
            .filter(|l| reachable.contains(&l.source) && reachable.contains(&l.target))
            .cloned()
            .collect();

        Ok(GraphData {
            nodes: filtered_nodes,
            links: filtered_links,
        })
    }

    async fn detect_cycles(
        &self,
        project: &ProjectId,
    ) -> Result<Vec<Vec<String>>, GraphStorageError> {
        let analysis = self
            .projects
            .get(project.as_str())
            .ok_or_else(|| GraphStorageError::NotFound(project.to_string()))?;

        Ok(detect_cycles_in_graph(&analysis))
    }

    async fn generate_system_flow(
        &self,
        project: &ProjectId,
        max_nodes: usize,
    ) -> Result<String, GraphStorageError> {
        let analysis = self
            .projects
            .get(project.as_str())
            .ok_or_else(|| GraphStorageError::NotFound(project.to_string()))?;

        Ok(generate_mermaid_system_flow(&analysis, max_nodes))
    }

    async fn generate_feature_flow(
        &self,
        project: &ProjectId,
        keyword: &str,
        max_depth: u32,
    ) -> Result<String, GraphStorageError> {
        let analysis = self
            .projects
            .get(project.as_str())
            .ok_or_else(|| GraphStorageError::NotFound(project.to_string()))?;

        Ok(generate_mermaid_feature_flow(&analysis, keyword, max_depth))
    }
}
