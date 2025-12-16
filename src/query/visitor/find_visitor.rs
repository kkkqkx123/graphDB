//! FindVisitor - 用于查找表达式中特定类型子表达式的访问器
//! 对应 NebulaGraph FindVisitor.h/.cpp 的功能

use crate::graph::expression::Expression;
use std::collections::HashSet;

pub struct FindVisitor {
    /// 要查找的表达式类型集合
    target_types: HashSet<ExpressionType>,
    /// 找到的表达式列表
    found_exprs: Vec<Expression>,
}

/// 表达式类型枚举，用于标识不同类型的表达式
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ExpressionType {
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
    TagProperty,
    EdgeProperty,
    InputProperty,
    VariableProperty,
    SourceProperty,
    DestinationProperty,
}

impl FindVisitor {
    pub fn new() -> Self {
        Self {
            target_types: HashSet::new(),
            found_exprs: Vec::new(),
        }
    }

    /// 设置要查找的表达式类型
    pub fn set_target_types(&mut self, types: Vec<ExpressionType>) -> &mut Self {
        self.target_types.clear();
        for expr_type in types {
            self.target_types.insert(expr_type);
        }
        self
    }

    /// 添加要查找的表达式类型
    pub fn add_target_type(&mut self, expr_type: ExpressionType) -> &mut Self {
        self.target_types.insert(expr_type);
        self
    }

    /// 搜索表达式中匹配类型的所有子表达式
    pub fn find(&mut self, expr: &Expression) -> Vec<Expression> {
        self.found_exprs.clear();
        self.visit(expr);
        self.found_exprs.clone()
    }

    /// 检查表达式中是否存在匹配类型的子表达式
    pub fn exist(&mut self, expr: &Expression) -> bool {
        self.found_exprs.clear();
        self.visit(expr);
        !self.found_exprs.is_empty()
    }

    fn visit(&mut self, expr: &Expression) {
        // 检查当前表达式是否匹配目标类型
        if self.target_types.contains(&Self::get_expression_type(expr)) {
            self.found_exprs.push(expr.clone());
        }

        // 递归访问子表达式
        self.visit_children(expr);
    }

    fn visit_children(&mut self, expr: &Expression) {
        match expr {
            Expression::Literal(_) => {}
            Expression::Variable(_) => {}
            Expression::Property { object, .. } => {
                self.visit(object);
            }
            Expression::Binary { left, right, .. } => {
                self.visit(left);
                self.visit(right);
            }
            Expression::Unary { operand, .. } => {
                self.visit(operand);
            }
            Expression::Function { args, .. } => {
                for arg in args {
                    self.visit(arg);
                }
            }
            Expression::Aggregate { arg, .. } => {
                self.visit(arg);
            }
            Expression::List(elements) => {
                for elem in elements {
                    self.visit(elem);
                }
            }
            Expression::Map(pairs) => {
                for (_, value) in pairs {
                    self.visit(value);
                }
            }
            Expression::Case {
                conditions,
                default,
            } => {
                for (condition, value) in conditions {
                    self.visit(condition);
                    self.visit(value);
                }
                if let Some(default_expr) = default {
                    self.visit(default_expr);
                }
            }
            Expression::TypeCast { expr, .. } => {
                self.visit(expr);
            }
            Expression::Subscript { collection, index } => {
                self.visit(collection);
                self.visit(index);
            }
            Expression::Range {
                collection,
                start,
                end,
            } => {
                self.visit(collection);
                if let Some(start_expr) = start {
                    self.visit(start_expr);
                }
                if let Some(end_expr) = end {
                    self.visit(end_expr);
                }
            }
            Expression::Path(elements) => {
                for elem in elements {
                    self.visit(elem);
                }
            }
            Expression::Label(_) => {}
            Expression::TagProperty { .. } => {}
            Expression::EdgeProperty { .. } => {}
            Expression::InputProperty(_) => {}
            Expression::VariableProperty { .. } => {}
            Expression::SourceProperty { .. } => {}
            Expression::DestinationProperty { .. } => {}

            // 新增表达式类型的处理
            Expression::UnaryPlus(expr)
            | Expression::UnaryNegate(expr)
            | Expression::UnaryNot(expr)
            | Expression::UnaryIncr(expr)
            | Expression::UnaryDecr(expr)
            | Expression::IsNull(expr)
            | Expression::IsNotNull(expr)
            | Expression::IsEmpty(expr)
            | Expression::IsNotEmpty(expr) => {
                self.visit(expr);
            }
            Expression::TypeCasting { expr, .. } => {
                self.visit(expr);
            }
            Expression::ListComprehension {
                generator,
                condition,
            } => {
                self.visit(generator);
                if let Some(cond) = condition {
                    self.visit(cond);
                }
            }
            Expression::Predicate { list, condition } => {
                self.visit(list);
                self.visit(condition);
            }
            Expression::Reduce {
                list,
                initial,
                expr,
                ..
            } => {
                self.visit(list);
                self.visit(initial);
                self.visit(expr);
            }
            Expression::PathBuild(elements) => {
                for elem in elements {
                    self.visit(elem);
                }
            }
            Expression::ESQuery(_) => {}
            Expression::UUID => {}
            Expression::SubscriptRange {
                collection,
                start,
                end,
            } => {
                self.visit(collection);
                if let Some(start_expr) = start {
                    self.visit(start_expr);
                }
                if let Some(end_expr) = end {
                    self.visit(end_expr);
                }
            }
            Expression::MatchPathPattern { patterns, .. } => {
                for pattern in patterns {
                    self.visit(pattern);
                }
            }
        }
    }

