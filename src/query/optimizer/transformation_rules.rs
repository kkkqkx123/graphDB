//! 转换规则
//! 这些规则负责将计划节点转换为等效但更高效的节点

use super::optimizer::{OptContext, OptGroupNode, OptRule, OptimizerError, Pattern};
use super::rule_patterns::PatternBuilder;
use super::rule_traits::BaseOptRule;
use crate::query::planner::plan::core::plan_node_traits::PlanNodeMutable;
use crate::query::planner::plan::PlanNodeKind;

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
        node: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        // 检查是否为Limit操作
        if node.plan_node.kind() != PlanNodeKind::Limit {
            return Ok(None);
        }

        // 检查Limit下是否是Sort操作
        if node.dependencies.len() == 1 {
            let child_dep_id = node.dependencies[0];
            if let Some(child_node) = ctx.find_group_node_by_plan_node_id(child_dep_id) {
                if child_node.plan_node.kind() == PlanNodeKind::Sort {
                    // 根据NebulaGraph的实现，将Limit和Sort转换为TopN
                    if let Some(limit_plan_node) =
                        node.plan_node
                            .as_any()
                            .downcast_ref::<crate::query::planner::plan::core::nodes::LimitNode>()
                    {
                        if let Some(sort_plan_node) = child_node
                            .plan_node
                            .as_any()
                            .downcast_ref::<crate::query::planner::plan::core::nodes::SortNode>(
                        ) {
                            // 创建新的OptGroupNode
                            let mut new_node = child_node.clone(); // 从Sort节点克隆

                            // 获取Sort节点的输入作为TopN的输入
                            let sort_input = child_node.plan_node.clone();
                            
                            // 创建TopN节点并设置输出变量
                            let mut topn_node = crate::query::planner::plan::core::nodes::TopNNode::new(
                                sort_input,                           // 使用Sort的输入
                                sort_plan_node.sort_items().to_vec(), // 使用Sort的排序项
                                limit_plan_node.count(), // 使用Limit的计数值作为TopN的限制
                            ).unwrap();

                            // 保持输出变量不变
                            if let Some(output_var) = node.plan_node.output_var() {
                                topn_node.set_output_var(output_var.clone());
                            }

                            new_node.plan_node = std::sync::Arc::new(topn_node);

                            // 保持原始Sort节点的依赖（即TopN的输入）
                            if !child_node.dependencies.is_empty() {
                                new_node.dependencies = child_node.dependencies.clone();
                            } else {
                                new_node.dependencies = vec![];
                            }

                            return Ok(Some(new_node));
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    fn pattern(&self) -> Pattern {
        use crate::query::planner::plan::PlanNodeKind;
        // Limit节点，依赖一个Sort节点
        PatternBuilder::with_dependency(PlanNodeKind::Limit, PlanNodeKind::Sort)
    }
}

impl BaseOptRule for TopNRule {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::context::QueryContext;
    use crate::query::optimizer::optimizer::{OptContext, OptGroupNode};
    use crate::query::planner::plan::Sort;

    fn create_test_context() -> OptContext {
        OptContext::new(QueryContext::default())
    }

    #[test]
    fn test_top_n_rule() {
        let rule = TopNRule;
        let mut ctx = create_test_context();

        // 创建一个Sort节点
        let sort_node = std::sync::Arc::new(Sort::new(1, vec!["col1".to_string()]))
            as std::sync::Arc<dyn crate::query::planner::plan::core::plan_node_traits::PlanNode>;
        let opt_node = OptGroupNode::new(1, sort_node);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        assert!(result.is_none());
    }
}
