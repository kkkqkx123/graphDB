//! 将边过滤条件下推到Traverse节点的规则
//!
//! 该规则识别 Traverse 节点中的 eFilter，
//! 并将其重写为具体的边属性表达式。

use crate::core::Expression;
use crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum;
use crate::query::planner::rewrite::context::RewriteContext;
use crate::query::planner::rewrite::pattern::Pattern;
use crate::query::planner::rewrite::result::{RewriteResult, TransformResult};
use crate::query::planner::rewrite::rule::{RewriteRule, PushDownRule};

/// 将边过滤条件下推到Traverse节点的规则
///
/// # 转换示例
///
/// Before:
/// ```text
///   Traverse(eFilter: *.likeness > 78)
/// ```
///
/// After:
/// ```text
///   Traverse(filter: e.likeness > 78)
/// ```
///
/// # 适用条件
///
/// - Traverse 节点存在 eFilter
/// - eFilter 包含通配符边属性表达式
/// - Traverse 不为零步遍历
#[derive(Debug)]
pub struct PushEFilterDownRule;

impl PushEFilterDownRule {
    /// 创建规则实例
    pub fn new() -> Self {
        Self
    }
}

impl Default for PushEFilterDownRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RewriteRule for PushEFilterDownRule {
    fn name(&self) -> &'static str {
        "PushEFilterDownRule"
    }

    fn pattern(&self) -> Pattern {
        Pattern::new_with_name("Traverse")
    }

    fn apply(
        &self,
        _ctx: &mut RewriteContext,
        node: &PlanNodeEnum,
    ) -> RewriteResult<Option<TransformResult>> {
        // 检查是否为 Traverse 节点
        let traverse = match node {
            PlanNodeEnum::Traverse(t) => t,
            _ => return Ok(None),
        };

        // 获取 eFilter
        let e_filter = match traverse.e_filter() {
            Some(filter) => filter,
            None => return Ok(None),
        };

        // 获取边别名
        let edge_alias = match traverse.edge_alias() {
            Some(alias) => alias.clone(),
            None => return Ok(None),
        };

        // 重写表达式，将通配符替换为具体的边别名
        let rewritten_filter = rewrite_wildcard_to_alias(e_filter, &edge_alias);

        // 创建新的 Traverse 节点
        let mut new_traverse = traverse.clone();

        // 设置新的 eFilter
        new_traverse.set_e_filter(rewritten_filter);

        // 构建转换结果
        let mut result = TransformResult::new();
        result.erase_curr = true;
        result.add_new_node(PlanNodeEnum::Traverse(new_traverse));

        Ok(Some(result))
    }
}

impl PushDownRule for PushEFilterDownRule {
    fn can_push_down(&self, node: &PlanNodeEnum, _target: &PlanNodeEnum) -> bool {
        match node {
            PlanNodeEnum::Traverse(traverse) => {
                traverse.e_filter().is_some() && traverse.min_steps() > 0
            }
            _ => false,
        }
    }

    fn push_down(
        &self,
        ctx: &mut RewriteContext,
        node: &PlanNodeEnum,
        _target: &PlanNodeEnum,
    ) -> RewriteResult<Option<TransformResult>> {
        self.apply(ctx, node)
    }
}

/// 将表达式中的通配符替换为具体的边别名
///
/// 通配符通常表示为 `*` 或 `_`，在属性访问中表示任意边
fn rewrite_wildcard_to_alias(expr: &Expression, edge_alias: &str) -> Expression {
    match expr {
        Expression::Property { object, property } => {
            // 检查对象是否为通配符
            let new_object = match object.as_ref() {
                Expression::Variable(name) if name == "*" || name == "_" => {
                    Box::new(Expression::Variable(edge_alias.to_string()))
                }
                _ => Box::new(rewrite_wildcard_to_alias(object, edge_alias)),
            };

            Expression::Property {
                object: new_object,
                property: property.clone(),
            }
        }
        Expression::Binary { left, op, right } => {
            Expression::Binary {
                left: Box::new(rewrite_wildcard_to_alias(left, edge_alias)),
                op: *op,
                right: Box::new(rewrite_wildcard_to_alias(right, edge_alias)),
            }
        }
        Expression::Unary { op, operand } => {
            Expression::Unary {
                op: *op,
                operand: Box::new(rewrite_wildcard_to_alias(operand, edge_alias)),
            }
        }
        Expression::Function { name, args } => {
            Expression::Function {
                name: name.clone(),
                args: args.iter()
                    .map(|arg| rewrite_wildcard_to_alias(arg, edge_alias))
                    .collect(),
            }
        }
        // 其他表达式类型保持不变
        _ => expr.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::operators::BinaryOperator;

    #[test]
    fn test_rule_name() {
        let rule = PushEFilterDownRule::new();
        assert_eq!(rule.name(), "PushEFilterDownRule");
    }

    #[test]
    fn test_rule_pattern() {
        let rule = PushEFilterDownRule::new();
        let pattern = rule.pattern();
        assert!(pattern.node.is_some());
    }

    #[test]
    fn test_rewrite_wildcard_to_alias() {
        // 测试通配符属性访问
        let wildcard_expr = Expression::Property {
            object: Box::new(Expression::Variable("*".to_string())),
            property: "likeness".to_string(),
        };

        let rewritten = rewrite_wildcard_to_alias(&wildcard_expr, "e");

        match rewritten {
            Expression::Property { object, property } => {
                assert_eq!(property, "likeness");
                match object.as_ref() {
                    Expression::Variable(name) => assert_eq!(name, "e"),
                    _ => panic!("期望变量表达式"),
                }
            }
            _ => panic!("期望属性表达式"),
        }
    }

    #[test]
    fn test_rewrite_binary_expr() {
        // 测试二元表达式中的通配符
        let binary_expr = Expression::Binary {
            left: Box::new(Expression::Property {
                object: Box::new(Expression::Variable("*".to_string())),
                property: "likeness".to_string(),
            }),
            op: BinaryOperator::GreaterThan,
            right: Box::new(Expression::Literal(78.into())),
        };

        let rewritten = rewrite_wildcard_to_alias(&binary_expr, "e");

        match rewritten {
            Expression::Binary { left, op, right } => {
                assert!(matches!(op, BinaryOperator::GreaterThan));
                match left.as_ref() {
                    Expression::Property { object, property } => {
                        assert_eq!(property, "likeness");
                        match object.as_ref() {
                            Expression::Variable(name) => assert_eq!(name, "e"),
                            _ => panic!("期望变量表达式"),
                        }
                    }
                    _ => panic!("期望属性表达式"),
                }
                match right.as_ref() {
                    Expression::Literal(val) => assert_eq!(val, &78.into()),
                    _ => panic!("期望字面量表达式"),
                }
            }
            _ => panic!("期望二元表达式"),
        }
    }
}