    /// 获取表达式的类型
    fn get_expression_type(expr: &Expression) -> ExpressionType {
        match expr {
            Expression::Literal(_) => ExpressionType::Literal,
            Expression::Variable(_) => ExpressionType::Variable,
            Expression::Property { .. } => ExpressionType::Property,
            Expression::Binary { .. } => ExpressionType::Binary,
            Expression::Unary { .. } => ExpressionType::Unary,
            Expression::Function { .. } => ExpressionType::Function,
            Expression::Aggregate { .. } => ExpressionType::Aggregate,
            Expression::List(_) => ExpressionType::List,
            Expression::Map(_) => ExpressionType::Map,
            Expression::Case { .. } => ExpressionType::Case,
            Expression::TypeCast { .. } => ExpressionType::TypeCast,
            Expression::Subscript { .. } => ExpressionType::Subscript,
            Expression::Range { .. } => ExpressionType::Range,
            Expression::Path(_) => ExpressionType::Path,
            Expression::Label(_) => ExpressionType::Label,
            Expression::TagProperty { .. } => ExpressionType::TagProperty,
            Expression::EdgeProperty { .. } => ExpressionType::EdgeProperty,
            Expression::InputProperty(_) => ExpressionType::InputProperty,
            Expression::VariableProperty { .. } => ExpressionType::VariableProperty,
            Expression::SourceProperty { .. } => ExpressionType::SourceProperty,
            Expression::DestinationProperty { .. } => ExpressionType::DestinationProperty,

            // 新增表达式类型的处理
            Expression::UnaryPlus(_)
            | Expression::UnaryNegate(_)
            | Expression::UnaryNot(_)
            | Expression::UnaryIncr(_)
            | Expression::UnaryDecr(_)
            | Expression::IsNull(_)
            | Expression::IsNotNull(_)
            | Expression::IsEmpty(_)
            | Expression::IsNotEmpty(_) => ExpressionType::Unary,
            Expression::TypeCasting { .. } => ExpressionType::TypeCast,
            Expression::ListComprehension { .. } => ExpressionType::List,
            Expression::Predicate { .. } => ExpressionType::Property,
            Expression::Reduce { .. } => ExpressionType::Aggregate,
            Expression::PathBuild(_) => ExpressionType::Path,
            Expression::ESQuery(_) => ExpressionType::Function,
            Expression::UUID => ExpressionType::Literal,
            Expression::SubscriptRange { .. } => ExpressionType::Subscript,
            Expression::MatchPathPattern { .. } => ExpressionType::Path,
        }
    }

    /// 搜索表达式中匹配特定条件的子表达式
    pub fn find_if<F>(&mut self, expr: &Expression, predicate: F) -> Vec<Expression>
    where
        F: Fn(&Expression) -> bool,
    {
        let mut results = Vec::new();
        self.visit_with_predicate(expr, &predicate, &mut results);
        results
    }

