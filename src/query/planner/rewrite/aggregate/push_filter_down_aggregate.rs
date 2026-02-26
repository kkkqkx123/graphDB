//! 过滤下推到聚合节点的优化规则
//!
//! 此规则将过滤操作下推到聚合节点之前执行，以减少进入聚合的数据量。
//!
//! # 转换示例
//!
//! Before:
//! ```text
//! Filter(condition)
//!       |
//!   Aggregate(group_keys, agg_funcs)
//!       |
//!     Input
//! ```
//!
//! After:
//! ```text
//! Aggregate(group_keys, agg_funcs)
//!             |
//!       Filter(condition)
//!             |
//!           Input
//! ```
//!
//! # 适用条件
//!
//! - Filter 节点的子节点是 Aggregate 节点
//! - Filter 条件不涉及聚合函数（只涉及聚合的输入列）

use crate::query::planner::plan::PlanNodeEnum;
use crate::query::planner::rewrite::context::RewriteContext;
use crate::query::planner::rewrite::pattern::Pattern;
use crate::query::planner::rewrite::result::{RewriteResult, TransformResult};
use crate::query::planner::rewrite::rule::{PushDownRule, RewriteRule};
use crate::query::planner::plan::core::nodes::aggregate_node::AggregateNode;
use crate::query::planner::plan::core::nodes::filter_node::FilterNode;
use crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode;
use crate::core::Expression;
use crate::core::types::operators::AggregateFunction;

/// 将过滤下推到聚合之前的规则
#[derive(Debug)]
pub struct PushFilterDownAggregateRule;

impl PushFilterDownAggregateRule {
    /// 创建规则实例
    pub fn new() -> Self {
        Self
    }

    /// 检查条件是否包含聚合函数引用
    ///
    /// 如果条件引用了聚合函数的结果（如 COUNT(*), SUM(amount) 等），
    /// 则不能将 Filter 下推，因为聚合结果在聚合之前不存在。
    fn has_aggregate_function_reference(
        condition: &Expression,
        group_keys: &[String],
        agg_funcs: &[AggregateFunction],
    ) -> bool {
        fn check_expr(expr: &Expression, group_keys: &[String], agg_funcs: &[AggregateFunction]) -> bool {
            match expr {
                // 直接包含聚合表达式
                Expression::Aggregate { .. } => true,
                // 二元运算：检查左右两边
                Expression::Binary { left, right, .. } => {
                    check_expr(left, group_keys, agg_funcs) || check_expr(right, group_keys, agg_funcs)
                }
                // 一元运算：检查操作数
                Expression::Unary { operand, .. } => check_expr(operand, group_keys, agg_funcs),
                // 属性访问：检查对象
                Expression::Property { object, .. } => check_expr(object, group_keys, agg_funcs),
                // 函数调用：检查是否是聚合函数或参数中包含聚合
                Expression::Function { name, args, .. } => {
                    let func_name = name.to_lowercase();
                    // 检查是否是聚合函数名称
                    if matches!(
                        func_name.as_str(),
                        "sum" | "avg" | "count" | "max" | "min" | "collect" | "collect_set" | "distinct" | "std"
                    ) {
                        return true;
                    }
                    // 检查参数中是否包含聚合
                    args.iter().any(|arg| check_expr(arg, group_keys, agg_funcs))
                }
                // 变量：检查是否是分组键
                // 如果不是分组键，则可能是聚合输出列
                Expression::Variable(name) => {
                    // 如果是分组键，可以下推
                    if group_keys.contains(name) {
                        return false;
                    }
                    // 检查是否是聚合函数的输出列名
                    for agg_func in agg_funcs {
                        if agg_func.name() == name || 
                           agg_func.field_name().map(|f| f == name).unwrap_or(false) {
                            return true;
                        }
                    }
                    // 其他变量，假设可以下推（是输入列）
                    false
                }
                // 其他表达式类型
                _ => false,
            }
        }

        check_expr(condition, group_keys, agg_funcs)
    }

    /// 重写过滤条件中的变量引用
    ///
    /// 将 Filter 中的变量引用转换为聚合输入的列引用。
    ///
    /// # 为什么不需要重写
    ///
    /// 当前实现直接返回原条件，这是正确的，因为：
    ///
    /// 1. **分组键**：分组键在聚合前后名称相同，不需要重写
    /// 2. **聚合函数输出列**：已被 `has_aggregate_function_reference` 阻止，不会下推
    /// 3. **其他输入列**：输入列在聚合前后名称相同，不需要重写
    ///
    /// # 何时需要重写
    ///
    /// 如果未来需要支持以下场景，可以使用 `expression_utils::rewrite_expression`：
    ///
    /// - 聚合输出列名与输入列名不同
    /// - 需要将聚合输出列映射回输入列
    /// - 需要处理复杂的列名转换
    fn rewrite_filter_condition(condition: &Expression, _group_keys: &[String]) -> Expression {
        condition.clone()
    }
}

