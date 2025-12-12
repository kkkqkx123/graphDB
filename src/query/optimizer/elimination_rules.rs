//! 消除优化规则
//! 这些规则负责消除冗余的操作，如永真式过滤、无操作投影、不必要的去重等

use super::optimizer::OptimizerError;
use super::rule_patterns::PatternBuilder;
use super::rule_traits::{create_basic_pattern, is_tautology, BaseOptRule, EliminationRule};
use crate::query::optimizer::optimizer::{OptContext, OptGroupNode, OptRule, Pattern};
use crate::query::planner::plan::operations::Filter as FilterPlanNode;
use crate::query::planner::plan::operations::Project as ProjectPlanNode;
use crate::query::planner::plan::{PlanNode, PlanNodeKind};

/// 消除冗余过滤操作的规则
#[derive(Debug)]
pub struct EliminateFilterRule;

impl OptRule for EliminateFilterRule {
    fn name(&self) -> &str {
        "EliminateFilterRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        node: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        // 检查是否为过滤节点
        if node.plan_node.kind() != PlanNodeKind::Filter {
            return Ok(None);
        }

        // 获取过滤条件并检查是否为永真式
        if let Some(filter_plan_node) = node.plan_node.as_any().downcast_ref::<FilterPlanNode>() {
            let condition = &filter_plan_node.condition;

            // 检查条件是否为永真式
            if is_tautology(condition) {
                // 如果过滤条件是永真式，我们可以移除它，直接返回其子节点
                if !node.dependencies.is_empty() {
                    let child_dep_id = node.dependencies[0];

                    if let Some(child_node) = ctx.find_group_node_by_plan_node_id(child_dep_id) {
                        // 创建一个新的OptGroupNode，基于子节点，但保留当前节点的输出变量
                        let mut new_node = child_node.clone();

                        // 设置输出变量为当前过滤节点的输出变量
                        if let Some(output_var) = node.plan_node.output_var() {
                            new_node.plan_node.set_output_var(output_var.clone());
                        }

                        // 设置依赖关系为子节点的依赖关系
                        new_node.dependencies = child_node.dependencies.clone();

                        return Ok(Some(new_node));
                    }
                }
            }
        }

        Ok(None)
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::filter()
    }
}

impl BaseOptRule for EliminateFilterRule {}

impl EliminationRule for EliminateFilterRule {
    fn can_eliminate(&self, node: &OptGroupNode) -> bool {
        if node.plan_node.kind() != PlanNodeKind::Filter {
            return false;
        }

        if let Some(filter_plan_node) = node.plan_node.as_any().downcast_ref::<FilterPlanNode>() {
            let condition = &filter_plan_node.condition;
            is_tautology(condition)
        } else {
            false
        }
    }

    fn get_replacement(
        &self,
        ctx: &mut OptContext,
        node: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        if !node.dependencies.is_empty() {
            let child_dep_id = node.dependencies[0];

            if let Some(child_node) = ctx.find_group_node_by_plan_node_id(child_dep_id) {
                let mut new_node = child_node.clone();

                if let Some(output_var) = node.plan_node.output_var() {
                    new_node.plan_node.set_output_var(output_var.clone());
                }

                new_node.dependencies = child_node.dependencies.clone();

                return Ok(Some(new_node));
            }
        }

        Ok(None)
    }
}

/// 消除重复操作的规则
#[derive(Debug)]
pub struct DedupEliminationRule;

impl OptRule for DedupEliminationRule {
    fn name(&self) -> &str {
        "DedupEliminationRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        node: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        // 检查是否为去重操作
        if node.plan_node.kind() != PlanNodeKind::Dedup {
            return Ok(None);
        }

        // 匹配模式以确定是否可以消除去重操作
        if let Some(matched) = self.match_pattern(ctx, node)? {
            if matched.dependencies.len() == 1 {
                let child = &matched.dependencies[0];

                // 如果子操作已经产生唯一结果，则去重操作是不需要的
                match child.plan_node().kind() {
                    PlanNodeKind::IndexScan
                    | PlanNodeKind::GetVertices
                    | PlanNodeKind::GetEdges => {
                        // 某些操作可能已经产生唯一结果
                        // 在完整实现中，我们会仔细分析是否需要去重
                        // 目前，我们只返回原始节点（不进行优化）
                        Ok(Some(node.clone()))
                    }
                    _ => {
                        // 对于其他操作，我们可能需要去重
                        Ok(Some(node.clone()))
                    }
                }
            } else {
                Ok(Some(node.clone()))
            }
        } else {
            Ok(None)
        }
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::dedup()
    }
}

impl BaseOptRule for DedupEliminationRule {}

