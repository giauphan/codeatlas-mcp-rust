use super::super::protocol::{ToolHandler, ToolRegistry};
use async_trait::async_trait;
use codeatlas_application::dto::analysis_dto::AnalyzeRequest;
use codeatlas_application::use_cases::analyze_project::AnalyzeProjectUseCase;
use codeatlas_application::use_cases::query_graph::QueryGraphUseCase;
use codeatlas_domain::value_objects::project_id::ProjectId;
use serde_json::{json, Value};
use std::sync::Arc;

pub async fn resolve_project(query_uc: &QueryGraphUseCase, project_arg: Option<&str>) -> ProjectId {
    if let Some(p) = project_arg {
        let trimmed = p.trim();
        if !trimmed.is_empty() && trimmed != "default" {
            return ProjectId::new(trimmed);
        }
    }
    if let Ok(mut projects) = query_uc.list_projects().await {
        if !projects.is_empty() {
            projects.sort_by(|a, b| a.0.as_str().cmp(b.0.as_str()));
            return projects[0].0.clone();
        }
    }
    ProjectId::new("default")
}

struct AnalyzeHandler(Arc<AnalyzeProjectUseCase>);
#[async_trait]
impl ToolHandler for AnalyzeHandler {
    async fn call(&self, args: Value) -> Result<String, String> {
        let path = args.get("path").and_then(|p| p.as_str()).unwrap_or("");
        let max_files = args
            .get("maxFiles")
            .and_then(|m| m.as_u64())
            .map(|m| m as usize);
        let req = AnalyzeRequest {
            path: path.to_string(),
            max_files,
        };
        let project_id = ProjectId::from_path(std::path::Path::new(path));
        let res = self.0.execute(req).await.map_err(|e| e.to_string())?;
        let output = json!({
            "project": project_id.as_str(),
            "result": res
        });
        serde_json::to_string_pretty(&output).map_err(|e| e.to_string())
    }
}

struct ListProjectsHandler(Arc<QueryGraphUseCase>);
#[async_trait]
impl ToolHandler for ListProjectsHandler {
    async fn call(&self, _args: Value) -> Result<String, String> {
        let mut projects = self.0.list_projects().await.map_err(|e| e.to_string())?;
        projects.sort_by(|a, b| a.0.as_str().cmp(b.0.as_str()));
        let json_arr: Vec<Value> = projects
            .into_iter()
            .map(|(id, name)| json!({ "id": id.as_str(), "name": name }))
            .collect();
        serde_json::to_string_pretty(&json_arr).map_err(|e| e.to_string())
    }
}

struct GetStructureHandler(Arc<QueryGraphUseCase>);
#[async_trait]
impl ToolHandler for GetStructureHandler {
    async fn call(&self, args: Value) -> Result<String, String> {
        let project_arg = args.get("project").and_then(|p| p.as_str());
        let project_id = resolve_project(&self.0, project_arg).await;
        let analysis = self
            .0
            .get_analysis(&project_id)
            .await
            .map_err(|e| e.to_string())?;
        match analysis {
            Some(a) => serde_json::to_string_pretty(&a.graph.nodes).map_err(|e| e.to_string()),
            None => Err(format!("Project not found: {}", project_id)),
        }
    }
}

struct GetInsightsHandler(Arc<QueryGraphUseCase>);
#[async_trait]
impl ToolHandler for GetInsightsHandler {
    async fn call(&self, args: Value) -> Result<String, String> {
        let project_arg = args.get("project").and_then(|p| p.as_str());
        let project_id = resolve_project(&self.0, project_arg).await;
        let analysis = self
            .0
            .get_analysis(&project_id)
            .await
            .map_err(|e| e.to_string())?;
        match analysis {
            Some(a) => serde_json::to_string_pretty(&a.insights).map_err(|e| e.to_string()),
            None => Ok("[]".to_string()),
        }
    }
}

struct ProjectContextHandler(Arc<QueryGraphUseCase>);
#[async_trait]
impl ToolHandler for ProjectContextHandler {
    async fn call(&self, args: Value) -> Result<String, String> {
        let project_arg = args.get("project").and_then(|p| p.as_str());
        let project_id = resolve_project(&self.0, project_arg).await;
        let analysis = self
            .0
            .get_analysis(&project_id)
            .await
            .map_err(|e| e.to_string())?;
        let res = json!({
            "project": project_id.as_str(),
            "hasAnalysis": analysis.is_some(),
            "nodesCount": analysis.as_ref().map(|a| a.graph.nodes.len()).unwrap_or(0),
            "linksCount": analysis.as_ref().map(|a| a.graph.links.len()).unwrap_or(0),
        });
        serde_json::to_string_pretty(&res).map_err(|e| e.to_string())
    }
}

pub fn register_analysis_tools(
    registry: &mut ToolRegistry,
    analyze_uc: Arc<AnalyzeProjectUseCase>,
    query_uc: Arc<QueryGraphUseCase>,
) {
    registry.register(
        "analyze",
        "Perform deep code analysis on a local project directory.",
        json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "Absolute path to project directory" },
                "maxFiles": { "type": "number", "description": "Maximum files to analyze" }
            },
            "required": ["path"]
        }),
        AnalyzeHandler(analyze_uc),
    );

    registry.register(
        "list_projects",
        "List all projects analyzed by CodeAtlas.",
        json!({ "type": "object", "properties": {} }),
        ListProjectsHandler(query_uc.clone()),
    );

    registry.register(
        "get_project_structure",
        "Get all modules, classes, and functions in the analyzed project.",
        json!({
            "type": "object",
            "properties": { "project": { "type": "string" } }
        }),
        GetStructureHandler(query_uc.clone()),
    );

    registry.register(
        "get_insights",
        "Get code insights including refactoring suggestions and security issues.",
        json!({
            "type": "object",
            "properties": { "project": { "type": "string" } }
        }),
        GetInsightsHandler(query_uc.clone()),
    );

    registry.register(
        "project_context",
        "Get comprehensive overview of a project in one call.",
        json!({
            "type": "object",
            "properties": { "project": { "type": "string" } }
        }),
        ProjectContextHandler(query_uc.clone()),
    );
}
