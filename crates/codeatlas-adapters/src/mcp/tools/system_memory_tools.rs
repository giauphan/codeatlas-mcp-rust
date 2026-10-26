use super::super::protocol::{ToolHandler, ToolRegistry};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::path::PathBuf;

fn resolve_memory_dir(project_arg: Option<&str>) -> PathBuf {
    let base = std::path::Path::new(project_arg.unwrap_or("."));
    base.join(".agents").join("memory")
}

struct SyncSystemMemoryHandler;
#[async_trait]
impl ToolHandler for SyncSystemMemoryHandler {
    async fn call(&self, args: Value) -> Result<String, String> {
        let business_rule = args
            .get("businessRule")
            .and_then(|b| b.as_str())
            .filter(|s| !s.trim().is_empty());
        let change_log = args
            .get("changeLog")
            .and_then(|c| c.as_str())
            .filter(|s| !s.trim().is_empty());

        if business_rule.is_none() && change_log.is_none() {
            return Err("At least one of businessRule or changeLog must be supplied".to_string());
        }

        let project_arg = args.get("project").and_then(|p| p.as_str());
        let dir = resolve_memory_dir(project_arg);
        if !dir.exists() {
            std::fs::create_dir_all(&dir)
                .map_err(|e| format!("Failed to create directory {:?}: {}", dir, e))?;
        }

        let flow_file = dir.join("system-flow.md");
        let mut content = if flow_file.exists() {
            std::fs::read_to_string(&flow_file)
                .map_err(|e| format!("Failed to read memory file {:?}: {}", flow_file, e))?
        } else {
            "# System Architecture & Memory Checkpoint\n\n".to_string()
        };

        if let Some(br) = business_rule {
            content.push_str(&format!("## Business Rules\n{}\n\n", br));
        }
        if let Some(cl) = change_log {
            content.push_str(&format!("## Change Log\n{}\n\n", cl));
        }

        let tmp_file = dir.join(format!("system-flow.tmp.{}", std::process::id()));
        std::fs::write(&tmp_file, &content)
            .map_err(|e| format!("Failed to write temporary memory file: {}", e))?;
        std::fs::rename(&tmp_file, &flow_file)
            .map_err(|e| format!("Failed to finalize memory file: {}", e))?;

        let res = json!({
            "status": "synced",
            "path": flow_file.to_string_lossy(),
        });
        serde_json::to_string_pretty(&res).map_err(|e| e.to_string())
    }
}

struct GetSystemMemoryHandler;
#[async_trait]
impl ToolHandler for GetSystemMemoryHandler {
    async fn call(&self, args: Value) -> Result<String, String> {
        let project_arg = args.get("project").and_then(|p| p.as_str());
        let dir = resolve_memory_dir(project_arg);
        let flow_file = dir.join("system-flow.md");

        if flow_file.exists() {
            let content = std::fs::read_to_string(&flow_file)
                .map_err(|e| format!("Failed to read memory file: {}", e))?;
            let res = json!({
                "status": "found",
                "path": flow_file.to_string_lossy(),
                "content": content,
            });
            serde_json::to_string_pretty(&res).map_err(|e| e.to_string())
        } else {
            let res = json!({
                "status": "not_found",
                "message": "No system memory found. Run 'sync_system_memory' to generate it.",
            });
            serde_json::to_string_pretty(&res).map_err(|e| e.to_string())
        }
    }
}

struct MemoryBootstrapHandler;
#[async_trait]
impl ToolHandler for MemoryBootstrapHandler {
    async fn call(&self, args: Value) -> Result<String, String> {
        let project_arg = args.get("project").and_then(|p| p.as_str());
        let dir = resolve_memory_dir(project_arg);
        let flow_file = dir.join("system-flow.md");

        if flow_file.exists() {
            let content = std::fs::read_to_string(&flow_file)
                .map_err(|e| format!("Failed to read checkpoint: {}", e))?;
            let res = json!({
                "status": "ready",
                "memoryAvailable": true,
                "memoryDir": dir.to_string_lossy(),
                "checkpoint": content,
            });
            serde_json::to_string_pretty(&res).map_err(|e| e.to_string())
        } else {
            let res = json!({
                "status": "not_found",
                "memoryAvailable": false,
                "memoryDir": dir.to_string_lossy(),
                "message": "No checkpoint found at system-flow.md",
            });
            serde_json::to_string_pretty(&res).map_err(|e| e.to_string())
        }
    }
}

pub fn register_system_memory_tools(registry: &mut ToolRegistry) {
    registry.register(
        "sync_system_memory",
        "Create or update .agents/memory/ with auto-generated system documentation.",
        json!({
            "type": "object",
            "properties": {
                "project": { "type": "string" },
                "businessRule": { "type": "string" },
                "changeLog": { "type": "string" }
            }
        }),
        SyncSystemMemoryHandler,
    );

    registry.register(
        "get_system_memory",
        "Retrieve the auto-generated system documentation and episodic memories.",
        json!({
            "type": "object",
            "properties": {
                "project": { "type": "string" },
                "eventType": { "type": "string" }
            }
        }),
        GetSystemMemoryHandler,
    );

    registry.register(
        "memory_bootstrap",
        "Fetch the latest verified checkpoint for this project as an explicit MCP tool response.",
        json!({
            "type": "object",
            "properties": {
                "project": { "type": "string" }
            }
        }),
        MemoryBootstrapHandler,
    );
}
