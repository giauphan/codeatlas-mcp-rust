use codeatlas_domain::entities::graph::{GraphLink, GraphNode};
use std::path::Path;

pub(crate) fn make_module_node(path_str: &str) -> GraphNode {
    GraphNode {
        id: path_str.to_string(),
        label: path_str.to_string(),
        node_type: "module".to_string(),
        val: Some(1.0),
        color: None,
        file_path: Some(path_str.to_string()),
        line: Some(1),
    }
}

pub(crate) fn make_class_node(path_str: &str, class_name: &str, line: u32) -> GraphNode {
    GraphNode {
        id: format!("{path_str}#{class_name}"),
        label: class_name.to_string(),
        node_type: "class".to_string(),
        val: Some(2.0),
        color: None,
        file_path: Some(path_str.to_string()),
        line: Some(line),
    }
}

pub(crate) fn make_function_node(path_str: &str, label: &str, line: u32) -> GraphNode {
    GraphNode {
        id: format!("{path_str}#{label}"),
        label: label.to_string(),
        node_type: "function".to_string(),
        val: Some(1.0),
        color: None,
        file_path: Some(path_str.to_string()),
        line: Some(line),
    }
}

pub(crate) fn make_variable_node(path_str: &str, name: &str, line: u32) -> GraphNode {
    GraphNode {
        id: format!("{path_str}#{name}"),
        label: name.to_string(),
        node_type: "variable".to_string(),
        val: Some(0.5),
        color: None,
        file_path: Some(path_str.to_string()),
        line: Some(line),
    }
}

pub(crate) fn make_call_link(source: &str, target: &str) -> GraphLink {
    GraphLink {
        source: source.to_string(),
        target: target.to_string(),
        link_type: "call".to_string(),
        label: None,
    }
}

pub(crate) fn make_import_link(source: &str, target: &str) -> GraphLink {
    GraphLink {
        source: source.to_string(),
        target: target.to_string(),
        link_type: "import".to_string(),
        label: None,
    }
}

pub(crate) fn init_parsed_graph(file_path: &Path) -> (String, Vec<GraphNode>, Vec<GraphLink>) {
    let path_str = file_path.to_string_lossy().to_string();
    let nodes = vec![make_module_node(&path_str)];
    let links = Vec::new();
    (path_str, nodes, links)
}

pub(crate) fn node_text<'a>(node: tree_sitter::Node<'_>, content: &'a str) -> &'a str {
    &content[node.start_byte()..node.end_byte()]
}

pub(crate) fn child_field_text<'a>(
    node: tree_sitter::Node<'_>,
    field: &str,
    content: &'a str,
) -> Option<&'a str> {
    node.child_by_field_name(field)
        .map(|n| node_text(n, content))
}
