//! 表达式工具类
//!
//! 提供表达式分析和转换的实用函数，类似于 nebula-graph 的 ExpressionUtils

use crate::core::types::expression::Expression;
use crate::core::types::operators::BinaryOperator;
use std::collections::HashSet;

/// 表达式工具类
///
/// 提供表达式分析、分割、重写等实用函数
pub struct ExpressionUtils;

impl ExpressionUtils {
    /// 检查是否为单步边属性表达式
    ///
    /// # 参数
    /// * `edge_alias` - 边别名
    /// * `expr` - 要检查的表达式
    ///
    /// # 返回值
    /// 如果表达式是单步边属性表达式，返回 true
    pub fn is_one_step_edge_prop(edge_alias: &str, expr: &Expression) -> bool {
        match expr {
            Expression::EdgeProperty { edge, .. } => {
                edge == edge_alias
            }
            _ => false,
        }
    }

    /// 检查是否为单步顶点属性表达式
    ///
    /// # 参数
    /// * `tag` - 标签名
    /// * `expr` - 要检查的表达式
    ///
    /// # 返回值
    /// 如果表达式是单步顶点属性表达式，返回 true
    pub fn is_one_step_tag_prop(tag: &str, expr: &Expression) -> bool {
        match expr {
            Expression::TagProperty { tag: t, .. } => {
                t == tag
            }
            _ => false,
        }
    }

    /// 分割过滤条件
    ///
    /// 将过滤条件分割为满足 picker 条件的部分和不满足的部分
    ///
    /// # 参数
    /// * `filter` - 要分割的过滤条件
    /// * `picker` - 选择器函数，决定表达式是否应该被选中
    ///
    /// # 返回值
    /// 返回一个元组：(选中的表达式, 未选中的表达式)
    pub fn split_filter(
        filter: &Expression,
        picker: impl Fn(&Expression) -> bool,
    ) -> (Option<Expression>, Option<Expression>) {
        let mut picked_exprs = Vec::new();
        let mut unpicked_exprs = Vec::new();

        Self::split_filter_recursive(filter, &picker, &mut picked_exprs, &mut unpicked_exprs);

        let picked = if picked_exprs.is_empty() {
            None
        } else {
            Some(Self::and_all(picked_exprs))
        };

        let unpicked = if unpicked_exprs.is_empty() {
            None
        } else {
            Some(Self::and_all(unpicked_exprs))
        };

        (picked, unpicked)
    }

    /// 递归分割过滤条件
    fn split_filter_recursive(
        expr: &Expression,
        picker: &impl Fn(&Expression) -> bool,
        picked: &mut Vec<Expression>,
        unpicked: &mut Vec<Expression>,
    ) {
        match expr {
            Expression::Binary {
                left,
                op: BinaryOperator::And,
                right,
            } => {
                // 对于 AND 表达式，递归分割左右两边
                Self::split_filter_recursive(left, picker, picked, unpicked);
                Self::split_filter_recursive(right, picker, picked, unpicked);
            }
            _ => {
                // 对于其他表达式，使用 picker 决定
                if picker(expr) {
                    picked.push(expr.clone());
                } else {
                    unpicked.push(expr.clone());
                }
            }
        }
    }

    /// 重写边属性过滤条件
    ///
    /// 将属性表达式重写为边属性表达式
    ///
    /// # 参数
    /// * `edge_alias` - 边别名
    /// * `filter` - 要重写的过滤条件
    ///
    /// # 返回值
    /// 返回重写后的表达式
    pub fn rewrite_edge_property_filter(
        edge_alias: &str,
        filter: Expression,
    ) -> Expression {
        match filter {
            Expression::Property { object, property } => {
                // 检查是否是边属性
                if let Expression::Variable(var_name) = *object {
                    if var_name == edge_alias {
                        // 重写为边属性表达式
                        return Expression::EdgeProperty {
                            edge: edge_alias.to_string(),
                            prop: property,
                        };
                    }
                }
                Expression::Property { object, property }
            }
            Expression::Binary { left, op, right } => {
                // 递归重写左右两边
                Expression::Binary {
                    left: Box::new(Self::rewrite_edge_property_filter(edge_alias, *left)),
                    op,
                    right: Box::new(Self::rewrite_edge_property_filter(edge_alias, *right)),
                }
            }
            Expression::Unary { op, operand } => {
                // 递归重写操作数
                Expression::Unary {
                    op,
                    operand: Box::new(Self::rewrite_edge_property_filter(edge_alias, *operand)),
                }
            }
            _ => filter,
        }
    }