    fn visit_with_predicate<F>(
        &self,
        expr: &Expression,
        predicate: &F,
        results: &mut Vec<Expression>,
    ) where
        F: Fn(&Expression) -> bool,
    {
        if predicate(expr) {
            results.push(expr.clone());
        }

        // 递归访问子表达式
        match expr {
            Expression::Literal(_) => {}
            Expression::Variable(_) => {}
            Expression::Property { object, .. } => {
                self.visit_with_predicate(object, predicate, results);
            }
            Expression::Binary { left, right, .. } => {
                self.visit_with_predicate(left, predicate, results);
                self.visit_with_predicate(right, predicate, results);
            }
            Expression::Unary { operand, .. } => {
                self.visit_with_predicate(operand, predicate, results);
            }
            Expression::Function { args, .. } => {
                for arg in args {
                    self.visit_with_predicate(arg, predicate, results);
                }
            }
            Expression::Aggregate { arg, .. } => {
                self.visit_with_predicate(arg, predicate, results);
            }
            Expression::List(elements) => {
                for elem in elements {
                    self.visit_with_predicate(elem, predicate, results);
                }
            }
            Expression::Map(pairs) => {
                for (_, value) in pairs {
                    self.visit_with_predicate(value, predicate, results);
                }
            }
            Expression::Case {
                conditions,
                default,
            } => {
                for (condition, value) in conditions {
                    self.visit_with_predicate(condition, predicate, results);
                    self.visit_with_predicate(value, predicate, results);
                }
                if let Some(default_expr) = default {
                    self.visit_with_predicate(default_expr, predicate, results);
                }
            }
            Expression::TypeCast { expr, .. } => {
                self.visit_with_predicate(expr, predicate, results);
            }
            Expression::Subscript { collection, index } => {
                self.visit_with_predicate(collection, predicate, results);
                self.visit_with_predicate(index, predicate, results);
            }
            Expression::Range {
                collection,
                start,
                end,
            } => {
                self.visit_with_predicate(collection, predicate, results);
                if let Some(start_expr) = start {
                    self.visit_with_predicate(start_expr, predicate, results);
                }
                if let Some(end_expr) = end {
                    self.visit_with_predicate(end_expr, predicate, results);
                }
            }
            Expression::Path(elements) => {
                for elem in elements {
                    self.visit_with_predicate(elem, predicate, results);
                }
            }
            Expression::Label(_) => {}
            Expression::TagProperty { .. } => {}
            Expression::EdgeProperty { .. } => {}
            Expression::InputProperty(_) => {}
            Expression::VariableProperty { .. } => {}
            Expression::SourceProperty { .. } => {}
            Expression::DestinationProperty { .. } => {}

            // 新增表达式类型的处理
            Expression::UnaryPlus(expr)
            | Expression::UnaryNegate(expr)
            | Expression::UnaryNot(expr)
            | Expression::UnaryIncr(expr)
            | Expression::UnaryDecr(expr)
            | Expression::IsNull(expr)
            | Expression::IsNotNull(expr)
            | Expression::IsEmpty(expr)
            | Expression::IsNotEmpty(expr) => {
                self.visit_with_predicate(expr, predicate, results);
            }
            Expression::TypeCasting { expr, .. } => {
                self.visit_with_predicate(expr, predicate, results);
            }
            Expression::ListComprehension {
                generator,
                condition,
            } => {
                self.visit_with_predicate(generator, predicate, results);
                if let Some(cond) = condition {
                    self.visit_with_predicate(cond, predicate, results);
                }
            }
            Expression::Predicate { list, condition } => {
                self.visit_with_predicate(list, predicate, results);
                self.visit_with_predicate(condition, predicate, results);
            }
            Expression::Reduce {
                list,
                initial,
                expr,
                ..
            } => {
                self.visit_with_predicate(list, predicate, results);
                self.visit_with_predicate(initial, predicate, results);
                self.visit_with_predicate(expr, predicate, results);
            }
            Expression::PathBuild(elements) => {
                for elem in elements {
                    self.visit_with_predicate(elem, predicate, results);
                }
            }
            Expression::ESQuery(_) => {}
            Expression::UUID => {}
            Expression::SubscriptRange {
                collection,
                start,
                end,
            } => {
                self.visit_with_predicate(collection, predicate, results);
                if let Some(start_expr) = start {
                    self.visit_with_predicate(start_expr, predicate, results);
                }
                if let Some(end_expr) = end {
                    self.visit_with_predicate(end_expr, predicate, results);
                }
            }
            Expression::MatchPathPattern { patterns, .. } => {
                for pattern in patterns {
                    self.visit_with_predicate(pattern, predicate, results);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::expression::{BinaryOperator, LiteralValue};

    #[test]
    fn test_find_literals() {
        let mut visitor = FindVisitor::new();

        // 创建一个包含字面量的表达式: 1 + 2 * 3
        let expr = Expression::Binary {
            left: Box::new(Expression::Literal(LiteralValue::Int(1))),
            op: BinaryOperator::Add,
            right: Box::new(Expression::Binary {
                left: Box::new(Expression::Literal(LiteralValue::Int(2))),
                op: BinaryOperator::Multiply,
                right: Box::new(Expression::Literal(LiteralValue::Int(3))),
            }),
        };

        let literals = visitor.add_target_type(ExpressionType::Literal).find(&expr);

        // 应该找到3个字面量
        assert_eq!(literals.len(), 3);
    }

    #[test]
    fn test_find_with_predicate() {
        let mut visitor = FindVisitor::new();

        // 创建一个包含整数字面量的表达式: 1 + 2 * 3
        let expr = Expression::Binary {
            left: Box::new(Expression::Literal(LiteralValue::Int(1))),
            op: BinaryOperator::Add,
            right: Box::new(Expression::Binary {
                left: Box::new(Expression::Literal(LiteralValue::Int(2))),
                op: BinaryOperator::Multiply,
                right: Box::new(Expression::Literal(LiteralValue::Int(3))),
            }),
        };

        let literals = visitor.find_if(&expr, |e| {
            matches!(e, Expression::Literal(LiteralValue::Int(_)))
        });

        // 应该找到3个整数字面量
        assert_eq!(literals.len(), 3);
    }
}
