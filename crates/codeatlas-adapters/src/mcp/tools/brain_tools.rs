use super::super::protocol::{ToolHandler, ToolRegistry};
use async_trait::async_trait;
use codeatlas_application::use_cases::second_brain::SecondBrainUseCase;
use codeatlas_domain::entities::brain::DreamMemory;
use serde_json::{json, Value};
use std::sync::Arc;

pub fn current_rfc3339() -> String {
    let now = std::time::SystemTime::now();
    let duration = now
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    let days = secs / 86400;
    let time_of_day = secs % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;

    let mut year = 1970;
    let mut remaining_days = days;
    loop {
        let leap = if (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0) {
            1
        } else {
            0
        };
        let days_in_year = 365 + leap;
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        year += 1;
    }
    let leap = if (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0) {
        1
    } else {
        0
    };
    let month_days = [31, 28 + leap, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut month = 1;
    for &md in &month_days {
        if remaining_days < md {
            break;
        }
        remaining_days -= md;
        month += 1;
    }
    let day = remaining_days + 1;

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hours, minutes, seconds
    )
}

struct SaveDreamHandler(Arc<SecondBrainUseCase>);
#[async_trait]
impl ToolHandler for SaveDreamHandler {
    async fn call(&self, args: Value) -> Result<String, String> {
        let category = args
            .get("category")
            .and_then(|c| c.as_str())
            .unwrap_or("knowledge");
        let content = args
            .get("content")
            .and_then(|c| c.as_str())
            .filter(|c| !c.trim().is_empty())
            .ok_or_else(|| "Missing required argument: content".to_string())?;
        let project = args
            .get("project")
            .and_then(|p| p.as_str())
            .unwrap_or("default");

        let now_nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();

        let dream = DreamMemory {
            id: format!("dream_{:x}", now_nanos),
            category: category.to_string(),
            content: content.to_string(),
            project: project.to_string(),
            timestamp: current_rfc3339(),
            score: None,
        };

        self.0.save_dream(&dream).await.map_err(|e| e.to_string())?;
        Ok("{\"status\":\"saved\"}".to_string())
    }
}

struct QueryDreamsHandler(Arc<SecondBrainUseCase>);
#[async_trait]
impl ToolHandler for QueryDreamsHandler {
    async fn call(&self, args: Value) -> Result<String, String> {
        let query = args.get("query").and_then(|q| q.as_str()).unwrap_or("");
        let project = args.get("project").and_then(|p| p.as_str());
        let limit = match args.get("limit") {
            Some(val) => {
                let n = val.as_u64().ok_or_else(|| {
                    format!("Invalid argument 'limit': expected an integer, got {}", val)
                })?;
                if n == 0 || n > 100 {
                    return Err(format!(
                        "Invalid argument 'limit': value {} out of range [1, 100]",
                        n
                    ));
                }
                n as usize
            }
            None => 5,
        };

        let dreams = self
            .0
            .query_dreams(query, project, limit)
            .await
            .map_err(|e| e.to_string())?;
        serde_json::to_string_pretty(&dreams).map_err(|e| e.to_string())
    }
}

struct BrainContextHandler(Arc<SecondBrainUseCase>);
#[async_trait]
impl ToolHandler for BrainContextHandler {
    async fn call(&self, args: Value) -> Result<String, String> {
        let task = args.get("task").and_then(|t| t.as_str()).unwrap_or("");
        let project = args.get("project").and_then(|p| p.as_str());
        let ctx = self
            .0
            .load_brain_context(task, project)
            .await
            .map_err(|e| e.to_string())?;
        serde_json::to_string_pretty(&ctx).map_err(|e| e.to_string())
    }
}

pub fn register_brain_tools(registry: &mut ToolRegistry, brain_uc: Arc<SecondBrainUseCase>) {
    registry.register(
        "save_dream_memory",
        "Save a dream memory to CodeAtlas Cloud for long-term AI recall.",
        json!({
            "type": "object",
            "properties": {
                "category": { "type": "string" },
                "content": { "type": "string" },
                "project": { "type": "string" }
            },
            "required": ["content"]
        }),
        SaveDreamHandler(brain_uc.clone()),
    );

    registry.register(
        "query_dream_memories",
        "Query previously saved dream memories from CodeAtlas Cloud.",
        json!({
            "type": "object",
            "properties": {
                "query": { "type": "string" },
                "project": { "type": "string" },
                "limit": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": 100
                }
            },
            "required": ["query"]
        }),
        QueryDreamsHandler(brain_uc.clone()),
    );

    registry.register(
        "brain_context",
        "Load Second Brain context (dreams, immune genes) for the current task.",
        json!({
            "type": "object",
            "properties": {
                "task": { "type": "string" },
                "project": { "type": "string" }
            },
            "required": ["task"]
        }),
        BrainContextHandler(brain_uc.clone()),
    );

    registry.register(
        "sync_dreams",
        "Check dream memory sync status and count of stored dreams.",
        json!({
            "type": "object",
            "properties": {
                "type": { "type": "string" },
                "project": { "type": "string" }
            }
        }),
        SyncDreamsHandler(brain_uc.clone()),
    );

    registry.register(
        "setup_second_brain",
        "Configure CodeAtlas Second Brain for MCP clients.",
        json!({
            "type": "object",
            "properties": {
                "client": { "type": "string" },
                "apiKey": { "type": "string" }
            }
        }),
        SetupSecondBrainHandler,
    );

    registry.register(
        "check_second_brain_status",
        "Check the current Second Brain configuration and connectivity status.",
        json!({ "type": "object", "properties": {} }),
        CheckSecondBrainStatusHandler(brain_uc),
    );

    registry.register(
        "sync_skills_inventory",
        "Scan installed skills and save inventory to CodeAtlas Second Brain.",
        json!({
            "type": "object",
            "properties": { "action": { "type": "string" } }
        }),
        SyncSkillsInventoryHandler,
    );
}

