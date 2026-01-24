//! 转换规则
//! 这些规则负责将计划节点转换为等效但更高效的节点

use super::optimizer::{OptContext, OptGroupNode, OptRule, OptimizerError, Pattern};
use super::rule_patterns::PatternBuilder;
use super::rule_traits::BaseOptRule;
use crate::query::planner::plan::core::nodes::PlanNodeEnum;

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
        if !node.plan_node.is_limit() {
            return Ok(None);
        }

        // 检查Limit下是否是Sort操作
        if node.dependencies.len() == 1 {
            let child_dep_id = node.dependencies[0];
            if let Some(child_node) = ctx.find_group_node_by_plan_node_id(child_dep_id) {
                if child_node.plan_node.is_sort() {
                    // 根据NebulaGraph的实现，将Limit和Sort转换为TopN
                    if let Some(limit_plan_node) = node.plan_node.as_limit() {
                        if let Some(sort_plan_node) = child_node.plan_node.as_sort() {
                            // 创建新的OptGroupNode
                            let mut new_node = child_node.clone(); // 从Sort节点克隆

                            // 获取Sort节点的输入作为TopN的输入
                            let sort_input = (*sort_plan_node.dependencies()[0].clone()).clone();

                            // 创建TopN节点并设置输出变量
                            let mut topn_node =
                                crate::query::planner::plan::core::nodes::TopNNode::new(
                                    sort_input,                           // 使用Sort的输入
                                    sort_plan_node.sort_items().to_vec(), // 使用Sort的排序项
                                    limit_plan_node.count(), // 使用Limit的计数值作为TopN的限制
                                )
                                .expect("TopN node should be created successfully");

                            // 保持输出变量不变
                            if let Some(output_var) = limit_plan_node.output_var() {
                                topn_node.set_output_var(output_var.clone());
                            }

                            new_node.plan_node = PlanNodeEnum::TopN(topn_node);

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
        // Limit节点，依赖一个Sort节点
        PatternBuilder::with_dependency("Limit", "Sort")
    }
}

impl BaseOptRule for TopNRule {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::context::execution::QueryContext;
    use crate::query::optimizer::optimizer::{OptContext, OptGroupNode};
    use crate::query::planner::plan::core::nodes::SortNode;

    fn create_test_context() -> OptContext {
        let _session_info = crate::api::session::session_manager::SessionInfo {
            session_id: 1,
            user_name: "test_user".to_string(),
            space_name: None,
            graph_addr: None,
            create_time: std::time::SystemTime::now(),
            last_access_time: std::time::SystemTime::now(),
            active_queries: 0,
            timezone: None,
        };
        let query_context = QueryContext::new();
        OptContext::new(query_context)
    }

    #[test]
    fn test_top_n_rule() {
        let rule = TopNRule;
        let mut ctx = create_test_context();

        // 创建一个Sort节点
        let sort_node = PlanNodeEnum::Sort(
            SortNode::new(
                PlanNodeEnum::Start(crate::query::planner::plan::core::nodes::StartNode::new()),
                vec![],
            )
            .expect("Sort node should be created successfully"),
        );
        let opt_node = OptGroupNode::new(1, sort_node);

        let result = rule
            .apply(&mut ctx, &opt_node)
            .expect("Rule should apply successfully");
        assert!(result.is_none());
    }
}
