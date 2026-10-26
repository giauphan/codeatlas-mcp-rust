pub mod adr;
pub mod brain;
pub mod graph;
pub mod insight;
pub mod security;

pub use adr::{AdrStatus, ADR};
pub use brain::{BrainContext, DreamCategory, DreamMemory, ImmuneGene};
pub use graph::{
    AnalysisResult, CompactEdge, CompactNode, EdgeType, EntityCounts, GraphData, GraphLink,
    GraphNode, NodeType,
};
pub use insight::{AIInsight, InsightSeverity, InsightType};
pub use security::SecurityFinding;
