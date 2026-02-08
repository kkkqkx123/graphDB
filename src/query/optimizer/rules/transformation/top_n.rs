//! 转换Limit-Sort为TopN的规则
//!
//! 该规则识别 Limit -> Sort 模式，并将其转换为更高效的 TopN 操作。
//!
//! # 转换示例
//!
//! Before:
//! ```text
//! Limit(count)
//!     |
//!   Sort(items)
//!     |
//!   Input
//! ```
//!
//! After:
//! ```text
//! TopN(count, items)
//!         |
//!       Input
//! ```
//!
//! # 适用条件
//!
//! - Limit 节点只有一个子节点
//! - 子节点是 Sort 节点
//! - Limit 的 offset 为 0（当前无法知道输入数据的总量，只有offset为0时才应用）
//! - Sort 节点有排序项

use crate::query::optimizer::plan::{OptContext, OptGroupNode, OptRule, Pattern, TransformResult, Result as OptResult};
use crate::query::optimizer::rule_patterns::PatternBuilder;
use crate::query::optimizer::rule_traits::BaseOptRule;
use crate::query::planner::plan::core::nodes::PlanNodeEnum;
use crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode;
use std::cell::RefCell;
use std::rc::Rc;

/// 转换Limit-Sort为TopN的规则
#[derive(Debug)]
pub struct TopNRule;

impl OptRule for TopNRule {
    fn name(&self) -> &str {
        "TopNRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        node: &Rc<RefCell<OptGroupNode>>,
    ) -> OptResult<Option<TransformResult>> {
        let node_ref = node.borrow();

        if !node_ref.plan_node.is_limit() {
            return Ok(None);
        }

        let limit_node = match node_ref.plan_node.as_limit() {
            Some(n) => n,
            None => return Ok(None),
        };

        let limit_offset = limit_node.offset();
        let limit_count = limit_node.count();

        if limit_offset != 0 {
            return Ok(None);
        }

        if node_ref.dependencies.is_empty() {
            return Ok(None);
        }

        let child_dep_id = node_ref.dependencies[0];
        let child_node = match ctx.find_group_node_by_plan_node_id(child_dep_id) {
            Some(n) => n,
            None => return Ok(None),
        };

        let child_node_ref = child_node.borrow();

        if !child_node_ref.plan_node.is_sort() {
            return Ok(None);
        }

        let sort_node = match child_node_ref.plan_node.as_sort() {
            Some(n) => n,
            None => return Ok(None),
        };

        let sort_items = sort_node.sort_items().to_vec();
        let sort_input = SingleInputNode::input(sort_node).clone();

        let topn_node = PlanNodeEnum::TopN(
            crate::query::planner::plan::core::nodes::TopNNode::new(
                sort_input,
                sort_items,
                limit_count,
            )
            .expect("TopN node should be created successfully"),
        );

        let mut new_group_node = child_node_ref.clone();
        new_group_node.plan_node = topn_node;

        if !child_node_ref.dependencies.is_empty() {
            new_group_node.dependencies = child_node_ref.dependencies.clone();
        } else {
            new_group_node.dependencies = vec![];
        }

        drop(child_node_ref);

        let mut transform_result = TransformResult::new();
        transform_result.add_new_group_node(Rc::new(RefCell::new(new_group_node)));
        transform_result.erase_all = true;

        Ok(Some(transform_result))
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::with_dependency("Limit", "Sort")
    }
}

impl BaseOptRule for TopNRule {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::context::execution::QueryContext;
    use crate::query::optimizer::plan::{OptContext, OptGroupNode};
    use crate::query::planner::plan::core::nodes::{LimitNode, SortNode};

    fn create_test_context() -> OptContext {
        let query_context = QueryContext::new();
        OptContext::new(query_context)
    }

