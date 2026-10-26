use super::super::protocol::{ToolHandler, ToolRegistry};
use super::analysis_tools::resolve_project;
use async_trait::async_trait;
use codeatlas_application::use_cases::manage_adr::ManageAdrUseCase;
use codeatlas_application::use_cases::query_graph::QueryGraphUseCase;
use codeatlas_domain::entities::adr::ADR;
use serde_json::{json, Value};
use std::sync::Arc;

fn require_id(args: &Value) -> Result<&str, String> {
    args.get("id")
        .and_then(|i| i.as_str())
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| "Missing required argument: id".to_string())
}

struct ManageAdrHandler {
    adr_uc: Arc<ManageAdrUseCase>,
    query_uc: Arc<QueryGraphUseCase>,
}

#[async_trait]
impl ToolHandler for ManageAdrHandler {
    async fn call(&self, args: Value) -> Result<String, String> {
        let action = args
            .get("action")
            .and_then(|a| a.as_str())
            .unwrap_or("list");
        let project_arg = args.get("project").and_then(|p| p.as_str());
        let project_id = resolve_project(&self.query_uc, project_arg).await;
        let project = project_id.as_str();

        match action {
            "list" => {
                let adrs = self
                    .adr_uc
                    .list(Some(project))
                    .await
                    .map_err(|e| e.to_string())?;
                serde_json::to_string_pretty(&adrs).map_err(|e| e.to_string())
            }
            "get" => {
                let id = require_id(&args)?;
                let adr = self
                    .adr_uc
                    .get(project, id)
                    .await
                    .map_err(|e| e.to_string())?;
                match adr {
                    Some(a) => serde_json::to_string_pretty(&a).map_err(|e| e.to_string()),
                    None => Err(format!("ADR not found: {}", id)),
                }
            }
            "save" => {
                let id = require_id(&args)?;

                let existing = self
                    .adr_uc
                    .get(project, id)
                    .await
                    .map_err(|e| e.to_string())?;

                let title = args
                    .get("title")
                    .and_then(|t| t.as_str())
                    .map(|s| s.to_string())
                    .or_else(|| existing.as_ref().map(|e| e.title.clone()))
                    .unwrap_or_default();

                let decision = args
                    .get("decision")
                    .and_then(|d| d.as_str())
                    .map(|s| s.to_string())
                    .or_else(|| existing.as_ref().map(|e| e.decision.clone()))
                    .unwrap_or_default();

                let status = args
                    .get("status")
                    .and_then(|s| s.as_str())
                    .map(|s| s.to_string())
                    .or_else(|| existing.as_ref().map(|e| e.status.clone()))
                    .unwrap_or_else(|| "proposed".to_string());

                let context = args
                    .get("context")
                    .and_then(|c| c.as_str())
                    .map(|s| s.to_string())
                    .or_else(|| existing.as_ref().map(|e| e.context.clone()))
                    .unwrap_or_default();

                let consequences = args
                    .get("consequences")
                    .and_then(|c| c.as_str())
                    .map(|s| s.to_string())
                    .or_else(|| existing.as_ref().map(|e| e.consequences.clone()))
                    .unwrap_or_default();

                let date = match args.get("date") {
                    Some(Value::Null) => None,
                    Some(Value::String(s)) => Some(s.clone()),
                    Some(_) => existing.as_ref().and_then(|e| e.date.clone()),
                    None => existing.as_ref().and_then(|e| e.date.clone()),
                };

                let superseded_by = match args.get("supersededBy") {
                    Some(Value::Null) => None,
                    Some(Value::String(s)) => Some(s.clone()),
                    Some(_) => existing.as_ref().and_then(|e| e.superseded_by.clone()),
                    None => existing.as_ref().and_then(|e| e.superseded_by.clone()),
                };

                let adr = ADR {
                    id: id.to_string(),
                    title,
                    status,
                    context,
                    decision,
                    consequences,
                    project: project.to_string(),
                    date,
                    superseded_by,
                };
                let saved = self.adr_uc.save(adr).await.map_err(|e| e.to_string())?;
                serde_json::to_string_pretty(&saved).map_err(|e| e.to_string())
            }
            "delete" => {
                let id = require_id(&args)?;
                let ok = self
                    .adr_uc
                    .delete(project, id)
                    .await
                    .map_err(|e| e.to_string())?;
                Ok(json!({ "deleted": ok }).to_string())
            }
            _ => Err(format!("Unknown action: {}", action)),
        }
    }
}

struct RouteTaskHandler;
#[async_trait]
impl ToolHandler for RouteTaskHandler {
    async fn call(&self, args: Value) -> Result<String, String> {
        let task = args.get("task").and_then(|t| t.as_str()).unwrap_or("");
        let lower = task.to_lowercase();
        let (model, effort) = if lower.contains("architect")
            || lower.contains("refactor")
            || lower.contains("migrate")
        {
            ("Claude Opus 4.6", "high")
        } else {
            ("Gemini 3.8 Flash", "medium")
        };
        Ok(json!({ "model": model, "effort": effort }).to_string())
    }
}

pub fn register_workspace_tools(
    registry: &mut ToolRegistry,
    adr_uc: Arc<ManageAdrUseCase>,
    query_uc: Arc<QueryGraphUseCase>,
) {
    registry.register(
        "manage_adr",
        "Manage Architecture Decision Records (ADRs). Actions: list, get, save, delete.",
        json!({
            "type": "object",
            "properties": {
                "action": { "type": "string" },
                "project": { "type": "string" },
                "id": { "type": "string" },
                "title": { "type": "string" },
                "decision": { "type": "string" },
                "status": { "type": "string" },
                "context": { "type": "string" },
                "consequences": { "type": "string" },
                "date": { "type": ["string", "null"] },
                "supersededBy": { "type": ["string", "null"] }
            },
            "required": ["action"]
        }),
        ManageAdrHandler { adr_uc, query_uc },
    );

    registry.register(
        "route_task",
        "Suggest a model and effort level for a coding task.",
        json!({
            "type": "object",
            "properties": { "task": { "type": "string" } },
            "required": ["task"]
        }),
        RouteTaskHandler,
    );
}
