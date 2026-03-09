//! 配置解析器
//!
//! 负责解析权重配置和启发式配置

use crate::query::executor::data_processing::graph_traversal::algorithms::{
    EdgeWeightConfig, HeuristicFunction,
};

/// 解析权重表达式为权重配置
///
/// 支持的权重表达式格式：
/// - None: 无权图
/// - "ranking": 使用边的ranking作为权重
/// - 其他字符串: 使用指定属性名作为权重
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

/// 解析启发式表达式为启发式配置
///
/// 支持的启发式表达式格式：
/// - None: 零启发式（退化为Dijkstra）
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

            // 默认使用零启发式
            HeuristicFunction::Zero
        }
    }
}
