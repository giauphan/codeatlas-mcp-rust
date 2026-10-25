use super::super::protocol::{ToolHandler, ToolRegistry};
use async_trait::async_trait;
use codeatlas_application::use_cases::search_code::SearchCodeUseCase;
use codeatlas_domain::value_objects::project_id::ProjectId;
use serde_json::{json, Value};
use std::sync::Arc;

struct SearchEntitiesHandler(Arc<SearchCodeUseCase>);
#[async_trait]
impl ToolHandler for SearchEntitiesHandler {
    async fn call(&self, args: Value) -> Result<String, String> {
        let query = args.get("query").and_then(|q| q.as_str()).unwrap_or("");
        let project = args
            .get("project")
            .and_then(|p| p.as_str())
            .unwrap_or("default");
        let project_id = ProjectId::new(project);
        let matches = self
            .0
            .search_entities(&project_id, query)
            .await
            .map_err(|e| e.to_string())?;
        serde_json::to_string_pretty(&matches).map_err(|e| e.to_string())
    }
}

struct GetCodeSnippetHandler(Arc<SearchCodeUseCase>);
#[async_trait]
impl ToolHandler for GetCodeSnippetHandler {
    async fn call(&self, args: Value) -> Result<String, String> {
        let symbol = args.get("symbol").and_then(|s| s.as_str()).unwrap_or("");
        let project = args
            .get("project")
            .and_then(|p| p.as_str())
            .unwrap_or("default");
        let project_id = ProjectId::new(project);
        let snippet = self
            .0
            .get_code_snippet(&project_id, symbol)
            .await
            .map_err(|e| e.to_string())?;
        match snippet {
            Some(s) => Ok(s),
            None => Err(format!("Symbol snippet not found: {}", symbol)),
        }
    }
}

pub fn register_search_tools(registry: &mut ToolRegistry, search_uc: Arc<SearchCodeUseCase>) {
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
        SearchEntitiesHandler(search_uc.clone()),
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
        GetCodeSnippetHandler(search_uc.clone()),
    );
}
