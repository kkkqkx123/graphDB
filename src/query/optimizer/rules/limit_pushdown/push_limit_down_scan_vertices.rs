//! 将LIMIT下推到扫描顶点操作的规则
//!
//! 该规则识别 Limit -> ScanVertices 模式，
//! 并将LIMIT值集成到ScanVertices操作中。

use crate::query::optimizer::rule_patterns::PatternBuilder;

crate::define_limit_pushdown_rule! {
    /// 将LIMIT下推到扫描顶点操作的规则
    ///
    /// # 转换示例
    ///
    /// Before:
    /// ```text
    ///   Limit(100)
    ///       |
    ///   ScanVertices
    /// ```
    ///
    /// After:
    /// ```text
    ///   ScanVertices(limit=100)
    /// ```
    ///
    /// # 适用条件
    ///
    /// - 当前节点为Limit节点
    /// - 子节点为ScanVertices节点
    /// - Limit节点只有一个子节点
    pub struct PushLimitDownScanVerticesRule {
        target: ScanVertices,
        target_check: is_scan_vertices,
        target_as: as_scan_vertices,
        enum_variant: ScanVertices,
        pattern: PatternBuilder::with_dependency("Limit", "ScanVertices")
    }
}
