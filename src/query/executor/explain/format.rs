//! Plan Formatting Utilities
//!
//! Provides formatting for query plan descriptions in different output formats:
//! - Table format (default): Human-readable tabular output
//! - Dot format: Graphviz DOT format for visualization

use crate::query::planning::plan::core::explain::{PlanDescription, PlanNodeDescription};

/// Format plan description as a table
pub fn format_plan_as_table(plan_desc: &PlanDescription) -> String {
    let mut output = String::new();

    // Header
    output.push_str("+----+---------------+--------------+------------------+--------------------------------------------------+\n");
    output.push_str("| id | name          | dependencies | profiling_data   | operator info                                    |\n");
    output.push_str("+----+---------------+--------------+------------------+--------------------------------------------------+\n");

    // Rows
    for node in &plan_desc.plan_node_descs {
        let id = format!("{:>2}", node.id);
        let name = truncate_or_pad(&node.name, 13);

        let deps = node
            .dependencies
            .as_ref()
            .map(|d| {
                d.iter()
                    .map(|id| id.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_default();
        let deps = truncate_or_pad(&deps, 12);

        let profile = if let Some(ref profiles) = node.profiles {
            profiles
                .iter()
                .map(|p| format!("rows: {}, time: {}us", p.rows, p.exec_duration_in_us))
                .collect::<Vec<_>>()
                .join("\n")
        } else {
            "N/A".to_string()
        };
        let profile = truncate_or_pad(&profile, 16);

        let info = node
            .description
            .as_ref()
            .map(|descs| {
                descs
                    .iter()
                    .map(|p| format!("{}: {}", p.key, p.value))
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_default();
        let info = truncate_or_pad(&info, 48);

        output.push_str(&format!(
            "| {} | {} | {} | {} | {} |\n",
            id, name, deps, profile, info
        ));
        output.push_str("+----+---------------+--------------+------------------+--------------------------------------------------+\n");
    }

    output
}

/// Format plan description as DOT (Graphviz) format
pub fn format_plan_as_dot(plan_desc: &PlanDescription) -> String {
    let mut output = String::new();

    output.push_str("digraph G {\n");
    output.push_str("    node[shape=box, style=filled, fillcolor=lightblue];\n");
    output.push_str("    edge[arrowhead=none];\n\n");

    // Nodes
    for node in &plan_desc.plan_node_descs {
        let label = format_plan_node_label(node);
        output.push_str(&format!(
            "    {}[label={}];\n",
            node.id,
            escape_dot_label(&label)
        ));
    }

    output.push('\n');

    // Edges
    for node in &plan_desc.plan_node_descs {
        if let Some(ref deps) = node.dependencies {
            for dep_id in deps {
                output.push_str(&format!("    {} -> {};\n", node.id, dep_id));
            }
        }
    }

    output.push('}');
    output
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
