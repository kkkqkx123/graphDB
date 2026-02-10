//! 将LIMIT下推到获取顶点操作的规则
//!
//! 该规则识别 Limit -> GetVertices 模式，
//! 并将LIMIT值集成到GetVertices操作中。

use crate::query::optimizer::rule_patterns::PatternBuilder;

/// 将LIMIT下推到获取顶点操作的规则
///
/// # 转换示例
///
/// Before:
/// ```text
///   Limit(100)
///       |
///   GetVertices
/// ```
///
/// After:
/// ```text
///   GetVertices(limit=100)
/// ```
///
/// # 适用条件
///
/// - 当前节点为Limit节点
/// - 子节点为GetVertices节点
/// - Limit节点只有一个子节点
crate::define_limit_pushdown_rule! {
    pub struct PushLimitDownGetVerticesRule {
        target: GetVertices,
        target_check: is_get_vertices,
        target_as: as_get_vertices,
        enum_variant: GetVertices,
        pattern: PatternBuilder::with_dependency("Limit", "GetVertices")
    }
}