    /// 重写顶点属性过滤条件
    ///
    /// 将属性表达式重写为顶点属性表达式
    ///
    /// # 参数
    /// * `tag` - 标签名
    /// * `filter` - 要重写的过滤条件
    ///
    /// # 返回值
    /// 返回重写后的表达式
    pub fn rewrite_tag_property_filter(tag: &str, filter: Expression) -> Expression {
        match filter {
            Expression::Property { object, property } => {
                // 检查是否是顶点属性
                if let Expression::Variable(var_name) = *object {
                    if var_name == tag {
                        // 重写为顶点属性表达式
                        return Expression::TagProperty {
                            tag: tag.to_string(),
                            prop: property,
                        };
                    }
                }
                Expression::Property { object, property }
            }
            Expression::Binary { left, op, right } => {
                // 递归重写左右两边
                Expression::Binary {
                    left: Box::new(Self::rewrite_tag_property_filter(tag, *left)),
                    op,
                    right: Box::new(Self::rewrite_tag_property_filter(tag, *right)),
                }
            }
            Expression::Unary { op, operand } => {
                // 递归重写操作数
                Expression::Unary {
                    op,
                    operand: Box::new(Self::rewrite_tag_property_filter(tag, *operand)),
                }
            }
            _ => filter,
        }
    }

    /// 检查表达式是否包含特定类型的表达式
    ///
    /// # 参数
    /// * `expr` - 要检查的表达式
    /// * `kinds` - 要查找的表达式类型集合
    ///
    /// # 返回值
    /// 如果表达式包含任何指定类型的表达式，返回 true
    pub fn has_any(expr: &Expression, kinds: &[&str]) -> bool {
        match expr {
            Expression::Binary { left, right, .. } => {
                kinds.contains(&expr.type_name()) ||
                Self::has_any(left, kinds) ||
                Self::has_any(right, kinds)
            }
            Expression::Unary { operand, .. } => {
                kinds.contains(&expr.type_name()) ||
                Self::has_any(operand, kinds)
            }
            Expression::Function { args, .. } => {
                kinds.contains(&expr.type_name()) ||
                args.iter().any(|a| Self::has_any(a, kinds))
            }
            Expression::List(items) => {
                kinds.contains(&expr.type_name()) ||
                items.iter().any(|a| Self::has_any(a, kinds))
            }
            Expression::Map(pairs) => {
                kinds.contains(&expr.type_name()) ||
                pairs.iter().any(|(_, v)| Self::has_any(v, kinds))
            }
            Expression::Case { conditions, default, .. } => {
                kinds.contains(&expr.type_name()) ||
                conditions.iter().any(|(c, _)| Self::has_any(c, kinds)) ||
                conditions.iter().any(|(_, e)| Self::has_any(e, kinds)) ||
                default.as_ref().map_or(false, |d| Self::has_any(d, kinds))
            }
            _ => kinds.contains(&expr.type_name()),
        }
    }

    /// 收集表达式中的所有特定类型表达式
    ///
    /// # 参数
    /// * `expr` - 要搜索的表达式
    /// * `kinds` - 要收集的表达式类型集合
    ///
    /// # 返回值
    /// 返回所有匹配的表达式
    pub fn collect_all(expr: &Expression, kinds: &[&str]) -> Vec<Expression> {
        let mut results = Vec::new();
        Self::collect_all_recursive(expr, kinds, &mut results);
        results
    }

    /// 递归收集表达式
    fn collect_all_recursive(
        expr: &Expression,
        kinds: &[&str],
        results: &mut Vec<Expression>,
    ) {
        if kinds.contains(&expr.type_name()) {
            results.push(expr.clone());
        }

        match expr {
            Expression::Binary { left, right, .. } => {
                Self::collect_all_recursive(left, kinds, results);
                Self::collect_all_recursive(right, kinds, results);
            }
            Expression::Unary { operand, .. } => {
                Self::collect_all_recursive(operand, kinds, results);
            }
            Expression::Function { args, .. } => {
                for arg in args {
                    Self::collect_all_recursive(arg, kinds, results);
                }
            }
            Expression::List(items) => {
                for item in items {
                    Self::collect_all_recursive(item, kinds, results);
                }
            }
            Expression::Map(pairs) => {
                for (_, value) in pairs {
                    Self::collect_all_recursive(value, kinds, results);
                }
            }
            Expression::Case { conditions, default, .. } => {
                for (condition, expr) in conditions {
                    Self::collect_all_recursive(condition, kinds, results);
                    Self::collect_all_recursive(expr, kinds, results);
                }
                if let Some(default_expr) = default {
                    Self::collect_all_recursive(default_expr, kinds, results);
                }
            }
            _ => {}
        }
    }

