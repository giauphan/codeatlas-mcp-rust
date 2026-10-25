use crate::entities::graph::GraphData;
use crate::value_objects::file_path::SanitizedPath;
use async_trait::async_trait;
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("I/O error during file reading: {0}")]
    Io(#[from] std::io::Error),
    #[error("Tree-sitter parse failure: {0}")]
    TreeSitter(String),
    #[error("Unsupported language for path: {0}")]
    UnsupportedLanguage(String),
    #[error("Internal parser error: {0}")]
    Internal(String),
}

#[derive(Debug, Clone)]
pub struct ParsedFile {
    pub file_path: String,
    pub language: String,
    pub nodes: Vec<crate::entities::graph::GraphNode>,
    pub links: Vec<crate::entities::graph::GraphLink>,
}

#[async_trait]
pub trait AstParserPort: Send + Sync {
    async fn parse_file(&self, path: &Path, content: &str) -> Result<ParsedFile, ParserError>;
    async fn parse_batch(&self, paths: &[SanitizedPath]) -> Result<GraphData, ParserError>;
    fn supports_language(&self, path: &Path) -> bool;
}
