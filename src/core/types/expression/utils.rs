//! 表达式工具函数
//!
//! 提供表达式分析和转换的工具函数，替代访问者模式。
//!
//! 这些函数使用递归和模式匹配，比访问者模式更简洁直观。

use crate::core::types::expression::Expression;
use crate::core::types::operators::AggregateFunction;
use crate::core::{BinaryOperator, DataType, UnaryOperator, Value};
use crate::expression::evaluator::ExpressionEvaluator;

/// 分组套件
#[derive(Debug, Clone, Default)]
pub struct GroupSuite {
    /// 分组键集合
    pub group_keys: Vec<Expression>,
    /// 分组项集合
    pub group_items: Vec<Expression>,
    /// 聚合函数集合
    pub aggregates: Vec<Expression>,
}

impl GroupSuite {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_group_key(&mut self, expression: Expression) {
        if !self.group_keys.contains(&expression) {
            self.group_keys.push(expression);
        }
    }

    pub fn add_group_item(&mut self, expression: Expression) {
        if !self.group_items.contains(&expression) {
            self.group_items.push(expression);
        }
    }

    pub fn add_aggregate(&mut self, expression: Expression) {
        if !self.aggregates.contains(&expression) {
            self.aggregates.push(expression);
        }
    }

    pub fn is_empty(&self) -> bool {
        self.group_keys.is_empty()
            && self.group_items.is_empty()
            && self.aggregates.is_empty()
    }

    pub fn union(&mut self, other: &GroupSuite) {
        for key in &other.group_keys {
            self.add_group_key(key.clone());
        }
        for item in &other.group_items {
            self.add_group_item(item.clone());
        }
        for agg in &other.aggregates {
            self.add_aggregate(agg.clone());
        }
    }
}

/// 从表达式中提取分组套件
///
/// 用于 GROUP BY 优化，识别可用于分组的表达式和聚合函数。
///
/// # 参数
/// - `expression`: 要分析的表达式
///
/// # 返回
/// - `Ok(GroupSuite)`: 提取到的分组套件
/// - `Err(String)`: 错误信息
pub fn extract_group_suite(expression: &Expression) -> Result<GroupSuite, String> {
    let mut group_suite = GroupSuite::new();
    extract_group_suite_recursive(expression, &mut group_suite);
    Ok(group_suite)
}

