use super::super::protocol::{ToolHandler, ToolRegistry};
use super::analysis_tools::resolve_project;
use async_trait::async_trait;
use codeatlas_application::use_cases::query_graph::QueryGraphUseCase;
use codeatlas_application::use_cases::search_code::SearchCodeUseCase;
use serde_json::{json, Value};
use std::sync::Arc;

struct SearchEntitiesHandler {
    search_uc: Arc<SearchCodeUseCase>,
    query_uc: Arc<QueryGraphUseCase>,
}

#[async_trait]
impl ToolHandler for SearchEntitiesHandler {
    async fn call(&self, args: Value) -> Result<String, String> {
        let query = args
            .get("query")
            .and_then(|q| q.as_str())
            .filter(|q| !q.trim().is_empty())
            .ok_or_else(|| "Missing required argument: query".to_string())?;
        let project_arg = args.get("project").and_then(|p| p.as_str());
        let project_id = resolve_project(&self.query_uc, project_arg).await;
        let matches = self
            .search_uc
            .search_entities(&project_id, query)
            .await
            .map_err(|e| e.to_string())?;
        serde_json::to_string_pretty(&matches).map_err(|e| e.to_string())
    }
}

struct GetCodeSnippetHandler {
    search_uc: Arc<SearchCodeUseCase>,
    query_uc: Arc<QueryGraphUseCase>,
}

#[async_trait]
impl ToolHandler for GetCodeSnippetHandler {
    async fn call(&self, args: Value) -> Result<String, String> {
        let symbol = args
            .get("symbol")
            .and_then(|s| s.as_str())
            .filter(|s| !s.trim().is_empty())
            .ok_or_else(|| "Missing required argument: symbol".to_string())?;
        let project_arg = args.get("project").and_then(|p| p.as_str());
        let project_id = resolve_project(&self.query_uc, project_arg).await;
        let snippet = self
            .search_uc
            .get_code_snippet(&project_id, symbol)
            .await
            .map_err(|e| e.to_string())?;
        match snippet {
            Some(s) => Ok(s),
            None => Err(format!("Symbol snippet not found: {}", symbol)),
        }
    }
}

pub fn register_search_tools(
    registry: &mut ToolRegistry,
    search_uc: Arc<SearchCodeUseCase>,
    query_uc: Arc<QueryGraphUseCase>,
) {
    registry.register(
        "search_entities",
        "Search for functions, classes, or modules by name.",
        json!({
            "type": "object",
            "properties": {
                "query": { "type": "string" },
                "project": { "type": "string" }
            },
            "required": ["query"]
        }),
        SearchEntitiesHandler {
            search_uc: search_uc.clone(),
            query_uc: query_uc.clone(),
        },
    );

    registry.register(
        "get_code_snippet",
        "Read source code of a specific function or class by name.",
        json!({
            "type": "object",
            "properties": {
                "symbol": { "type": "string" },
                "project": { "type": "string" }
            },
            "required": ["symbol"]
        }),
        GetCodeSnippetHandler {
            search_uc,
            query_uc: query_uc.clone(),
        },
    );

    registry.register(
        "code_search",
        "Search source file contents across the entire project for any text string.",
        json!({
            "type": "object",
            "properties": {
                "query": { "type": "string" },
                "project": { "type": "string" }
            },
            "required": ["query"]
        }),
        CodeSearchHandler { query_uc },
    );
}

struct CodeSearchHandler {
    query_uc: Arc<QueryGraphUseCase>,
}

#[async_trait]
impl ToolHandler for CodeSearchHandler {
    async fn call(&self, args: Value) -> Result<String, String> {
        let query = args
            .get("query")
            .and_then(|q| q.as_str())
            .filter(|q| !q.trim().is_empty())
            .ok_or_else(|| "Missing required argument: query".to_string())?;
        let project_arg = args.get("project").and_then(|p| p.as_str());
        let project_id = resolve_project(&self.query_uc, project_arg).await;

        let analysis = self
            .query_uc
            .get_analysis(&project_id)
            .await
            .map_err(|e| e.to_string())?;

        let mut matches = Vec::new();
        let query_lower = query.to_lowercase();

        let mut searched_files = std::collections::HashSet::new();
        if let Some(res) = analysis {
            for node in &res.graph.nodes {
                if let Some(fp) = &node.file_path {
                    if searched_files.insert(fp.clone()) {
                        let path = std::path::Path::new(fp);
                        if let Ok(content) = std::fs::read_to_string(path) {
                            for (line_idx, line) in content.lines().enumerate() {
                                if line.to_lowercase().contains(&query_lower) {
                                    matches.push(json!({
                                        "file": fp,
                                        "line": line_idx + 1,
                                        "content": line.trim(),
                                    }));
                                    if matches.len() >= 50 {
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    if matches.len() >= 50 {
                        break;
                    }
                }
            }
        }

        let res = json!({
            "project": project_id.as_str(),
            "query": query,
            "matches": matches,
            "count": matches.len(),
        });
        serde_json::to_string_pretty(&res).map_err(|e| e.to_string())
    }
}
