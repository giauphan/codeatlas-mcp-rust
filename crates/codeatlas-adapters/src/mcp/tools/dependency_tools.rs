use super::super::protocol::{ToolHandler, ToolRegistry};
use super::analysis_tools::resolve_project;
use async_trait::async_trait;
use codeatlas_application::use_cases::query_graph::QueryGraphUseCase;
use serde_json::{json, Value};
use std::sync::Arc;

struct GetCallersHandler(Arc<QueryGraphUseCase>);
#[async_trait]
impl ToolHandler for GetCallersHandler {
    async fn call(&self, args: Value) -> Result<String, String> {
        let symbol = args.get("symbol").and_then(|s| s.as_str()).unwrap_or("");
        let project_arg = args.get("project").and_then(|p| p.as_str());
        let project_id = resolve_project(&self.0, project_arg).await;
        let callers = self
            .0
            .get_callers(&project_id, symbol)
            .await
            .map_err(|e| e.to_string())?;
        serde_json::to_string_pretty(&callers).map_err(|e| e.to_string())
    }
}

struct GetCalleesHandler(Arc<QueryGraphUseCase>);
#[async_trait]
impl ToolHandler for GetCalleesHandler {
    async fn call(&self, args: Value) -> Result<String, String> {
        let symbol = args.get("symbol").and_then(|s| s.as_str()).unwrap_or("");
        let project_arg = args.get("project").and_then(|p| p.as_str());
        let project_id = resolve_project(&self.0, project_arg).await;
        let callees = self
            .0
            .get_callees(&project_id, symbol)
            .await
            .map_err(|e| e.to_string())?;
        serde_json::to_string_pretty(&callees).map_err(|e| e.to_string())
    }
}

struct GetDependenciesHandler(Arc<QueryGraphUseCase>);
#[async_trait]
impl ToolHandler for GetDependenciesHandler {
    async fn call(&self, args: Value) -> Result<String, String> {
        let symbol = args.get("symbol").and_then(|s| s.as_str()).unwrap_or("");
        let depth = args.get("depth").and_then(|d| d.as_u64()).unwrap_or(2) as u32;
        let project_arg = args.get("project").and_then(|p| p.as_str());
        let project_id = resolve_project(&self.0, project_arg).await;
        let deps = self
            .0
            .get_dependencies(&project_id, symbol, depth)
            .await
            .map_err(|e| e.to_string())?;
        serde_json::to_string_pretty(&deps).map_err(|e| e.to_string())
    }
}

struct SystemFlowHandler(Arc<QueryGraphUseCase>);
#[async_trait]
impl ToolHandler for SystemFlowHandler {
    async fn call(&self, args: Value) -> Result<String, String> {
        let project_arg = args.get("project").and_then(|p| p.as_str());
        let project_id = resolve_project(&self.0, project_arg).await;
        let max_nodes = args.get("maxNodes").and_then(|m| m.as_u64()).unwrap_or(50) as usize;
        self.0
            .generate_system_flow(&project_id, max_nodes)
            .await
            .map_err(|e| e.to_string())
    }
}

struct FeatureFlowHandler(Arc<QueryGraphUseCase>);
#[async_trait]
impl ToolHandler for FeatureFlowHandler {
    async fn call(&self, args: Value) -> Result<String, String> {
        let project_arg = args.get("project").and_then(|p| p.as_str());
        let project_id = resolve_project(&self.0, project_arg).await;
        let keyword = args.get("keyword").and_then(|k| k.as_str()).unwrap_or("");
        let max_depth = args.get("maxDepth").and_then(|m| m.as_u64()).unwrap_or(3) as u32;
        self.0
            .generate_feature_flow(&project_id, keyword, max_depth)
            .await
            .map_err(|e| e.to_string())
    }
}

pub fn register_dependency_tools(registry: &mut ToolRegistry, query_uc: Arc<QueryGraphUseCase>) {
    registry.register(
        "get_callers",
        "Find all functions or classes that call or reference a symbol.",
        json!({
            "type": "object",
            "properties": {
                "symbol": { "type": "string" },
                "project": { "type": "string" }
            },
            "required": ["symbol"]
        }),
        GetCallersHandler(query_uc.clone()),
    );

    registry.register(
        "get_callees",
        "Find everything a function or class calls or depends on.",
        json!({
            "type": "object",
            "properties": {
                "symbol": { "type": "string" },
                "project": { "type": "string" }
            },
            "required": ["symbol"]
        }),
        GetCalleesHandler(query_uc.clone()),
    );

    registry.register(
        "get_dependencies",
        "Get import/call relationships for a symbol.",
        json!({
            "type": "object",
            "properties": {
                "symbol": { "type": "string" },
                "depth": { "type": "number" },
                "project": { "type": "string" }
            },
            "required": ["symbol"]
        }),
        GetDependenciesHandler(query_uc.clone()),
    );

    registry.register(
        "generate_system_flow",
        "Auto-generate Mermaid flowchart showing module connections.",
        json!({
            "type": "object",
            "properties": {
                "project": { "type": "string" },
                "maxNodes": { "type": "number" }
            }
        }),
        SystemFlowHandler(query_uc.clone()),
    );

    registry.register(
        "generate_feature_flow_diagram",
        "Generate Mermaid diagram showing the runtime execution flow of a feature.",
        json!({
            "type": "object",
            "properties": {
                "keyword": { "type": "string" },
                "project": { "type": "string" },
                "maxDepth": { "type": "number" }
            },
            "required": ["keyword"]
        }),
        FeatureFlowHandler(query_uc.clone()),
    );

    registry.register(
        "trace_feature_flow",
        "Trace the complete execution flow of a feature keyword.",
        json!({
            "type": "object",
            "properties": {
                "keyword": { "type": "string" },
                "project": { "type": "string" }
            },
            "required": ["keyword"]
        }),
        FeatureFlowHandler(query_uc.clone()),
    );

    registry.register(
        "impact_analysis",
        "Full blast radius analysis for changing a symbol. Traces both callers and callees in one view.",
        json!({
            "type": "object",
            "properties": {
                "symbol": { "type": "string" },
                "project": { "type": "string" }
            },
            "required": ["symbol"]
        }),
        ImpactAnalysisHandler(query_uc),
    );
}

struct ImpactAnalysisHandler(Arc<QueryGraphUseCase>);
#[async_trait]
impl ToolHandler for ImpactAnalysisHandler {
    async fn call(&self, args: Value) -> Result<String, String> {
        let symbol = args
            .get("symbol")
            .and_then(|s| s.as_str())
            .filter(|s| !s.trim().is_empty())
            .ok_or_else(|| "Missing required argument: symbol".to_string())?;
        let project_arg = args.get("project").and_then(|p| p.as_str());
        let project_id = resolve_project(&self.0, project_arg).await;

        let callers = self
            .0
            .get_callers(&project_id, symbol)
            .await
            .map_err(|e| e.to_string())?;
        let callees = self
            .0
            .get_callees(&project_id, symbol)
            .await
            .map_err(|e| e.to_string())?;

        let callers_count = callers.len();
        let callees_count = callees.len();
        let blast_radius = callers_count + callees_count;

        let res = json!({
            "project": project_id.as_str(),
            "symbol": symbol,
            "blastRadiusScore": blast_radius,
            "upstreamDependents": callers,
            "upstreamCount": callers_count,
            "downstreamDependencies": callees,
            "downstreamCount": callees_count,
        });
        serde_json::to_string_pretty(&res).map_err(|e| e.to_string())
    }
}