impl Default for PushFilterDownAggregateRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RewriteRule for PushFilterDownAggregateRule {
    fn name(&self) -> &'static str {
        "PushFilterDownAggregateRule"
    }

    fn pattern(&self) -> Pattern {
        Pattern::new_with_name("Filter").with_dependency_name("Aggregate")
    }

    fn apply(
        &self,
        _ctx: &mut RewriteContext,
        node: &PlanNodeEnum,
    ) -> RewriteResult<Option<TransformResult>> {
        // 检查是否为 Filter 节点
        let filter_node = match node {
            PlanNodeEnum::Filter(n) => n,
            _ => return Ok(None),
        };

        // 获取过滤条件
        let filter_condition = filter_node.condition();

        // 获取输入节点
        let input = filter_node.input();

        // 检查输入节点是否为 Aggregate
        let agg_node = match input {
            PlanNodeEnum::Aggregate(n) => n,
            _ => return Ok(None),
        };

        // 获取聚合的分组键和聚合函数
        let group_keys = agg_node.group_keys();
        let agg_funcs = agg_node.aggregation_functions();

        // 检查过滤条件是否包含聚合函数引用
        // 如果条件引用了聚合结果（如 HAVING COUNT(*) > 10），则不能下推
        if Self::has_aggregate_function_reference(filter_condition, group_keys, agg_funcs) {
            return Ok(None);
        }

        // 获取聚合的输入节点
        let agg_input = agg_node.input();

        // 重写过滤条件（将输出列引用转换为输入列引用）
        let rewritten_condition = Self::rewrite_filter_condition(filter_condition, group_keys);

        // 创建新的 Filter 节点，放在 Aggregate 之前
        let new_filter = FilterNode::new(agg_input.clone(), rewritten_condition)
            .map_err(|e| crate::query::planner::rewrite::result::RewriteError::rewrite_failed(
                format!("创建 FilterNode 失败: {:?}", e)
            ))?;

        // 创建新的 Aggregate 节点，输入为新的 Filter 节点
        let new_aggregate = AggregateNode::new(
            PlanNodeEnum::Filter(new_filter),
            group_keys.to_vec(),
            agg_funcs.to_vec(),
        ).map_err(|e| crate::query::planner::rewrite::result::RewriteError::rewrite_failed(
            format!("创建 AggregateNode 失败: {:?}", e)
        ))?;

        // 构建转换结果
        let mut result = TransformResult::new();
        result.erase_curr = true; // 删除原来的 Filter 节点
        result.add_new_node(PlanNodeEnum::Aggregate(new_aggregate));

        Ok(Some(result))
    }
}

impl PushDownRule for PushFilterDownAggregateRule {
    fn can_push_down(&self, node: &PlanNodeEnum, target: &PlanNodeEnum) -> bool {
        matches!((node, target), (PlanNodeEnum::Filter(_), PlanNodeEnum::Aggregate(_)))
    }

