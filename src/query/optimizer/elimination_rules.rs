//! 消除优化规则
//! 这些规则负责消除冗余的操作，如永真式过滤、无操作投影、不必要的去重等

use super::optimizer::OptimizerError;
use super::rule_patterns::PatternBuilder;
use super::rule_traits::{
    create_basic_pattern, is_expression_tautology, BaseOptRule, EliminationRule,
};
use crate::query::optimizer::optimizer::{OptContext, OptGroupNode, OptRule, Pattern};
use crate::query::planner::plan::PlanNodeEnum;
use crate::query::planner::plan::ProjectNode;

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
        if !node.plan_node.is_filter() {
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
    fn can_eliminate(&self, _ctx: &OptContext, node: &OptGroupNode) -> bool {
        if !node.plan_node.is_filter() {
            return false;
        }

        if let Some(filter_plan_node) = node.plan_node.as_filter() {
            let condition = filter_plan_node.condition();
            is_expression_tautology(condition)
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
                // 创建一个全新的节点，而不是修改现有的节点
                let new_plan_node = child_node.plan_node.clone();

                // 创建新的OptGroupNode
                let mut new_node = OptGroupNode {
                    id: child_node.id,
                    plan_node: new_plan_node,
                    dependencies: child_node.dependencies.clone(),
                    cost: child_node.cost,
                    properties: child_node.properties.clone(),
                    explored_rules: child_node.explored_rules.clone(),
                    group_id: child_node.group_id,
                };

                // 尝试设置输出变量
                if let Some(output_var) = node.plan_node.output_var() {
                    // 创建一个新的计划节点并设置输出变量
                    // 由于PlanNodeEnum是不可变的，我们需要基于原节点创建一个新节点
                    // 这需要PlanNode有具体类型才能设置输出变量
                    let new_plan_node_with_output =
                        create_plan_node_with_output_var(&child_node.plan_node, output_var.clone());
                    new_node.plan_node = new_plan_node_with_output;
                }

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
        if !node.plan_node.is_dedup() {
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
        if !node.plan_node.is_dedup() {
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
            child_node.plan_node.is_index_scan()
                || child_node.plan_node.is_get_vertices()
                || child_node.plan_node.is_get_edges()
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
                if child_node.plan_node.is_index_scan()
                    || child_node.plan_node.is_get_vertices()
                    || child_node.plan_node.is_get_edges()
                {
                    // 这些操作已经产生唯一结果，可以移除去重
                    let new_plan_node = child_node.plan_node.clone();

                    // 创建新的OptGroupNode
                    let mut new_node = OptGroupNode {
                        id: child_node.id,
                        plan_node: new_plan_node,
                        dependencies: child_node.dependencies.clone(),
                        cost: child_node.cost,
                        properties: child_node.properties.clone(),
                        explored_rules: child_node.explored_rules.clone(),
                        group_id: child_node.group_id,
                    };

                    // 保留当前节点的输出变量
                    if let Some(output_var) = node.plan_node.output_var() {
                        new_node.plan_node = create_plan_node_with_output_var(
                            &child_node.plan_node,
                            output_var.clone(),
                        );
                    }

                    return Ok(Some(new_node));
                } else {
                    // 其他类型不能移除去重
                    return Ok(None);
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
        if !node.plan_node.is_project() {
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
        project_node: &ProjectNode,
        child_node: &OptGroupNode,
    ) -> Result<bool, OptimizerError> {
        // 获取投影列
        let columns = project_node.columns();

        // 如果投影列为空，则无法判断，返回false
        if columns.is_empty() {
            return Ok(false);
        }

        // 获取子节点的输出列名
        let child_col_names = child_node.plan_node.col_names();

        // 如果子节点没有输出列，则无法判断，返回false
        if child_col_names.is_empty() {
            // 如果投影只有一列且为 "*"，认为是无操作投影
            if columns.len() == 1 {
                if let crate::core::Expression::Variable(var_name) = &columns[0].expr {
                    if var_name == "*" {
                        return Ok(true);
                    }
                }
            }
            // 如果投影列都是简单的变量引用且没有别名，认为是无操作投影
            return Ok(true);
        }

        // 检查投影列是否包含别名或表达式
        if self.has_aliases_or_expressions_in_columns(columns)? {
            return Ok(false);
        }

        // 从投影列中提取列名
        let projected_columns: Vec<String> = columns.iter().map(|col| col.alias.clone()).collect();

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

    /// 检查投影列是否包含别名或复杂表达式
    fn has_aliases_or_expressions_in_columns(
        &self,
        columns: &[crate::query::validator::YieldColumn],
    ) -> Result<bool, OptimizerError> {
        for column in columns {
            // 检查是否是表达式（不是简单的变量引用）
            match &column.expr {
                crate::core::Expression::Variable(_) => {
                    // 简单变量，继续检查
                }
                _ => {
                    // 其他类型的表达式，认为是复杂表达式
                    return Ok(true);
                }
            }

            // 检查别名是否与原始表达式不同
            if let crate::core::Expression::Variable(var_name) = &column.expr {
                if var_name != &column.alias {
                    // 别名与变量名不同，认为是别名
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }
}

impl EliminationRule for RemoveNoopProjectRule {
    fn can_eliminate(&self, ctx: &OptContext, node: &OptGroupNode) -> bool {
        // 检查是否为投影节点
        if !node.plan_node.is_project() {
            return false;
        }

        // 检查是否有且只有一个依赖
        if node.dependencies.len() != 1 {
            return false;
        }

        // 检查投影是否为无操作
        if let Some(project_plan_node) = node.plan_node.as_project() {
            let child_dep_id = node.dependencies[0];
            if let Some(child_node) = ctx.find_group_node_by_plan_node_id(child_dep_id) {
                // 在实际实现中，需要比较投影表达式和输入列
                // 这里简化实现，假设投影不是无操作
                self.is_noop_projection(project_plan_node, child_node)
                    .unwrap_or(false)
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
                if let Some(project_plan_node) = node.plan_node.as_project() {
                    if self.is_noop_projection(project_plan_node, child_node)? {
                        let new_plan_node = child_node.plan_node.clone();

                        // 创建新的OptGroupNode
                        let mut new_node = OptGroupNode {
                            id: child_node.id,
                            plan_node: new_plan_node,
                            dependencies: child_node.dependencies.clone(),
                            cost: child_node.cost,
                            properties: child_node.properties.clone(),
                            explored_rules: child_node.explored_rules.clone(),
                            group_id: child_node.group_id,
                        };

                        // 保留当前节点的输出变量
                        if let Some(output_var) = node.plan_node.output_var() {
                            new_node.plan_node = create_plan_node_with_output_var(
                                &child_node.plan_node,
                                output_var.clone(),
                            );
                        }

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
        if !node.plan_node.is_append_vertices() {
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
        create_basic_pattern("AppendVertices")
    }
}

impl BaseOptRule for EliminateAppendVerticesRule {}

impl EliminationRule for EliminateAppendVerticesRule {
    fn can_eliminate(&self, _ctx: &OptContext, node: &OptGroupNode) -> bool {
        if !node.plan_node.is_append_vertices() {
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
                let new_plan_node = child_node.plan_node.clone();

                // 创建新的OptGroupNode
                let mut new_node = OptGroupNode {
                    id: child_node.id,
                    plan_node: new_plan_node,
                    dependencies: child_node.dependencies.clone(),
                    cost: child_node.cost,
                    properties: child_node.properties.clone(),
                    explored_rules: child_node.explored_rules.clone(),
                    group_id: child_node.group_id,
                };

                // 保留当前节点的输出变量
                if let Some(output_var) = node.plan_node.output_var() {
                    new_node.plan_node =
                        create_plan_node_with_output_var(&child_node.plan_node, output_var.clone());
                }

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
        if !node.plan_node.is_append_vertices() {
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
        PatternBuilder::with_dependency("AppendVertices", "InnerJoin")
    }
}

impl BaseOptRule for RemoveAppendVerticesBelowJoinRule {}

impl EliminationRule for RemoveAppendVerticesBelowJoinRule {
    fn can_eliminate(&self, ctx: &OptContext, node: &OptGroupNode) -> bool {
        if !node.plan_node.is_append_vertices() {
            return false;
        }

        // 检查是否只有一个依赖且依赖是连接操作
        if node.dependencies.len() == 1 {
            let child_dep_id = node.dependencies[0];
            if let Some(child_node) = ctx.find_group_node_by_plan_node_id(child_dep_id) {
                return child_node.plan_node.is_inner_join()
                    || child_node.plan_node.is_hash_inner_join()
                    || child_node.plan_node.is_hash_left_join();
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
                let new_plan_node = child_node.plan_node.clone();

                // 创建新的OptGroupNode
                let mut new_node = OptGroupNode {
                    id: child_node.id,
                    plan_node: new_plan_node,
                    dependencies: child_node.dependencies.clone(),
                    cost: child_node.cost,
                    properties: child_node.properties.clone(),
                    explored_rules: child_node.explored_rules.clone(),
                    group_id: child_node.group_id,
                };

                // 保留当前节点的输出变量
                if let Some(output_var) = node.plan_node.output_var() {
                    new_node.plan_node =
                        create_plan_node_with_output_var(&child_node.plan_node, output_var.clone());
                }

                return Ok(Some(new_node));
            }
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::context::QueryContext;
    use crate::query::optimizer::optimizer::{OptContext, OptGroupNode};
    use crate::query::optimizer::rule_traits::is_tautology;
    use crate::query::planner::plan::algorithms::IndexScan;
    use crate::query::planner::plan::core::nodes::{
        AppendVerticesNode, DedupNode, FilterNode, ProjectNode, SortNode, StartNode,
    };

    fn create_test_context() -> OptContext {
        let session_info = crate::core::context::session::SessionInfo::new(
            "test_session",
            "test_user",
            vec!["user".to_string()],
            "127.0.0.1",
            8080,
            "test_client",
            "test_connection",
        );
        let query_context = QueryContext::new(
            "test_query",
            crate::core::context::query::QueryType::DataQuery,
            "TEST QUERY",
            session_info,
        );
        OptContext::new(query_context)
    }

    #[test]
    fn test_eliminate_filter_rule() {
        let rule = EliminateFilterRule;
        let mut ctx = create_test_context();

        use crate::core::types::expression::Expression;
        use crate::core::types::operators::BinaryOperator;

        let start_node = PlanNodeEnum::Start(StartNode::new());
        let filter_node = PlanNodeEnum::Filter(
            FilterNode::new(
                start_node,
                Expression::Binary {
                    left: Box::new(Expression::Literal(crate::core::Value::Int(1))),
                    op: BinaryOperator::Equal,
                    right: Box::new(Expression::Literal(crate::core::Value::Int(1))),
                },
            )
            .expect("Filter node should be created successfully"),
        );
        let mut opt_node = OptGroupNode::new(1, filter_node);

        let child_node = PlanNodeEnum::ScanVertices(
            crate::query::planner::plan::core::nodes::ScanVerticesNode::new(1),
        );
        let child_opt_node = OptGroupNode::new(2, child_node);
        ctx.add_plan_node_and_group_node(2, &child_opt_node);
        opt_node.dependencies.push(2);

        let result = rule
            .apply(&mut ctx, &opt_node)
            .expect("Rule should apply successfully");
        assert!(result.is_some());
    }

    #[test]
    fn test_dedup_elimination_rule() {
        let rule = DedupEliminationRule;
        let mut ctx = create_test_context();

        // 创建一个去重节点
        let start_node = PlanNodeEnum::Start(StartNode::new());
        let dedup_node = PlanNodeEnum::Dedup(
            DedupNode::new(start_node).expect("Dedup node should be created successfully"),
        );
        let mut opt_node = OptGroupNode::new(1, dedup_node);

        // 添加一个IndexScan子节点作为依赖（IndexScan产生唯一结果）
        let child_node = PlanNodeEnum::IndexScan(IndexScan::new(2, 1, 1, 1, "UNIQUE"));
        let child_opt_node = OptGroupNode::new(2, child_node);
        ctx.add_plan_node_and_group_node(2, &child_opt_node);
        opt_node.dependencies.push(2);

        let result = rule
            .apply(&mut ctx, &opt_node)
            .expect("Rule should apply successfully");
        assert!(result.is_some());
    }

    #[test]
    fn test_remove_noop_project_rule() {
        let rule = RemoveNoopProjectRule;
        let mut ctx = create_test_context();

        // 创建一个子节点，设置输出列
        let mut child_node = PlanNodeEnum::ScanVertices(
            crate::query::planner::plan::core::nodes::ScanVerticesNode::new(1),
        );
        // 注意：需要使用 PlanNode trait 中的 set_col_names 方法
        child_node.set_col_names(vec![
            "id".to_string(),
            "name".to_string(),
            "age".to_string(),
        ]);
        let child_opt_node = OptGroupNode::new(2, child_node);
        ctx.add_plan_node_and_group_node(2, &child_opt_node);

        // 测试1: 创建一个投影所有列的投影节点（应该被消除）
        let columns_all = vec![crate::query::validator::YieldColumn {
            expr: crate::core::Expression::Variable("*".to_string()),
            alias: "*".to_string(),
            is_matched: false,
        }];
        let scan_node = PlanNodeEnum::ScanVertices(
            crate::query::planner::plan::core::nodes::ScanVerticesNode::new(1),
        );
        let project_node_all = PlanNodeEnum::Project(
            ProjectNode::new(scan_node, columns_all)
                .expect("Project node should be created successfully"),
        );
        let mut opt_node_all = OptGroupNode::new(1, project_node_all);
        opt_node_all.dependencies.push(2);

        let result_all = rule
            .apply(&mut ctx, &opt_node_all)
            .expect("Failed to apply rule");
        assert!(result_all.is_some(), "投影所有列的节点应该被消除");

        // 测试2: 创建一个投影相同列的投影节点（应该被消除）
        let columns_same = vec![
            crate::query::validator::YieldColumn {
                expr: crate::core::Expression::Variable("id".to_string()),
                alias: "id".to_string(),
                is_matched: false,
            },
            crate::query::validator::YieldColumn {
                expr: crate::core::Expression::Variable("name".to_string()),
                alias: "name".to_string(),
                is_matched: false,
            },
            crate::query::validator::YieldColumn {
                expr: crate::core::Expression::Variable("age".to_string()),
                alias: "age".to_string(),
                is_matched: false,
            },
        ];
        let scan_node = PlanNodeEnum::ScanVertices(
            crate::query::planner::plan::core::nodes::ScanVerticesNode::new(1),
        );
        let project_node_same = PlanNodeEnum::Project(
            ProjectNode::new(scan_node, columns_same)
                .expect("Project node should be created successfully"),
        );
        let mut opt_node_same = OptGroupNode::new(3, project_node_same);
        opt_node_same.dependencies.push(2);

        let result_same = rule
            .apply(&mut ctx, &opt_node_same)
            .expect("Failed to apply rule");
        assert!(result_same.is_some(), "投影相同列的节点应该被消除");

        // 测试3: 创建一个投影不同列的投影节点（不应该被消除）
        let columns_diff = vec![
            crate::query::validator::YieldColumn {
                expr: crate::core::Expression::Variable("id".to_string()),
                alias: "id".to_string(),
                is_matched: false,
            },
            crate::query::validator::YieldColumn {
                expr: crate::core::Expression::Variable("name".to_string()),
                alias: "name".to_string(),
                is_matched: false,
            },
        ];
        let scan_node = PlanNodeEnum::ScanVertices(
            crate::query::planner::plan::core::nodes::ScanVerticesNode::new(1),
        );
        let project_node_diff = PlanNodeEnum::Project(
            ProjectNode::new(scan_node, columns_diff)
                .expect("Project node should be created successfully"),
        );
        let mut opt_node_diff = OptGroupNode::new(4, project_node_diff);
        opt_node_diff.dependencies.push(2);

        let result_diff = rule
            .apply(&mut ctx, &opt_node_diff)
            .expect("Failed to apply rule");
        assert!(result_diff.is_none(), "投影不同列的节点不应该被消除");

        // 测试4: 创建一个投影带别名的节点（不应该被消除）
        let columns_alias = vec![
            crate::query::validator::YieldColumn {
                expr: crate::core::Expression::Variable("id".to_string()),
                alias: "vertex_id".to_string(),
                is_matched: false,
            },
            crate::query::validator::YieldColumn {
                expr: crate::core::Expression::Variable("name".to_string()),
                alias: "vertex_name".to_string(),
                is_matched: false,
            },
            crate::query::validator::YieldColumn {
                expr: crate::core::Expression::Variable("age".to_string()),
                alias: "age".to_string(),
                is_matched: false,
            },
        ];
        let scan_node = PlanNodeEnum::ScanVertices(
            crate::query::planner::plan::core::nodes::ScanVerticesNode::new(1),
        );
        let project_node_alias = PlanNodeEnum::Project(
            ProjectNode::new(scan_node, columns_alias)
                .expect("Project node should be created successfully"),
        );
        let mut opt_node_alias = OptGroupNode::new(5, project_node_alias);
        opt_node_alias.dependencies.push(2);

        let result_alias = rule
            .apply(&mut ctx, &opt_node_alias)
            .expect("Failed to apply rule");
        assert!(result_alias.is_none(), "投影带别名的节点不应该被消除");

        // 测试5: 创建一个投影包含表达式的节点（不应该被消除）
        let columns_expr = vec![
            crate::query::validator::YieldColumn {
                expr: crate::core::Expression::Variable("id".to_string()),
                alias: "id".to_string(),
                is_matched: false,
            },
            crate::query::validator::YieldColumn {
                expr: crate::core::Expression::Variable("name".to_string()),
                alias: "name".to_string(),
                is_matched: false,
            },
            crate::query::validator::YieldColumn {
                expr: crate::core::Expression::Binary {
                    left: Box::new(crate::core::Expression::Variable("age".to_string())),
                    op: crate::core::BinaryOperator::Add,
                    right: Box::new(crate::core::Expression::Literal(
                        crate::core::Value::String("1".to_string()),
                    )),
                },
                alias: "age_plus_1".to_string(),
                is_matched: false,
            },
        ];
        let start_node = PlanNodeEnum::Start(StartNode::new());
        let project_node_expr = PlanNodeEnum::Project(
            ProjectNode::new(start_node, columns_expr)
                .expect("Project node should be created successfully"),
        );
        let mut opt_node_expr = OptGroupNode::new(6, project_node_expr);
        opt_node_expr.dependencies.push(2);

        let result_expr = rule
            .apply(&mut ctx, &opt_node_expr)
            .expect("Failed to apply rule");
        assert!(result_expr.is_none(), "投影包含表达式的节点不应该被消除");
    }

    #[test]
    fn test_eliminate_append_vertices_rule() {
        let rule = EliminateAppendVerticesRule;
        let mut ctx = create_test_context();

        // 创建一个添加顶点节点
        let append_vertices_node =
            PlanNodeEnum::AppendVertices(AppendVerticesNode::new(1, vec![], vec![]));
        let mut opt_node = OptGroupNode::new(1, append_vertices_node);

        // 添加一个子节点作为依赖
        let child_node = PlanNodeEnum::ScanVertices(
            crate::query::planner::plan::core::nodes::ScanVerticesNode::new(1),
        );
        let child_opt_node = OptGroupNode::new(2, child_node);
        ctx.add_plan_node_and_group_node(2, &child_opt_node);
        opt_node.dependencies.push(2);

        let result = rule
            .apply(&mut ctx, &opt_node)
            .expect("Rule should apply successfully");
        assert!(result.is_some());
    }

    #[test]
    fn test_remove_append_vertices_below_join_rule() {
        let rule = RemoveAppendVerticesBelowJoinRule;
        let mut ctx = create_test_context();

        // 创建一个添加顶点节点
        let append_vertices_node =
            PlanNodeEnum::AppendVertices(AppendVerticesNode::new(1, vec![], vec![]));
        let mut opt_node = OptGroupNode::new(1, append_vertices_node);

        // 添加一个HashInnerJoin子节点作为依赖
        let start_node1 = PlanNodeEnum::Start(StartNode::new());
        let start_node2 = PlanNodeEnum::Start(StartNode::new());
        let child_node = PlanNodeEnum::InnerJoin(
            crate::query::planner::plan::core::nodes::InnerJoinNode::new(
                start_node1,
                start_node2,
                vec![],
                vec![],
            )
            .expect("InnerJoin node should be created successfully"),
        );
        let child_opt_node = OptGroupNode::new(2, child_node);
        ctx.add_plan_node_and_group_node(2, &child_opt_node);
        opt_node.dependencies.push(2);

        let result = rule
            .apply(&mut ctx, &opt_node)
            .expect("Rule should apply successfully");
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

/// 创建具有指定输出变量的PlanNode副本
fn create_plan_node_with_output_var(
    plan_node: &PlanNodeEnum,
    output_var: crate::query::context::validate::types::Variable,
) -> PlanNodeEnum {
    use crate::query::planner::plan::core::nodes::*;

    // 尝试将plan_node向下转换为具体类型，并创建带有新输出变量的新实例
    // 这里我们只处理一些常见的节点类型作为示例，实际中需要处理所有类型
    match plan_node {
        PlanNodeEnum::Filter(filter_node) => {
            let input = *filter_node
                .dependencies()
                .get(0)
                .expect("Filter should have at least one dependency")
                .clone();
            let condition = filter_node.condition().clone();
            let mut new_node = FilterNode::new(input, condition)
                .expect("FilterNode creation should succeed with valid input");
            new_node.set_output_var(output_var);
            PlanNodeEnum::Filter(new_node)
        }
        PlanNodeEnum::Project(project_node) => {
            let input = project_node.input().clone();
            let columns = project_node.columns().to_vec();
            let mut new_node = ProjectNode::new(input, columns)
                .expect("ProjectNode creation should succeed with valid input");
            new_node.set_output_var(output_var);
            PlanNodeEnum::Project(new_node)
        }
        PlanNodeEnum::Dedup(dedup_node) => {
            let input = *dedup_node
                .dependencies()
                .get(0)
                .expect("Dedup should have at least one dependency")
                .clone();
            let mut new_node =
                DedupNode::new(input).expect("DedupNode creation should succeed with valid input");
            new_node.set_output_var(output_var);
            PlanNodeEnum::Dedup(new_node)
        }
        PlanNodeEnum::Sort(sort_node) => {
            let input = sort_node.input().clone();
            let sort_items = sort_node.sort_items().to_vec();
            let mut new_node = SortNode::new(input, sort_items)
                .expect("SortNode creation should succeed with valid input");
            new_node.set_output_var(output_var);
            PlanNodeEnum::Sort(new_node)
        }
        PlanNodeEnum::Limit(limit_node) => {
            let input = limit_node.input().clone();
            let offset = limit_node.offset();
            let count = limit_node.count();
            let mut new_node = LimitNode::new(input, offset, count)
                .expect("LimitNode creation should succeed with valid input");
            new_node.set_output_var(output_var);
            PlanNodeEnum::Limit(new_node)
        }
        PlanNodeEnum::ScanVertices(scan_vertices_node) => {
            // 创建新的扫描顶点节点，需要使用正确的构造函数
            let space_id = scan_vertices_node.space_id();
            let mut new_node = ScanVerticesNode::new(space_id);
            new_node.set_output_var(output_var);
            PlanNodeEnum::ScanVertices(new_node)
        }
        PlanNodeEnum::AppendVertices(append_vertices_node) => {
            // 创建新的添加顶点节点，需要使用正确的构造函数
            let space_id = append_vertices_node.space_id();
            let vids = append_vertices_node.vids().to_vec();
            let tag_ids = append_vertices_node.tag_ids().to_vec();
            let mut new_node = AppendVerticesNode::new(space_id, vids, tag_ids);
            new_node.set_output_var(output_var);
            PlanNodeEnum::AppendVertices(new_node)
        }
        PlanNodeEnum::ScanEdges(scan_edges_node) => {
            let mut new_node =
                ScanEdgesNode::new(scan_edges_node.space_id(), scan_edges_node.edge_type());
            new_node.set_output_var(output_var);
            PlanNodeEnum::ScanEdges(new_node)
        }
        PlanNodeEnum::GetVertices(get_vertices_node) => {
            let mut new_node =
                GetVerticesNode::new(get_vertices_node.space_id(), get_vertices_node.src_vids());
            new_node.set_output_var(output_var);
            PlanNodeEnum::GetVertices(new_node)
        }
        PlanNodeEnum::GetEdges(get_edges_node) => {
            let mut new_node = GetEdgesNode::new(
                get_edges_node.space_id(),
                get_edges_node.src(),
                get_edges_node.edge_type(),
                get_edges_node.rank(),
                get_edges_node.dst(),
            );
            new_node.set_output_var(output_var);
            PlanNodeEnum::GetEdges(new_node)
        }
        PlanNodeEnum::InnerJoin(inner_join_node) => {
            let deps = inner_join_node.dependencies();
            if deps.len() >= 2 {
                let left = *deps[0].clone();
                let right = *deps[1].clone();
                let hash_keys = inner_join_node.hash_keys().to_vec();
                let probe_keys = inner_join_node.probe_keys().to_vec();
                let mut new_node = InnerJoinNode::new(left, right, hash_keys, probe_keys)
                    .expect("InnerJoinNode creation should succeed with valid input");
                new_node.set_output_var(output_var);
                PlanNodeEnum::InnerJoin(new_node)
            } else {
                plan_node.clone()
            }
        }
        PlanNodeEnum::LeftJoin(left_join_node) => {
            let deps = left_join_node.dependencies();
            if deps.len() >= 2 {
                let left = *deps[0].clone();
                let right = *deps[1].clone();
                let hash_keys = left_join_node.hash_keys().to_vec();
                let probe_keys = left_join_node.probe_keys().to_vec();
                let mut new_node = LeftJoinNode::new(left, right, hash_keys, probe_keys)
                    .expect("LeftJoinNode creation should succeed with valid input");
                new_node.set_output_var(output_var);
                PlanNodeEnum::LeftJoin(new_node)
            } else {
                plan_node.clone()
            }
        }
        _ => {
            // 如果无法识别具体类型，则返回原节点的克隆（不改变输出变量）
            plan_node.clone()
        }
    }
}
