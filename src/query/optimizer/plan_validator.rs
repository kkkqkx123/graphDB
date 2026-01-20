//! 计划验证器
//!
//! 提供计划验证功能，确保优化后的计划是正确的

use crate::core::types::expression::Expression;
use crate::core::types::operators::Operator;
use crate::query::optimizer::optimizer::{OptContext, OptGroup, OptGroupNode};
use crate::query::optimizer::OptimizerError;
use std::collections::HashMap;

/// 计划验证器
///
/// 用于验证优化后的计划是否正确
#[derive(Debug)]
pub struct PlanValidator;

impl PlanValidator {
    /// 验证计划
    ///
    /// # 参数
    /// * `ctx` - 优化上下文
    /// * `root_group` - 根优化组
    ///
    /// # 返回值
    /// 如果验证成功，返回 Ok(())，否则返回错误
    pub fn validate_plan(
        ctx: &OptContext,
        root_group: &OptGroup,
    ) -> Result<(), OptimizerError> {
        // 验证数据流
        Self::validate_data_flow(ctx, root_group)?;

        // 验证变量使用
        Self::validate_variable_usage(ctx, root_group)?;

        // 验证表达式
        Self::validate_expressions(ctx, root_group)?;

        // 验证计划节点属性
        Self::validate_plan_node_properties(ctx, root_group)?;

        Ok(())
    }

    /// 验证数据流
    ///
    /// 确保计划中的数据流是正确的
    fn validate_data_flow(
        ctx: &OptContext,
        root_group: &OptGroup,
    ) -> Result<(), OptimizerError> {
        let boundary = vec![root_group];
        Self::validate_data_flow_recursive(ctx, root_group, &boundary)
    }

    /// 递归验证数据流
    fn validate_data_flow_recursive(
        ctx: &OptContext,
        group: &OptGroup,
        boundary: &[&OptGroup],
    ) -> Result<(), OptimizerError> {
        for node in &group.nodes {
            // 验证节点的数据流
            if !ctx.validate_data_flow(node, boundary) {
                return Err(OptimizerError::Validation {
                    message: format!(
                        "数据流验证失败：节点 {} 的依赖关系不正确",
                        node.id
                    ),
                });
            }

            // 递归验证依赖组
            for dep_id in &node.dependencies {
                if let Some(dep_group) = Self::find_group_by_id(ctx, *dep_id) {
                    Self::validate_data_flow_recursive(ctx, dep_group, boundary)?;
                }
            }

            // 递归验证主体组
            for body_id in &node.bodies {
                if let Some(body_group) = Self::find_group_by_id(ctx, *body_id) {
                    Self::validate_data_flow_recursive(ctx, body_group, boundary)?;
                }
            }
        }

        Ok(())
    }

    /// 验证变量使用
    ///
    /// 确保计划中的变量使用是正确的
    fn validate_variable_usage(
        ctx: &OptContext,
        root_group: &OptGroup,
    ) -> Result<(), OptimizerError> {
        let mut defined_vars = HashMap::new();
        Self::validate_variable_usage_recursive(ctx, root_group, &mut defined_vars)
    }

    /// 递归验证变量使用
    fn validate_variable_usage_recursive(
        ctx: &OptContext,
        group: &OptGroup,
        defined_vars: &mut HashMap<String, usize>,
    ) -> Result<(), OptimizerError> {
        for node in &group.nodes {
            // 验证输出变量是否已定义
            if let Some(output_var) = node.plan_node.output_var() {
                let var_name = &output_var.name;
                if defined_vars.contains_key(var_name) {
                    return Err(OptimizerError::Validation {
                        message: format!(
                            "变量使用验证失败：变量 {} 被多次定义",
                            var_name
                        ),
                    });
                }
                defined_vars.insert(var_name.to_string(), node.id);
            }

            // 验证输入变量是否已定义
            for input_var in &node.properties.input_vars {
                if !defined_vars.contains_key(input_var) {
                    return Err(OptimizerError::Validation {
                        message: format!(
                            "变量使用验证失败：变量 {} 未定义就被使用",
                            input_var
                        ),
                    });
                }
            }

            // 递归验证依赖组
            for dep_id in &node.dependencies {
                if let Some(dep_group) = Self::find_group_by_id(ctx, *dep_id) {
                    Self::validate_variable_usage_recursive(ctx, dep_group, defined_vars)?;
                }
            }

            // 递归验证主体组
            for body_id in &node.bodies {
                if let Some(body_group) = Self::find_group_by_id(ctx, *body_id) {
                    Self::validate_variable_usage_recursive(ctx, body_group, defined_vars)?;
                }
            }
        }

        Ok(())
    }

    /// 验证表达式
    ///
    /// 确保计划中的表达式是正确的
    fn validate_expressions(
        ctx: &OptContext,
        root_group: &OptGroup,
    ) -> Result<(), OptimizerError> {
        Self::validate_expressions_recursive(ctx, root_group)
    }

