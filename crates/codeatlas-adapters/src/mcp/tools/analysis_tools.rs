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

    registry.register(
        "get_file_entities",
        "Get all entities (classes, functions, variables) defined in a specific file.",
        json!({
            "type": "object",
            "properties": {
                "filePath": { "type": "string" },
                "project": { "type": "string" }
            },
            "required": ["filePath"]
        }),
        GetFileEntitiesHandler(query_uc.clone()),
    );

    registry.register(
        "index_coverage",
        "Check what files and entity types are indexed for a project.",
        json!({
            "type": "object",
            "properties": { "project": { "type": "string" } }
        }),
        IndexCoverageHandler(query_uc.clone()),
    );

    registry.register(
        "detect_code_similarities",
        "Scan project files to find near-duplicate or semantically similar functions/classes.",
        json!({
            "type": "object",
            "properties": {
                "project": { "type": "string" },
                "threshold": { "type": "number" }
            }
        }),
        DetectCodeSimilaritiesHandler(query_uc.clone()),
    );

    registry.register(
        "export_team_artifact",
        "Write and export a compressed snapshot of the project's analysis to .codeatlas/artifact.json.",
        json!({
            "type": "object",
            "properties": { "project": { "type": "string" } }
        }),
        ExportTeamArtifactHandler(query_uc),
    );
}

struct GetFileEntitiesHandler(Arc<QueryGraphUseCase>);
#[async_trait]
impl ToolHandler for GetFileEntitiesHandler {
    async fn call(&self, args: Value) -> Result<String, String> {
        let file_path_arg = args
            .get("filePath")
            .and_then(|f| f.as_str())
            .filter(|f| !f.trim().is_empty())
            .ok_or_else(|| "Missing required argument: filePath".to_string())?;
        let project_arg = args.get("project").and_then(|p| p.as_str());
        let project_id = resolve_project(&self.0, project_arg).await;

        let analysis = self
            .0
            .get_analysis(&project_id)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| {
                "No analysis data found for project. Run 'analyze' first.".to_string()
            })?;

        let entities: Vec<_> = analysis
            .graph
            .nodes
            .into_iter()
            .filter(|n| {
                n.file_path
                    .as_deref()
                    .is_some_and(|p| p.contains(file_path_arg) || p.ends_with(file_path_arg))
            })
            .collect();

        let count = entities.len();
        let res = json!({
            "project": project_id.as_str(),
            "filePath": file_path_arg,
            "entities": entities,
            "count": count,
        });
        serde_json::to_string_pretty(&res).map_err(|e| e.to_string())
    }
}

struct IndexCoverageHandler(Arc<QueryGraphUseCase>);
#[async_trait]
impl ToolHandler for IndexCoverageHandler {
    async fn call(&self, args: Value) -> Result<String, String> {
        let project_arg = args.get("project").and_then(|p| p.as_str());
        let project_id = resolve_project(&self.0, project_arg).await;

        let analysis = self
            .0
            .get_analysis(&project_id)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| {
                "No analysis data found for project. Run 'analyze' first.".to_string()
            })?;

        let mut type_counts = std::collections::HashMap::new();
        let mut file_counts: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();

        for node in &analysis.graph.nodes {
            *type_counts.entry(node.node_type.clone()).or_insert(0) += 1;
            if let Some(fp) = &node.file_path {
                *file_counts.entry(fp.clone()).or_insert(0) += 1;
            }
        }

        let unique_files = file_counts.len();
        let mut top_files: Vec<_> = file_counts.into_iter().collect();
        top_files.sort_by_key(|a| std::cmp::Reverse(a.1));
        top_files.truncate(10);

        let res = json!({
            "project": project_id.as_str(),
            "totalEntities": analysis.graph.nodes.len(),
            "totalLinks": analysis.graph.links.len(),
            "uniqueFiles": unique_files,
            "typeDistribution": type_counts,
            "topFilesByEntities": top_files,
        });
        serde_json::to_string_pretty(&res).map_err(|e| e.to_string())
    }
}

struct DetectCodeSimilaritiesHandler(Arc<QueryGraphUseCase>);
#[async_trait]
impl ToolHandler for DetectCodeSimilaritiesHandler {
    async fn call(&self, args: Value) -> Result<String, String> {
        let project_arg = args.get("project").and_then(|p| p.as_str());
        let threshold = args
            .get("threshold")
            .and_then(|t| t.as_f64())
            .unwrap_or(0.8);
        let project_id = resolve_project(&self.0, project_arg).await;

        let analysis = self
            .0
            .get_analysis(&project_id)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| {
                "No analysis data found for project. Run 'analyze' first.".to_string()
            })?;

        let funcs: Vec<(String, Option<String>)> = analysis
            .graph
            .nodes
            .iter()
            .filter(|n| n.node_type == "function")
            .map(|n| (n.label.clone(), n.file_path.clone()))
            .collect();

        let similarities = tokio::task::spawn_blocking(move || {
            let mut matches = Vec::new();
            'outer: for i in 0..funcs.len() {
                for j in (i + 1)..funcs.len() {
                    let (s1, f1) = &funcs[i];
                    let (s2, f2) = &funcs[j];
                    if s1 == s2 {
                        continue;
                    }
                    let sim = strsim::jaro_winkler(s1, s2);
                    if sim >= threshold {
                        matches.push(json!({
                            "symbolA": s1,
                            "fileA": f1,
                            "symbolB": s2,
                            "fileB": f2,
                            "similarity": (sim * 100.0).round() / 100.0,
                        }));
                        if matches.len() >= 30 {
                            break 'outer;
                        }
                    }
                }
            }
            matches
        })
        .await
        .map_err(|e| e.to_string())?;

        let res = json!({
            "project": project_id.as_str(),
            "threshold": threshold,
            "matches": similarities,
            "count": similarities.len(),
        });
        serde_json::to_string_pretty(&res).map_err(|e| e.to_string())
    }
}

struct ExportTeamArtifactHandler(Arc<QueryGraphUseCase>);
#[async_trait]
impl ToolHandler for ExportTeamArtifactHandler {
    async fn call(&self, args: Value) -> Result<String, String> {
        let project_arg = args.get("project").and_then(|p| p.as_str());
        let project_id = resolve_project(&self.0, project_arg).await;

        let analysis = self
            .0
            .get_analysis(&project_id)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| {
                "No analysis data found for project. Run 'analyze' first.".to_string()
            })?;

        let project_path = std::path::Path::new(project_arg.unwrap_or("."));
        let dir = project_path.join(".codeatlas");
        if !dir.exists() {
            std::fs::create_dir_all(&dir)
                .map_err(|e| format!("Failed to create directory {:?}: {}", dir, e))?;
        }
        let artifact_path = dir.join("artifact.json");
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let payload = json!({
            "project": project_id.as_str(),
            "timestamp": ts,
            "nodesCount": analysis.graph.nodes.len(),
            "linksCount": analysis.graph.links.len(),
            "graph": analysis.graph,
        });

        let json_data = serde_json::to_string_pretty(&payload)
            .map_err(|e| format!("Failed to serialize artifact: {}", e))?;

        std::fs::write(&artifact_path, json_data)
            .map_err(|e| format!("Failed to write artifact: {}", e))?;

        let res = json!({
            "status": "success",
            "path": artifact_path.to_string_lossy(),
            "nodesCount": payload["nodesCount"],
            "linksCount": payload["linksCount"],
        });
        serde_json::to_string_pretty(&res).map_err(|e| e.to_string())
    }
}
