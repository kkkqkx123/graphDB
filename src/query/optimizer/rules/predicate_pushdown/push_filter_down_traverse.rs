//! 将过滤条件下推到遍历操作的规则
//!
//! 该规则识别 Filter -> Traverse 模式，
//! 并将边属性过滤条件下推到 Traverse 节点中。

use crate::query::optimizer::plan::{OptContext, OptGroupNode, OptRule, Pattern, TransformResult, OptimizerError};
use crate::query::optimizer::rule_patterns::PatternBuilder;
use crate::query::optimizer::rule_traits::BaseOptRule;
use crate::core::Expression;
use crate::query::optimizer::expression_utils::split_filter;
use crate::query::planner::plan::core::nodes::PlanNodeEnum;
use std::rc::Rc;
use std::cell::RefCell;

/// 将过滤条件下推到遍历操作的规则
///
/// # 转换示例
///
/// Before:
/// ```text
///   Filter(e.likeness > 78)
///           |
///   AppendVertices
///           |
///   Traverse
/// ```
///
/// After:
/// ```text
///   AppendVertices
///           |
///   Traverse(eFilter: *.likeness > 78)
/// ```
///
/// # 适用条件
///
/// - 过滤条件包含边属性表达式
/// - Traverse 节点为单步遍历
#[derive(Debug)]
pub struct PushFilterDownTraverseRule;

impl OptRule for PushFilterDownTraverseRule {
    fn name(&self) -> &str {
        "PushFilterDownTraverseRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> Result<Option<TransformResult>, OptimizerError> {
        let node_ref = group_node.borrow();
        
        if !node_ref.plan_node.is_filter() {
            return Ok(None);
        }

        if node_ref.dependencies.len() != 1 {
            return Ok(None);
        }

        let child_id = node_ref.dependencies[0];
        let child_node = match ctx.find_group_node_by_id(child_id) {
            Some(node) => node,
            None => return Ok(None),
        };

        let child_ref = child_node.borrow();
        
        if child_ref.plan_node.name() != "Traverse" {
            return Ok(None);
        }

        let filter_condition = match node_ref.plan_node.as_filter() {
            Some(filter) => filter.condition().clone(),
            None => return Ok(None),
        };

        let traverse = match &child_ref.plan_node {
            PlanNodeEnum::Traverse(t) => t,
            _ => return Ok(None),
        };

        if !traverse.is_one_step() {
            return Ok(None);
        }

        let edge_alias = match traverse.edge_alias() {
            Some(alias) => alias,
            None => return Ok(None),
        };

        let picker = |expr: &Expression| -> bool {
            is_edge_property_expression(edge_alias, expr)
        };

        let (filter_picked, filter_unpicked) = split_filter(&filter_condition, picker);

        let filter_picked = match filter_picked {
            Some(expr) => expr,
            None => return Ok(None),
        };

        let new_filter_picked = rewrite_edge_property_filter(edge_alias, &filter_picked);

        let mut new_traverse = traverse.clone();
        
        let new_e_filter = match (traverse.e_filter(), new_filter_picked) {
            (Some(ef), Some(nf)) => {
                Some(Expression::Binary {
                    left: Box::new(ef.clone()),
                    op: crate::core::BinaryOperator::And,
                    right: Box::new(nf),
                })
            }
            (Some(ef), None) => Some(ef.clone()),
            (None, Some(nf)) => Some(nf),
            (None, None) => None,
        };

        if let Some(ef) = new_e_filter {
            new_traverse.set_e_filter(ef);
        }

        let mut result = TransformResult::new();
        result.erase_curr = true;

        if let Some(unpicked) = filter_unpicked {
            let new_filter_node = match node_ref.plan_node.as_filter() {
                Some(filter) => {
                    let mut new_filter = filter.clone();
                    new_filter.set_condition(unpicked);
                    new_filter
                }
                None => return Ok(None),
            };

            let mut new_filter_group_node = node_ref.clone();
            new_filter_group_node.plan_node = PlanNodeEnum::Filter(new_filter_node);
            new_filter_group_node.dependencies = vec![child_id];

            result.add_new_group_node(Rc::new(RefCell::new(new_filter_group_node)));
        } else {
            let mut new_traverse_group_node = child_ref.clone();
            new_traverse_group_node.plan_node = PlanNodeEnum::Traverse(new_traverse);
            new_traverse_group_node.dependencies = child_ref.dependencies.clone();

            if let Some(output_var) = node_ref.plan_node.output_var() {
                new_traverse_group_node.plan_node.set_output_var(output_var.to_string());
            }

            result.add_new_group_node(Rc::new(RefCell::new(new_traverse_group_node)));
        }
        
        Ok(Some(result))
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::with_dependency("Filter", "Traverse")
    }
}

impl BaseOptRule for PushFilterDownTraverseRule {}

/// 检查表达式是否为边属性表达式
fn is_edge_property_expression(edge_alias: &str, expr: &Expression) -> bool {
    match expr {
        Expression::Property { object, .. } => {
            if let Expression::Variable(name) = object.as_ref() {
                name == edge_alias
            } else {
                false
            }
        }
        Expression::Binary { left, right, .. } => {
            is_edge_property_expression(edge_alias, left) && is_edge_property_expression(edge_alias, right)
        }
        Expression::Unary { operand, .. } => is_edge_property_expression(edge_alias, operand),
        Expression::Function { args, .. } => {
            args.iter().all(|arg| is_edge_property_expression(edge_alias, arg))
        }
        _ => false,
    }
}

/// 重写边属性表达式
fn rewrite_edge_property_filter(edge_alias: &str, expr: &Expression) -> Option<Expression> {
    match expr {
        Expression::Property { object, property } => {
            if let Expression::Variable(name) = object.as_ref() {
                if name == edge_alias {
                    Some(Expression::Property {
                        object: Box::new(Expression::Variable("*".to_string())),
                        property: property.clone(),
                    })
                } else {
                    Some(Expression::Property {
                        object: object.clone(),
                        property: property.clone(),
                    })
                }
            } else {
                Some(Expression::Property {
                    object: object.clone(),
                    property: property.clone(),
                })
            }
        }
        Expression::Binary { left, op, right } => {
            let new_left = rewrite_edge_property_filter(edge_alias, left)?;
            let new_right = rewrite_edge_property_filter(edge_alias, right)?;
            Some(Expression::Binary {
                left: Box::new(new_left),
                op: op.clone(),
                right: Box::new(new_right),
            })
        }
        Expression::Unary { op, operand } => {
            let new_operand = rewrite_edge_property_filter(edge_alias, operand)?;
            Some(Expression::Unary {
                op: op.clone(),
                operand: Box::new(new_operand),
            })
        }
        Expression::Function { name, args } => {
            let new_args: Vec<Expression> = args
                .iter()
                .map(|arg| rewrite_edge_property_filter(edge_alias, arg))
                .collect::<Option<Vec<_>>>()?;
            Some(Expression::Function {
                name: name.clone(),
                args: new_args,
            })
        }
        _ => Some(expr.clone()),
    }
}
