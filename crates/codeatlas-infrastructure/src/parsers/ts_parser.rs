use codeatlas_domain::entities::graph::{GraphLink, GraphNode};
use std::path::Path;
use tree_sitter::Parser;

pub struct TypeScriptParser {
    parser: Parser,
}

fn extract_ts_calls<'a>(node: tree_sitter::Node<'a>, content: &'a str, calls: &mut Vec<String>) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "call_expression" {
            if let Some(func) = child.child_by_field_name("function") {
                let callee = &content[func.start_byte()..func.end_byte()];
                calls.push(callee.to_string());
            }
        }
        extract_ts_calls(child, content, calls);
    }
}

fn unwrap_export<'a>(node: tree_sitter::Node<'a>) -> tree_sitter::Node<'a> {
    if node.kind() == "export_statement" {
        node.child_by_field_name("declaration").unwrap_or(node)
    } else {
        node
    }
}

fn resolve_import_target(clean_source: &str, file_path: &Path) -> String {
    if !clean_source.starts_with('.') {
        return clean_source.to_string();
    }
    let parent = file_path.parent().unwrap_or(Path::new("."));
    let mut resolved = parent.join(clean_source);
    if resolved.extension().is_none() {
        if resolved.with_extension("ts").exists() {
            resolved = resolved.with_extension("ts");
        } else if resolved.with_extension("tsx").exists() {
            resolved = resolved.with_extension("tsx");
        } else if resolved.with_extension("js").exists() {
            resolved = resolved.with_extension("js");
        }
    }
    resolved.to_string_lossy().to_string()
}

fn add_ts_function_node_and_calls(
    node: tree_sitter::Node<'_>,
    content: &str,
    path_str: &str,
    label: &str,
    line: u32,
    nodes: &mut Vec<GraphNode>,
    links: &mut Vec<GraphLink>,
) {
    nodes.push(GraphNode {
        id: format!("{}#{}", path_str, label),
        label: label.to_string(),
        node_type: "function".to_string(),
        val: Some(1.0),
        color: None,
        file_path: Some(path_str.to_string()),
        line: Some(line),
    });

    let mut calls = Vec::new();
    extract_ts_calls(node, content, &mut calls);
    for callee in calls {
        links.push(GraphLink {
            source: label.to_string(),
            target: callee,
            link_type: "call".to_string(),
            label: None,
        });
    }
}

fn handle_ts_function(
    child: tree_sitter::Node<'_>,
    content: &str,
    path_str: &str,
    nodes: &mut Vec<GraphNode>,
    links: &mut Vec<GraphLink>,
) {
    let Some(name_node) = child.child_by_field_name("name") else {
        return;
    };
    let name = &content[name_node.start_byte()..name_node.end_byte()];
    let line = child.start_position().row as u32 + 1;
    add_ts_function_node_and_calls(child, content, path_str, name, line, nodes, links);
}

fn handle_ts_class(
    child: tree_sitter::Node<'_>,
    content: &str,
    path_str: &str,
    nodes: &mut Vec<GraphNode>,
    links: &mut Vec<GraphLink>,
) {
    let Some(name_node) = child.child_by_field_name("name") else {
        return;
    };
    let class_name = &content[name_node.start_byte()..name_node.end_byte()];
    nodes.push(GraphNode {
        id: format!("{}#{}", path_str, class_name),
        label: class_name.to_string(),
        node_type: "class".to_string(),
        val: Some(2.0),
        color: None,
        file_path: Some(path_str.to_string()),
        line: Some(child.start_position().row as u32 + 1),
    });

    let Some(body) = child.child_by_field_name("body") else {
        return;
    };
    let mut body_cursor = body.walk();
    for member in body.children(&mut body_cursor) {
        if member.kind() != "method_definition" {
            continue;
        }
        let Some(m_name) = member.child_by_field_name("name") else {
            continue;
        };
        let method_name = &content[m_name.start_byte()..m_name.end_byte()];
        let method_label = format!("{}::{}", class_name, method_name);
        let line = member.start_position().row as u32 + 1;
        add_ts_function_node_and_calls(
            member,
            content,
            path_str,
            &method_label,
            line,
            nodes,
            links,
        );
    }
}

