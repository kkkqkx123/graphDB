//! 将LIMIT下推到扫描边操作的规则
//!
//! 该规则识别 Limit -> ScanEdges 模式，
//! 并将LIMIT值集成到ScanEdges操作中。

use crate::query::optimizer::rule_patterns::PatternBuilder;

crate::define_limit_pushdown_rule! {
    /// 将LIMIT下推到扫描边操作的规则
    ///
    /// # 转换示例
    ///
    /// Before:
    /// ```text
    ///   Limit(100)
    ///       |
    ///   ScanEdges
    /// ```
    ///
    /// After:
    /// ```text
    ///   ScanEdges(limit=100)
    /// ```
    ///
    /// # 适用条件
    ///
    /// - 当前节点为Limit节点
    /// - 子节点为ScanEdges节点
    /// - Limit节点只有一个子节点
    pub struct PushLimitDownScanEdgesRule {
        target: ScanEdges,
        target_check: is_scan_edges,
        target_as: as_scan_edges,
        enum_variant: ScanEdges,
        pattern: PatternBuilder::with_dependency("Limit", "ScanEdges")
    }
}
