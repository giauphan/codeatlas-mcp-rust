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

    registry.register(
        "run_script",
        "Run a project script from package.json or Cargo.toml.",
        json!({
            "type": "object",
            "properties": {
                "script": { "type": "string" },
                "project": { "type": "string" }
            },
            "required": ["script"]
        }),
        RunScriptHandler,
    );

    registry.register(
        "git_changes",
        "Get recent git changes: uncommitted files and recent commit history.",
        json!({
            "type": "object",
            "properties": {
                "commits": { "type": "number" },
                "project": { "type": "string" }
            }
        }),
        GitChangesHandler,
    );
}

struct RunScriptHandler;
#[async_trait]
impl ToolHandler for RunScriptHandler {
    async fn call(&self, args: Value) -> Result<String, String> {
        let script = args
            .get("script")
            .and_then(|s| s.as_str())
            .filter(|s| !s.trim().is_empty())
            .ok_or_else(|| "Missing required argument: script".to_string())?;

        if !script
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            return Err(
                "Invalid script name: only alphanumeric, hyphen, and underscore allowed"
                    .to_string(),
            );
        }

        let is_npm = std::path::Path::new("package.json").exists();
        let is_cargo = std::path::Path::new("Cargo.toml").exists();

        let (cmd, cmd_args) = if is_npm {
            ("npm", vec!["run", script])
        } else if is_cargo {
            ("cargo", vec![script])
        } else {
            return Err(
                "Neither package.json nor Cargo.toml found in current directory".to_string(),
            );
        };

        let output = std::process::Command::new(cmd)
            .args(&cmd_args)
            .output()
            .map_err(|e| format!("Failed to execute script: {}", e))?;

        let res = json!({
            "command": format!("{} {}", cmd, cmd_args.join(" ")),
            "exitCode": output.status.code().unwrap_or(-1),
            "stdout": String::from_utf8_lossy(&output.stdout).trim(),
            "stderr": String::from_utf8_lossy(&output.stderr).trim(),
        });
        serde_json::to_string_pretty(&res).map_err(|e| e.to_string())
    }
}

struct GitChangesHandler;
#[async_trait]
impl ToolHandler for GitChangesHandler {
    async fn call(&self, args: Value) -> Result<String, String> {
        let commits = args.get("commits").and_then(|c| c.as_u64()).unwrap_or(5);
        let commits_arg = format!("-{}", commits.min(20));

        let status_out = std::process::Command::new("git")
            .args(["status", "-s"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default();

        let log_out = std::process::Command::new("git")
            .args(["log", &commits_arg, "--oneline"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default();

        let branch_out = std::process::Command::new("git")
            .args(["branch", "--show-current"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default();

        let res = json!({
            "branch": branch_out,
            "uncommittedChanges": status_out,
            "recentCommits": log_out,
        });
        serde_json::to_string_pretty(&res).map_err(|e| e.to_string())
    }
}
