use lasso::Spur;
use serde::{Deserialize, Serialize};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NodeType {
    Module = 0,
    Function = 1,
    Class = 2,
    Variable = 3,
}

impl NodeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            NodeType::Module => "module",
            NodeType::Function => "function",
            NodeType::Class => "class",
            NodeType::Variable => "variable",
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EdgeType {
    Import = 0,
    Call = 1,
    Contains = 2,
    Implements = 3,
}

impl EdgeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EdgeType::Import => "import",
            EdgeType::Call => "call",
            EdgeType::Contains => "contains",
            EdgeType::Implements => "implements",
        }
    }
}

/// Compact 16-byte aligned graph node for minimal RAM usage
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CompactNode {
    pub label: Spur,
    pub file_path: Spur,
    pub line: u32,
    pub node_type: NodeType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CompactEdge {
    pub edge_type: EdgeType,
}

/// JSON-compatible Graph Node representation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    #[serde(rename = "type")]
    pub node_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub val: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
}

/// JSON-compatible Graph Link representation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphLink {
    pub source: String,
    pub target: String,
    #[serde(rename = "type")]
    pub link_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct GraphData {
    pub nodes: Vec<GraphNode>,
    pub links: Vec<GraphLink>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntityCounts {
    pub modules: usize,
    pub functions: usize,
    pub classes: usize,
    pub variables: usize,
    pub dependencies: usize,
    pub circular_deps: usize,
    pub dead_code: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisResult {
    pub graph: GraphData,
    pub insights: Vec<crate::entities::insight::AIInsight>,
    pub entity_counts: EntityCounts,
    pub total_files_analyzed: usize,
    pub total_files_skipped: usize,
}
