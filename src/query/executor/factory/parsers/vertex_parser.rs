//! 顶点解析器
//!
//! 负责解析顶点ID字符串，从计划节点提取顶点ID

use crate::core::Value;
use crate::query::planning::plan::core::nodes::base::plan_node_enum::PlanNodeEnum;

/// 从 PlanNode 提取顶点 ID 列表
/// 用于多源最短路径等算法获取起始和目标顶点
pub fn extract_vertex_ids_from_node(node: &PlanNodeEnum) -> Vec<Value> {
    match node {
        PlanNodeEnum::GetVertices(n) => {
            vec![Value::from(format!("vertex_{}", n.id()))]
        }
        PlanNodeEnum::ScanVertices(n) => {
            vec![Value::from(format!("scan_{}", n.id()))]
        }
        PlanNodeEnum::Project(n) => {
            vec![Value::from(format!("project_{}", n.id()))]
        }
        PlanNodeEnum::Start(_) => {
            vec![Value::from("__start__")]
        }
        _ => {
            vec![Value::from(format!("node_{}", node.id()))]
        }
    }
}

/// 解析顶点ID字符串为 Value 列表
/// 支持逗号分隔的多个ID
pub fn parse_vertex_ids(src_vids: &str) -> Vec<Value> {
    src_vids
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| Value::String(s.to_string()))
        .collect()
}