    fn push_down(
        &self,
        _ctx: &mut RewriteContext,
        node: &PlanNodeEnum,
        _target: &PlanNodeEnum,
    ) -> RewriteResult<Option<TransformResult>> {
        self.apply(_ctx, node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::planner::plan::core::nodes::start_node::StartNode;

    #[test]
    fn test_rule_name() {
        let rule = PushFilterDownAggregateRule::new();
        assert_eq!(rule.name(), "PushFilterDownAggregateRule");
    }

    #[test]
    fn test_rule_pattern() {
        let rule = PushFilterDownAggregateRule::new();
        let pattern = rule.pattern();
        assert!(pattern.node.is_some());
    }

    #[test]
    fn test_has_aggregate_function_reference_with_aggregate() {
        let condition = Expression::Aggregate {
            func: AggregateFunction::Count(None),
            arg: Box::new(Expression::Variable("amount".to_string())),
            distinct: false,
        };

        assert!(PushFilterDownAggregateRule::has_aggregate_function_reference(
            &condition,
            &[],
            &[AggregateFunction::Count(None)]
        ));
    }

    #[test]
    fn test_no_aggregate_function_reference() {
        let condition = Expression::Binary {
            op: crate::core::types::operators::BinaryOperator::Equal,
            left: Box::new(Expression::Variable("name".to_string())),
            right: Box::new(Expression::Literal(crate::core::Value::String("test".to_string()))),
        };

        assert!(!PushFilterDownAggregateRule::has_aggregate_function_reference(
            &condition,
            &["name".to_string()],
            &[]
        ));
    }

    #[test]
    fn test_has_aggregate_function_reference_with_function() {
        let condition = Expression::Function {
            name: "sum".to_string(),
            args: vec![Expression::Variable("amount".to_string())],
        };

        assert!(PushFilterDownAggregateRule::has_aggregate_function_reference(
            &condition,
            &[],
            &[AggregateFunction::Sum("amount".to_string())]
        ));
    }

    #[test]
    fn test_rewrite_filter_condition() {
        let condition = Expression::Binary {
            op: crate::core::types::operators::BinaryOperator::Equal,
            left: Box::new(Expression::Variable("name".to_string())),
            right: Box::new(Expression::Literal(crate::core::Value::String("test".to_string()))),
        };

        let rewritten = PushFilterDownAggregateRule::rewrite_filter_condition(
            &condition,
            &["name".to_string()]
        );

        assert_eq!(rewritten, condition);
    }

    #[test]
    fn test_apply_with_group_key_filter() {
        // 创建 Start 节点
        let start_node = StartNode::new();
        let start_enum = PlanNodeEnum::Start(start_node);

        // 创建 Aggregate 节点
        let group_keys = vec!["category".to_string()];
        let agg_funcs = vec![AggregateFunction::Count(None)];
        let aggregate = AggregateNode::new(start_enum.clone(), group_keys, agg_funcs)
            .expect("创建 AggregateNode 失败");
        let aggregate_enum = PlanNodeEnum::Aggregate(aggregate);

        // 创建 Filter 节点（条件只涉及分组键）
        let condition = Expression::Binary {
            op: crate::core::types::operators::BinaryOperator::Equal,
            left: Box::new(Expression::Variable("category".to_string())),
            right: Box::new(Expression::Literal(crate::core::Value::String("A".to_string()))),
        };
        let filter = FilterNode::new(aggregate_enum, condition)
            .expect("创建 FilterNode 失败");
        let filter_enum = PlanNodeEnum::Filter(filter);

        // 应用规则
        let rule = PushFilterDownAggregateRule::new();
        let mut ctx = RewriteContext::new();
        let result = rule.apply(&mut ctx, &filter_enum)
            .expect("应用规则失败");

        // 验证转换成功
        assert!(result.is_some());
        let transform_result = result.expect("Failed to apply rewrite rule");
        assert!(transform_result.erase_curr);
        assert_eq!(transform_result.new_nodes.len(), 1);
    }

    #[test]
    fn test_apply_with_aggregate_filter() {
        // 创建 Start 节点
        let start_node = StartNode::new();
        let start_enum = PlanNodeEnum::Start(start_node);

        // 创建 Aggregate 节点
        let group_keys = vec!["category".to_string()];
        let agg_funcs = vec![AggregateFunction::Count(None)];
        let aggregate = AggregateNode::new(start_enum.clone(), group_keys, agg_funcs)
            .expect("创建 AggregateNode 失败");
        let aggregate_enum = PlanNodeEnum::Aggregate(aggregate);

        // 创建 Filter 节点（条件涉及聚合函数结果，如 HAVING COUNT(*) > 10）
        let condition = Expression::Binary {
            op: crate::core::types::operators::BinaryOperator::GreaterThan,
            left: Box::new(Expression::Variable("COUNT".to_string())),
            right: Box::new(Expression::Literal(crate::core::Value::Int(10))),
        };
        let filter = FilterNode::new(aggregate_enum, condition)
            .expect("创建 FilterNode 失败");
        let filter_enum = PlanNodeEnum::Filter(filter);

        // 应用规则
        let rule = PushFilterDownAggregateRule::new();
        let mut ctx = RewriteContext::new();
        let result = rule.apply(&mut ctx, &filter_enum)
            .expect("应用规则失败");

        // 验证转换未执行（因为条件涉及聚合结果）
        assert!(result.is_none());
    }

    #[test]
    fn test_apply_with_non_aggregate_input() {
        // 创建 Start 节点
        let start_node = StartNode::new();
        let start_enum = PlanNodeEnum::Start(start_node);

        // 创建 Filter 节点，但输入不是 Aggregate
        let condition = Expression::Binary {
            op: crate::core::types::operators::BinaryOperator::Equal,
            left: Box::new(Expression::Variable("name".to_string())),
            right: Box::new(Expression::Literal(crate::core::Value::String("test".to_string()))),
        };
        let filter = FilterNode::new(start_enum, condition)
            .expect("创建 FilterNode 失败");
        let filter_enum = PlanNodeEnum::Filter(filter);

        // 应用规则
        let rule = PushFilterDownAggregateRule::new();
        let mut ctx = RewriteContext::new();
        let result = rule.apply(&mut ctx, &filter_enum)
            .expect("应用规则失败");

        // 验证转换未执行（因为输入不是 Aggregate）
        assert!(result.is_none());
    }
}
