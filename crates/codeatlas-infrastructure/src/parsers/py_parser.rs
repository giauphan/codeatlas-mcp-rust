use codeatlas_domain::entities::graph::{GraphLink, GraphNode};
use std::path::Path;
use tree_sitter::Parser;

pub struct PythonParser {
    parser: Parser,
}

fn extract_py_calls<'a>(node: tree_sitter::Node<'a>, content: &'a str, calls: &mut Vec<String>) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "call" {
            if let Some(func) = child.child_by_field_name("function") {
                let callee = &content[func.start_byte()..func.end_byte()];
                calls.push(callee.to_string());
            }
        }
        extract_py_calls(child, content, calls);
    }
}

fn unwrap_decorated<'a>(node: tree_sitter::Node<'a>) -> tree_sitter::Node<'a> {
    if node.kind() == "decorated_definition" {
        node.child_by_field_name("definition").unwrap_or(node)
    } else {
        node
    }
}

fn emit_import_link(source: &str, target: &str, links: &mut Vec<GraphLink>) {
    links.push(GraphLink {
        source: source.to_string(),
        target: target.to_string(),
        link_type: "import".to_string(),
        label: None,
    });
}

fn add_py_function_node_and_calls(
    node: tree_sitter::Node<'_>,
    content: &str,
    path_str: &str,
    label: &str,
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
        line: Some(node.start_position().row as u32 + 1),
    });

    let mut calls = Vec::new();
    extract_py_calls(node, content, &mut calls);
    for callee in calls {
        links.push(GraphLink {
            source: label.to_string(),
            target: callee,
            link_type: "call".to_string(),
            label: None,
        });
    }
}

fn handle_py_function(
    child: tree_sitter::Node<'_>,
    content: &str,
    path_str: &str,
    nodes: &mut Vec<GraphNode>,
    links: &mut Vec<GraphLink>,
) {
    let Some(name_node) = child.child_by_field_name("name") else { return };
    let name = &content[name_node.start_byte()..name_node.end_byte()];
    add_py_function_node_and_calls(child, content, path_str, name, nodes, links);
}

fn handle_py_class(
    child: tree_sitter::Node<'_>,
    content: &str,
    path_str: &str,
    nodes: &mut Vec<GraphNode>,
    links: &mut Vec<GraphLink>,
) {
    let Some(name_node) = child.child_by_field_name("name") else { return };
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

    let Some(body) = child.child_by_field_name("body") else { return };
    let mut body_cursor = body.walk();
    for raw_member in body.children(&mut body_cursor) {
        let member = unwrap_decorated(raw_member);
        if member.kind() != "function_definition" {
            continue;
        }
        let Some(m_name) = member.child_by_field_name("name") else { continue };
        let method_name = &content[m_name.start_byte()..m_name.end_byte()];
        let method_label = format!("{}::{}", class_name, method_name);
        add_py_function_node_and_calls(member, content, path_str, &method_label, nodes, links);
    }
}

fn handle_py_import(
    child: tree_sitter::Node<'_>,
    content: &str,
    path_str: &str,
    links: &mut Vec<GraphLink>,
) {
    if let Some(mod_node) = child.child_by_field_name("module_name") {
        let mod_name = &content[mod_node.start_byte()..mod_node.end_byte()];
        emit_import_link(path_str, mod_name, links);
        return;
    }

    let mut sub_cursor = child.walk();
    for sub_child in child.children(&mut sub_cursor) {
        let name_node = match sub_child.kind() {
            "dotted_name" => Some(sub_child),
            "aliased_import" => sub_child.child_by_field_name("name"),
            _ => None,
        };
        if let Some(n) = name_node {
            let mod_name = &content[n.start_byte()..n.end_byte()];
            emit_import_link(path_str, mod_name, links);
        }
    }
}

impl PythonParser {
    pub fn new() -> Result<Self, String> {
        let mut parser = Parser::new();
        let lang = tree_sitter_python::language();
        parser
            .set_language(&lang)
            .map_err(|e| format!("Failed to set Python language: {:?}", e))?;
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
            .ok_or_else(|| "Failed to parse Python code".to_string())?;

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
            let child = unwrap_decorated(top);

            match child.kind() {
                "function_definition" => {
                    handle_py_function(child, content, &path_str, &mut nodes, &mut links);
                }
                "class_definition" => {
                    handle_py_class(child, content, &path_str, &mut nodes, &mut links);
                }
                "import_statement" | "import_from_statement" => {
                    handle_py_import(child, content, &path_str, &mut links);
                }
                _ => {}
            }
        }

        Ok((nodes, links))
    }
}
