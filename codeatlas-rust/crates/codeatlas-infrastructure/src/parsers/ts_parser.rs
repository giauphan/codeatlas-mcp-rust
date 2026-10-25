use codeatlas_domain::entities::graph::{GraphLink, GraphNode};
use std::path::Path;
use tree_sitter::Parser;

pub struct TypeScriptParser {
    parser: Parser,
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

        let root_node = tree.root_node();
        let mut cursor = root_node.walk();

        for top in root_node.children(&mut cursor) {
            let child = if top.kind() == "export_statement" {
                top.child_by_field_name("declaration").unwrap_or(top)
            } else {
                top
            };

            match child.kind() {
                "function_declaration" => {
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
                    }
                }
                "class_declaration" => {
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

                        // Extract methods from class_body
                        if let Some(body) = child.child_by_field_name("body") {
                            let mut body_cursor = body.walk();
                            for member in body.children(&mut body_cursor) {
                                if member.kind() == "method_definition" {
                                    if let Some(m_name) = member.child_by_field_name("name") {
                                        let method_name =
                                            &content[m_name.start_byte()..m_name.end_byte()];
                                        nodes.push(GraphNode {
                                            id: format!(
                                                "{}#{}::{}",
                                                path_str, class_name, method_name
                                            ),
                                            label: format!("{}::{}", class_name, method_name),
                                            node_type: "function".to_string(),
                                            val: Some(1.0),
                                            color: None,
                                            file_path: Some(path_str.clone()),
                                            line: Some(member.start_position().row as u32 + 1),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
                "lexical_declaration" => {
                    let mut decl_cursor = child.walk();
                    for decl in child.children(&mut decl_cursor) {
                        if decl.kind() == "variable_declarator" {
                            if let Some(name_node) = decl.child_by_field_name("name") {
                                let name = &content[name_node.start_byte()..name_node.end_byte()];
                                nodes.push(GraphNode {
                                    id: format!("{}#{}", path_str, name),
                                    label: name.to_string(),
                                    node_type: "variable".to_string(),
                                    val: Some(0.5),
                                    color: None,
                                    file_path: Some(path_str.clone()),
                                    line: Some(decl.start_position().row as u32 + 1),
                                });
                            }
                        }
                    }
                }
                "import_statement" => {
                    if let Some(source_node) = child.child_by_field_name("source") {
                        let raw_source = &content[source_node.start_byte()..source_node.end_byte()];
                        let clean_source = raw_source.trim_matches(|c| c == '\'' || c == '"');
                        links.push(GraphLink {
                            source: path_str.clone(),
                            target: clean_source.to_string(),
                            link_type: "import".to_string(),
                            label: None,
                        });
                    }
                }
                _ => {}
            }
        }

        Ok((nodes, links))
    }
}
