use codeatlas_domain::entities::graph::{GraphLink, GraphNode};
use std::path::Path;

pub struct PhpParser;

fn is_valid_php_ident(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let mut chars = s.chars();
    if let Some(first) = chars.next() {
        if !first.is_alphabetic() && first != '_' {
            return false;
        }
    }
    chars.all(|c| c.is_alphanumeric() || c == '_')
}

impl PhpParser {
    pub fn new() -> Self {
        Self
    }

    pub fn parse(&self, file_path: &Path, content: &str) -> (Vec<GraphNode>, Vec<GraphLink>) {
        let mut nodes = Vec::new();
        let mut links = Vec::new();
        let path_str = file_path.to_string_lossy().to_string();
        let mut current_class: Option<String> = None;

        // Emit module node
        nodes.push(GraphNode {
            id: path_str.clone(),
            label: path_str.clone(),
            node_type: "module".to_string(),
            val: Some(3.0),
            color: None,
            file_path: Some(path_str.clone()),
            line: Some(1),
        });

        for (line_idx, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            let line_num = (line_idx + 1) as u32;

            if trimmed.is_empty()
                || trimmed.starts_with("//")
                || trimmed.starts_with('#')
                || trimmed.starts_with('*')
                || trimmed.starts_with("/*")
            {
                continue;
            }

            // Strip class modifiers: abstract, final, readonly
            let mut decl = trimmed;
            loop {
                if let Some(rest) = decl.strip_prefix("abstract ") {
                    decl = rest.trim_start();
                } else if let Some(rest) = decl.strip_prefix("final ") {
                    decl = rest.trim_start();
                } else if let Some(rest) = decl.strip_prefix("readonly ") {
                    decl = rest.trim_start();
                } else {
                    break;
                }
            }

            // Class, Interface, Trait definition
            if (decl.starts_with("class ")
                || decl.starts_with("interface ")
                || decl.starts_with("trait "))
                && !trimmed.contains("::class")
            {
                let parts: Vec<&str> = decl.split_whitespace().collect();
                if parts.len() >= 2 {
                    let name = parts[1].trim_matches(|c: char| !c.is_alphanumeric() && c != '_');
                    if is_valid_php_ident(name) {
                        current_class = Some(name.to_string());
                        nodes.push(GraphNode {
                            id: format!("{}#{}", path_str, name),
                            label: name.to_string(),
                            node_type: "class".to_string(),
                            val: Some(2.0),
                            color: None,
                            file_path: Some(path_str.clone()),
                            line: Some(line_num),
                        });
                    }
                }
            }
            // Function / Method definition
            else if trimmed.starts_with("function ")
                || trimmed.contains(" function ")
                || trimmed.starts_with("public function ")
                || trimmed.starts_with("protected function ")
                || trimmed.starts_with("private function ")
            {
                if let Some(pos) = trimmed.find("function ") {
                    let after = &trimmed[pos + 9..];
                    let name_part = after.split('(').next().unwrap_or("").trim();
                    let name = name_part.trim_matches(|c: char| !c.is_alphanumeric() && c != '_');
                    if is_valid_php_ident(name) {
                        let node_id = if let Some(ref cls) = current_class {
                            format!("{}#{}::{}", path_str, cls, name)
                        } else {
                            format!("{}#{}", path_str, name)
                        };
                        nodes.push(GraphNode {
                            id: node_id,
                            label: name.to_string(),
                            node_type: "function".to_string(),
                            val: Some(1.0),
                            color: None,
                            file_path: Some(path_str.clone()),
                            line: Some(line_num),
                        });
                    }
                }
            }
            // Use statement (import)
            else if trimmed.starts_with("use ") && trimmed.ends_with(';') {
                let target = trimmed
                    .trim_start_matches("use ")
                    .trim_end_matches(';')
                    .trim();
                if !target.is_empty() {
                    links.push(GraphLink {
                        source: path_str.clone(),
                        target: target.to_string(),
                        link_type: "import".to_string(),
                        label: None,
                    });
                }
            }
        }

        (nodes, links)
    }
}

impl Default for PhpParser {
    fn default() -> Self {
        Self::new()
    }
}