/// 递归提取分组套件的辅助函数
fn extract_group_suite_recursive(expression: &Expression, group_suite: &mut GroupSuite) {
    match expression {
        Expression::Literal(value) => {
            group_suite.add_group_key(Expression::Literal(value.clone()));
        }
        Expression::Variable(name) => {
            group_suite.add_group_key(Expression::Variable(name.clone()));
        }
        Expression::Property { object, property } => {
            let prop_expression = Expression::Property {
                object: Box::new(object.as_ref().clone()),
                property: property.clone(),
            };
            group_suite.add_group_key(prop_expression);
            extract_group_suite_recursive(object, group_suite);
        }
        Expression::Binary { left, right, .. } => {
            if is_groupable(left) {
                group_suite.add_group_key(left.as_ref().clone());
            }
            if is_groupable(right) {
                group_suite.add_group_key(right.as_ref().clone());
            }
            extract_group_suite_recursive(left, group_suite);
            extract_group_suite_recursive(right, group_suite);
        }
        Expression::Unary { operand, .. } => {
            if is_groupable(operand) {
                group_suite.add_group_key(operand.as_ref().clone());
            }
            extract_group_suite_recursive(operand, group_suite);
        }
        Expression::Function { name, args } => {
            let name_upper = name.to_uppercase();
            if matches!(name_upper.as_str(), "ID" | "SRC" | "DST") && args.len() == 1 {
                let func_expression = Expression::Function {
                    name: name.clone(),
                    args: args.clone(),
                };
                group_suite.add_group_key(func_expression);
            }
            for arg in args {
                extract_group_suite_recursive(arg, group_suite);
            }
        }
        Expression::Aggregate { func, arg, distinct } => {
            let agg_expression = Expression::Aggregate {
                func: func.clone(),
                arg: Box::new(arg.as_ref().clone()),
                distinct: *distinct,
            };
            group_suite.add_aggregate(agg_expression);
            extract_group_suite_recursive(arg, group_suite);
        }
        Expression::List(items) => {
            for item in items {
                extract_group_suite_recursive(item, group_suite);
            }
        }
        Expression::Map(pairs) => {
            for (_, expression) in pairs {
                extract_group_suite_recursive(expression, group_suite);
            }
        }
        Expression::Case {
            test_expr,
            conditions,
            default,
        } => {
            if let Some(test) = test_expr {
                extract_group_suite_recursive(test, group_suite);
            }
            for (cond, expr) in conditions {
                extract_group_suite_recursive(cond, group_suite);
                extract_group_suite_recursive(expr, group_suite);
            }
            if let Some(def) = default {
                extract_group_suite_recursive(def, group_suite);
            }
        }
        Expression::TypeCast { expression, .. } => {
            extract_group_suite_recursive(expression, group_suite);
        }
        Expression::Subscript { collection, index } => {
            extract_group_suite_recursive(collection, group_suite);
            extract_group_suite_recursive(index, group_suite);
        }
        Expression::Range {
            collection,
            start,
            end,
        } => {
            extract_group_suite_recursive(collection, group_suite);
            if let Some(s) = start {
                extract_group_suite_recursive(s, group_suite);
            }
            if let Some(e) = end {
                extract_group_suite_recursive(e, group_suite);
            }
        }
        Expression::Path(items) => {
            for item in items {
                extract_group_suite_recursive(item, group_suite);
            }
        }
        Expression::Label(name) => {
            group_suite.add_group_key(Expression::Label(name.clone()));
        }
        Expression::ListComprehension {
            variable,
            source,
            filter,
            map,
        } => {
            group_suite.add_group_key(Expression::Variable(variable.clone()));
            extract_group_suite_recursive(source, group_suite);
            if let Some(f) = filter {
                extract_group_suite_recursive(f, group_suite);
            }
            if let Some(m) = map {
                extract_group_suite_recursive(m, group_suite);
            }
        }
        Expression::Parameter(name) => {
            group_suite.add_group_key(Expression::Parameter(name.clone()));
        }
        _ => {}
    }
}

/// 检查表达式是否为可分组的表达式
fn is_groupable(expression: &Expression) -> bool {
    match expression {
        Expression::Literal(_) => true,
        Expression::Variable(_) => true,
        Expression::Property { .. } => true,
        Expression::Function { name, args } => {
            let name_upper = name.to_uppercase();
            matches!(name_upper.as_str(), "ID" | "SRC" | "DST") && args.len() == 1
        }
        _ => false,
    }
}

/// 检查表达式是否可以在编译时求值（静态可求值性检查）
///
/// 此方法委托给 ExpressionEvaluator::can_evaluate
pub fn is_evaluable(expression: &Expression) -> bool {
    ExpressionEvaluator::can_evaluate(expression)
}

/// 检查表达式是否为常量
pub fn is_constant(expression: &Expression) -> bool {
    matches!(expression, Expression::Literal(_))
}

/// 查找表达式中所有匹配条件的表达式
///
/// # 参数
/// - `expression`: 要搜索的表达式
/// - `predicate`: 匹配条件函数
///
/// # 返回
/// 所有匹配的表达式列表
pub fn find_all<F>(expression: &Expression, predicate: F) -> Vec<Expression>
where
    F: Fn(&Expression) -> bool,
{
    let mut results = Vec::new();
    find_all_recursive(expression, &predicate, &mut results);
    results
}

/// 递归查找表达式的辅助函数
fn find_all_recursive<F>(
    expression: &Expression,
    predicate: &F,
    results: &mut Vec<Expression>,
) where
    F: Fn(&Expression) -> bool,
{
    if predicate(expression) {
        results.push(expression.clone());
    }
    for child in expression.children() {
        find_all_recursive(child, predicate, results);
    }
}

