use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

#[async_trait]
pub trait ToolHandler: Send + Sync {
    async fn call(&self, arguments: Value) -> Result<String, String>;
}

#[derive(Default)]
pub struct ToolRegistry {
    definitions: Vec<ToolDefinition>,
    handlers: HashMap<String, Arc<dyn ToolHandler>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            definitions: Vec::new(),
            handlers: HashMap::new(),
        }
    }

    pub fn register<H>(&mut self, name: &str, description: &str, schema: Value, handler: H)
    where
        H: ToolHandler + 'static,
    {
        self.definitions.push(ToolDefinition {
            name: name.to_string(),
            description: description.to_string(),
            input_schema: schema,
        });
        self.handlers.insert(name.to_string(), Arc::new(handler));
    }

    pub fn list_tools(&self) -> &[ToolDefinition] {
        &self.definitions
    }

    pub async fn dispatch(&self, name: &str, args: Value) -> Result<String, String> {
        let handler = self
            .handlers
            .get(name)
            .ok_or_else(|| format!("Unknown tool: {}", name))?;
        handler.call(args).await
    }
}
