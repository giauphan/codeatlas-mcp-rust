use codeatlas_domain::entities::graph::AnalysisResult;
use std::collections::{HashMap, HashSet, VecDeque};

/// Fast breadth-first search on adjacency list with depth and max node limits.
/// ⚡ Bolt Optimization: Uses `&str` instead of `String` to avoid O(N) heap allocations when building
/// the adjacency list and queue. Reduces memory footprint and speeds up dependency resolution.
pub fn bfs_reachable<'a>(
    adj_list: &HashMap<&'a str, Vec<&'a str>>,
    seed: &'a str,
    max_depth: u32,
    max_nodes: usize,
) -> HashSet<&'a str> {
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();

    visited.insert(seed);
    queue.push_back((seed, 0));

    while let Some((current, depth)) = queue.pop_front() {
        if depth >= max_depth || visited.len() >= max_nodes {
            continue;
        }

        if let Some(neighbors) = adj_list.get(current) {
            for &neighbor in neighbors {
                if !visited.contains(neighbor) {
                    visited.insert(neighbor);
                    queue.push_back((neighbor, depth + 1));
                    if visited.len() >= max_nodes {
                        break;
                    }
                }
            }
        }
    }

    visited
}

/// Detect cycles using Tarjan's Strongly Connected Components.
/// ⚡ Bolt Optimization: Uses borrowed `&str` for adjacency maps and internal structures
/// (`TarjanContext`, stacks) instead of cloning strings. Eliminates unnecessary allocations during cycle detection.
pub fn detect_cycles_in_graph(analysis: &AnalysisResult) -> Vec<Vec<String>> {
    let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
    for link in &analysis.graph.links {
        if link.link_type == "import" || link.link_type == "call" {
            adj.entry(link.source.as_str())
                .or_default()
                .push(link.target.as_str());
        }
    }

    struct TarjanContext<'a> {
        index: usize,
        stack: Vec<&'a str>,
        indices: HashMap<&'a str, usize>,
        lowlinks: HashMap<&'a str, usize>,
        on_stack: HashSet<&'a str>,
        adj: &'a HashMap<&'a str, Vec<&'a str>>,
        sccs: Vec<Vec<String>>,
    }

    impl<'a> TarjanContext<'a> {
        fn strongconnect(&mut self, node: &'a str) {
            self.indices.insert(node, self.index);
            self.lowlinks.insert(node, self.index);
            self.index += 1;
            self.stack.push(node);
            self.on_stack.insert(node);

            if let Some(neighbors) = self.adj.get(node) {
                for &neighbor in neighbors {
                    if !self.indices.contains_key(neighbor) {
                        self.strongconnect(neighbor);
                        let n_low = *self.lowlinks.get(neighbor).unwrap();
                        let node_low = self.lowlinks.get_mut(node).unwrap();
                        *node_low = (*node_low).min(n_low);
                    } else if self.on_stack.contains(neighbor) {
                        let n_idx = *self.indices.get(neighbor).unwrap();
                        let node_low = self.lowlinks.get_mut(node).unwrap();
                        *node_low = (*node_low).min(n_idx);
                    }
                }
            }

            if self.lowlinks.get(node) == self.indices.get(node) {
                let mut scc = Vec::new();
                while let Some(w) = self.stack.pop() {
                    self.on_stack.remove(w);
                    scc.push(w.to_string());
                    if w == node {
                        break;
                    }
                }
                if scc.len() > 1 {
                    self.sccs.push(scc);
                }
            }
        }
    }

    let mut ctx = TarjanContext {
        index: 0,
        stack: Vec::new(),
        indices: HashMap::new(),
        lowlinks: HashMap::new(),
        on_stack: HashSet::new(),
        adj: &adj,
        sccs: Vec::new(),
    };

    for node in &analysis.graph.nodes {
        if !ctx.indices.contains_key(node.label.as_str()) {
            ctx.strongconnect(node.label.as_str());
        }
    }

    ctx.sccs
}

/// Auto-generate Mermaid flowchart diagram
pub fn generate_mermaid_system_flow(analysis: &AnalysisResult, max_nodes: usize) -> String {
    let mut diagram = String::from("flowchart TD\n");
    let mut count = 0;

    for link in &analysis.graph.links {
        if count >= max_nodes {
            break;
        }
        let clean_src = sanitize_mermaid_id(&link.source);
        let clean_tgt = sanitize_mermaid_id(&link.target);
        if clean_src != clean_tgt {
            diagram.push_str(&format!("  {} --> {}\n", clean_src, clean_tgt));
            count += 1;
        }
    }

    if count == 0 {
        diagram.push_str("  Empty[\"No system flow relationships found\"]\n");
    }

    diagram
}

/// Auto-generate Mermaid feature sequence flow
pub fn generate_mermaid_feature_flow(
    analysis: &AnalysisResult,
    keyword: &str,
    _max_depth: u32,
) -> String {
    let mut diagram = String::from("sequenceDiagram\n  autonumber\n");
    let kw_lower = keyword.to_lowercase();
    let mut count = 0;

    for link in &analysis.graph.links {
        if link.source.to_lowercase().contains(&kw_lower)
            || link.target.to_lowercase().contains(&kw_lower)
        {
            let clean_src = sanitize_mermaid_id(&link.source);
            let clean_tgt = sanitize_mermaid_id(&link.target);
            diagram.push_str(&format!(
                "  {}->>{}: {}\n",
                clean_src, clean_tgt, link.link_type
            ));
            count += 1;
            if count >= 30 {
                break;
            }
        }
    }

    if count == 0 {
        diagram.push_str(&format!(
            "  Note over System: No feature flow found for keyword '{}'\n",
            keyword
        ));
    }

    diagram
}

fn sanitize_mermaid_id(raw: &str) -> String {
    let s: String = raw
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    if s.is_empty() {
        "node".to_string()
    } else {
        s
    }
}
