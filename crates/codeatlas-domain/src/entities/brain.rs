use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DreamCategory {
    Mistake,
    Preference,
    Knowledge,
    Pattern,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DreamMemory {
    pub id: String,
    pub category: String,
    pub content: String,
    pub project: String,
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImmuneGene {
    pub id: String,
    pub failure_pattern: String,
    pub fix_recommendation: String,
    pub severity: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrainContext {
    pub project: String,
    pub dream_memories: Vec<DreamMemory>,
    pub immune_genes: Vec<ImmuneGene>,
    pub system_memory: Option<String>,
}