    /// 收集表达式中的所有变量
    ///
    /// # 参数
    /// * `expr` - 要搜索的表达式
    ///
    /// # 返回值
    /// 返回所有变量名
    pub fn collect_variables(expr: &Expression) -> Vec<String> {
        let mut variables = Vec::new();
        Self::collect_variables_recursive(expr, &mut variables);
        variables
    }

    /// 递归收集变量
    fn collect_variables_recursive(expr: &Expression, variables: &mut Vec<String>) {
        match expr {
            Expression::Variable(name) => {
                if !variables.contains(name) {
                    variables.push(name.clone());
                }
            }
            Expression::Binary { left, right, .. } => {
                Self::collect_variables_recursive(left, variables);
                Self::collect_variables_recursive(right, variables);
            }
            Expression::Unary { operand, .. } => {
                Self::collect_variables_recursive(operand, variables);
            }
            Expression::Function { args, .. } => {
                for arg in args {
                    Self::collect_variables_recursive(arg, variables);
                }
            }
            Expression::List(items) => {
                for item in items {
                    Self::collect_variables_recursive(item, variables);
                }
            }
            Expression::Map(pairs) => {
                for (_, value) in pairs {
                    Self::collect_variables_recursive(value, variables);
                }
            }
            Expression::Case { conditions, default, .. } => {
                for (condition, expr) in conditions {
                    Self::collect_variables_recursive(condition, variables);
                    Self::collect_variables_recursive(expr, variables);
                }
                if let Some(default_expr) = default {
                    Self::collect_variables_recursive(default_expr, variables);
                }
            }
            _ => {}
        }
    }

    /// 收集表达式中的所有属性
    ///
    /// # 参数
    /// * `expr` - 要搜索的表达式
    ///
    /// # 返回值
    /// 返回所有属性名
    pub fn collect_properties(expr: &Expression) -> Vec<String> {
        let mut properties = Vec::new();
        Self::collect_properties_recursive(expr, &mut properties);
        properties
    }

    /// 递归收集属性
    fn collect_properties_recursive(expr: &Expression, properties: &mut Vec<String>) {
        match expr {
            Expression::Property { property, .. } => {
                if !properties.contains(property) {
                    properties.push(property.clone());
                }
            }
            Expression::TagProperty { prop, .. } => {
                if !properties.contains(prop) {
                    properties.push(prop.clone());
                }
            }
            Expression::EdgeProperty { prop, .. } => {
                if !properties.contains(prop) {
                    properties.push(prop.clone());
                }
            }
            Expression::InputProperty(prop) => {
                if !properties.contains(prop) {
                    properties.push(prop.clone());
                }
            }
            Expression::VariableProperty { prop, .. } => {
                if !properties.contains(prop) {
                    properties.push(prop.clone());
                }
            }
            Expression::SourceProperty { prop, .. } => {
                if !properties.contains(prop) {
                    properties.push(prop.clone());
                }
            }
            Expression::DestinationProperty { prop, .. } => {
                if !properties.contains(prop) {
                    properties.push(prop.clone());
                }
            }
            Expression::Binary { left, right, .. } => {
                Self::collect_properties_recursive(left, properties);
                Self::collect_properties_recursive(right, properties);
            }
            Expression::Unary { operand, .. } => {
                Self::collect_properties_recursive(operand, properties);
            }
            Expression::Function { args, .. } => {
                for arg in args {
                    Self::collect_properties_recursive(arg, properties);
                }
            }
            Expression::List(items) => {
                for item in items {
                    Self::collect_properties_recursive(item, properties);
                }
            }
            Expression::Map(pairs) => {
                for (_, value) in pairs {
                    Self::collect_properties_recursive(value, properties);
                }
            }
            Expression::Case { conditions, default, .. } => {
                for (condition, expr) in conditions {
                    Self::collect_properties_recursive(condition, properties);
                    Self::collect_properties_recursive(expr, properties);
                }
                if let Some(default_expr) = default {
                    Self::collect_properties_recursive(default_expr, properties);
                }
            }
            _ => {}
        }
    }

