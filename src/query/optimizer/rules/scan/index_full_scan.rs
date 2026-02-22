//! 优化索引全扫描为更高效的全表扫描的规则
//!
//! 该规则识别索引全扫描操作，在特定场景下可以优化为更高效的全表扫描。
//!
//! # 适用条件
//!
//! - 节点是 IndexScan 节点
//! - 查询上下文中没有设置索引 ID（表示还没有被优化过）
//! - 存在可用的索引

use crate::query::optimizer::plan::{OptContext, OptGroupNode, OptRule, Pattern, TransformResult, Result};
use crate::query::optimizer::rule_patterns::PatternBuilder;
use crate::query::optimizer::rule_traits::BaseOptRule;
use crate::query::planner::plan::core::nodes::PlanNodeEnum;
use std::cell::RefCell;
use std::rc::Rc;

/// 优化索引全扫描为更高效的全表扫描的规则
#[derive(Debug)]
pub struct IndexFullScanRule;

impl OptRule for IndexFullScanRule {
    fn name(&self) -> &str {
        "IndexFullScanRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        node: &Rc<RefCell<OptGroupNode>>,
    ) -> Result<Option<TransformResult>> {
        let node_ref = node.borrow();

        if !node_ref.plan_node.is_index_scan() {
            return Ok(None);
        }

        let index_scan = match node_ref.plan_node.as_index_scan() {
            Some(scan) => scan,
            None => return Ok(None),
        };

        if index_scan.index_id > 0 {
            return Ok(None);
        }

        let space_id = index_scan.space_id;
        let tag_id = index_scan.tag_id;

        let available_index_id = match self.find_best_index(ctx, space_id, tag_id) {
            Some(id) => id,
            None => return Ok(None),
        };

        let mut new_index_scan = index_scan.clone();
        new_index_scan.index_id = available_index_id;

        let mut new_group_node = node_ref.clone();
        new_group_node.plan_node = PlanNodeEnum::IndexScan(new_index_scan);

        let mut transform_result = TransformResult::new();
        transform_result.add_new_group_node(Rc::new(RefCell::new(new_group_node)));
        transform_result.erase_curr = true;

        Ok(Some(transform_result))
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::index_scan()
    }
}

impl BaseOptRule for IndexFullScanRule {}

impl IndexFullScanRule {
    fn find_best_index(&self, ctx: &OptContext, space_id: u64, tag_id: i32) -> Option<i32> {
        let index_metadata_manager = ctx.qctx().index_metadata_manager()?;

        let space_name = match ctx.qctx().schema_manager()?.get_space_by_id(space_id) {
            Ok(Some(space)) => space.space_name,
            _ => return None,
        };

        let tag_name = match ctx.qctx().schema_manager()?.get_tag(&space_name, &tag_id.to_string()) {
            Ok(Some(tag)) => tag.tag_name,
            _ => return None,
        };

        let indexes = match index_metadata_manager.list_tag_indexes(space_id) {
            Ok(indexes) => indexes,
            Err(_) => return None,
        };

        let schema_indexes: Vec<_> = indexes
            .into_iter()
            .filter(|idx| idx.schema_name == tag_name)
            .collect();

        if schema_indexes.is_empty() {
            return None;
        }

        let best_index = schema_indexes
            .iter()
            .min_by_key(|idx| idx.fields.len())?;

        Some(best_index.id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::QueryContext;
    use crate::query::optimizer::plan::{OptContext, OptGroupNode};
    use crate::query::planner::plan::PlanNodeEnum;
    use std::sync::Arc;

    fn create_test_context() -> OptContext {
        let query_context = Arc::new(QueryContext::default());
        OptContext::new(query_context)
    }

    #[test]
    fn test_index_full_scan_rule_without_index_id() {
        use crate::query::planner::plan::algorithms::ScanType;

        let rule = IndexFullScanRule;
        let mut ctx = create_test_context();

        let index_scan_node =
            crate::query::planner::plan::algorithms::IndexScan::new(1, 1, 1, 0, ScanType::Range);
        let index_scan_enum = PlanNodeEnum::IndexScan(index_scan_node);

        let opt_node = OptGroupNode::new(1, index_scan_enum);

        let result = rule
            .apply(&mut ctx, &Rc::new(RefCell::new(opt_node)))
            .expect("Rule should apply successfully");
        // 当前规则实现返回 Ok(None)，因为元数据客户端没有可用的索引
        assert!(result.is_none());
    }

    #[test]
    fn test_index_full_scan_rule_with_index_id() {
        use crate::query::planner::plan::algorithms::ScanType;

        let rule = IndexFullScanRule;
        let mut ctx = create_test_context();

        let index_scan_node =
            crate::query::planner::plan::algorithms::IndexScan::new(1, 1, 1, 5, ScanType::Range);
        let index_scan_enum = PlanNodeEnum::IndexScan(index_scan_node);

        let opt_node = OptGroupNode::new(1, index_scan_enum);

        let result = rule
            .apply(&mut ctx, &Rc::new(RefCell::new(opt_node)))
            .expect("Rule should apply successfully");
        // 已经有索引 ID，不应该再次优化
        assert!(result.is_none());
    }
}
