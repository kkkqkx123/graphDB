//! 将顶点过滤条件下推到ScanVertices节点的规则
//!
//! 该规则识别 Traverse 节点中的 vFilter，
//! 并将其重写为具体的顶点属性表达式。

use crate::core::Expression;
use crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum;
use crate::query::planner::rewrite::context::RewriteContext;
use crate::query::planner::rewrite::pattern::Pattern;
use crate::query::planner::rewrite::result::{RewriteResult, TransformResult};
use crate::query::planner::rewrite::rule::{RewriteRule, PushDownRule};

/// 将顶点过滤条件下推到ScanVertices节点的规则
///
/// # 转换示例
///
/// Before:
/// ```text
///   Traverse(vFilter: *.age > 18)
/// ```
///
/// After:
/// ```text
///   Traverse(filter: v.age > 18)
/// ```
///
/// # 适用条件
///
/// - Traverse 节点存在 vFilter
/// - vFilter 包含通配符顶点属性表达式
/// - Traverse 不为零步遍历
#[derive(Debug)]
pub struct PushVFilterDownScanVerticesRule;

impl PushVFilterDownScanVerticesRule {
    /// 创建规则实例
    pub fn new() -> Self {
        Self
    }
}

impl Default for PushVFilterDownScanVerticesRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RewriteRule for PushVFilterDownScanVerticesRule {
    fn name(&self) -> &'static str {
        "PushVFilterDownScanVerticesRule"
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

        // 获取 vFilter
        let v_filter = match traverse.v_filter() {
            Some(filter) => filter,
            None => return Ok(None),
        };

        // 获取顶点别名
        let vertex_alias = match traverse.vertex_alias() {
            Some(alias) => alias.clone(),
            None => return Ok(None),
        };

        // 重写表达式，将通配符替换为具体的顶点别名
        let rewritten_filter = rewrite_wildcard_to_alias(v_filter, &vertex_alias);

        // 创建新的 Traverse 节点
        let mut new_traverse = traverse.clone();

        // 设置新的 vFilter
        new_traverse.set_v_filter(rewritten_filter);

        // 构建转换结果
        let mut result = TransformResult::new();
        result.erase_curr = true;
        result.add_new_node(PlanNodeEnum::Traverse(new_traverse));

        Ok(Some(result))
    }
}

impl PushDownRule for PushVFilterDownScanVerticesRule {
    fn can_push_down(&self, node: &PlanNodeEnum, _target: &PlanNodeEnum) -> bool {
        match node {
            PlanNodeEnum::Traverse(traverse) => {
                traverse.v_filter().is_some() && traverse.min_steps() > 0
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

/// 将表达式中的通配符替换为具体的顶点别名
fn rewrite_wildcard_to_alias(expr: &Expression, vertex_alias: &str) -> Expression {
    match expr {
        Expression::Property { object, property } => {
            let new_object = match object.as_ref() {
                Expression::Variable(name) if name == "*" || name == "_" => {
                    Box::new(Expression::Variable(vertex_alias.to_string()))
                }
                _ => Box::new(rewrite_wildcard_to_alias(object, vertex_alias)),
            };

            Expression::Property {
                object: new_object,
                property: property.clone(),
            }
        }
        Expression::Binary { left, op, right } => {
            Expression::Binary {
                left: Box::new(rewrite_wildcard_to_alias(left, vertex_alias)),
                op: *op,
                right: Box::new(rewrite_wildcard_to_alias(right, vertex_alias)),
            }
        }
        Expression::Unary { op, operand } => {
            Expression::Unary {
                op: *op,
                operand: Box::new(rewrite_wildcard_to_alias(operand, vertex_alias)),
            }
        }
        Expression::Function { name, args } => {
            Expression::Function {
                name: name.clone(),
                args: args.iter()
                    .map(|arg| rewrite_wildcard_to_alias(arg, vertex_alias))
                    .collect(),
            }
        }
        _ => expr.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::operators::BinaryOperator;

    #[test]
    fn test_rule_name() {
        let rule = PushVFilterDownScanVerticesRule::new();
        assert_eq!(rule.name(), "PushVFilterDownScanVerticesRule");
    }

    #[test]
    fn test_rule_pattern() {
        let rule = PushVFilterDownScanVerticesRule::new();
        let pattern = rule.pattern();
        assert!(pattern.node.is_some());
    }

    #[test]
    fn test_rewrite_wildcard_to_alias() {
        // 测试通配符属性访问
        let wildcard_expr = Expression::Property {
            object: Box::new(Expression::Variable("*".to_string())),
            property: "age".to_string(),
        };

        let rewritten = rewrite_wildcard_to_alias(&wildcard_expr, "v");

        match rewritten {
            Expression::Property { object, property } => {
                assert_eq!(property, "age");
                match object.as_ref() {
                    Expression::Variable(name) => assert_eq!(name, "v"),
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
                property: "age".to_string(),
            }),
            op: BinaryOperator::GreaterThan,
            right: Box::new(Expression::Literal(18.into())),
        };

        let rewritten = rewrite_wildcard_to_alias(&binary_expr, "v");

        match rewritten {
            Expression::Binary { left, op, right } => {
                assert!(matches!(op, BinaryOperator::GreaterThan));
                match left.as_ref() {
                    Expression::Property { object, property } => {
                        assert_eq!(property, "age");
                        match object.as_ref() {
                            Expression::Variable(name) => assert_eq!(name, "v"),
                            _ => panic!("期望变量表达式"),
                        }
                    }
                    _ => panic!("期望属性表达式"),
                }
                match right.as_ref() {
                    Expression::Literal(val) => assert_eq!(val, &18.into()),
                    _ => panic!("期望字面量表达式"),
                }
            }
            _ => panic!("期望二元表达式"),
        }
    }
}