/// 收集表达式中所有的变量
///
/// # 参数
/// - `expression`: 要分析的表达式
///
/// # 返回
/// 所有变量名称的列表
pub fn collect_variables(expression: &Expression) -> Vec<String> {
    let mut variables = Vec::new();
    collect_variables_recursive(expression, &mut variables);
    variables.sort();
    variables.dedup();
    variables
}

/// 递归收集变量的辅助函数
fn collect_variables_recursive(expression: &Expression, variables: &mut Vec<String>) {
    match expression {
        Expression::Variable(name) => {
            if !variables.contains(name) {
                variables.push(name.clone());
            }
        }
        Expression::Property { object, .. } => {
            collect_variables_recursive(object, variables);
        }
        Expression::Binary { left, right, .. } => {
            collect_variables_recursive(left, variables);
            collect_variables_recursive(right, variables);
        }
        Expression::Unary { operand, .. } => {
            collect_variables_recursive(operand, variables);
        }
        Expression::Function { args, .. } => {
            for arg in args {
                collect_variables_recursive(arg, variables);
            }
        }
        Expression::Aggregate { arg, .. } => {
            collect_variables_recursive(arg, variables);
        }
        Expression::List(items) => {
            for item in items {
                collect_variables_recursive(item, variables);
            }
        }
        Expression::Map(pairs) => {
            for (_, expr) in pairs {
                collect_variables_recursive(expr, variables);
            }
        }
        Expression::Case {
            test_expr,
            conditions,
            default,
        } => {
            if let Some(test) = test_expr {
                collect_variables_recursive(test, variables);
            }
            for (cond, expr) in conditions {
                collect_variables_recursive(cond, variables);
                collect_variables_recursive(expr, variables);
            }
            if let Some(def) = default {
                collect_variables_recursive(def, variables);
            }
        }
        Expression::TypeCast { expression, .. } => {
            collect_variables_recursive(expression, variables);
        }
        Expression::Subscript { collection, index } => {
            collect_variables_recursive(collection, variables);
            collect_variables_recursive(index, variables);
        }
        Expression::Range {
            collection,
            start,
            end,
        } => {
            collect_variables_recursive(collection, variables);
            if let Some(s) = start {
                collect_variables_recursive(s, variables);
            }
            if let Some(e) = end {
                collect_variables_recursive(e, variables);
            }
        }
        Expression::Path(items) => {
            for item in items {
                collect_variables_recursive(item, variables);
            }
        }
        Expression::ListComprehension {
            variable,
            source,
            filter,
            map,
        } => {
            if !variables.contains(variable) {
                variables.push(variable.clone());
            }
            collect_variables_recursive(source, variables);
            if let Some(f) = filter {
                collect_variables_recursive(f, variables);
            }
            if let Some(m) = map {
                collect_variables_recursive(m, variables);
            }
        }
        _ => {}
    }
}

/// 检查表达式中是否包含聚合函数
///
/// # 参数
/// - `expression`: 要检查的表达式
///
/// # 返回
/// 如果包含聚合函数返回 true，否则返回 false
pub fn has_aggregate_function(expression: &Expression) -> bool {
    match expression {
        Expression::Aggregate { .. } => true,
        Expression::Binary { left, right, .. } => {
            has_aggregate_function(left) || has_aggregate_function(right)
        }
        Expression::Unary { operand, .. } => has_aggregate_function(operand),
        Expression::Function { args, .. } => args.iter().any(|arg| has_aggregate_function(arg)),
        Expression::List(items) => items.iter().any(|item| has_aggregate_function(item)),
        Expression::Map(pairs) => {
            pairs.iter().any(|(_, expr)| has_aggregate_function(expr))
        }
        Expression::Case {
            test_expr,
            conditions,
            default,
        } => {
            test_expr.as_ref().map_or(false, |e| has_aggregate_function(e))
                || conditions
                    .iter()
                    .any(|(cond, expr)| has_aggregate_function(cond) || has_aggregate_function(expr))
                || default.as_ref().map_or(false, |e| has_aggregate_function(e))
        }
        Expression::TypeCast { expression, .. } => has_aggregate_function(expression),
        Expression::Subscript { collection, index } => {
            has_aggregate_function(collection) || has_aggregate_function(index)
        }
        Expression::Range {
            collection,
            start,
            end,
        } => {
            has_aggregate_function(collection)
                || start.as_ref().map_or(false, |e| has_aggregate_function(e))
                || end.as_ref().map_or(false, |e| has_aggregate_function(e))
        }
        Expression::Path(items) => items.iter().any(|item| has_aggregate_function(item)),
        Expression::ListComprehension {
            source,
            filter,
            map,
            ..
        } => {
            has_aggregate_function(source)
                || filter.as_ref().map_or(false, |e| has_aggregate_function(e))
                || map.as_ref().map_or(false, |e| has_aggregate_function(e))
        }
        Expression::Property { object, .. } => has_aggregate_function(object),
        _ => false,
    }
}