    #[test]
    fn test_top_n_rule_with_offset_zero() {
        let rule = TopNRule;
        let mut ctx = create_test_context();

        let start_node = PlanNodeEnum::Start(crate::query::planner::plan::core::nodes::StartNode::new());

        let sort_node = PlanNodeEnum::Sort(
            SortNode::new(start_node.clone(), vec!["name".to_string()])
                .expect("Sort node should be created successfully"),
        );
        let sort_node_id = sort_node.id() as usize;

        // 注册 Sort 节点到上下文（使用克隆的节点）
        let sort_opt_node = Rc::new(RefCell::new(OptGroupNode::new(2, sort_node)));
        ctx.add_group_node(sort_opt_node).expect("Failed to add sort node");

        // 创建 Limit 节点
        let sort_node_for_limit = PlanNodeEnum::Sort(
            SortNode::new(start_node, vec!["name".to_string()])
                .expect("Sort node should be created successfully"),
        );
        let limit_node = PlanNodeEnum::Limit(
            LimitNode::new(sort_node_for_limit, 0, 10)
                .expect("Limit node should be created successfully"),
        );

        // 创建 Limit 节点并设置依赖
        let mut limit_opt_node = OptGroupNode::new(1, limit_node);
        limit_opt_node.dependencies = vec![sort_node_id];

        let result = rule
            .apply(&mut ctx, &Rc::new(RefCell::new(limit_opt_node)))
            .expect("Rule should apply successfully");

        assert!(result.is_some());
    }

    #[test]
    fn test_top_n_rule_with_non_zero_offset() {
        let rule = TopNRule;
        let mut ctx = create_test_context();

        let start_node = PlanNodeEnum::Start(crate::query::planner::plan::core::nodes::StartNode::new());

        let sort_node = PlanNodeEnum::Sort(
            SortNode::new(start_node.clone(), vec!["name".to_string()])
                .expect("Sort node should be created successfully"),
        );
        let sort_node_id = sort_node.id() as usize;

        // 注册 Sort 节点到上下文
        let sort_opt_node = Rc::new(RefCell::new(OptGroupNode::new(2, sort_node)));
        ctx.add_group_node(sort_opt_node).expect("Failed to add sort node");

        // 创建 Limit 节点
        let sort_node_for_limit = PlanNodeEnum::Sort(
            SortNode::new(start_node, vec!["name".to_string()])
                .expect("Sort node should be created successfully"),
        );
        let limit_node = PlanNodeEnum::Limit(
            LimitNode::new(sort_node_for_limit, 5, 10)
                .expect("Limit node should be created successfully"),
        );

        // 创建 Limit 节点并设置依赖
        let mut limit_opt_node = OptGroupNode::new(1, limit_node);
        limit_opt_node.dependencies = vec![sort_node_id];

        let result = rule
            .apply(&mut ctx, &Rc::new(RefCell::new(limit_opt_node)))
            .expect("Rule should apply successfully");

        assert!(result.is_none());
    }

    #[test]
    fn test_top_n_rule_without_sort() {
        let rule = TopNRule;
        let mut ctx = create_test_context();

        let start_node = PlanNodeEnum::Start(crate::query::planner::plan::core::nodes::StartNode::new());
        let start_node_id = start_node.id() as usize;

        let limit_node = PlanNodeEnum::Limit(
            LimitNode::new(start_node, 0, 10)
                .expect("Limit node should be created successfully"),
        );

        // 注册 Start 节点到上下文
        let start_opt_node = Rc::new(RefCell::new(OptGroupNode::new(2, PlanNodeEnum::Start(crate::query::planner::plan::core::nodes::StartNode::new()))));
        ctx.add_group_node(start_opt_node).expect("Failed to add start node");

        // 创建 Limit 节点并设置依赖
        let mut limit_opt_node = OptGroupNode::new(1, limit_node);
        limit_opt_node.dependencies = vec![start_node_id];

        let result = rule
            .apply(&mut ctx, &Rc::new(RefCell::new(limit_opt_node)))
            .expect("Rule should apply successfully");

        assert!(result.is_none());
    }
}
