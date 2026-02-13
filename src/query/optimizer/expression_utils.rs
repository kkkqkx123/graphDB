//! 表达式工具函数
//! 对应 NebulaGraph ExpressionUtils.h/.cpp 的功能

use crate::core::types::expression::Expression;
use crate::core::types::operators::{BinaryOperator, UnaryOperator};
use std::collections::HashSet;

/// 检查表达式中是否包含指定类型的表达式
pub fn find_any(expr: &Expression, kinds: &[ExpressionKind]) -> bool {
    if kinds.contains(&expr.kind()) {
        return true;
    }
    match expr {
        Expression::Binary { left, right, .. } => {
            find_any(left, kinds) || find_any(right, kinds)
        }
        Expression::Unary { operand, .. } => find_any(operand, kinds),
        Expression::Function { args, .. } => args.iter().any(|arg| find_any(arg, kinds)),
        Expression::List(items) => items.iter().any(|item| find_any(item, kinds)),
        Expression::Map(pairs) => pairs.iter().any(|(_, value)| find_any(value, kinds)),
        Expression::Case {
            test_expr,
            conditions,
            default,
        } => {
            test_expr.as_ref().map_or(false, |e| find_any(e, kinds))
                || conditions
                    .iter()
                    .any(|(cond, value)| find_any(cond, kinds) || find_any(value, kinds))
                || default.as_ref().map_or(false, |e| find_any(e, kinds))
        }
        Expression::TypeCast { expression, .. } => find_any(expression, kinds),
        Expression::Subscript { collection, index } => {
            find_any(collection, kinds) || find_any(index, kinds)
        }
        Expression::Range {
            collection,
            start,
            end,
        } => {
            find_any(collection, kinds)
                || start.as_ref().map_or(false, |e| find_any(e, kinds))
                || end.as_ref().map_or(false, |e| find_any(e, kinds))
        }
        Expression::Path(items) => items.iter().any(|item| find_any(item, kinds)),
        Expression::ListComprehension {
            source,
            filter,
            map,
            ..
        } => {
            find_any(source, kinds)
                || filter.as_ref().map_or(false, |e| find_any(e, kinds))
                || map.as_ref().map_or(false, |e| find_any(e, kinds))
        }
        Expression::LabelTagProperty { tag, .. } => find_any(tag, kinds),
        Expression::Predicate { args, .. } => args.iter().any(|arg| find_any(arg, kinds)),
        Expression::Reduce {
            initial,
            source,
            mapping,
            ..
        } => {
            find_any(initial, kinds)
                || find_any(source, kinds)
                || find_any(mapping, kinds)
        }
        Expression::PathBuild(exprs) => exprs.iter().any(|expr| find_any(expr, kinds)),
        _ => false,
    }
}

/// 检查表达式中是否包含NOT操作符
pub fn contains_not(expr: &Expression) -> bool {
    match expr {
        Expression::Unary {
            op: UnaryOperator::Not,
            operand: _,
        } => true,
        Expression::Binary { left, right, .. } => {
            contains_not(left) || contains_not(right)
        }
        Expression::Unary { operand, .. } => contains_not(operand),
        Expression::Function { args, .. } => args.iter().any(|arg| contains_not(arg)),
        Expression::List(items) => items.iter().any(|item| contains_not(item)),
        Expression::Map(pairs) => pairs.iter().any(|(_, value)| contains_not(value)),
        Expression::Case {
            test_expr,
            conditions,
            default,
        } => {
            test_expr.as_ref().map_or(false, |e| contains_not(e))
                || conditions
                    .iter()
                    .any(|(cond, value)| contains_not(cond) || contains_not(value))
                || default.as_ref().map_or(false, |e| contains_not(e))
        }
        Expression::TypeCast { expression, .. } => contains_not(expression),
        Expression::Subscript { collection, index } => {
            contains_not(collection) || contains_not(index)
        }
        Expression::Range {
            collection,
            start,
            end,
        } => {
            contains_not(collection)
                || start.as_ref().map_or(false, |e| contains_not(e))
                || end.as_ref().map_or(false, |e| contains_not(e))
        }
        Expression::Path(items) => items.iter().any(|item| contains_not(item)),
        Expression::ListComprehension {
            source,
            filter,
            map,
            ..
        } => {
            contains_not(source)
                || filter.as_ref().map_or(false, |e| contains_not(e))
                || map.as_ref().map_or(false, |e| contains_not(e))
        }
        Expression::LabelTagProperty { tag, .. } => contains_not(tag),
        Expression::Predicate { args, .. } => args.iter().any(|arg| contains_not(arg)),
        Expression::Reduce {
            initial,
            source,
            mapping,
            ..
        } => {
            contains_not(initial)
                || contains_not(source)
                || contains_not(mapping)
        }
        Expression::PathBuild(exprs) => exprs.iter().any(|expr| contains_not(expr)),
        _ => false,
    }
}

/// 扁平化嵌套的逻辑AND表达式
/// 例如：(A AND (B AND C)) AND D => (A AND B AND C AND D)
fn flatten_inner_logical_and_expr(expr: &Expression) -> Expression {
    match expr {
        Expression::Binary {
            left,
            op: BinaryOperator::And,
            right,
        } => {
            let left_flattened = flatten_inner_logical_and_expr(left);
            let right_flattened = flatten_inner_logical_and_expr(right);
            
            let mut operands = Vec::new();
            collect_and_operands(&left_flattened, &mut operands);
            collect_and_operands(&right_flattened, &mut operands);
            
            if operands.len() == 1 {
                operands.into_iter().next().unwrap()
            } else {
                let mut result = None;
                for operand in operands {
                    result = Some(match result {
                        None => operand,
                        Some(acc) => Expression::Binary {
                            left: Box::new(acc),
                            op: BinaryOperator::And,
                            right: Box::new(operand),
                        },
                    });
                }
                result.unwrap()
            }
        }
        _ => expr.clone(),
    }
}