    /// 递归验证表达式
    fn validate_expressions_recursive(
        ctx: &OptContext,
        group: &OptGroup,
    ) -> Result<(), OptimizerError> {
        for node in &group.nodes {
            // 验证节点中的表达式
            Self::validate_node_expressions(node)?;

            // 递归验证依赖组
            for dep_id in &node.dependencies {
                if let Some(dep_group) = Self::find_group_by_id(ctx, *dep_id) {
                    Self::validate_expressions_recursive(ctx, dep_group)?;
                }
            }

            // 递归验证主体组
            for body_id in &node.bodies {
                if let Some(body_group) = Self::find_group_by_id(ctx, *body_id) {
                    Self::validate_expressions_recursive(ctx, body_group)?;
                }
            }
        }

        Ok(())
    }

    /// 验证节点中的表达式
    fn validate_node_expressions(node: &OptGroupNode) -> Result<(), OptimizerError> {
        match &node.plan_node {
            crate::query::planner::plan::PlanNodeEnum::Filter(filter_node) => {
                Self::validate_expression(&filter_node.condition())?;
            }
            crate::query::planner::plan::PlanNodeEnum::Project(project_node) => {
                for column in project_node.columns() {
                    Self::validate_expression(&column.expr)?;
                }
            }
            crate::query::planner::plan::PlanNodeEnum::Aggregate(aggregate_node) => {
                for group_key in aggregate_node.group_keys() {
                    if group_key.is_empty() {
                        return Err(OptimizerError::Validation {
                            message: format!(
                                "聚合节点验证失败：分组键为空"
                            ),
                        });
                    }
                }
                for agg_func in aggregate_node.aggregation_functions() {
                    match agg_func {
                        crate::core::types::operators::AggregateFunction::Count(None) => {
                        }
                        crate::core::types::operators::AggregateFunction::Count(Some(field)) => {
                            if field.is_empty() {
                                return Err(OptimizerError::Validation {
                                    message: format!(
                                        "聚合节点验证失败：COUNT 函数字段名为空"
                                    ),
                                });
                            }
                        }
                        crate::core::types::operators::AggregateFunction::Sum(field)
                        | crate::core::types::operators::AggregateFunction::Avg(field)
                        | crate::core::types::operators::AggregateFunction::Min(field)
                        | crate::core::types::operators::AggregateFunction::Max(field)
                        | crate::core::types::operators::AggregateFunction::Collect(field)
                        | crate::core::types::operators::AggregateFunction::Distinct(field) => {
                            if field.is_empty() {
                                return Err(OptimizerError::Validation {
                                    message: format!(
                                        "聚合节点验证失败：{} 函数字段名为空",
                                        agg_func.name()
                                    ),
                                });
                            }
                        }
                        crate::core::types::operators::AggregateFunction::Percentile(field, _) => {
                            if field.is_empty() {
                                return Err(OptimizerError::Validation {
                                    message: format!(
                                        "聚合节点验证失败：PERCENTILE 函数字段名为空"
                                    ),
                                });
                            }
                        }
                    }
                }
            }
            crate::query::planner::plan::PlanNodeEnum::Select(select_node) => {
                let condition = select_node.condition();
                if condition.is_empty() {
                    return Err(OptimizerError::Validation {
                        message: format!(
                            "Select 节点验证失败：条件为空"
                        ),
                    });
                }
                if select_node.if_branch().is_none() && select_node.else_branch().is_none() {
                    return Err(OptimizerError::Validation {
                        message: format!(
                            "Select 节点验证失败：至少需要一个分支（if 或 else）"
                        ),
                    });
                }
            }
            crate::query::planner::plan::PlanNodeEnum::Loop(loop_node) => {
                let condition = loop_node.condition();
                if condition.is_empty() {
                    return Err(OptimizerError::Validation {
                        message: format!(
                            "Loop 节点验证失败：条件为空"
                        ),
                    });
                }
                if loop_node.body().is_none() {
                    return Err(OptimizerError::Validation {
                        message: format!(
                            "Loop 节点验证失败：缺少循环体"
                        ),
                    });
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// 验证表达式
    fn validate_expression(expr: &Expression) -> Result<(), OptimizerError> {
        match expr {
            Expression::Binary { left, right, .. } => {
                Self::validate_expression(left)?;
                Self::validate_expression(right)?;
            }
            Expression::Unary { operand, .. } => {
                Self::validate_expression(operand)?;
            }
            Expression::Function { args, .. } => {
                for arg in args {
                    Self::validate_expression(arg)?;
                }
            }
            Expression::List(items) => {
                for item in items {
                    Self::validate_expression(item)?;
                }
            }
            Expression::Map(pairs) => {
                for (_, value) in pairs {
                    Self::validate_expression(value)?;
                }
            }
            Expression::Case { conditions, default, .. } => {
                for (condition, expr) in conditions {
                    Self::validate_expression(condition)?;
                    Self::validate_expression(expr)?;
                }
                if let Some(default_expr) = default {
                    Self::validate_expression(default_expr)?;
                }
            }
            Expression::TypeCast { expr, .. } => {
                Self::validate_expression(expr)?;
            }
            Expression::Subscript { collection, index } => {
                Self::validate_expression(collection)?;
                Self::validate_expression(index)?;
            }
            Expression::Range { collection, start, end } => {
                Self::validate_expression(collection)?;
                if let Some(start_expr) = start {
                    Self::validate_expression(start_expr)?;
                }
                if let Some(end_expr) = end {
                    Self::validate_expression(end_expr)?;
                }
            }
            Expression::Path(items) => {
                for item in items {
                    Self::validate_expression(item)?;
                }
            }
            Expression::ListComprehension { generator, condition } => {
                Self::validate_expression(generator)?;
                if let Some(cond) = condition {
                    Self::validate_expression(cond)?;
                }
            }
            Expression::Predicate { list, condition } => {
                Self::validate_expression(list)?;
                Self::validate_expression(condition)?;
            }
            Expression::Reduce { list, initial, expr, .. } => {
                Self::validate_expression(list)?;
                Self::validate_expression(initial)?;
                Self::validate_expression(expr)?;
            }
            _ => {}
        }

        Ok(())
    }

    /// 验证计划节点属性
    ///
    /// 确保计划节点的属性是正确的
    fn validate_plan_node_properties(
        ctx: &OptContext,
        root_group: &OptGroup,
    ) -> Result<(), OptimizerError> {
        Self::validate_plan_node_properties_recursive(ctx, root_group)
    }

    /// 递归验证计划节点属性
    fn validate_plan_node_properties_recursive(
        ctx: &OptContext,
        group: &OptGroup,
    ) -> Result<(), OptimizerError> {
        for node in &group.nodes {
            // 验证节点的属性
            Self::validate_node_properties(node)?;

            // 递归验证依赖组
            for dep_id in &node.dependencies {
                if let Some(dep_group) = Self::find_group_by_id(ctx, *dep_id) {
                    Self::validate_plan_node_properties_recursive(ctx, dep_group)?;
                }
            }

            // 递归验证主体组
            for body_id in &node.bodies {
                if let Some(body_group) = Self::find_group_by_id(ctx, *body_id) {
                    Self::validate_plan_node_properties_recursive(ctx, body_group)?;
                }
            }
        }

        Ok(())
    }

    /// 验证节点属性
    fn validate_node_properties(node: &OptGroupNode) -> Result<(), OptimizerError> {
        // 验证成本非负
        if node.cost < 0.0 {
            return Err(OptimizerError::Validation {
                message: format!(
                    "节点属性验证失败：节点 {} 的成本为负数 {}",
                    node.id, node.cost
                ),
            });
        }

        // 验证输出变量
        if let Some(output_var) = node.plan_node.output_var() {
            if output_var.name.is_empty() {
                return Err(OptimizerError::Validation {
                    message: format!(
                        "节点属性验证失败：节点 {} 的输出变量名为空",
                        node.id
                    ),
                });
            }
        }

        // 验证列名（仅当有输出变量时需要列名）
        if node.plan_node.output_var().is_some() {
            let col_names = node.plan_node.col_names();
            if col_names.is_empty() {
                return Err(OptimizerError::Validation {
                    message: format!(
                        "节点属性验证失败：节点 {} 的列名为空",
                        node.id
                    ),
                });
            }
        }

        Ok(())
    }

    /// 根据ID查找优化组
    fn find_group_by_id(ctx: &OptContext, group_id: usize) -> Option<&OptGroup> {
        ctx.group_map.get(&group_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::context::execution::QueryContext;
    use crate::api::session::session_manager::SessionInfo;
    use crate::core::types::expression::Expression;
    use crate::query::optimizer::optimizer::{OptContext, OptGroup, OptGroupNode};
    use crate::query::planner::plan::PlanNodeEnum;

    fn create_test_context() -> OptContext {
        let query_context = QueryContext::new();
        OptContext::new(query_context)
    }

    #[test]
    fn test_validate_expression() {
        let expr = Expression::Binary {
            left: Box::new(Expression::Variable("x".to_string())),
            op: crate::core::types::operators::BinaryOperator::Equal,
            right: Box::new(Expression::Literal(crate::core::Value::Int(42))),
        };

        let result = PlanValidator::validate_expression(&expr);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_node_properties() {
        let node = OptGroupNode::new(1, PlanNodeEnum::Start(crate::query::planner::plan::core::nodes::start_node::StartNode::new()));
        let result = PlanValidator::validate_node_properties(&node);
        assert!(result.is_ok());
    }
}