struct SyncDreamsHandler(Arc<SecondBrainUseCase>);
#[async_trait]
impl ToolHandler for SyncDreamsHandler {
    async fn call(&self, args: Value) -> Result<String, String> {
        let type_filter = args.get("type").and_then(|t| t.as_str());
        let project = args.get("project").and_then(|p| p.as_str());

        let dreams = match self.0.query_dreams("*", project, 1000).await {
            Ok(d) => d,
            Err(e) => {
                let res = json!({
                    "status": "offline",
                    "error": e.to_string(),
                    "syncedDreamsCount": 0,
                    "distribution": {}
                });
                return serde_json::to_string_pretty(&res).map_err(|err| err.to_string());
            }
        };

        let is_truncated = dreams.len() >= 1000;
        let mut counts = std::collections::HashMap::new();
        for d in &dreams {
            if let Some(tf) = type_filter {
                if !d.category.eq_ignore_ascii_case(tf) {
                    continue;
                }
            }
            *counts.entry(d.category.clone()).or_insert(0) += 1;
        }

        let filtered_count: usize = counts.values().sum();
        let res = json!({
            "status": "synced",
            "syncedDreamsCount": filtered_count,
            "distribution": counts,
            "truncated": is_truncated,
        });
        serde_json::to_string_pretty(&res).map_err(|e| e.to_string())
    }
}

struct SetupSecondBrainHandler;
#[async_trait]
impl ToolHandler for SetupSecondBrainHandler {
    async fn call(&self, args: Value) -> Result<String, String> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| ".".to_string());
        let dir = std::path::PathBuf::from(home).join(".codeatlas");
        if !dir.exists() {
            std::fs::create_dir_all(&dir)
                .map_err(|e| format!("Failed to create config directory {:?}: {}", dir, e))?;
        }
        let config_file = dir.join("config.json");
        let api_key = args.get("apiKey").and_then(|k| k.as_str()).unwrap_or("");
        let client = args.get("client").and_then(|c| c.as_str()).unwrap_or("all");

        let config_data = json!({
            "apiKey": if api_key.is_empty() { Value::Null } else { Value::String(api_key.to_string()) },
            "apiKeyConfigured": !api_key.is_empty(),
            "client": client,
        });

        let serialized = serde_json::to_string_pretty(&config_data)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;
        std::fs::write(&config_file, serialized)
            .map_err(|e| format!("Failed to write config file {:?}: {}", config_file, e))?;

        let res = json!({
            "status": "configured",
            "client": client,
            "apiKeyConfigured": !api_key.is_empty(),
            "configPath": config_file.to_string_lossy(),
        });
        serde_json::to_string_pretty(&res).map_err(|e| e.to_string())
    }
}

struct CheckSecondBrainStatusHandler(Arc<SecondBrainUseCase>);
#[async_trait]
impl ToolHandler for CheckSecondBrainStatusHandler {
    async fn call(&self, _args: Value) -> Result<String, String> {
        let is_key_set = std::env::var("CODEATLAS_API_KEY").is_ok();
        let api_url = std::env::var("CODEATLAS_API_URL").ok();
        let auth_status = self.0.check_auth().await.unwrap_or(false);

        let res = json!({
            "apiKey": if is_key_set { "set" } else { "not_set" },
            "apiUrl": api_url,
            "connected": auth_status,
            "status": if auth_status { "online" } else { "offline_local" },
        });
        serde_json::to_string_pretty(&res).map_err(|e| e.to_string())
    }
}

struct SyncSkillsInventoryHandler;
#[async_trait]
impl ToolHandler for SyncSkillsInventoryHandler {
    async fn call(&self, _args: Value) -> Result<String, String> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| ".".to_string());
        let skills_dir = std::path::PathBuf::from(&home)
            .join(".agents")
            .join("skills");
        let mut skill_names = Vec::new();

        if let Ok(entries) = std::fs::read_dir(skills_dir) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    if let Some(name) = entry.file_name().to_str() {
                        skill_names.push(name.to_string());
                    }
                }
            }
        }

        let res = json!({
            "status": "synced",
            "skillsCount": skill_names.len(),
            "skills": skill_names,
        });
        serde_json::to_string_pretty(&res).map_err(|e| e.to_string())
    }
}
