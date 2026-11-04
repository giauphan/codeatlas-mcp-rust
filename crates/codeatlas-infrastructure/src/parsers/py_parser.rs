use crate::parsers::ast_utils::{
    child_field_text, init_parsed_graph, make_class_node, make_import_link, node_text, GraphTarget,
};
use codeatlas_domain::entities::graph::{GraphLink, GraphNode};
use std::path::Path;
use tree_sitter::Parser;

pub struct PythonParser {
    parser: Parser,
}

fn unwrap_decorated<'a>(node: tree_sitter::Node<'a>) -> tree_sitter::Node<'a> {
    if node.kind() == "decorated_definition" {
        node.child_by_field_name("definition").unwrap_or(node)
    } else {
        node
    }
}

fn handle_py_function(child: tree_sitter::Node<'_>, content: &str, target: &mut GraphTarget<'_>) {
    let Some(name) = child_field_text(child, "name", content) else {
        return;
    };
    let line = child.start_position().row as u32 + 1;
    target.add_function_with_calls(child, content, name, line, "call");
}

fn handle_py_class(child: tree_sitter::Node<'_>, content: &str, target: &mut GraphTarget<'_>) {
    let Some(class_name) = child_field_text(child, "name", content) else {
        return;
    };
    let line = child.start_position().row as u32 + 1;
    target
        .nodes
        .push(make_class_node(target.path_str, class_name, line));

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
        let m_line = member.start_position().row as u32 + 1;
        target.add_function_with_calls(member, content, &method_label, m_line, "call");
    }
}

fn handle_py_import(child: tree_sitter::Node<'_>, content: &str, target: &mut GraphTarget<'_>) {
    if let Some(mod_name) = child_field_text(child, "module_name", content) {
        target
            .links
            .push(make_import_link(target.path_str, mod_name));
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
            target
                .links
                .push(make_import_link(target.path_str, node_text(n, content)));
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
        let mut target = GraphTarget {
            path_str: &path_str,
            nodes: &mut nodes,
            links: &mut links,
        };
        let root_node = tree.root_node();
        let mut cursor = root_node.walk();

        for top in root_node.children(&mut cursor) {
            let child = unwrap_decorated(top);

            match child.kind() {
                "function_definition" => {
                    handle_py_function(child, content, &mut target);
                }
                "class_definition" => {
                    handle_py_class(child, content, &mut target);
                }
                "import_statement" | "import_from_statement" => {
                    handle_py_import(child, content, &mut target);
                }
                _ => {}
            }
        }

        Ok((nodes, links))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_python_parser_functions_and_calls() {
        let mut parser = PythonParser::new().expect("Should create Python parser");
        let code = r#"
def helper():
    print("hello")

class Service:
    def execute(self):
        helper()
"#;
        let (nodes, links) = parser
            .parse(Path::new("test.py"), code)
            .expect("Should parse");

        let node_labels: Vec<&str> = nodes.iter().map(|n| n.label.as_str()).collect();
        assert!(node_labels.contains(&"helper"));
        assert!(node_labels.contains(&"Service"));
        assert!(node_labels.contains(&"Service::execute"));

        let has_call = links
            .iter()
            .any(|l| l.source == "Service::execute" && l.target == "helper");
        assert!(
            has_call,
            "Expected call link from Service::execute to helper"
        );
    }
}