    /// 将多个表达式用 AND 连接
    ///
    /// # 参数
    /// * `exprs` - 要连接的表达式列表
    ///
    /// # 返回值
    /// 返回用 AND 连接的表达式
    pub fn and_all(mut exprs: Vec<Expression>) -> Expression {
        match exprs.len() {
            0 => Expression::Literal(crate::core::Value::Bool(true)),
            1 => exprs.pop().expect("Should have one element"),
            _ => {
                let mut result = exprs.pop().expect("Should have elements");
                while let Some(expr) = exprs.pop() {
                    result = Expression::Binary {
                        left: Box::new(expr),
                        op: BinaryOperator::And,
                        right: Box::new(result),
                    };
                }
                result
            }
        }
    }

    /// 将多个表达式用 OR 连接
    ///
    /// # 参数
    /// * `exprs` - 要连接的表达式列表
    ///
    /// # 返回值
    /// 返回用 OR 连接的表达式
    pub fn or_all(mut exprs: Vec<Expression>) -> Expression {
        match exprs.len() {
            0 => Expression::Literal(crate::core::Value::Bool(false)),
            1 => exprs.pop().expect("Should have one element"),
            _ => {
                let mut result = exprs.pop().expect("Should have elements");
                while let Some(expr) = exprs.pop() {
                    result = Expression::Binary {
                        left: Box::new(expr),
                        op: BinaryOperator::Or,
                        right: Box::new(result),
                    };
                }
                result
            }
        }
    }

    /// 检查表达式是否为常量表达式
    ///
    /// # 参数
    /// * `expr` - 要检查的表达式
    ///
    /// # 返回值
    /// 如果表达式是常量表达式，返回 true
    pub fn is_constant(expr: &Expression) -> bool {
        matches!(expr, Expression::Literal(_))
    }

    /// 检查表达式是否为变量表达式
    ///
    /// # 参数
    /// * `expr` - 要检查的表达式
    ///
    /// # 返回值
    /// 如果表达式是变量表达式，返回 true
    pub fn is_variable(expr: &Expression) -> bool {
        matches!(expr, Expression::Variable(_))
    }

    /// 检查表达式是否为属性表达式
    ///
    /// # 参数
    /// * `expr` - 要检查的表达式
    ///
    /// # 返回值
    /// 如果表达式是属性表达式，返回 true
    pub fn is_property(expr: &Expression) -> bool {
        matches!(
            expr,
            Expression::Property { .. }
                | Expression::TagProperty { .. }
                | Expression::EdgeProperty { .. }
                | Expression::InputProperty(_)
                | Expression::VariableProperty { .. }
                | Expression::SourceProperty { .. }
                | Expression::DestinationProperty { .. }
        )
    }

    /// 检查表达式是否为比较表达式
    ///
    /// # 参数
    /// * `expr` - 要检查的表达式
    ///
    /// # 返回值
    /// 如果表达式是比较表达式，返回 true
    pub fn is_comparison(expr: &Expression) -> bool {
        match expr {
            Expression::Binary { op, .. } => {
                matches!(
                    op,
                    BinaryOperator::Equal
                        | BinaryOperator::NotEqual
                        | BinaryOperator::LessThan
                        | BinaryOperator::LessThanOrEqual
                        | BinaryOperator::GreaterThan
                        | BinaryOperator::GreaterThanOrEqual
                )
            }
            _ => false,
        }
    }

    /// 检查表达式是否为逻辑表达式
    ///
    /// # 参数
    /// * `expr` - 要检查的表达式
    ///
    /// # 返回值
    /// 如果表达式是逻辑表达式，返回 true
    pub fn is_logical(expr: &Expression) -> bool {
        match expr {
            Expression::Binary { op, .. } => {
                matches!(op, BinaryOperator::And | BinaryOperator::Or | BinaryOperator::Xor)
            }
            Expression::UnaryNot(_) => true,
            _ => false,
        }
    }

    /// 检查表达式是否为算术表达式
    ///
    /// # 参数
    /// * `expr` - 要检查的表达式
    ///
    /// # 返回值
    /// 如果表达式是算术表达式，返回 true
    pub fn is_arithmetic(expr: &Expression) -> bool {
        match expr {
            Expression::Binary { op, .. } => {
                matches!(
                    op,
                    BinaryOperator::Add
                        | BinaryOperator::Subtract
                        | BinaryOperator::Multiply
                        | BinaryOperator::Divide
                        | BinaryOperator::Modulo
                )
            }
            Expression::UnaryPlus(_) | Expression::UnaryNegate(_) => true,
            _ => false,
        }
    }

