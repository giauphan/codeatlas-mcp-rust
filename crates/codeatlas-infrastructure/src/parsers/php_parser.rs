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

fn strip_class_modifiers(mut decl: &str) -> &str {
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
    decl
}

fn extract_class_name(decl: &str, trimmed: &str) -> Option<String> {
    if trimmed.contains("::class") {
        return None;
    }
    if !decl.starts_with("class ") && !decl.starts_with("interface ") && !decl.starts_with("trait ") {
        return None;
    }
    let name = decl.split_whitespace().nth(1)?;
    let clean = name.trim_matches(|c: char| !c.is_alphanumeric() && c != '_');
    if is_valid_php_ident(clean) {
        Some(clean.to_string())
    } else {
        None
    }
}

fn extract_function_name(trimmed: &str) -> Option<String> {
    let is_fn = trimmed.starts_with("function ")
        || trimmed.contains(" function ")
        || trimmed.starts_with("public function ")
        || trimmed.starts_with("protected function ")
        || trimmed.starts_with("private function ");
    if !is_fn {
        return None;
    }
    let pos = trimmed.find("function ")?;
    let after = &trimmed[pos + 9..];
    let name_part = after.split('(').next()?.trim();
    let name = name_part.trim_matches(|c: char| !c.is_alphanumeric() && c != '_');
    if is_valid_php_ident(name) {
        Some(name.to_string())
    } else {
        None
    }
}

fn extract_use_target(trimmed: &str) -> Option<String> {
    if trimmed.starts_with("use ") && trimmed.ends_with(';') {
        let target = trimmed
            .trim_start_matches("use ")
            .trim_end_matches(';')
            .trim();
        if !target.is_empty() {
            return Some(target.to_string());
        }
    }
    None
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

            let decl = strip_class_modifiers(trimmed);

            if let Some(name) = extract_class_name(decl, trimmed) {
                current_class = Some(name.clone());
                nodes.push(GraphNode {
                    id: format!("{}#{}", path_str, name),
                    label: name,
                    node_type: "class".to_string(),
                    val: Some(2.0),
                    color: None,
                    file_path: Some(path_str.clone()),
                    line: Some(line_num),
                });
            } else if let Some(name) = extract_function_name(trimmed) {
                let node_id = match &current_class {
                    Some(cls) => format!("{}#{}::{}", path_str, cls, name),
                    None => format!("{}#{}", path_str, name),
                };
                nodes.push(GraphNode {
                    id: node_id,
                    label: name,
                    node_type: "function".to_string(),
                    val: Some(1.0),
                    color: None,
                    file_path: Some(path_str.clone()),
                    line: Some(line_num),
                });
            } else if let Some(target) = extract_use_target(trimmed) {
                links.push(GraphLink {
                    source: path_str.clone(),
                    target,
                    link_type: "import".to_string(),
                    label: None,
                });
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
