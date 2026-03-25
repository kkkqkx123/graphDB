//! Configure the parser.
//!
//! Responsible for parsing weight configurations and heuristic configurations.

use crate::query::executor::data_processing::graph_traversal::algorithms::{
    EdgeWeightConfig, HeuristicFunction,
};

/// Analyzing the weight expression results in the weight configuration.
///
/// Supported weight expression formats:
/// “No authority diagram”
/// “ranking”: Use the ranking of the edges as the weight.
/// Use the specified attribute name as the weight.
pub fn parse_weight_config(weight_expr: &Option<String>) -> EdgeWeightConfig {
    match weight_expr {
        None => EdgeWeightConfig::Unweighted,
        Some(expr) => {
            let expr_lower = expr.to_lowercase();
            if expr_lower == "ranking" {
                EdgeWeightConfig::Ranking
            } else {
                EdgeWeightConfig::Property(expr.clone())
            }
        }
    }
}

/// Parsing heuristic expressions into heuristic configurations
///
/// Supported heuristic expression formats:
/// Zero heuristics (degrades to Dijkstra’s algorithm).
/// - "distance(lat,lon)": 使用顶点的经纬度属性计算欧几里得距离
/// - "scale(factor)": 使用固定缩放因子
pub fn parse_heuristic_config(heuristic_expr: &Option<String>) -> HeuristicFunction {
    match heuristic_expr {
        None => HeuristicFunction::Zero,
        Some(expr) => {
            let expr_lower = expr.to_lowercase().replace(' ', "");

            // 解析 distance(lat,lon) 格式
            if expr_lower.starts_with("distance(") && expr_lower.ends_with(')') {
                let inner = &expr_lower[9..expr_lower.len() - 1];
                let parts: Vec<&str> = inner.split(',').collect();
                if parts.len() == 2 {
                    return HeuristicFunction::PropertyDistance(
                        parts[0].to_string(),
                        parts[1].to_string(),
                    );
                }
            }

            // 解析 scale(factor) 格式
            if expr_lower.starts_with("scale(") && expr_lower.ends_with(')') {
                let inner = &expr_lower[6..expr_lower.len() - 1];
                if let Ok(factor) = inner.parse::<f64>() {
                    return HeuristicFunction::ScaleFactor(factor);
                }
            }

            // The default option uses zero heuristics.
            HeuristicFunction::Zero
        }
    }
}