    /// 简化表达式
    ///
    /// 对表达式进行常量折叠和其他简化
    ///
    /// # 参数
    /// * `expr` - 要简化的表达式
    ///
    /// # 返回值
    /// 返回简化后的表达式
    pub fn simplify(expr: &Expression) -> Expression {
        match expr {
            Expression::Binary { left, op, right } => {
                let left_simplified = Self::simplify(left);
                let right_simplified = Self::simplify(right);

                // 尝试常量折叠
                if Self::is_constant(&left_simplified) && Self::is_constant(&right_simplified) {
                    if let Ok(result) = Self::evaluate_binary(&left_simplified, *op, &right_simplified) {
                        return Expression::Literal(result);
                    }
                }

                Expression::Binary {
                    left: Box::new(left_simplified),
                    op: *op,
                    right: Box::new(right_simplified),
                }
            }
            Expression::Unary { op, operand } => {
                let operand_simplified = Self::simplify(operand);

                // 尝试常量折叠
                if Self::is_constant(&operand_simplified) {
                    if let Ok(result) = Self::evaluate_unary(*op, &operand_simplified) {
                        return Expression::Literal(result);
                    }
                }

                Expression::Unary {
                    op: *op,
                    operand: Box::new(operand_simplified),
                }
            }
            _ => expr.clone(),
        }
    }

    /// 评估二元表达式
    fn evaluate_binary(
        _left: &Expression,
        _op: BinaryOperator,
        _right: &Expression,
    ) -> Result<crate::core::Value, String> {
        // 简化实现：不支持表达式评估
        Err("表达式评估功能尚未实现".to_string())
    }

    /// 评估一元表达式
    fn evaluate_unary(
        _op: crate::core::types::operators::UnaryOperator,
        _operand: &Expression,
    ) -> Result<crate::core::Value, String> {
        // 简化实现：不支持表达式评估
        Err("表达式评估功能尚未实现".to_string())
    }

    /// 获取常量值
    fn get_constant_value(expr: &Expression) -> Result<crate::core::Value, String> {
        match expr {
            Expression::Literal(val) => Ok(val.clone()),
            _ => Err("Expression is not a constant".to_string()),
        }
    }
}