/// 从表达式中提取所有聚合函数
///
/// # 参数
/// - `expression`: 要分析的表达式
///
/// # 返回
/// 所有聚合函数的列表
pub fn extract_aggregate_functions(expression: &Expression) -> Vec<AggregateFunction> {
    let mut functions = Vec::new();
    extract_aggregate_functions_recursive(expression, &mut functions);
    functions
}

/// 递归提取聚合函数的辅助函数
fn extract_aggregate_functions_recursive(
    expression: &Expression,
    functions: &mut Vec<AggregateFunction>,
) {
    match expression {
        Expression::Aggregate { func, .. } => {
            functions.push(func.clone());
        }
        Expression::Binary { left, right, .. } => {
            extract_aggregate_functions_recursive(left, functions);
            extract_aggregate_functions_recursive(right, functions);
        }
        Expression::Unary { operand, .. } => {
            extract_aggregate_functions_recursive(operand, functions);
        }
        Expression::Function { args, .. } => {
            for arg in args {
                extract_aggregate_functions_recursive(arg, functions);
            }
        }
        Expression::List(items) => {
            for item in items {
                extract_aggregate_functions_recursive(item, functions);
            }
        }
        Expression::Map(pairs) => {
            for (_, expr) in pairs {
                extract_aggregate_functions_recursive(expr, functions);
            }
        }
        Expression::Case {
            test_expr,
            conditions,
            default,
        } => {
            if let Some(test) = test_expr {
                extract_aggregate_functions_recursive(test, functions);
            }
            for (cond, expr) in conditions {
                extract_aggregate_functions_recursive(cond, functions);
                extract_aggregate_functions_recursive(expr, functions);
            }
            if let Some(def) = default {
                extract_aggregate_functions_recursive(def, functions);
            }
        }
        Expression::TypeCast { expression, .. } => {
            extract_aggregate_functions_recursive(expression, functions);
        }
        Expression::Subscript { collection, index } => {
            extract_aggregate_functions_recursive(collection, functions);
            extract_aggregate_functions_recursive(index, functions);
        }
        Expression::Range {
            collection,
            start,
            end,
        } => {
            extract_aggregate_functions_recursive(collection, functions);
            if let Some(s) = start {
                extract_aggregate_functions_recursive(s, functions);
            }
            if let Some(e) = end {
                extract_aggregate_functions_recursive(e, functions);
            }
        }
        Expression::Path(items) => {
            for item in items {
                extract_aggregate_functions_recursive(item, functions);
            }
        }
        Expression::ListComprehension {
            source,
            filter,
            map,
            ..
        } => {
            extract_aggregate_functions_recursive(source, functions);
            if let Some(f) = filter {
                extract_aggregate_functions_recursive(f, functions);
            }
            if let Some(m) = map {
                extract_aggregate_functions_recursive(m, functions);
            }
        }
        Expression::Property { object, .. } => {
            extract_aggregate_functions_recursive(object, functions);
        }
        _ => {}
    }
}
