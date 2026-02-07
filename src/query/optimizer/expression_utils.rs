//! 表达式工具函数
//! 对应 NebulaGraph ExpressionUtils.h/.cpp 的功能

use crate::core::types::expression::Expression;
use crate::core::BinaryOperator;
use std::collections::HashSet;

/// 分离过滤器表达式
/// 
/// # 参数
/// * `condition` - 原始过滤条件
/// * `picker` - 判断表达式是否应该被提取的函数
/// 
/// # 返回
/// 返回一个元组：(被提取的表达式, 剩余的表达式)
pub fn split_filter(
    condition: &Expression,
    picker: impl Fn(&Expression) -> bool + Copy,
) -> (Option<Expression>, Option<Expression>) {
    match condition {
        Expression::Binary { left, op, right } => {
            let (left_picked, left_remained) = split_filter(left, picker);
            let (right_picked, right_remained) = split_filter(right, picker);

            match (left_picked, right_picked) {
                (Some(lp), Some(rp)) => {
                    let picked = Some(Expression::Binary {
                        left: Box::new(lp),
                        op: op.clone(),
                        right: Box::new(rp),
                    });
                    let remained = match (left_remained, right_remained) {
                        (Some(lr), Some(rr)) => Some(Expression::Binary {
                            left: Box::new(lr),
                            op: op.clone(),
                            right: Box::new(rr),
                        }),
                        (Some(lr), None) => Some(lr),
                        (None, Some(rr)) => Some(rr),
                        (None, None) => None,
                    };
                    (picked, remained)
                }
                (Some(lp), None) => {
                    let remained = match (left_remained, right_remained) {
                        (Some(lr), Some(rr)) => Some(Expression::Binary {
                            left: Box::new(lr),
                            op: op.clone(),
                            right: Box::new(rr),
                        }),
                        (Some(lr), None) => Some(lr),
                        (None, Some(rr)) => Some(rr),
                        (None, None) => None,
                    };
                    (Some(lp), remained)
                }
                (None, Some(rp)) => {
                    let remained = match (left_remained, right_remained) {
                        (Some(lr), Some(rr)) => Some(Expression::Binary {
                            left: Box::new(lr),
                            op: op.clone(),
                            right: Box::new(rr),
                        }),
                        (Some(lr), None) => Some(lr),
                        (None, Some(rr)) => Some(rr),
                        (None, None) => None,
                    };
                    (Some(rp), remained)
                }
                (None, None) => {
                    let remained = match (left_remained, right_remained) {
                        (Some(lr), Some(rr)) => Some(Expression::Binary {
                            left: Box::new(lr),
                            op: op.clone(),
                            right: Box::new(rr),
                        }),
                        (Some(lr), None) => Some(lr),
                        (None, Some(rr)) => Some(rr),
                        (None, None) => None,
                    };
                    (None, remained)
                }
            }
        }
        Expression::Unary { op, operand } => {
            if picker(condition) {
                (Some(condition.clone()), None)
            } else {
                (None, Some(condition.clone()))
            }
        }
        Expression::Function { name, args } => {
            if picker(condition) {
                (Some(condition.clone()), None)
            } else {
                (None, Some(condition.clone()))
            }
        }
        _ => {
            if picker(condition) {
                (Some(condition.clone()), None)
            } else {
                (None, Some(condition.clone()))
            }
        }
    }
}

/// 创建 AND 逻辑表达式
pub fn make_and(left: Expression, right: Expression) -> Expression {
    Expression::Binary {
        left: Box::new(left),
        op: BinaryOperator::And,
        right: Box::new(right),
    }
}

/// 创建 OR 逻辑表达式
pub fn make_or(left: Expression, right: Expression) -> Expression {
    Expression::Binary {
        left: Box::new(left),
        op: BinaryOperator::Or,
        right: Box::new(right),
    }
}

/// 检查表达式是否为单步边属性表达式
/// 
/// 单步边属性表达式是指形如 `e.prop` 的表达式，其中 `e` 是边别名
pub fn is_one_step_edge_prop(edge_alias: &str, expr: &Expression) -> bool {
    match expr {
        Expression::Property { object, property } => {
            if let Expression::Variable(name) = object.as_ref() {
                name == edge_alias
            } else {
                false
            }
        }
        _ => false,
    }
}

