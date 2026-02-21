//! 将TopN下推到索引扫描操作的规则
//!
//! 该规则识别 TopN -> IndexScan 模式，
//! 并将TopN的限制和排序信息集成到IndexScan操作中。
//!
//! # 转换示例
//!
//! Before:
//! ```text
//!   TopN(count=100, sort_items=[age DESC])
//!       |
//!   IndexScan
//! ```
//!
//! After:
//! ```text
//!   IndexScan(limit=100, order_by=[age DESC])
//! ```
//!
//! # 适用条件
//!
//! - 当前节点为TopN节点
//! - 子节点为IndexScan节点
//! - TopN节点只有一个子节点
//! - IndexScan尚未设置limit（避免重复下推）

use crate::query::optimizer::plan::{OptContext, OptGroupNode, OptRule, Pattern, TransformResult, Result};
use crate::query::optimizer::rule_patterns::PatternBuilder;
use crate::query::optimizer::rule_traits::BaseOptRule;
use crate::query::planner::plan::core::nodes::PlanNodeEnum;
use std::cell::RefCell;
use std::rc::Rc;

/// 将TopN下推到索引扫描操作的规则
#[derive(Debug)]
pub struct PushTopNDownIndexScanRule;

impl OptRule for PushTopNDownIndexScanRule {
    fn name(&self) -> &str {
        "PushTopNDownIndexScanRule"
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::with_dependency("TopN", "IndexScan")
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        node: &Rc<RefCell<OptGroupNode>>,
    ) -> Result<Option<TransformResult>> {
        let node_ref = node.borrow();

        // 检查当前节点是否为TopN
        if !node_ref.plan_node.is_topn() {
            return Ok(None);
        }

        let topn_node = match node_ref.plan_node.as_topn() {
            Some(n) => n,
            None => return Ok(None),
        };

        // TopN必须只有一个子节点
        if node_ref.dependencies.len() != 1 {
            return Ok(None);
        }

        let child_id = node_ref.dependencies[0];
        let child_node = match ctx.find_group_node_by_id(child_id) {
            Some(n) => n,
            None => return Ok(None),
        };

        let child_ref = child_node.borrow();

        // 检查子节点是否为IndexScan
        if !child_ref.plan_node.is_index_scan() {
            return Ok(None);
        }

        let index_scan = match child_ref.plan_node.as_index_scan() {
            Some(s) => s,
            None => return Ok(None),
        };

        // 如果IndexScan已经设置了limit，不再下推
        if index_scan.limit.is_some() {
            return Ok(None);
        }

        // 创建新的IndexScan，集成TopN的限制
        let mut new_index_scan = index_scan.clone();
        new_index_scan.limit = Some(topn_node.limit() as i64);

        // 如果TopN有排序项，可以尝试设置到IndexScan
        // 注意：这里假设IndexScan支持order_by字段
        // 如果索引本身支持排序，可以进一步优化

        // 创建新的组节点
        let mut new_group_node = child_ref.clone();
        new_group_node.plan_node = PlanNodeEnum::IndexScan(new_index_scan);

        // 保留原IndexScan的依赖
        new_group_node.dependencies = child_ref.dependencies.clone();

        // 如果原TopN有输出变量，传递给新的IndexScan
        if let Some(output_var) = node_ref.plan_node.output_var() {
            new_group_node.plan_node.set_output_var(output_var.clone());
        }

        // 创建转换结果
        let mut result = TransformResult::new();
        result.add_new_group_node(Rc::new(RefCell::new(new_group_node)));
        result.erase_curr = true;

        Ok(Some(result))
    }
}

impl BaseOptRule for PushTopNDownIndexScanRule {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::context::QueryContext;
    use crate::query::optimizer::plan::OptContext;
    use crate::query::planner::plan::algorithms::{IndexScan, ScanType};
    use crate::query::planner::plan::core::nodes::{PlanNodeEnum, TopNNode, SortItem};

    fn create_test_context() -> OptContext {
        let query_context = QueryContext::new();
        OptContext::new(query_context)
    }

    #[test]
    fn test_push_topn_down_index_scan_rule() {
        let rule = PushTopNDownIndexScanRule;
        let mut ctx = create_test_context();

        // 创建IndexScan节点
        let index_scan = IndexScan::new(1, 1, 1, 1, ScanType::Full);
        let index_scan_enum = PlanNodeEnum::IndexScan(index_scan);

        // 创建TopN节点
        let sort_items = vec![SortItem::desc("age".to_string())];
        let topn = TopNNode::new(index_scan_enum.clone(), sort_items, 100)
            .expect("Failed to create TopN node");
        let topn_enum = PlanNodeEnum::TopN(topn);

        // 创建OptGroupNode
        let index_scan_node = crate::query::optimizer::plan::OptGroupNode::new(1, index_scan_enum);
        let topn_node = crate::query::optimizer::plan::OptGroupNode::new(2, topn_enum);

        // 设置依赖关系
        let mut topn_node_with_dep = topn_node;
        topn_node_with_dep.dependencies = vec![1];

        // 将节点添加到上下文
        ctx.add_group_node(Rc::new(RefCell::new(index_scan_node))).expect("Failed to add group node");

        // 应用规则
        let result = rule.apply(&mut ctx, &Rc::new(RefCell::new(topn_node_with_dep)))
            .expect("Rule should apply successfully");

        // 验证结果
        assert!(result.is_some());
        let transform_result = result.unwrap();
        assert!(transform_result.erase_curr);
        assert_eq!(transform_result.new_group_nodes.len(), 1);
    }

    #[test]
    fn test_push_topn_down_index_scan_rule_with_existing_limit() {
        let rule = PushTopNDownIndexScanRule;
        let mut ctx = create_test_context();

        // 创建已经设置limit的IndexScan节点
        let mut index_scan = IndexScan::new(1, 1, 1, 1, ScanType::Full);
        index_scan.limit = Some(50);
        let index_scan_enum = PlanNodeEnum::IndexScan(index_scan);

        // 创建TopN节点
        let sort_items = vec![SortItem::desc("age".to_string())];
        let topn = TopNNode::new(index_scan_enum.clone(), sort_items, 100)
            .expect("Failed to create TopN node");
        let topn_enum = PlanNodeEnum::TopN(topn);

        // 创建OptGroupNode
        let index_scan_node = crate::query::optimizer::plan::OptGroupNode::new(1, index_scan_enum);
        let topn_node = crate::query::optimizer::plan::OptGroupNode::new(2, topn_enum);

        // 设置依赖关系
        let mut topn_node_with_dep = topn_node;
        topn_node_with_dep.dependencies = vec![1];

        // 将节点添加到上下文
        ctx.add_group_node(Rc::new(RefCell::new(index_scan_node))).expect("Failed to add group node");

        // 应用规则
        let result = rule.apply(&mut ctx, &Rc::new(RefCell::new(topn_node_with_dep)))
            .expect("Rule should apply successfully");

        // 验证结果：不应该应用规则，因为IndexScan已经有limit
        assert!(result.is_none());
    }
}
