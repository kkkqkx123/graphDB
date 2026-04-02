//! Plan Formatting Utilities
//!
//! Provides formatting for query plan descriptions in different output formats:
//! - Table format (default): Human-readable tabular output
//! - Dot format: Graphviz DOT format for visualization
//! - Tree format: Hierarchical tree structure showing plan relationships

use crate::query::planning::plan::core::explain::{PlanDescription, PlanNodeDescription};

/// Format plan description as a table
pub fn format_plan_as_table(plan_desc: &PlanDescription) -> String {
    let mut output = String::new();

    // Header with clear column names
    output.push_str("+------+------------------+------------+------------------+--------------------------------------------------+------------------+\n");
    output.push_str("| id   | name             | deps       | profiling_data   | operator_info                                    | output_var       |\n");
    output.push_str("+------+------------------+------------+------------------+--------------------------------------------------+------------------+\n");

    // Rows
    for node in &plan_desc.plan_node_descs {
        let id = format!("{:>4}", node.id);
        let name = truncate_or_pad(&node.name, 16);

        // Format dependencies
        let deps = node
            .dependencies
            .as_ref()
            .map(|d| {
                if d.is_empty() {
                    "-".to_string()
                } else {
                    d.iter()
                        .map(|id| id.to_string())
                        .collect::<Vec<_>>()
                        .join(",")
                }
            })
            .unwrap_or_else(|| "-".to_string());
        let deps = truncate_or_pad(&deps, 10);

        // Format profiling data
        let profile = if let Some(ref profiles) = node.profiles {
            profiles
                .iter()
                .map(|p| format!("rows:{},time:{}us", p.rows, p.exec_duration_in_us))
                .collect::<Vec<_>>()
                .join(";")
        } else {
            "-".to_string()
        };
        let profile = truncate_or_pad(&profile, 16);

        // Format operator info
        let info = node
            .description
            .as_ref()
            .map(|descs| {
                descs
                    .iter()
                    .map(|p| format!("{}:{}", p.key, p.value))
                    .collect::<Vec<_>>()
                    .join(",")
            })
            .unwrap_or_else(|| "-".to_string());
        let info = truncate_or_pad(&info, 48);

        // Format output variable
        let output_var_str = if node.output_var.is_empty() {
            "-"
        } else {
            &node.output_var
        };
        let output_var = truncate_or_pad(output_var_str, 16);

        output.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} |\n",
            id, name, deps, profile, info, output_var
        ));
        output.push_str("+------+------------------+------------+------------------+--------------------------------------------------+------------------+\n");
    }

    output
}

/// Format plan description as DOT (Graphviz) format
pub fn format_plan_as_dot(plan_desc: &PlanDescription) -> String {
    let mut output = String::new();

    output.push_str("digraph G {\n");
    output.push_str("    rankdir=BT;\n");  // Bottom to top layout for better flow visualization
    output.push_str("    node[shape=box, style=filled, fillcolor=lightblue, fontname=\"Arial\"];\n");
    output.push_str("    edge[arrowhead=none, fontname=\"Arial\"];\n\n");

    // Find root nodes (nodes that are not dependencies of any other node)
    let mut all_deps = std::collections::HashSet::new();
    for node in &plan_desc.plan_node_descs {
        if let Some(ref deps) = node.dependencies {
            for dep_id in deps {
                all_deps.insert(*dep_id);
            }
        }
    }

    // Nodes
    for node in &plan_desc.plan_node_descs {
        let label = format_plan_node_label(node);
        let is_root = !all_deps.contains(&node.id);
        let fillcolor = if is_root { "lightgreen" } else { "lightblue" };
        output.push_str(&format!(
            "    {}[label={}, fillcolor={}];\n",
            node.id,
            escape_dot_label(&label),
            fillcolor
        ));
    }

    output.push('\n');

    // Edges with labels showing the relationship
    for node in &plan_desc.plan_node_descs {
        if let Some(ref deps) = node.dependencies {
            for (idx, dep_id) in deps.iter().enumerate() {
                let edge_label = if deps.len() > 1 {
                    format!("label=\"input{}\"", idx + 1)
                } else {
                    "".to_string()
                };
                if edge_label.is_empty() {
                    output.push_str(&format!("    {} -> {};\n", node.id, dep_id));
                } else {
                    output.push_str(&format!("    {} -> {} [{}];\n", node.id, dep_id, edge_label));
                }
            }
        }
    }

    output.push('}');
    output
}

