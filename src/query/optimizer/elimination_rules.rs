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

        // 使用 EliminationRule trait 的方法来保持一致性
        if self.can_eliminate(ctx, node) {
            self.get_replacement(ctx, node)
        } else {
            Ok(None)
        }
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::filter()
    }
}

impl BaseOptRule for EliminateFilterRule {}

impl EliminationRule for EliminateFilterRule {
    fn can_eliminate(&self, ctx: &OptContext, node: &OptGroupNode) -> bool {
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

        // 使用 EliminationRule trait 的方法来保持一致性
        if self.can_eliminate(ctx, node) {
            self.get_replacement(ctx, node)
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
    fn can_eliminate(&self, ctx: &OptContext, node: &OptGroupNode) -> bool {
        // 检查是否为去重节点
        if node.plan_node.kind() != PlanNodeKind::Dedup {
            return false;
        }

        // 检查是否有且只有一个依赖
        if node.dependencies.len() != 1 {
            return false;
        }

        // 检查依赖节点的类型
        let child_dep_id = node.dependencies[0];
        if let Some(child_node) = ctx.find_group_node_by_plan_node_id(child_dep_id) {
            // 如果依赖节点已经产生唯一结果（如IndexScan、GetVertices等），则可以消除去重操作
            matches!(
                child_node.plan_node.kind(),
                PlanNodeKind::IndexScan | PlanNodeKind::GetVertices | PlanNodeKind::GetEdges
            )
        } else {
            false
        }
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

        // 使用 EliminationRule trait 的方法来保持一致性
        if self.can_eliminate(ctx, node) {
            self.get_replacement(ctx, node)
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
        // 获取投影表达式
        let yield_expr = &project_node.yield_expr;
        
        // 如果投影表达式是 "*"，则表示投影所有列，这是无操作投影
        if yield_expr == "*" {
            return Ok(true);
        }
        
        // 获取子节点的输出列名
        let child_col_names = child_node.plan_node.col_names();
        
        // 如果子节点没有输出列，则无法判断，返回false
        if child_col_names.is_empty() {
            return Ok(false);
        }
        
        // 检查投影表达式是否包含别名或表达式
        if self.has_aliases_or_expressions(yield_expr)? {
            return Ok(false);
        }
        
        // 解析投影表达式，提取列名
        let projected_columns = self.extract_columns_from_yield_expr(yield_expr)?;
        
        // 如果投影的列与子节点的输出列完全相同，则是无操作投影
        if projected_columns.len() == child_col_names.len() {
            for (i, col_name) in projected_columns.iter().enumerate() {
                if i < child_col_names.len() && col_name != &child_col_names[i] {
                    return Ok(false);
                }
            }
            return Ok(true);
        }
        
        Ok(false)
    }
    
    /// 检查投影表达式是否包含别名或复杂表达式
    fn has_aliases_or_expressions(&self, yield_expr: &str) -> Result<bool, OptimizerError> {
        // 检查是否包含 AS 关键字（别名）
        if yield_expr.to_lowercase().contains(" as ") {
            return Ok(true);
        }
        
        // 按逗号分割表达式
        for part in yield_expr.split(',') {
            let part = part.trim();
            
            // 如果包含运算符，则是表达式
            if part.contains('+') || part.contains('-') || part.contains('*') || part.contains('/') {
                return Ok(true);
            }
            
            // 如果包含函数调用，则是表达式
            if part.contains('(') && part.contains(')') {
                return Ok(true);
            }
        }
        
        Ok(false)
    }
    
    /// 从投影表达式中提取列名
    fn extract_columns_from_yield_expr(&self, yield_expr: &str) -> Result<Vec<String>, OptimizerError> {
        // 简单实现：按逗号分割表达式，并去除空格
        // 在实际实现中，可能需要更复杂的解析来处理表达式和别名
        let mut columns = Vec::new();
        
        // 处理特殊情况：如果表达式为空或为 "*"
        if yield_expr.is_empty() || yield_expr == "*" {
            return Ok(columns);
        }
        
        // 按逗号分割表达式
        for part in yield_expr.split(',') {
            let part = part.trim();
            
            // 如果包含 AS 关键字，提取别名前的部分
            if let Some(as_pos) = part.to_lowercase().find(" as ") {
                let expr_part = part[..as_pos].trim();
                columns.push(expr_part.to_string());
            } else {
                // 否则直接使用整个部分
                columns.push(part.to_string());
            }
        }
        
        Ok(columns)
    }
}

impl EliminationRule for RemoveNoopProjectRule {
    fn can_eliminate(&self, ctx: &OptContext, node: &OptGroupNode) -> bool {
        // 检查是否为投影节点
        if node.plan_node.kind() != PlanNodeKind::Project {
            return false;
        }

        // 检查是否有且只有一个依赖
        if node.dependencies.len() != 1 {
            return false;
        }

        // 检查投影是否为无操作
        if let Some(project_plan_node) = node.plan_node.as_any().downcast_ref::<ProjectPlanNode>() {
            let child_dep_id = node.dependencies[0];
            if let Some(child_node) = ctx.find_group_node_by_plan_node_id(child_dep_id) {
                // 在实际实现中，需要比较投影表达式和输入列
                // 这里简化实现，假设投影不是无操作
                self.is_noop_projection(project_plan_node, child_node).unwrap_or(false)
            } else {
                false
            }
        } else {
            false
        }
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

        // 使用 EliminationRule trait 的方法来保持一致性
        if self.can_eliminate(ctx, node) {
            self.get_replacement(ctx, node)
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
    fn can_eliminate(&self, ctx: &OptContext, node: &OptGroupNode) -> bool {
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

        // 使用 EliminationRule trait 的方法来保持一致性
        if self.can_eliminate(ctx, node) {
            self.get_replacement(ctx, node)
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
    fn can_eliminate(&self, ctx: &OptContext, node: &OptGroupNode) -> bool {
        if node.plan_node.kind() != PlanNodeKind::AppendVertices {
            return false;
        }

        // 检查是否只有一个依赖且依赖是连接操作
        if node.dependencies.len() == 1 {
            let child_dep_id = node.dependencies[0];
            if let Some(child_node) = ctx.find_group_node_by_plan_node_id(child_dep_id) {
                return matches!(
                    child_node.plan_node.kind(),
                    PlanNodeKind::InnerJoin
                        | PlanNodeKind::HashInnerJoin
                        | PlanNodeKind::HashLeftJoin
                );
            }
        }

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
    use crate::query::planner::plan::IndexScan;

    fn create_test_context() -> OptContext {
        OptContext::new(QueryContext::default())
    }

    #[test]
    fn test_eliminate_filter_rule() {
        let rule = EliminateFilterRule;
        let mut ctx = create_test_context();

        // 创建一个带有永真式条件的过滤节点
        let filter_node = Box::new(Filter::new(1, "1 = 1"));
        let mut opt_node = OptGroupNode::new(1, filter_node);
        
        // 添加一个子节点作为依赖
        let child_node = Box::new(crate::query::planner::plan::operations::ScanVertices::new(2, 1));
        let child_opt_node = OptGroupNode::new(2, child_node);
        ctx.add_plan_node_and_group_node(2, &child_opt_node);
        opt_node.dependencies.push(2);

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
        let mut opt_node = OptGroupNode::new(1, dedup_node);
        
        // 添加一个IndexScan子节点作为依赖（IndexScan产生唯一结果）
        let child_node = Box::new(IndexScan::new(2, 1, 1, 1, "UNIQUE"));
        let child_opt_node = OptGroupNode::new(2, child_node);
        ctx.add_plan_node_and_group_node(2, &child_opt_node);
        opt_node.dependencies.push(2);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_remove_noop_project_rule() {
        let rule = RemoveNoopProjectRule;
        let mut ctx = create_test_context();

        // 创建一个子节点，设置输出列
        let mut child_node = Box::new(crate::query::planner::plan::operations::ScanVertices::new(2, 1));
        child_node.set_col_names(vec!["id".to_string(), "name".to_string(), "age".to_string()]);
        let child_opt_node = OptGroupNode::new(2, child_node);
        ctx.add_plan_node_and_group_node(2, &child_opt_node);

        // 测试1: 创建一个投影所有列的投影节点（应该被消除）
        let project_node_all = Box::new(Project::new(1, "*"));
        let mut opt_node_all = OptGroupNode::new(1, project_node_all);
        opt_node_all.dependencies.push(2);
        
        let result_all = rule.apply(&mut ctx, &opt_node_all).unwrap();
        assert!(result_all.is_some(), "投影所有列的节点应该被消除");

        // 测试2: 创建一个投影相同列的投影节点（应该被消除）
        let project_node_same = Box::new(Project::new(3, "id, name, age"));
        let mut opt_node_same = OptGroupNode::new(3, project_node_same);
        opt_node_same.dependencies.push(2);
        
        let result_same = rule.apply(&mut ctx, &opt_node_same).unwrap();
        assert!(result_same.is_some(), "投影相同列的节点应该被消除");

        // 测试3: 创建一个投影不同列的投影节点（不应该被消除）
        let project_node_diff = Box::new(Project::new(4, "id, name"));
        let mut opt_node_diff = OptGroupNode::new(4, project_node_diff);
        opt_node_diff.dependencies.push(2);
        
        let result_diff = rule.apply(&mut ctx, &opt_node_diff).unwrap();
        assert!(result_diff.is_none(), "投影不同列的节点不应该被消除");

        // 测试4: 创建一个投影带别名的节点（不应该被消除）
        let project_node_alias = Box::new(Project::new(5, "id as vertex_id, name as vertex_name, age"));
        let mut opt_node_alias = OptGroupNode::new(5, project_node_alias);
        opt_node_alias.dependencies.push(2);
        
        let result_alias = rule.apply(&mut ctx, &opt_node_alias).unwrap();
        assert!(result_alias.is_none(), "投影带别名的节点不应该被消除");
        
        // 测试5: 创建一个投影包含表达式的节点（不应该被消除）
        let project_node_expr = Box::new(Project::new(6, "id, name, age + 1"));
        let mut opt_node_expr = OptGroupNode::new(6, project_node_expr);
        opt_node_expr.dependencies.push(2);
        
        let result_expr = rule.apply(&mut ctx, &opt_node_expr).unwrap();
        assert!(result_expr.is_none(), "投影包含表达式的节点不应该被消除");
    }

    #[test]
    fn test_eliminate_append_vertices_rule() {
        let rule = EliminateAppendVerticesRule;
        let mut ctx = create_test_context();

        // 创建一个添加顶点节点
        let append_vertices_node = Box::new(AppendVertices::new(1, 1, vec![]));
        let mut opt_node = OptGroupNode::new(1, append_vertices_node);
        
        // 添加一个子节点作为依赖
        let child_node = Box::new(crate::query::planner::plan::operations::ScanVertices::new(2, 1));
        let child_opt_node = OptGroupNode::new(2, child_node);
        ctx.add_plan_node_and_group_node(2, &child_opt_node);
        opt_node.dependencies.push(2);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_remove_append_vertices_below_join_rule() {
        let rule = RemoveAppendVerticesBelowJoinRule;
        let mut ctx = create_test_context();

        // 创建一个添加顶点节点
        let append_vertices_node = Box::new(AppendVertices::new(1, 1, vec![]));
        let mut opt_node = OptGroupNode::new(1, append_vertices_node);
        
        // 添加一个HashInnerJoin子节点作为依赖
        let child_node = Box::new(crate::query::planner::plan::operations::HashInnerJoin::new(2));
        let child_opt_node = OptGroupNode::new(2, child_node);
        ctx.add_plan_node_and_group_node(2, &child_opt_node);
        opt_node.dependencies.push(2);

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
