use super::super::protocol::{ToolHandler, ToolRegistry};
use async_trait::async_trait;
use codeatlas_application::use_cases::second_brain::SecondBrainUseCase;
use codeatlas_domain::entities::brain::ImmuneGene;
use serde_json::{json, Value};
use std::sync::Arc;

struct SearchGenomeHandler(Arc<SecondBrainUseCase>);
#[async_trait]
impl ToolHandler for SearchGenomeHandler {
    async fn call(&self, args: Value) -> Result<String, String> {
        let query = args
            .get("query")
            .and_then(|q| q.as_str())
            .filter(|q| !q.trim().is_empty())
            .ok_or_else(|| "Missing required argument: query".to_string())?;
        let limit = args.get("limit").and_then(|l| l.as_u64()).unwrap_or(10) as usize;

        let genes = self
            .0
            .search_genome(query, limit)
            .await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&genes).map_err(|e| e.to_string())
    }
}

struct GetGeneHandler(Arc<SecondBrainUseCase>);
#[async_trait]
impl ToolHandler for GetGeneHandler {
    async fn call(&self, args: Value) -> Result<String, String> {
        let gene_id = args
            .get("geneId")
            .and_then(|g| g.as_str())
            .filter(|g| !g.trim().is_empty())
            .ok_or_else(|| "Missing required argument: geneId".to_string())?;

        let genes = self
            .0
            .search_genome(gene_id, 10)
            .await
            .map_err(|e| e.to_string())?;

        let gene = genes.into_iter().find(|g| g.id == gene_id);
        match gene {
            Some(g) => serde_json::to_string_pretty(&g).map_err(|e| e.to_string()),
            None => Err(format!("Gene not found: {}", gene_id)),
        }
    }
}

struct ScanImmuneGenesHandler(Arc<SecondBrainUseCase>);
#[async_trait]
impl ToolHandler for ScanImmuneGenesHandler {
    async fn call(&self, args: Value) -> Result<String, String> {
        let problem = args
            .get("problem")
            .and_then(|p| p.as_str())
            .filter(|p| !p.trim().is_empty())
            .ok_or_else(|| "Missing required argument: problem".to_string())?;

        let genes = self
            .0
            .scan_immune_genes(problem)
            .await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&genes).map_err(|e| e.to_string())
    }
}

struct SaveImmuneGeneHandler(Arc<SecondBrainUseCase>);
#[async_trait]
impl ToolHandler for SaveImmuneGeneHandler {
    async fn call(&self, args: Value) -> Result<String, String> {
        let problem = args
            .get("problem")
            .and_then(|p| p.as_str())
            .filter(|p| !p.trim().is_empty())
            .ok_or_else(|| "Missing required argument: problem".to_string())?;
        let failure = args
            .get("failure")
            .and_then(|f| f.as_str())
            .filter(|f| !f.trim().is_empty())
            .ok_or_else(|| "Missing required argument: failure".to_string())?;
        let prevention = args
            .get("prevention")
            .and_then(|p| p.as_str())
            .filter(|p| !p.trim().is_empty())
            .ok_or_else(|| "Missing required argument: prevention".to_string())?;

        let project_arg = args.get("project").and_then(|p| p.as_str());

        let now_nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();

        let gene = ImmuneGene {
            id: format!("gene_{:x}", now_nanos),
            failure_pattern: format!("Problem: {}\nFailure: {}", problem, failure),
            fix_recommendation: prevention.to_string(),
            severity: "high".to_string(),
            project: project_arg.map(|p| p.to_string()),
        };

        self.0
            .save_immune_gene(&gene)
            .await
            .map_err(|e| e.to_string())?;

        let res = json!({
            "status": "saved",
            "geneId": gene.id,
        });
        serde_json::to_string_pretty(&res).map_err(|e| e.to_string())
    }
}

pub fn register_genome_tools(registry: &mut ToolRegistry, brain_uc: Arc<SecondBrainUseCase>) {
    registry.register(
        "search_genome",
        "Search CodeAtlas Genome for relevant immune genes.",
        json!({
            "type": "object",
            "properties": {
                "query": { "type": "string" },
                "limit": { "type": "number" },
                "project": { "type": "string" }
            },
            "required": ["query"]
        }),
        SearchGenomeHandler(brain_uc.clone()),
    );

    registry.register(
        "get_gene",
        "Get a specific gene by ID from the CodeAtlas Genome.",
        json!({
            "type": "object",
            "properties": {
                "geneId": { "type": "string" }
            },
            "required": ["geneId"]
        }),
        GetGeneHandler(brain_uc.clone()),
    );

    registry.register(
        "scan_immune_genes",
        "Scan the CodeAtlas Immune System for previously encountered failure patterns.",
        json!({
            "type": "object",
            "properties": {
                "problem": { "type": "string" },
                "project": { "type": "string" }
            },
            "required": ["problem"]
        }),
        ScanImmuneGenesHandler(brain_uc.clone()),
    );

    registry.register(
        "save_immune_gene",
        "Record a failure pattern as an immune gene in CodeAtlas Genome.",
        json!({
            "type": "object",
            "properties": {
                "problem": { "type": "string" },
                "failure": { "type": "string" },
                "prevention": { "type": "string" },
                "project": { "type": "string" }
            },
            "required": ["problem", "failure", "prevention"]
        }),
        SaveImmuneGeneHandler(brain_uc),
    );
}