/// Format plan description as a tree structure
pub fn format_plan_as_tree(plan_desc: &PlanDescription) -> String {
    let mut output = String::new();

    // Build a map of node id to node for quick lookup
    let node_map: std::collections::HashMap<i64, &PlanNodeDescription> = plan_desc
        .plan_node_descs
        .iter()
        .map(|n| (n.id, n))
        .collect();

    // Build parent relationships
    let mut parent_map: std::collections::HashMap<i64, Vec<i64>> = std::collections::HashMap::new();
    for node in &plan_desc.plan_node_descs {
        if let Some(ref deps) = node.dependencies {
            for dep_id in deps {
                parent_map.entry(*dep_id).or_default().push(node.id);
            }
        }
    }

    // Find root nodes (nodes that are not dependencies of any other node)
    let mut root_nodes: Vec<&PlanNodeDescription> = Vec::new();
    for node in &plan_desc.plan_node_descs {
        let is_root = parent_map.get(&node.id).map(|v| v.is_empty()).unwrap_or(true)
            && node.dependencies.as_ref().map(|d| !d.is_empty()).unwrap_or(false);
        if is_root || parent_map.get(&node.id).is_none() {
            root_nodes.push(node);
        }
    }

    // If no clear root found, use nodes with no parents
    if root_nodes.is_empty() {
        for node in &plan_desc.plan_node_descs {
            if parent_map.get(&node.id).is_none() {
                root_nodes.push(node);
            }
        }
    }

    // Format tree starting from root nodes
    for (idx, root) in root_nodes.iter().enumerate() {
        if idx > 0 {
            output.push_str("\n");
        }
        format_tree_node(&mut output, root, &node_map, 0, true, &std::collections::HashSet::new());
    }

    output
}

/// Recursively format a tree node
fn format_tree_node(
    output: &mut String,
    node: &PlanNodeDescription,
    node_map: &std::collections::HashMap<i64, &PlanNodeDescription>,
    depth: usize,
    is_last: bool,
    visited: &std::collections::HashSet<i64>,
) {
    // Check for cycles
    if visited.contains(&node.id) {
        let indent = "  ".repeat(depth);
        output.push_str(&format!("{}[{}] {} (cycle detected)\n", indent, node.id, node.name));
        return;
    }

    // Format current node
    let _indent = if depth > 0 {
        let prefix = "  ".repeat(depth - 1);
        if is_last {
            format!("{}└── ", prefix)
        } else {
            format!("{}├── ", prefix)
        }
    } else {
        String::new()
    };

    // Build node info string
    let mut info_parts = vec![format!("[{}] {}", node.id, node.name)];

    if let Some(ref desc) = node.description {
        for pair in desc.iter().take(3) {
            info_parts.push(format!("{}:{}", pair.key, pair.value));
        }
    }

    if !node.output_var.is_empty() {
        info_parts.push(format!("-> {}", &node.output_var));
    }

    output.push_str(&format!("{}\n", info_parts.join(" | ")));

    // Mark as visited
    let mut new_visited = visited.clone();
    new_visited.insert(node.id);

    // Format children (dependencies)
    if let Some(ref deps) = node.dependencies {
        let dep_count = deps.len();
        for (idx, dep_id) in deps.iter().enumerate() {
            if let Some(dep_node) = node_map.get(dep_id) {
                format_tree_node(
                    output,
                    dep_node,
                    node_map,
                    depth + 1,
                    idx == dep_count - 1,
                    &new_visited,
                );
            }
        }
    }
}

/// Format a single plan node label for DOT output
fn format_plan_node_label(node: &PlanNodeDescription) -> String {
    let mut lines = vec![node.name.clone()];

    if let Some(ref profiles) = node.profiles {
        for profile in profiles {
            lines.push(format!("rows: {}", profile.rows));
            lines.push(format!("time: {}us", profile.exec_duration_in_us));
        }
    }

    if let Some(ref desc) = node.description {
        for pair in desc {
            lines.push(format!("{}: {}", pair.key, pair.value));
        }
    }

    lines.join("\\n")
}

/// Escape a string for use in DOT label
fn escape_dot_label(s: &str) -> String {
    format!("\"{}\"", s.replace('"', "\\\""))
}

/// Truncate or pad a string to fit in a fixed-width column
fn truncate_or_pad(s: &str, width: usize) -> String {
    if s.len() > width {
        format!("{}...", &s[..width.saturating_sub(3)])
    } else {
        format!("{:width$}", s, width = width)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::planning::plan::core::explain::PlanNodeDescription;

    #[test]
    fn test_format_plan_as_table() {
        let mut plan_desc = PlanDescription::new();
        let node = PlanNodeDescription::new("ScanVertices", 1).with_description("table", "Person");
        plan_desc.add_node_desc(node);

        let output = format_plan_as_table(&plan_desc);
        assert!(output.contains("ScanVertices"));
        assert!(output.contains("Person"));
    }

    #[test]
    fn test_format_plan_as_dot() {
        let mut plan_desc = PlanDescription::new();
        let node = PlanNodeDescription::new("ScanVertices", 1).with_description("table", "Person");
        plan_desc.add_node_desc(node);

        let output = format_plan_as_dot(&plan_desc);
        assert!(output.contains("digraph G"));
        assert!(output.contains("ScanVertices"));
    }

    #[test]
    fn test_truncate_or_pad() {
        assert_eq!(truncate_or_pad("short", 10), "short     ");
        assert_eq!(truncate_or_pad("very long string", 10), "very lo...");
    }
}