/// 扁平化嵌套的逻辑OR表达式
fn flatten_inner_logical_or_expr(expr: &Expression) -> Expression {
    match expr {
        Expression::Binary {
            left,
            op: BinaryOperator::Or,
            right,
        } => {
            let left_flattened = flatten_inner_logical_or_expr(left);
            let right_flattened = flatten_inner_logical_or_expr(right);
            
            let mut operands = Vec::new();
            collect_or_operands(&left_flattened, &mut operands);
            collect_or_operands(&right_flattened, &mut operands);
            
            if operands.len() == 1 {
                operands.into_iter().next().expect("operands不应为空")
            } else {
                let mut result = None;
                for operand in operands {
                    result = Some(match result {
                        None => operand,
                        Some(acc) => Expression::Binary {
                            left: Box::new(acc),
                            op: BinaryOperator::Or,
                            right: Box::new(operand),
                        },
                    });
                }
                result.expect("result不应为空")
            }
        }
        _ => expr.clone(),
    }
}

/// 收集AND表达式的所有操作数
fn collect_and_operands(expr: &Expression, operands: &mut Vec<Expression>) {
    match expr {
        Expression::Binary {
            left,
            op: BinaryOperator::And,
            right,
        } => {
            collect_and_operands(left, operands);
            collect_and_operands(right, operands);
        }
        _ => operands.push(expr.clone()),
    }
}

/// 收集OR表达式的所有操作数
fn collect_or_operands(expr: &Expression, operands: &mut Vec<Expression>) {
    match expr {
        Expression::Binary {
            left,
            op: BinaryOperator::Or,
            right,
        } => {
            collect_or_operands(left, operands);
            collect_or_operands(right, operands);
        }
        _ => operands.push(expr.clone()),
    }
}

/// 扁平化嵌套的逻辑表达式
/// 先扁平化AND，再扁平化OR
fn flatten_inner_logical_expr(expr: &Expression) -> Expression {
    let and_flattened = flatten_inner_logical_and_expr(expr);
    flatten_inner_logical_or_expr(&and_flattened)
}

/// 收集AND表达式的所有操作数到向量中
fn collect_and_operands_vec(expr: &Expression, operands: &mut Vec<Expression>) {
    match expr {
        Expression::Binary {
            left,
            op: BinaryOperator::And,
            right,
        } => {
            collect_and_operands_vec(left, operands);
            collect_and_operands_vec(right, operands);
        }
        _ => operands.push(expr.clone()),
    }
}

/// 分离过滤器表达式
/// 
/// # 参数
/// * `condition` - 原始过滤条件
/// * `picker` - 判断表达式是否应该被提取的函数
/// 
/// # 返回
/// 返回一个元组：(被提取的表达式, 剩余的表达式)
/// 
/// 参考 nebula-graph ExpressionUtils::splitFilter 实现
/// 
/// # 算法说明
/// 1. 如果表达式不是LogicalAnd，直接应用picker
/// 2. 如果是LogicalAnd，先flatten嵌套的逻辑表达式
/// 3. 遍历所有操作数，检查是否包含NOT表达式
/// 4. 如果包含NOT，放入filterUnpicked
/// 5. 否则应用picker判断：true放入filterPicked，false放入filterUnpicked
/// 6. 最后fold逻辑表达式，简化结果
pub fn split_filter(
    condition: &Expression,
    picker: impl Fn(&Expression) -> bool + Copy,
) -> (Option<Expression>, Option<Expression>) {
    match condition {
        Expression::Binary {
            left: _,
            op: BinaryOperator::And,
            right: _,
        } => {
            let flatten_expr = flatten_inner_logical_expr(condition);
            
            let mut picked_operands = Vec::new();
            let mut unpicked_operands = Vec::new();
            
            let mut all_operands = Vec::new();
            collect_and_operands_vec(&flatten_expr, &mut all_operands);
            
            for operand in all_operands {
                if contains_not(&operand) {
                    unpicked_operands.push(operand);
                } else if picker(&operand) {
                    picked_operands.push(operand);
                } else {
                    unpicked_operands.push(operand);
                }
            }
            
            let filter_picked = fold_logical_expr(&picked_operands, BinaryOperator::And);
            let filter_unpicked = fold_logical_expr(&unpicked_operands, BinaryOperator::And);
            
            (filter_picked, filter_unpicked)
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

/// 折叠逻辑表达式
/// 根据操作数数量决定返回单个表达式、组合表达式或None
fn fold_logical_expr(operands: &[Expression], op: BinaryOperator) -> Option<Expression> {
    match operands.len() {
        0 => None,
        1 => Some(operands[0].clone()),
        _ => {
            let mut result = None;
            for operand in operands {
                result = Some(match result {
                    None => operand.clone(),
                    Some(acc) => Expression::Binary {
                        left: Box::new(acc),
                        op: op.clone(),
                        right: Box::new(operand.clone()),
                    },
                });
            }
            result
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
        Expression::Property { object, property: _ } => {
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

/// 将多个表达式用 AND 连接成一个表达式
pub fn and_all(exprs: Vec<Expression>) -> Expression {
    if exprs.is_empty() {
        return Expression::Literal(crate::core::Value::Bool(true));
    }

    let mut iter = exprs.into_iter();
    let mut result = iter.next().expect("exprs不应为空");

    while let Some(expression) = iter.next() {
        result = Expression::Binary {
            left: Box::new(expression),
            op: BinaryOperator::And,
            right: Box::new(result),
        };
    }

    result
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
