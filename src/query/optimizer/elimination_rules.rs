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

        // 通过直接检查依赖来确定是否可以消除去重操作，避免使用match_pattern
        if node.dependencies.len() == 1 {
            let child_dep_id = node.dependencies[0];
            if let Some(child_node) = ctx.find_group_node_by_plan_node_id(child_dep_id) {
                // 如果子操作已经产生唯一结果，则去重操作是不需要的
                match child_node.plan_node.kind() {
                    PlanNodeKind::IndexScan
                    | PlanNodeKind::GetVertices
                    | PlanNodeKind::GetEdges => {
                        // 某些操作已经产生唯一结果，可以移除去重
                        if !child_node.dependencies.is_empty() {
                            let grandchild_dep_id = child_node.dependencies[0];
                            if let Some(grandchild_node) =
                                ctx.find_group_node_by_plan_node_id(grandchild_dep_id)
                            {
                                let mut new_node = grandchild_node.clone();
                                // 保留当前节点的输出变量
                                if let Some(output_var) = node.plan_node.output_var() {
                                    new_node.plan_node.set_output_var(output_var.clone());
                                }
                                new_node.dependencies = grandchild_node.dependencies.clone();
                                return Ok(Some(new_node));
                            }
                        }
                        Ok(None) // 如果没有子节点依赖，返回None
                    }
                    _ => {
                        // 对于其他操作，我们可能需要去重
                        Ok(None)
                    }
                }
            } else {
                Ok(None)
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
        // 检查是否为去重节点
        if node.plan_node.kind() != PlanNodeKind::Dedup {
            return false;
        }

        // 检查是否有且只有一个依赖
        if node.dependencies.len() != 1 {
            return false;
        }

        // 在实际实现中，这里需要检查依赖节点的类型
        // 如果依赖节点已经产生唯一结果（如IndexScan、GetVertices等），则可以消除去重操作
        // 由于这里缺少上下文信息，暂时返回true以保持与apply方法的一致性
        true
    }

    fn get_replacement(
        &self,
        ctx: &mut OptContext,
        node: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        if node.dependencies.len() == 1 {
            let child_dep_id = node.dependencies[0];
            if let Some(child_node) = ctx.find_group_node_by_plan_node_id(child_dep_id) {
                // 检查子节点是否已经是唯一结果的操作
                match child_node.plan_node.kind() {
                    PlanNodeKind::IndexScan
                    | PlanNodeKind::GetVertices
                    | PlanNodeKind::GetEdges => {
                        // 这些操作已经产生唯一结果，可以移除去重
                        let mut new_node = child_node.clone();

                        // 保留当前节点的输出变量
                        if let Some(output_var) = node.plan_node.output_var() {
                            new_node.plan_node.set_output_var(output_var.clone());
                        }

                        new_node.dependencies = child_node.dependencies.clone();
                        return Ok(Some(new_node));
                    }
                    _ => {
                        // 其他类型不能移除去重
                        return Ok(None);
                    }
                }
            }
        }
        Ok(None)
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
            // 目前，我们检查是否有依赖，如果有，且投影是无操作的，则可以移除
            if !node.dependencies.is_empty() {
                let child_dep_id = node.dependencies[0];
                if let Some(child_node) = ctx.find_group_node_by_plan_node_id(child_dep_id) {
                    // 如果投影是无操作的(即不改变列)，则可以移除
                    if self.is_noop_projection(project_plan_node, child_node)? {
                        let mut new_node = child_node.clone();

                        // 保留当前节点的输出变量
                        if let Some(output_var) = node.plan_node.output_var() {
                            new_node.plan_node.set_output_var(output_var.clone());
                        }

                        new_node.dependencies = child_node.dependencies.clone();
                        return Ok(Some(new_node));
                    }
                }
            }
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

impl RemoveNoopProjectRule {
    /// 检查投影是否为无操作（即投影的列与输入列相同）
    fn is_noop_projection(
        &self,
        project_node: &ProjectPlanNode,
        child_node: &OptGroupNode,
    ) -> Result<bool, OptimizerError> {
        // 检查投影的表达式是否与输入的列匹配
        // 在实际实现中，我们需要比较投影表达式和输入列
        // 简单起见，我们检查投影表达式的数量和依赖节点的输出是否匹配
        Ok(false) // 暂时设为false，实际实现中需要比较投影表达式和输入
    }
}

impl EliminationRule for RemoveNoopProjectRule {
    fn can_eliminate(&self, node: &OptGroupNode) -> bool {
        // 检查是否为投影节点
        if node.plan_node.kind() != PlanNodeKind::Project {
            return false;
        }

        // 检查是否有且只有一个依赖
        if node.dependencies.len() != 1 {
            return false;
        }

        // 在实际实现中，这里需要检查投影是否为无操作
        // 由于这里缺少上下文信息，暂时返回true以保持与apply方法的一致性
        true
    }

    fn get_replacement(
        &self,
        ctx: &mut OptContext,
        node: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        if node.dependencies.len() == 1 {
            let child_dep_id = node.dependencies[0];
            if let Some(child_node) = ctx.find_group_node_by_plan_node_id(child_dep_id) {
                // 检查投影是否为无操作
                if let Some(project_plan_node) =
                    node.plan_node.as_any().downcast_ref::<ProjectPlanNode>()
                {
                    if self.is_noop_projection(project_plan_node, child_node)? {
                        let mut new_node = child_node.clone();

                        // 保留当前节点的输出变量
                        if let Some(output_var) = node.plan_node.output_var() {
                            new_node.plan_node.set_output_var(output_var.clone());
                        }

                        new_node.dependencies = child_node.dependencies.clone();
                        return Ok(Some(new_node));
                    }
                }
            }
        }
        Ok(None)
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

        // 检查添加操作是否可以消除
        // 例如，如果只有一个源或添加操作不必要
        if node.dependencies.len() == 1 {
            let child_dep_id = node.dependencies[0];
            if let Some(child_node) = ctx.find_group_node_by_plan_node_id(child_dep_id) {
                // 如果只有一个依赖，我们可以直接使用子节点替换
                let mut new_node = child_node.clone();

                // 保留当前节点的输出变量
                if let Some(output_var) = node.plan_node.output_var() {
                    new_node.plan_node.set_output_var(output_var.clone());
                }

                new_node.dependencies = child_node.dependencies.clone();
                return Ok(Some(new_node));
            }
        } else if node.dependencies.len() == 0 {
            // 如果没有依赖，我们可以创建一个空的等效节点
            return Ok(None);
        }

        Ok(None)
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

        // 当添加顶点操作只有一个依赖时，可以移除该操作
        node.dependencies.len() == 1
    }

    fn get_replacement(
        &self,
        ctx: &mut OptContext,
        node: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        if node.dependencies.len() == 1 {
            let child_dep_id = node.dependencies[0];
            if let Some(child_node) = ctx.find_group_node_by_plan_node_id(child_dep_id) {
                let mut new_node = child_node.clone();

                // 保留当前节点的输出变量
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

        // 检查是否只有一个依赖且依赖是连接操作
        if node.dependencies.len() == 1 {
            let child_dep_id = node.dependencies[0];
            if let Some(child_node) = ctx.find_group_node_by_plan_node_id(child_dep_id) {
                if matches!(
                    child_node.plan_node.kind(),
                    PlanNodeKind::InnerJoin
                        | PlanNodeKind::HashInnerJoin
                        | PlanNodeKind::HashLeftJoin
                ) {
                    // 如果添加顶点操作在连接下方，我们可能需要调整操作顺序或消除不必要的操作
                    // 简单起见，我们返回节点本身，实际实现可能更复杂
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
        ctx: &mut OptContext,
        node: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        if node.dependencies.len() == 1 {
            let child_dep_id = node.dependencies[0];
            if let Some(child_node) = ctx.find_group_node_by_plan_node_id(child_dep_id) {
                // 在实际实现中，我们可能需要根据具体情况决定如何替换
                // 目前简单地返回子节点
                let mut new_node = child_node.clone();

                // 保留当前节点的输出变量
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
        let project_node = Box::new(Project::new(1, ""));
        let opt_node = OptGroupNode::new(1, project_node);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_eliminate_append_vertices_rule() {
        let rule = EliminateAppendVerticesRule;
        let mut ctx = create_test_context();

        // 创建一个添加顶点节点
        let append_vertices_node = Box::new(AppendVertices::new(1, 0, vec![]));
        let opt_node = OptGroupNode::new(1, append_vertices_node);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_remove_append_vertices_below_join_rule() {
        let rule = RemoveAppendVerticesBelowJoinRule;
        let mut ctx = create_test_context();

        // 创建一个添加顶点节点
        let append_vertices_node = Box::new(AppendVertices::new(1, 0, vec![]));
        let opt_node = OptGroupNode::new(1, append_vertices_node);

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