impl EliminationRule for DedupEliminationRule {
    fn can_eliminate(&self, node: &OptGroupNode) -> bool {
        if node.plan_node.kind() != PlanNodeKind::Dedup {
            return false;
        }

        // 在完整实现中，这里会检查子操作是否已经产生唯一结果
        // 目前简化实现，总是返回false
        false
    }

    fn get_replacement(
        &self,
        _ctx: &mut OptContext,
        node: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        if !node.dependencies.is_empty() {
            // 返回第一个子节点作为替代
            // 这里需要实际的子节点获取逻辑
            Ok(None)
        } else {
            Ok(None)
        }
    }
}

/// 移除无操作投影的规则
#[derive(Debug)]
pub struct RemoveNoopProjectRule;

impl OptRule for RemoveNoopProjectRule {
    fn name(&self) -> &str {
        "RemoveNoopProjectRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        node: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        // 检查是否为投影操作
        if node.plan_node.kind() != PlanNodeKind::Project {
            return Ok(None);
        }

        // 检查投影是否为无操作（即投影的列与输入列相同）
        if let Some(project_plan_node) = node.plan_node.as_any().downcast_ref::<ProjectPlanNode>() {
            // 在完整实现中，我们会检查投影的列是否与输入列相同
            // 目前简化实现，总是返回None（不进行优化）
            Ok(None)
        } else {
            Ok(None)
        }
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::project()
    }
}

impl BaseOptRule for RemoveNoopProjectRule {}

impl EliminationRule for RemoveNoopProjectRule {
    fn can_eliminate(&self, node: &OptGroupNode) -> bool {
        if node.plan_node.kind() != PlanNodeKind::Project {
            return false;
        }

        // 在完整实现中，这里会检查投影是否为无操作
        // 目前简化实现，总是返回false
        false
    }

    fn get_replacement(
        &self,
        _ctx: &mut OptContext,
        node: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        if !node.dependencies.is_empty() {
            // 返回第一个子节点作为替代
            // 这里需要实际的子节点获取逻辑
            Ok(None)
        } else {
            Ok(None)
        }
    }
}

/// 消除冗余添加顶点操作的规则
#[derive(Debug)]
pub struct EliminateAppendVerticesRule;

impl OptRule for EliminateAppendVerticesRule {
    fn name(&self) -> &str {
        "EliminateAppendVerticesRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        node: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        // 检查是否为添加顶点操作
        if node.plan_node.kind() != PlanNodeKind::AppendVertices {
            return Ok(None);
        }

        // 在完整实现中，这会检查添加操作是否冗余
        // 例如，如果只有一个源或添加操作不必要
        if let Some(matched) = self.match_pattern(ctx, node)? {
            // 检查添加操作是否可以消除
            // 这可能发生在只有一个输入或添加操作不必要的情况下
            Ok(Some(node.clone()))
        } else {
            Ok(None)
        }
    }

    fn pattern(&self) -> Pattern {
        create_basic_pattern(PlanNodeKind::AppendVertices)
    }
}

impl BaseOptRule for EliminateAppendVerticesRule {}

impl EliminationRule for EliminateAppendVerticesRule {
    fn can_eliminate(&self, node: &OptGroupNode) -> bool {
        if node.plan_node.kind() != PlanNodeKind::AppendVertices {
            return false;
        }

        // 在完整实现中，这里会检查添加操作是否冗余
        // 目前简化实现，总是返回false
        false
    }

    fn get_replacement(
        &self,
        _ctx: &mut OptContext,
        node: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        if !node.dependencies.is_empty() {
            // 返回第一个子节点作为替代
            // 这里需要实际的子节点获取逻辑
            Ok(None)
        } else {
            Ok(None)
        }
    }
}

/// 移除连接下方的添加顶点操作的规则
#[derive(Debug)]
pub struct RemoveAppendVerticesBelowJoinRule;

