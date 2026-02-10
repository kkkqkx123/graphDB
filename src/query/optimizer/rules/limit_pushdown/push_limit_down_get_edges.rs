//! 将LIMIT下推到获取边操作的规则
//!
//! 该规则识别 Limit -> GetEdges 模式，
//! 并将LIMIT值集成到GetEdges操作中。

use crate::query::optimizer::rule_patterns::PatternBuilder;

/// 将LIMIT下推到获取边操作的规则
///
/// # 转换示例
///
/// Before:
/// ```text
///   Limit(100)
///       |
///   GetEdges
/// ```
///
/// After:
/// ```text
///   GetEdges(limit=100)
/// ```
///
/// # 适用条件
///
/// - 当前节点为Limit节点
/// - 子节点为GetEdges节点
/// - Limit节点只有一个子节点
crate::define_limit_pushdown_rule! {
    pub struct PushLimitDownGetEdgesRule {
        target: GetEdges,
        target_check: is_get_edges,
        target_as: as_get_edges,
        enum_variant: GetEdges,
        pattern: PatternBuilder::with_dependency("Limit", "GetEdges")
    }
}