/// 重写边属性表达式
/// 
/// 将边属性表达式中的边别名替换为具体的边类型
/// 例如：将 `e.prop` 重写为 `follow.prop`
pub fn rewrite_edge_property_filter(
    edge_alias: &str,
    edge_name: &str,
    expr: Expression,
) -> Option<Expression> {
    match expr {
        Expression::Property { object, property } => {
            if let Expression::Variable(name) = object.as_ref() {
                if name == edge_alias {
                    Some(Expression::Property {
                        object: Box::new(Expression::Variable(edge_name.to_string())),
                        property,
                    })
                } else {
                    Some(Expression::Property { object, property })
                }
            } else {
                Some(Expression::Property { object, property })
            }
        }
        Expression::Binary { left, op, right } => {
            let new_left = rewrite_edge_property_filter(edge_alias, edge_name, *left)?;
            let new_right = rewrite_edge_property_filter(edge_alias, edge_name, *right)?;
            Some(Expression::Binary {
                left: Box::new(new_left),
                op,
                right: Box::new(new_right),
            })
        }
        Expression::Unary { op, operand } => {
            let new_operand = rewrite_edge_property_filter(edge_alias, edge_name, *operand)?;
            Some(Expression::Unary { op, operand: Box::new(new_operand) })
        }
        Expression::Function { name, args } => {
            let new_args: Vec<Expression> = args
                .into_iter()
                .map(|arg| rewrite_edge_property_filter(edge_alias, edge_name, arg))
                .collect::<Option<Vec<_>>>()?;
            Some(Expression::Function { name, args: new_args })
        }
        _ => Some(expr),
    }
}

/// 收集所有匹配的表达式
pub fn collect_all(
    expr: &Expression,
    kinds: &[ExpressionKind],
) -> Vec<Expression> {
    let mut results = Vec::new();
    collect_all_helper(expr, kinds, &mut results);
    results
}

fn collect_all_helper(
    expr: &Expression,
    kinds: &[ExpressionKind],
    results: &mut Vec<Expression>,
) {
    if kinds.contains(&expr.kind()) {
        results.push(expr.clone());
    }

    match expr {
        Expression::Binary { left, right, .. } => {
            collect_all_helper(left, kinds, results);
            collect_all_helper(right, kinds, results);
        }
        Expression::Unary { operand, .. } => {
            collect_all_helper(operand, kinds, results);
        }
        Expression::Function { args, .. } => {
            for arg in args {
                collect_all_helper(arg, kinds, results);
            }
        }
        Expression::List(items) => {
            for item in items {
                collect_all_helper(item, kinds, results);
            }
        }
        Expression::Map(pairs) => {
            for (_, value) in pairs {
                collect_all_helper(value, kinds, results);
            }
        }
        Expression::Case {
            test_expr,
            conditions,
            default,
        } => {
            if let Some(test) = test_expr {
                collect_all_helper(test, kinds, results);
            }
            for (cond, value) in conditions {
                collect_all_helper(cond, kinds, results);
                collect_all_helper(value, kinds, results);
            }
            if let Some(d) = default {
                collect_all_helper(d, kinds, results);
            }
        }
        Expression::TypeCast { expression, .. } => {
            collect_all_helper(expression, kinds, results);
        }
        Expression::Subscript { collection, index } => {
            collect_all_helper(collection, kinds, results);
            collect_all_helper(index, kinds, results);
        }
        Expression::Range {
            collection,
            start,
            end,
        } => {
            collect_all_helper(collection, kinds, results);
            if let Some(s) = start {
                collect_all_helper(s, kinds, results);
            }
            if let Some(e) = end {
                collect_all_helper(e, kinds, results);
            }
        }
        Expression::Path(items) => {
            for item in items {
                collect_all_helper(item, kinds, results);
            }
        }
        Expression::ListComprehension {
            source,
            filter,
            map,
            ..
        } => {
            collect_all_helper(source, kinds, results);
            if let Some(f) = filter {
                collect_all_helper(f, kinds, results);
            }
            if let Some(m) = map {
                collect_all_helper(m, kinds, results);
            }
        }
        Expression::LabelTagProperty { tag, .. } => {
            collect_all_helper(tag, kinds, results);
        }
        Expression::TagProperty { .. } => {}
        Expression::EdgeProperty { .. } => {}
        Expression::Predicate { args, .. } => {
            for arg in args {
                collect_all_helper(arg, kinds, results);
            }
        }
        Expression::Reduce {
            initial,
            source,
            mapping,
            ..
        } => {
            collect_all_helper(initial, kinds, results);
            collect_all_helper(source, kinds, results);
            collect_all_helper(mapping, kinds, results);
        }
        Expression::PathBuild(exprs) => {
            for expr in exprs {
                collect_all_helper(expr, kinds, results);
            }
        }
        _ => {}
    }
}

/// 表达式类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpressionKind {
    Literal,
    Variable,
    Property,
    Binary,
    Unary,
    Function,
    Aggregate,
    List,
    Map,
    Case,
    TypeCast,
    Subscript,
    Range,
    Path,
    Label,
    ListComprehension,
    LabelTagProperty,
    TagProperty,
    EdgeProperty,
    Predicate,
    Reduce,
    PathBuild,
}