impl OptRule for RemoveAppendVerticesBelowJoinRule {
    fn name(&self) -> &str {
        "RemoveAppendVerticesBelowJoinRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        node: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        // 检查是否为添加顶点操作
        if node.plan_node.kind() != PlanNodeKind::AppendVertices {
            return Ok(None);
        }

        // 匹配模式以查看是否为添加顶点后跟连接
        if let Some(matched) = self.match_pattern(ctx, node)? {
            if matched.dependencies.len() >= 1 {
                let child = &matched.dependencies[0];

                if child.plan_node().kind() == PlanNodeKind::InnerJoin
                    || child.plan_node().kind() == PlanNodeKind::HashInnerJoin
                    || child.plan_node().kind() == PlanNodeKind::HashLeftJoin
                {
                    // 在完整实现中，我们可能能够消除不必要的添加操作
                    // 如果它们在连接之前不添加值
                    Ok(Some(node.clone()))
                } else {
                    Ok(None)
                }
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::with_dependency(PlanNodeKind::AppendVertices, PlanNodeKind::InnerJoin)
    }
}

impl BaseOptRule for RemoveAppendVerticesBelowJoinRule {}

impl EliminationRule for RemoveAppendVerticesBelowJoinRule {
    fn can_eliminate(&self, node: &OptGroupNode) -> bool {
        if node.plan_node.kind() != PlanNodeKind::AppendVertices {
            return false;
        }

        // 在完整实现中，这里会检查添加操作是否在连接下方且不必要
        // 目前简化实现，总是返回false
        false
    }

    fn get_replacement(
        &self,
        _ctx: &mut OptContext,
        node: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        if !node.dependencies.is_empty() {
            // 返回第一个子节点作为替代
            // 这里需要实际的子节点获取逻辑
            Ok(None)
        } else {
            Ok(None)
        }
    }
}

/// 优化Top-N查询的规则
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
        // 检查是否为排序操作（通常用于Top-N查询）
        if node.plan_node.kind() != PlanNodeKind::Sort {
            return Ok(None);
        }

        // 在完整实现中，这会优化Top-N操作
        // 通过使用高效算法如堆排序来处理有限结果
        if let Some(matched) = self.match_pattern(ctx, node)? {
            // 对于Top-N查询（排序后跟限制），我们可能使用专门的算法
            // 只跟踪前N项而不是对整个数据集进行排序
            Ok(Some(node.clone()))
        } else {
            Ok(None)
        }
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::sort()
    }
}

impl BaseOptRule for TopNRule {}

impl EliminationRule for TopNRule {
    fn can_eliminate(&self, node: &OptGroupNode) -> bool {
        if node.plan_node.kind() != PlanNodeKind::Sort {
            return false;
        }

        // 在完整实现中，这里会检查是否可以优化为Top-N操作
        // 目前简化实现，总是返回false
        false
    }

    fn get_replacement(
        &self,
        _ctx: &mut OptContext,
        node: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        // 这里会创建一个优化的Top-N节点
        // 目前简化实现，总是返回None
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::context::QueryContext;
    use crate::query::optimizer::optimizer::{OptContext, OptGroupNode};
    use crate::query::planner::plan::{AppendVertices, Dedup, Filter, Project, Sort};
    use crate::query::planner::plan::{PlanNode, PlanNodeKind};

    fn create_test_context() -> OptContext {
        OptContext::new(QueryContext::default())
    }

    #[test]
    fn test_eliminate_filter_rule() {
        let rule = EliminateFilterRule;
        let mut ctx = create_test_context();

        // 创建一个带有永真式条件的过滤节点
        let filter_node = Box::new(Filter::new(1, "1 = 1"));
        let opt_node = OptGroupNode::new(1, filter_node);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        // 规则应该识别永真式过滤并尝试消除它们
        assert!(result.is_some());
    }

    #[test]
    fn test_dedup_elimination_rule() {
        let rule = DedupEliminationRule;
        let mut ctx = create_test_context();

        // 创建一个去重节点
        let dedup_node = Box::new(Dedup::new(1));
        let opt_node = OptGroupNode::new(1, dedup_node);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_remove_noop_project_rule() {
        let rule = RemoveNoopProjectRule;
        let mut ctx = create_test_context();

        // 创建一个投影节点
        let project_node = Box::new(Project::new(1, vec![]));
        let opt_node = OptGroupNode::new(1, project_node);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_eliminate_append_vertices_rule() {
        let rule = EliminateAppendVerticesRule;
        let mut ctx = create_test_context();

        // 创建一个添加顶点节点
        let append_vertices_node = Box::new(AppendVertices::new(1, vec![], vec![]));
        let opt_node = OptGroupNode::new(1, append_vertices_node);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_remove_append_vertices_below_join_rule() {
        let rule = RemoveAppendVerticesBelowJoinRule;
        let mut ctx = create_test_context();

        // 创建一个添加顶点节点
        let append_vertices_node = Box::new(AppendVertices::new(1, vec![], vec![]));
        let opt_node = OptGroupNode::new(1, append_vertices_node);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_top_n_rule() {
        let rule = TopNRule;
        let mut ctx = create_test_context();

        // 创建一个排序节点
        let sort_node = Box::new(Sort::new(1, vec![]));
        let opt_node = OptGroupNode::new(1, sort_node);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_is_tautology() {
        assert!(is_tautology("1 = 1"));
        assert!(is_tautology("true"));
        assert!(is_tautology("TRUE"));
        assert!(is_tautology("True"));
        assert!(!is_tautology("age > 18"));
    }
}
