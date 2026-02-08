//! 通用索引扫描操作的规则
//!
//! 该规则提供索引扫描的基础优化功能，
//! 包括索引选择、扫描类型确定等。

use crate::query::optimizer::plan::{OptContext, OptGroupNode, OptRule, Pattern, TransformResult};
use crate::query::optimizer::rule_patterns::PatternBuilder;
use crate::query::optimizer::rule_traits::BaseOptRule;
use crate::query::planner::plan::algorithms::IndexScan;
use std::rc::Rc;
use std::cell::RefCell;

/// 通用索引扫描操作的规则
///
/// # 功能
///
/// - 验证索引扫描的有效性
/// - 优化索引扫描参数
/// - 选择最优的索引扫描策略
#[derive(Debug)]
pub struct UnionAllEdgeIndexScanRule;

impl OptRule for UnionAllEdgeIndexScanRule {
    fn name(&self) -> &str {
        "UnionAllEdgeIndexScanRule"
    }

    fn apply(
        &self,
        _ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> Result<Option<TransformResult>, crate::query::optimizer::engine::OptimizerError> {
        let node_ref = group_node.borrow();
        
        if !node_ref.plan_node.is_index_scan() {
            return Ok(None);
        }

        let index_scan = match node_ref.plan_node.as_index_scan() {
            Some(scan) => scan,
            None => return Ok(None),
        };

        if !is_valid_index_scan(index_scan) {
            return Ok(None);
        }

        let mut new_index_scan = index_scan.clone();
        optimize_index_scan(&mut new_index_scan);

        let mut new_index_scan_group_node = node_ref.clone();
        new_index_scan_group_node.plan_node = crate::query::planner::plan::PlanNodeEnum::IndexScan(new_index_scan);

        let mut result = TransformResult::new();
        result.add_new_group_node(Rc::new(RefCell::new(new_index_scan_group_node)));
        
        Ok(Some(result))
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::index_scan()
    }
}

impl BaseOptRule for UnionAllEdgeIndexScanRule {}

/// 检查索引扫描是否有效
fn is_valid_index_scan(index_scan: &IndexScan) -> bool {
    !index_scan.index_name().is_empty()
}

/// 优化索引扫描
fn optimize_index_scan(index_scan: &mut IndexScan) {
    if index_scan.scan_limits.is_empty() {
        index_scan.scan_type = "FULL".to_string();
    } else {
        index_scan.scan_type = "RANGE".to_string();
    }
}