impl Expression {
    /// 获取表达式类型名称
    pub fn type_name(&self) -> &'static str {
        match self {
            Expression::Literal(_) => "Literal",
            Expression::Variable(_) => "Variable",
            Expression::Property { .. } => "Property",
            Expression::Binary { .. } => "Binary",
            Expression::Unary { .. } => "Unary",
            Expression::Function { .. } => "Function",
            Expression::Aggregate { .. } => "Aggregate",
            Expression::List(_) => "List",
            Expression::Map(_) => "Map",
            Expression::Case { .. } => "Case",
            Expression::TypeCast { .. } => "TypeCast",
            Expression::Subscript { .. } => "Subscript",
            Expression::Range { .. } => "Range",
            Expression::Path(_) => "Path",
            Expression::Label(_) => "Label",
            Expression::TagProperty { .. } => "TagProperty",
            Expression::EdgeProperty { .. } => "EdgeProperty",
            Expression::InputProperty(_) => "InputProperty",
            Expression::VariableProperty { .. } => "VariableProperty",
            Expression::SourceProperty { .. } => "SourceProperty",
            Expression::DestinationProperty { .. } => "DestinationProperty",
            Expression::UnaryPlus(_) => "UnaryPlus",
            Expression::UnaryNegate(_) => "UnaryNegate",
            Expression::UnaryNot(_) => "UnaryNot",
            Expression::UnaryIncr(_) => "UnaryIncr",
            Expression::UnaryDecr(_) => "UnaryDecr",
            Expression::IsNull(_) => "IsNull",
            Expression::IsNotNull(_) => "IsNotNull",
            Expression::IsEmpty(_) => "IsEmpty",
            Expression::IsNotEmpty(_) => "IsNotEmpty",
            Expression::ListComprehension { .. } => "ListComprehension",
            Expression::Predicate { .. } => "Predicate",
            Expression::Reduce { .. } => "Reduce",
            Expression::ESQuery(_) => "ESQuery",
            Expression::UUID => "UUID",
            Expression::MatchPathPattern { .. } => "MatchPathPattern",
        }
    }

    /// 获取子表达式
    pub fn children(&self) -> Vec<&Expression> {
        match self {
            Expression::Binary { left, right, .. } => vec![left, right],
            Expression::Unary { operand, .. } => vec![operand],
            Expression::Function { args, .. } => args.iter().collect(),
            Expression::List(items) => items.iter().collect(),
            Expression::Map(pairs) => pairs.iter().map(|(_, v)| v).collect(),
            Expression::Case { conditions, default, .. } => {
                let mut children: Vec<&Expression> = Vec::new();
                for (c, e) in conditions {
                    children.push(c);
                    children.push(e);
                }
                if let Some(d) = default {
                    children.push(d);
                }
                children
            }
            Expression::TypeCast { expr, .. } => vec![expr],
            Expression::Subscript { collection, index, .. } => vec![collection, index],
            Expression::Range { collection, start, end, .. } => {
                let mut children = vec![collection];
                if let Some(s) = start {
                    children.push(s);
                }
                if let Some(e) = end {
                    children.push(e);
                }
                children
            }
            Expression::Path(items) => items.iter().collect(),
            Expression::ListComprehension { generator, condition, .. } => {
                let mut children = vec![generator];
                if let Some(c) = condition {
                    children.push(c);
                }
                children
            }
            Expression::Predicate { list, condition, .. } => vec![list, condition],
            Expression::Reduce { list, initial, expr, .. } => vec![list, initial, expr],
            Expression::UnaryPlus(expr) => vec![expr],
            Expression::UnaryNegate(expr) => vec![expr],
            Expression::UnaryNot(expr) => vec![expr],
            Expression::UnaryIncr(expr) => vec![expr],
            Expression::UnaryDecr(expr) => vec![expr],
            Expression::IsNull(expr) => vec![expr],
            Expression::IsNotNull(expr) => vec![expr],
            Expression::IsEmpty(expr) => vec![expr],
            Expression::IsNotEmpty(expr) => vec![expr],
            _ => Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Value;

    #[test]
    fn test_is_one_step_edge_prop() {
        let expr = Expression::EdgeProperty {
            edge: "e".to_string(),
            prop: "name".to_string(),
        };
        assert!(ExpressionUtils::is_one_step_edge_prop("e", &expr));
        assert!(!ExpressionUtils::is_one_step_edge_prop("e2", &expr));
    }

    #[test]
    fn test_split_filter() {
        let expr = Expression::Binary {
            left: Box::new(Expression::EdgeProperty {
                edge: "e".to_string(),
                prop: "name".to_string(),
            }),
            op: BinaryOperator::And,
            right: Box::new(Expression::Variable("x".to_string())),
        };

        let (picked, unpicked) = ExpressionUtils::split_filter(&expr, |e| {
            ExpressionUtils::is_one_step_edge_prop("e", e)
        });

        assert!(picked.is_some());
        assert!(unpicked.is_some());
    }

    #[test]
    fn test_collect_variables() {
        let expr = Expression::Binary {
            left: Box::new(Expression::Variable("x".to_string())),
            op: BinaryOperator::Equal,
            right: Box::new(Expression::Variable("y".to_string())),
        };

        let vars = ExpressionUtils::collect_variables(&expr);
        assert_eq!(vars.len(), 2);
        assert!(vars.contains(&"x".to_string()));
        assert!(vars.contains(&"y".to_string()));
    }

    #[test]
    fn test_simplify() {
        let expr = Expression::Binary {
            left: Box::new(Expression::Literal(Value::Int(1))),
            op: BinaryOperator::Add,
            right: Box::new(Expression::Literal(Value::Int(2))),
        };

        let simplified = ExpressionUtils::simplify(&expr);
        assert_eq!(simplified, Expression::Literal(Value::Int(3)));
    }

    #[test]
    fn test_and_all() {
        let exprs = vec![
            Expression::Literal(Value::Bool(true)),
            Expression::Literal(Value::Bool(true)),
        ];

        let result = ExpressionUtils::and_all(exprs);
        assert!(matches!(result, Expression::Binary { .. }));
    }

    #[test]
    fn test_or_all() {
        let exprs = vec![
            Expression::Literal(Value::Bool(false)),
            Expression::Literal(Value::Bool(true)),
        ];

        let result = ExpressionUtils::or_all(exprs);
        assert!(matches!(result, Expression::Binary { .. }));
    }
}
