//! 合并获取顶点和去重操作的规则

use crate::query::optimizer::rule_patterns::PatternBuilder;

crate::define_merge_rule! {
    /// 合并获取顶点和去重操作的规则
    ///
    /// # 转换示例
    ///
    /// Before:
    /// ```text
    ///   GetVertices
    ///       |
    ///   Dedup
    ///       |
    ///   ScanVertices
    /// ```
    ///
    /// After:
    /// ```text
    ///   GetVertices
    ///       |
    ///   ScanVertices
    /// ```
    ///
    /// # 适用条件
    ///
    /// - 当前节点为GetVertices节点
    /// - 子节点为Dedup节点
    /// - 可以将去重操作合并到GetVertices中
    pub struct MergeGetVerticesAndDedupRule {
        parent: GetVertices,
        parent_check: is_get_vertices,
        child: Dedup,
        child_check: is_dedup,
        pattern: PatternBuilder::with_dependency("GetVertices", "Dedup")
    }
}
