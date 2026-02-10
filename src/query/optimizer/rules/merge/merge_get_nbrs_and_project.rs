//! 合并获取邻居和投影操作的规则

use crate::query::optimizer::rule_patterns::PatternBuilder;

/// 合并获取邻居和投影操作的规则
///
/// # 转换示例
///
/// Before:
/// ```text
///   GetNeighbors
///       |
///   Project(col1, col2)
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
/// - 子节点为Project节点
/// - 可以将投影操作合并到GetNeighbors中
crate::define_merge_rule! {
    pub struct MergeGetNbrsAndProjectRule {
        parent: GetNeighbors,
        parent_check: is_get_neighbors,
        child: Project,
        child_check: is_project,
        pattern: PatternBuilder::with_dependency("GetNeighbors", "Project")
    }
}