fn handle_ts_lexical_decl(
    child: tree_sitter::Node<'_>,
    content: &str,
    path_str: &str,
    nodes: &mut Vec<GraphNode>,
    links: &mut Vec<GraphLink>,
) {
    let mut decl_cursor = child.walk();
    for decl in child.children(&mut decl_cursor) {
        if decl.kind() != "variable_declarator" {
            continue;
        }
        let Some(name_node) = decl.child_by_field_name("name") else {
            continue;
        };
        let name = &content[name_node.start_byte()..name_node.end_byte()];
        let value_node = decl.child_by_field_name("value");
        let line = decl.start_position().row as u32 + 1;
        let is_func = value_node
            .as_ref()
            .is_some_and(|v| v.kind() == "arrow_function" || v.kind() == "function_expression");

        if is_func {
            let call_node = value_node.unwrap_or(decl);
            add_ts_function_node_and_calls(call_node, content, path_str, name, line, nodes, links);
        } else {
            nodes.push(GraphNode {
                id: format!("{}#{}", path_str, name),
                label: name.to_string(),
                node_type: "variable".to_string(),
                val: Some(0.5),
                color: None,
                file_path: Some(path_str.to_string()),
                line: Some(line),
            });
        }
    }
}

fn handle_ts_import(
    child: tree_sitter::Node<'_>,
    content: &str,
    path_str: &str,
    file_path: &Path,
    links: &mut Vec<GraphLink>,
) {
    let Some(source_node) = child.child_by_field_name("source") else {
        return;
    };
    let raw_source = &content[source_node.start_byte()..source_node.end_byte()];
    let clean_source = raw_source.trim_matches(|c| c == '\'' || c == '"');
    let target_path = resolve_import_target(clean_source, file_path);

    links.push(GraphLink {
        source: path_str.to_string(),
        target: target_path,
        link_type: "import".to_string(),
        label: None,
    });
}

impl TypeScriptParser {
    pub fn new() -> Result<Self, String> {
        let mut parser = Parser::new();
        let lang = tree_sitter_typescript::language_typescript();
        parser
            .set_language(&lang)
            .map_err(|e| format!("Failed to set TypeScript language: {:?}", e))?;
        Ok(Self { parser })
    }

    pub fn new_tsx() -> Result<Self, String> {
        let mut parser = Parser::new();
        let lang = tree_sitter_typescript::language_tsx();
        parser
            .set_language(&lang)
            .map_err(|e| format!("Failed to set TSX language: {:?}", e))?;
        Ok(Self { parser })
    }

    pub fn parse(
        &mut self,
        file_path: &Path,
        content: &str,
    ) -> Result<(Vec<GraphNode>, Vec<GraphLink>), String> {
        let tree = self
            .parser
            .parse(content, None)
            .ok_or_else(|| "Failed to parse TypeScript code".to_string())?;

        let mut nodes = Vec::new();
        let mut links = Vec::new();
        let path_str = file_path.to_string_lossy().to_string();

        // Emit module node for file
        nodes.push(GraphNode {
            id: path_str.clone(),
            label: path_str.clone(),
            node_type: "module".to_string(),
            val: Some(1.0),
            color: None,
            file_path: Some(path_str.clone()),
            line: Some(1),
        });

        let root_node = tree.root_node();
        let mut cursor = root_node.walk();

        for top in root_node.children(&mut cursor) {
            let child = unwrap_export(top);

            match child.kind() {
                "function_declaration" => {
                    handle_ts_function(child, content, &path_str, &mut nodes, &mut links);
                }
                "class_declaration" => {
                    handle_ts_class(child, content, &path_str, &mut nodes, &mut links);
                }
                "lexical_declaration" => {
                    handle_ts_lexical_decl(child, content, &path_str, &mut nodes, &mut links);
                }
                "import_statement" => {
                    handle_ts_import(child, content, &path_str, file_path, &mut links);
                }
                _ => {}
            }
        }

        Ok((nodes, links))
    }
}
