use crate::parsers::ast_utils::{
    child_field_text, init_parsed_graph, make_call_link, make_class_node, make_function_node,
    make_import_link, node_text,
};
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
                calls.push(node_text(func, content).to_string());
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

fn add_py_function_node_and_calls(
    node: tree_sitter::Node<'_>,
    content: &str,
    path_str: &str,
    label: &str,
    nodes: &mut Vec<GraphNode>,
    links: &mut Vec<GraphLink>,
) {
    let line = node.start_position().row as u32 + 1;
    nodes.push(make_function_node(path_str, label, line));

    let mut calls = Vec::new();
    extract_py_calls(node, content, &mut calls);
    for callee in calls {
        links.push(make_call_link(label, &callee));
    }
}

fn handle_py_function(
    child: tree_sitter::Node<'_>,
    content: &str,
    path_str: &str,
    nodes: &mut Vec<GraphNode>,
    links: &mut Vec<GraphLink>,
) {
    let Some(name) = child_field_text(child, "name", content) else {
        return;
    };
    add_py_function_node_and_calls(child, content, path_str, name, nodes, links);
}

fn handle_py_class(
    child: tree_sitter::Node<'_>,
    content: &str,
    path_str: &str,
    nodes: &mut Vec<GraphNode>,
    links: &mut Vec<GraphLink>,
) {
    let Some(class_name) = child_field_text(child, "name", content) else {
        return;
    };
    let line = child.start_position().row as u32 + 1;
    nodes.push(make_class_node(path_str, class_name, line));

    let Some(body) = child.child_by_field_name("body") else {
        return;
    };
    let mut body_cursor = body.walk();
    for raw_member in body.children(&mut body_cursor) {
        let member = unwrap_decorated(raw_member);
        if member.kind() != "function_definition" {
            continue;
        }
        let Some(method_name) = child_field_text(member, "name", content) else {
            continue;
        };
        let method_label = format!("{class_name}::{method_name}");
        add_py_function_node_and_calls(member, content, path_str, &method_label, nodes, links);
    }
}

fn handle_py_import(
    child: tree_sitter::Node<'_>,
    content: &str,
    path_str: &str,
    links: &mut Vec<GraphLink>,
) {
    if let Some(mod_name) = child_field_text(child, "module_name", content) {
        links.push(make_import_link(path_str, mod_name));
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
            links.push(make_import_link(path_str, node_text(n, content)));
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

        let (path_str, mut nodes, mut links) = init_parsed_graph(file_path);
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