impl Expression {
    pub fn kind(&self) -> ExpressionKind {
        match self {
            Expression::Literal(_) => ExpressionKind::Literal,
            Expression::Variable(_) => ExpressionKind::Variable,
            Expression::Property { .. } => ExpressionKind::Property,
            Expression::Binary { .. } => ExpressionKind::Binary,
            Expression::Unary { .. } => ExpressionKind::Unary,
            Expression::Function { .. } => ExpressionKind::Function,
            Expression::Aggregate { .. } => ExpressionKind::Aggregate,
            Expression::List(_) => ExpressionKind::List,
            Expression::Map(_) => ExpressionKind::Map,
            Expression::Case { .. } => ExpressionKind::Case,
            Expression::TypeCast { .. } => ExpressionKind::TypeCast,
            Expression::Subscript { .. } => ExpressionKind::Subscript,
            Expression::Range { .. } => ExpressionKind::Range,
            Expression::Path(_) => ExpressionKind::Path,
            Expression::Label(_) => ExpressionKind::Label,
            Expression::ListComprehension { .. } => ExpressionKind::ListComprehension,
            Expression::LabelTagProperty { .. } => ExpressionKind::LabelTagProperty,
            Expression::TagProperty { .. } => ExpressionKind::TagProperty,
            Expression::EdgeProperty { .. } => ExpressionKind::EdgeProperty,
            Expression::Predicate { .. } => ExpressionKind::Predicate,
            Expression::Reduce { .. } => ExpressionKind::Reduce,
            Expression::PathBuild(_) => ExpressionKind::PathBuild,
        }
    }
}

/// 检查表达式中的变量名是否在给定的列名列表中
/// 
/// # 参数
/// * `col_names` - 列名列表
/// * `expr` - 要检查的表达式
/// 
/// # 返回
/// 如果表达式中的所有变量名都在列名列表中，则返回 true
pub fn check_col_name(col_names: &[String], expr: &Expression) -> bool {
    let col_set: HashSet<&str> = col_names.iter().map(|s| s.as_str()).collect();
    check_col_name_helper(&col_set, expr)
}

fn check_col_name_helper(col_set: &HashSet<&str>, expr: &Expression) -> bool {
    match expr {
        Expression::Variable(name) => col_set.contains(name.as_str()),
        Expression::Property { object, .. } => check_col_name_helper(col_set, object),
        Expression::Binary { left, right, .. } => {
            check_col_name_helper(col_set, left) && check_col_name_helper(col_set, right)
        }
        Expression::Unary { operand, .. } => check_col_name_helper(col_set, operand),
        Expression::Function { args, .. } => {
            args.iter().all(|arg| check_col_name_helper(col_set, arg))
        }
        Expression::Aggregate { arg, .. } => {
            check_col_name_helper(col_set, arg)
        }
        Expression::List(items) => items.iter().all(|item| check_col_name_helper(col_set, item)),
        Expression::Map(pairs) => {
            pairs.iter().all(|(_, value)| check_col_name_helper(col_set, value))
        }
        Expression::Case {
            test_expr,
            conditions,
            default,
        } => {
            test_expr.as_ref().map_or(true, |e| check_col_name_helper(col_set, e))
                && conditions
                    .iter()
                    .all(|(cond, value)| {
                        check_col_name_helper(col_set, cond) && check_col_name_helper(col_set, value)
                    })
                && default.as_ref().map_or(true, |e| check_col_name_helper(col_set, e))
        }
        Expression::TypeCast { expression, .. } => check_col_name_helper(col_set, expression),
        Expression::Subscript { collection, index } => {
            check_col_name_helper(col_set, collection) && check_col_name_helper(col_set, index)
        }
        Expression::Range {
            collection,
            start,
            end,
        } => {
            check_col_name_helper(col_set, collection)
                && start.as_ref().map_or(true, |e| check_col_name_helper(col_set, e))
                && end.as_ref().map_or(true, |e| check_col_name_helper(col_set, e))
        }
        Expression::Path(items) => items.iter().all(|item| check_col_name_helper(col_set, item)),
        Expression::ListComprehension {
            source,
            filter,
            map,
            ..
        } => {
            check_col_name_helper(col_set, source)
                && filter.as_ref().map_or(true, |e| check_col_name_helper(col_set, e))
                && map.as_ref().map_or(true, |e| check_col_name_helper(col_set, e))
        }
        Expression::LabelTagProperty { tag, .. } => check_col_name_helper(col_set, tag),
        Expression::TagProperty { .. } => true,
        Expression::EdgeProperty { .. } => true,
        Expression::Predicate { args, .. } => {
            args.iter().all(|arg| check_col_name_helper(col_set, arg))
        }
        Expression::Reduce {
            initial,
            source,
            mapping,
            ..
        } => {
            check_col_name_helper(col_set, initial)
                && check_col_name_helper(col_set, source)
                && check_col_name_helper(col_set, mapping)
        }
        Expression::PathBuild(exprs) => exprs.iter().all(|expr| check_col_name_helper(col_set, expr)),
        _ => true,
    }
}
