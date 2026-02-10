//! 合并获取邻居和去重操作的规则

use crate::query::optimizer::rule_patterns::PatternBuilder;

/// 合并获取邻居和去重操作的规则
///
/// # 转换示例
///
/// Before:
/// ```text
///   GetNeighbors
///       |
///   Dedup
///       |
///   ScanVertices
/// ```
///
/// After:
/// ```text
///   GetNeighbors
///       |
///   ScanVertices
/// ```
///
/// # 适用条件
///
/// - 当前节点为GetNeighbors节点
/// - 子节点为Dedup节点
/// - 可以将去重操作合并到GetNeighbors中
crate::define_merge_rule! {
    pub struct MergeGetNbrsAndDedupRule {
        parent: GetNeighbors,
        parent_check: is_get_neighbors,
        child: Dedup,
        child_check: is_dedup,
        pattern: PatternBuilder::with_dependency("GetNeighbors", "Dedup")
    }
}
