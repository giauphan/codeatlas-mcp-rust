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
            let child = if top.kind() == "decorated_definition" {
                top.child_by_field_name("definition").unwrap_or(top)
            } else {
                top
            };

            match child.kind() {
                "function_definition" => {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let name = &content[name_node.start_byte()..name_node.end_byte()];
                        nodes.push(GraphNode {
                            id: format!("{}#{}", path_str, name),
                            label: name.to_string(),
                            node_type: "function".to_string(),
                            val: Some(1.0),
                            color: None,
                            file_path: Some(path_str.clone()),
                            line: Some(child.start_position().row as u32 + 1),
                        });

                        let mut calls = Vec::new();
                        extract_py_calls(child, content, &mut calls);
                        for callee in calls {
                            links.push(GraphLink {
                                source: name.to_string(),
                                target: callee,
                                link_type: "call".to_string(),
                                label: None,
                            });
                        }
                    }
                }
                "class_definition" => {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let class_name = &content[name_node.start_byte()..name_node.end_byte()];
                        nodes.push(GraphNode {
                            id: format!("{}#{}", path_str, class_name),
                            label: class_name.to_string(),
                            node_type: "class".to_string(),
                            val: Some(2.0),
                            color: None,
                            file_path: Some(path_str.clone()),
                            line: Some(child.start_position().row as u32 + 1),
                        });

                        // Extract methods from class body
                        if let Some(body) = child.child_by_field_name("body") {
                            let mut body_cursor = body.walk();
                            for raw_member in body.children(&mut body_cursor) {
                                let member = if raw_member.kind() == "decorated_definition" {
                                    raw_member
                                        .child_by_field_name("definition")
                                        .unwrap_or(raw_member)
                                } else {
                                    raw_member
                                };
                                if member.kind() == "function_definition" {
                                    if let Some(m_name) = member.child_by_field_name("name") {
                                        let method_name =
                                            &content[m_name.start_byte()..m_name.end_byte()];
                                        let method_label =
                                            format!("{}::{}", class_name, method_name);
                                        nodes.push(GraphNode {
                                            id: format!("{}#{}", path_str, method_label),
                                            label: method_label.clone(),
                                            node_type: "function".to_string(),
                                            val: Some(1.0),
                                            color: None,
                                            file_path: Some(path_str.clone()),
                                            line: Some(member.start_position().row as u32 + 1),
                                        });

                                        let mut calls = Vec::new();
                                        extract_py_calls(member, content, &mut calls);
                                        for callee in calls {
                                            links.push(GraphLink {
                                                source: method_label.clone(),
                                                target: callee,
                                                link_type: "call".to_string(),
                                                label: None,
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                "import_statement" | "import_from_statement" => {
                    if let Some(mod_node) = child.child_by_field_name("module_name") {
                        let mod_name = &content[mod_node.start_byte()..mod_node.end_byte()];
                        links.push(GraphLink {
                            source: path_str.clone(),
                            target: mod_name.to_string(),
                            link_type: "import".to_string(),
                            label: None,
                        });
                    } else {
                        let mut sub_cursor = child.walk();
                        for sub_child in child.children(&mut sub_cursor) {
                            let name_node = match sub_child.kind() {
                                "dotted_name" => Some(sub_child),
                                "aliased_import" => sub_child.child_by_field_name("name"),
                                _ => None,
                            };
                            if let Some(n) = name_node {
                                let mod_name = &content[n.start_byte()..n.end_byte()];
                                links.push(GraphLink {
                                    source: path_str.clone(),
                                    target: mod_name.to_string(),
                                    link_type: "import".to_string(),
                                    label: None,
                                });
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        Ok((nodes, links))
    }
}
