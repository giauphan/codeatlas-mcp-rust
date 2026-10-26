use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InsightSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InsightType {
    Refactor,
    Security,
    Maintainability,
    Performance,
    Architecture,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AIInsight {
    pub id: String,
    #[serde(rename = "type")]
    pub insight_type: String,
    pub title: String,
    pub description: String,
    pub severity: String,
    pub affected_nodes: Vec<String>,
}
