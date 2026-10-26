use super::super::protocol::{ToolHandler, ToolRegistry};
use super::analysis_tools::resolve_project;
use async_trait::async_trait;
use codeatlas_application::use_cases::query_graph::QueryGraphUseCase;
use codeatlas_application::use_cases::security_scan::SecurityScanUseCase;
use serde_json::{json, Value};
use std::sync::Arc;

struct ScanVulnHandler {
    security_uc: Arc<SecurityScanUseCase>,
    query_uc: Arc<QueryGraphUseCase>,
}

#[async_trait]
impl ToolHandler for ScanVulnHandler {
    async fn call(&self, args: Value) -> Result<String, String> {
        let project_arg = args.get("project").and_then(|p| p.as_str());
        let project_id = resolve_project(&self.query_uc, project_arg).await;
        let findings = self
            .security_uc
            .scan_project(&project_id)
            .await
            .map_err(|e| e.to_string())?;
        serde_json::to_string_pretty(&findings).map_err(|e| e.to_string())
    }
}

struct DetectSmellsHandler(Arc<QueryGraphUseCase>);
#[async_trait]
impl ToolHandler for DetectSmellsHandler {
    async fn call(&self, args: Value) -> Result<String, String> {
        let project_arg = args.get("project").and_then(|p| p.as_str());
        let project_id = resolve_project(&self.0, project_arg).await;
        let cycles = self
            .0
            .detect_cycles(&project_id)
            .await
            .map_err(|e| e.to_string())?;
        let res = json!({
            "project": project_id.as_str(),
            "circularDependencies": cycles,
            "cyclesCount": cycles.len(),
        });
        serde_json::to_string_pretty(&res).map_err(|e| e.to_string())
    }
}

pub fn register_security_tools(
    registry: &mut ToolRegistry,
    security_uc: Arc<SecurityScanUseCase>,
    query_uc: Arc<QueryGraphUseCase>,
) {
    registry.register(
        "scan_enterprise_vulnerabilities",
        "Scan analyzed projects for security vulnerabilities and hardcoded secrets.",
        json!({
            "type": "object",
            "properties": { "project": { "type": "string" } }
        }),
        ScanVulnHandler {
            security_uc,
            query_uc: query_uc.clone(),
        },
    );

    registry.register(
        "detect_architectural_smells",
        "Automatically detect architectural weaknesses, circular dependencies, and smells.",
        json!({
            "type": "object",
            "properties": { "project": { "type": "string" } }
        }),
        DetectSmellsHandler(query_uc),
    );
}
