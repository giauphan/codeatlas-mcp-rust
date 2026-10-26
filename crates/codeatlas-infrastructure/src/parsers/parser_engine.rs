use async_trait::async_trait;
use codeatlas_domain::entities::graph::{GraphData, GraphLink, GraphNode};
use codeatlas_domain::ports::analyzer::{AstParserPort, ParsedFile, ParserError};
use codeatlas_domain::value_objects::file_path::SanitizedPath;
use rayon::prelude::*;
use std::path::Path;

use super::php_parser::PhpParser;
use super::py_parser::PythonParser;
use super::source_buffer::SourceBuffer;
use super::ts_parser::TypeScriptParser;

pub struct TreeSitterParserEngine;

impl TreeSitterParserEngine {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TreeSitterParserEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AstParserPort for TreeSitterParserEngine {
    fn supports_language(&self, path: &Path) -> bool {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        matches!(ext, "ts" | "tsx" | "js" | "jsx" | "py" | "php")
    }

    async fn parse_file(&self, path: &Path, content: &str) -> Result<ParsedFile, ParserError> {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let (nodes, links) = match ext {
            "tsx" | "jsx" => {
                let mut p = TypeScriptParser::new_tsx().map_err(ParserError::TreeSitter)?;
                p.parse(path, content).map_err(ParserError::TreeSitter)?
            }
            "ts" | "js" => {
                let mut p = TypeScriptParser::new().map_err(ParserError::TreeSitter)?;
                p.parse(path, content).map_err(ParserError::TreeSitter)?
            }
            "py" => {
                let mut p = PythonParser::new().map_err(ParserError::TreeSitter)?;
                p.parse(path, content).map_err(ParserError::TreeSitter)?
            }
            "php" => {
                let p = PhpParser::new();
                p.parse(path, content)
            }
            _ => (Vec::new(), Vec::new()),
        };

        Ok(ParsedFile {
            file_path: path.to_string_lossy().to_string(),
            language: ext.to_string(),
            nodes,
            links,
        })
    }

    async fn parse_batch(&self, paths: &[SanitizedPath]) -> Result<GraphData, ParserError> {
        let results: Vec<(Vec<GraphNode>, Vec<GraphLink>)> = paths
            .par_iter()
            .filter_map(|sanitized| {
                let path = sanitized.as_path();
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                let buffer = SourceBuffer::open(path).ok()?;
                let content = buffer.as_str().ok()?;

                match ext {
                    "tsx" | "jsx" => {
                        let mut p = TypeScriptParser::new_tsx().ok()?;
                        p.parse(path, content).ok()
                    }
                    "ts" | "js" => {
                        let mut p = TypeScriptParser::new().ok()?;
                        p.parse(path, content).ok()
                    }
                    "py" => {
                        let mut p = PythonParser::new().ok()?;
                        p.parse(path, content).ok()
                    }
                    "php" => Some(PhpParser::new().parse(path, content)),
                    _ => None,
                }
            })
            .collect();

        let mut all_nodes = Vec::new();
        let mut all_links = Vec::new();

        for (nodes, links) in results {
            all_nodes.extend(nodes);
            all_links.extend(links);
        }

        Ok(GraphData {
            nodes: all_nodes,
            links: all_links,
        })
    }
}
