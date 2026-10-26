use super::super::protocol::{ToolHandler, ToolRegistry};
use async_trait::async_trait;
use codeatlas_application::use_cases::second_brain::SecondBrainUseCase;
use codeatlas_domain::entities::brain::DreamMemory;
use serde_json::{json, Value};
use std::sync::Arc;

fn current_rfc3339() -> String {
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
}
