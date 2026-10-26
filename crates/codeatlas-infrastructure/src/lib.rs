//! CodeAtlas Infrastructure Layer
//!
//! Engines: Tree-sitter AST, petgraph, notify filesystem watcher, memory caches.

pub mod graph;
pub mod parsers;
pub mod storage;
pub mod watcher;
