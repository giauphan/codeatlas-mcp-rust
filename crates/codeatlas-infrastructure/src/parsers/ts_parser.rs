use crate::parsers::ast_utils::{
    child_field_text, init_parsed_graph, make_class_node, make_import_link, make_variable_node,
    node_text, GraphTarget,
};
use codeatlas_domain::entities::graph::{GraphLink, GraphNode};
use std::path::Path;
use tree_sitter::Parser;

pub struct TypeScriptParser {
    parser: Parser,
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

fn handle_ts_function(child: tree_sitter::Node<'_>, content: &str, target: &mut GraphTarget<'_>) {
    let Some(name) = child_field_text(child, "name", content) else {
        return;
    };
    let line = child.start_position().row as u32 + 1;
    target.add_function_with_calls(child, content, name, line, "call_expression");
}

fn handle_ts_class(child: tree_sitter::Node<'_>, content: &str, target: &mut GraphTarget<'_>) {
    let Some(class_name) = child_field_text(child, "name", content) else {
        return;
    };
    let class_line = child.start_position().row as u32 + 1;
    target
        .nodes
        .push(make_class_node(target.path_str, class_name, class_line));

    let Some(body) = child.child_by_field_name("body") else {
        return;
    };
    let mut cursor = body.walk();
    for member in body.children(&mut cursor) {
        if member.kind() != "method_definition" {
            continue;
        }
        let Some(m_name) = child_field_text(member, "name", content) else {
            continue;
        };
        let method_label = format!("{class_name}::{m_name}");
        let line = member.start_position().row as u32 + 1;
        target.add_function_with_calls(member, content, &method_label, line, "call_expression");
    }
}

fn handle_ts_lexical_decl(
    child: tree_sitter::Node<'_>,
    content: &str,
    target: &mut GraphTarget<'_>,
) {
    let mut decl_cursor = child.walk();
    for decl in child.children(&mut decl_cursor) {
        if decl.kind() != "variable_declarator" {
            continue;
        }
        let Some(name) = child_field_text(decl, "name", content) else {
            continue;
        };
        let value_node = decl.child_by_field_name("value");
        let line = decl.start_position().row as u32 + 1;
        let is_func = value_node
            .as_ref()
            .is_some_and(|v| v.kind() == "arrow_function" || v.kind() == "function_expression");

        if is_func {
            let call_node = value_node.unwrap_or(decl);
            target.add_function_with_calls(call_node, content, name, line, "call_expression");
        } else {
            target
                .nodes
                .push(make_variable_node(target.path_str, name, line));
        }
    }
}

fn handle_ts_import(
    child: tree_sitter::Node<'_>,
    content: &str,
    file_path: &Path,
    target: &mut GraphTarget<'_>,
) {
    let Some(source_node) = child.child_by_field_name("source") else {
        return;
    };
    let clean_source = node_text(source_node, content).trim_matches(|c| c == '\'' || c == '"');
    let target_path = resolve_import_target(clean_source, file_path);
    target
        .links
        .push(make_import_link(target.path_str, &target_path));
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

        let (path_str, mut nodes, mut links) = init_parsed_graph(file_path);
        let mut target = GraphTarget {
            path_str: &path_str,
            nodes: &mut nodes,
            links: &mut links,
        };
        let root_node = tree.root_node();
        let mut cursor = root_node.walk();

        for top in root_node.children(&mut cursor) {
            let child = unwrap_export(top);

            match child.kind() {
                "function_declaration" => {
                    handle_ts_function(child, content, &mut target);
                }
                "class_declaration" => {
                    handle_ts_class(child, content, &mut target);
                }
                "lexical_declaration" => {
                    handle_ts_lexical_decl(child, content, &mut target);
                }
                "import_statement" => {
                    handle_ts_import(child, content, file_path, &mut target);
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
    fn test_typescript_parser_functions_and_calls() {
        let mut parser = TypeScriptParser::new().expect("Should create TypeScript parser");
        let code = r#"
function greet() {
    console.log("hello");
}

class Greeter {
    welcome() {
        greet();
    }
}
"#;
        let (nodes, links) = parser
            .parse(Path::new("test.ts"), code)
            .expect("Should parse");

        let node_labels: Vec<&str> = nodes.iter().map(|n| n.label.as_str()).collect();
        assert!(node_labels.contains(&"greet"));
        assert!(node_labels.contains(&"Greeter"));
        assert!(node_labels.contains(&"Greeter::welcome"));

        let has_call = links
            .iter()
            .any(|l| l.source == "Greeter::welcome" && l.target == "greet");
        assert!(
            has_call,
            "Expected call link from Greeter::welcome to greet"
        );
    }
}
