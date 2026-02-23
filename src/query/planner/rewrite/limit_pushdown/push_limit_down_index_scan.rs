//! 将LIMIT下推到索引扫描操作的规则
//!
//! 该规则识别 Limit -> IndexScan 模式，
//! 并将LIMIT值集成到IndexScan操作中。

use crate::query::optimizer::plan::Pattern;



crate::define_limit_pushdown_rule! {
    /// 将LIMIT下推到索引扫描操作的规则
    ///
    /// # 转换示例
    ///
    /// Before:
    /// ```text
    ///   Limit(100)
    ///       |
    ///   IndexScan
    /// ```
    ///
    /// After:
    /// ```text
    ///   IndexScan(limit=100)
    /// ```
    ///
    /// # 适用条件
    ///
    /// - 当前节点为Limit节点
    /// - 子节点为IndexScan节点
    /// - Limit节点只有一个子节点
    pub struct PushLimitDownIndexScanRule {
        target: IndexScan,
        target_check: is_index_scan,
        target_as: as_index_scan,
        enum_variant: IndexScan,
        pattern: Pattern::new_with_name("Limit").with_dependency_name("IndexScan")
    }
}
