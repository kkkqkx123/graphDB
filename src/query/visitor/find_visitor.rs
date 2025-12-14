//! FindVisitor - 用于查找表达式中特定类型子表达式的访问器
//! 对应 NebulaGraph FindVisitor.h/.cpp 的功能

use std::collections::HashSet;
use crate::graph::expression::expr_type::{Expression, ExpressionKind};

pub struct FindVisitor {
    /// 要查找的表达式类型集合
    target_kinds: HashSet<ExpressionKind>,
    /// 找到的表达式列表
    found_exprs: Vec<Expression>,
}

impl FindVisitor {
    pub fn new() -> Self {
        Self {
            target_kinds: HashSet::new(),
            found_exprs: Vec::new(),
        }
    }

    /// 设置要查找的表达式类型
    pub fn set_target_kinds(&mut self, kinds: Vec<ExpressionKind>) -> &mut Self {
        self.target_kinds.clear();
        for expr_kind in kinds {
            self.target_kinds.insert(expr_kind);
        }
        self
    }

    /// 添加要查找的表达式类型
    pub fn add_target_kind(&mut self, expr_kind: ExpressionKind) -> &mut Self {
        self.target_kinds.insert(expr_kind);
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
        if self.target_kinds.contains(&expr.kind()) {
            self.found_exprs.push(expr.clone());
        }

        // 递归访问子表达式
        self.visit_children(expr);
    }

    fn visit_children(&mut self, expr: &Expression) {
        match expr {
            Expression::UnaryOp(_, operand) => {
                self.visit(operand);
            },
            Expression::BinaryOp(left, _, right) => {
                self.visit(left);
                self.visit(right);
            },
            Expression::Function(_, args) => {
                for arg in args {
                    self.visit(arg);
                }
            },
            Expression::TagProperty { .. } |
            Expression::EdgeProperty { .. } |
            Expression::InputProperty(_) |
            Expression::VariableProperty { .. } |
            Expression::SourceProperty { .. } |
            Expression::DestinationProperty { .. } |
            Expression::Constant(_) |
            Expression::Property(_) => {
                // 没有子表达式，无需处理
            },
            Expression::UnaryPlus(operand) => {
                self.visit(operand);
            },
            Expression::UnaryNegate(operand) => {
                self.visit(operand);
            },
            Expression::UnaryNot(operand) => {
                self.visit(operand);
            },
            Expression::UnaryIncr(operand) => {
                self.visit(operand);
            },
            Expression::UnaryDecr(operand) => {
                self.visit(operand);
            },
            Expression::IsNull(operand) => {
                self.visit(operand);
            },
            Expression::IsNotNull(operand) => {
                self.visit(operand);
            },
            Expression::IsEmpty(operand) => {
                self.visit(operand);
            },
            Expression::IsNotEmpty(operand) => {
                self.visit(operand);
            },
            Expression::List(items) => {
                for item in items {
                    self.visit(item);
                }
            },
            Expression::Set(items) => {
                for item in items {
                    self.visit(item);
                }
            },
            Expression::Map(items) => {
                for (_, value) in items {
                    self.visit(value);
                }
            },
            Expression::TypeCasting { expr, .. } => {
                self.visit(expr);
            },
            Expression::Case { conditions, default } => {
                for (condition, value) in conditions {
                    self.visit(condition);
                    self.visit(value);
                }
                if let Some(default_expr) = default {
                    self.visit(default_expr);
                }
            },
            Expression::Aggregate { arg, .. } => {
                self.visit(arg);
            },
            Expression::ListComprehension { generator, condition } => {
                self.visit(generator);
                if let Some(condition_expr) = condition {
                    self.visit(condition_expr);
                }
            },
            Expression::Predicate { list, condition } => {
                self.visit(list);
                self.visit(condition);
            },
            Expression::Reduce { list, initial, expr, .. } => {
                self.visit(list);
                self.visit(initial);
                self.visit(expr);
            },
            Expression::PathBuild(items) => {
                for item in items {
                    self.visit(item);
                }
            },
            Expression::ESQuery(_) => {
                // ESQuery has no child expressions
            },
            Expression::UUID => {
                // UUID has no child expressions
            },
            Expression::Variable(_) => {
                // Variable has no child expressions
            },
            Expression::Subscript { collection, index } => {
                self.visit(collection);
                self.visit(index);
            },
            Expression::SubscriptRange { collection, start, end } => {
                self.visit(collection);
                if let Some(start_expr) = start {
                    self.visit(start_expr);
                }
                if let Some(end_expr) = end {
                    self.visit(end_expr);
                }
            },
            Expression::Label(_) => {
                // Label has no child expressions
            },
            Expression::PatternPattern { patterns, .. } => {
                for pattern in patterns {
                    self.visit(pattern);
                }
            },
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

    fn visit_with_predicate<F>(&self, expr: &Expression, predicate: &F, results: &mut Vec<Expression>)
    where
        F: Fn(&Expression) -> bool,
    {
        if predicate(expr) {
            results.push(expr.clone());
        }

        // 递归访问子表达式
        match expr {
            Expression::UnaryOp(_, operand) => {
                self.visit_with_predicate(operand, predicate, results);
            },
            Expression::BinaryOp(left, _, right) => {
                self.visit_with_predicate(left, predicate, results);
                self.visit_with_predicate(right, predicate, results);
            },
            Expression::Function(_, args) => {
                for arg in args {
                    self.visit_with_predicate(arg, predicate, results);
                }
            },
            Expression::TagProperty { .. } |
            Expression::EdgeProperty { .. } |
            Expression::InputProperty(_) |
            Expression::VariableProperty { .. } |
            Expression::SourceProperty { .. } |
            Expression::DestinationProperty { .. } |
            Expression::Constant(_) |
            Expression::Property(_) => {
                // 没有子表达式，无需处理
            },
            Expression::UnaryPlus(operand) => {
                self.visit_with_predicate(operand, predicate, results);
            },
            Expression::UnaryNegate(operand) => {
                self.visit_with_predicate(operand, predicate, results);
            },
            Expression::UnaryNot(operand) => {
                self.visit_with_predicate(operand, predicate, results);
            },
            Expression::UnaryIncr(operand) => {
                self.visit_with_predicate(operand, predicate, results);
            },
            Expression::UnaryDecr(operand) => {
                self.visit_with_predicate(operand, predicate, results);
            },
            Expression::IsNull(operand) => {
                self.visit_with_predicate(operand, predicate, results);
            },
            Expression::IsNotNull(operand) => {
                self.visit_with_predicate(operand, predicate, results);
            },
            Expression::IsEmpty(operand) => {
                self.visit_with_predicate(operand, predicate, results);
            },
            Expression::IsNotEmpty(operand) => {
                self.visit_with_predicate(operand, predicate, results);
            },
            Expression::List(items) => {
                for item in items {
                    self.visit_with_predicate(item, predicate, results);
                }
            },
            Expression::Set(items) => {
                for item in items {
                    self.visit_with_predicate(item, predicate, results);
                }
            },
            Expression::Map(items) => {
                for (_, value) in items {
                    self.visit_with_predicate(value, predicate, results);
                }
            },
            Expression::TypeCasting { expr, .. } => {
                self.visit_with_predicate(expr, predicate, results);
            },
            Expression::Case { conditions, default } => {
                for (condition, value) in conditions {
                    self.visit_with_predicate(condition, predicate, results);
                    self.visit_with_predicate(value, predicate, results);
                }
                if let Some(default_expr) = default {
                    self.visit_with_predicate(default_expr, predicate, results);
                }
            },
            Expression::Aggregate { arg, .. } => {
                self.visit_with_predicate(arg, predicate, results);
            },
            Expression::ListComprehension { generator, condition } => {
                self.visit_with_predicate(generator, predicate, results);
                if let Some(condition_expr) = condition {
                    self.visit_with_predicate(condition_expr, predicate, results);
                }
            },
            Expression::Predicate { list, condition } => {
                self.visit_with_predicate(list, predicate, results);
                self.visit_with_predicate(condition, predicate, results);
            },
            Expression::Reduce { list, initial, expr, .. } => {
                self.visit_with_predicate(list, predicate, results);
                self.visit_with_predicate(initial, predicate, results);
                self.visit_with_predicate(expr, predicate, results);
            },
            Expression::PathBuild(items) => {
                for item in items {
                    self.visit_with_predicate(item, predicate, results);
                }
            },
            Expression::ESQuery(_) => {
                // ESQuery has no child expressions
            },
            Expression::UUID => {
                // UUID has no child expressions
            },
            Expression::Variable(_) => {
                // Variable has no child expressions
            },
            Expression::Subscript { collection, index } => {
                self.visit_with_predicate(collection, predicate, results);
                self.visit_with_predicate(index, predicate, results);
            },
            Expression::SubscriptRange { collection, start, end } => {
                self.visit_with_predicate(collection, predicate, results);
                if let Some(start_expr) = start {
                    self.visit_with_predicate(start_expr, predicate, results);
                }
                if let Some(end_expr) = end {
                    self.visit_with_predicate(end_expr, predicate, results);
                }
            },
            Expression::Label(_) => {
                // Label has no child expressions
            },
            Expression::PatternPattern { patterns, .. } => {
                for pattern in patterns {
                    self.visit_with_predicate(pattern, predicate, results);
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Value;

    #[test]
    fn test_find_constants() {
        let mut visitor = FindVisitor::new();
        
        // 创建一个包含常量的表达式: 1 + 2 * 3
        let expr = Expression::BinaryOp(
            Box::new(Expression::Constant(Value::Int(1))),
            crate::graph::expression::BinaryOperator::Add,
            Box::new(Expression::BinaryOp(
                Box::new(Expression::Constant(Value::Int(2))),
                crate::graph::expression::BinaryOperator::Mul,
                Box::new(Expression::Constant(Value::Int(3))),
            )),
        );

        let constants = visitor
            .add_target_kind(ExpressionKind::Constant)
            .find(&expr);

        // 应该找到3个常量
        assert_eq!(constants.len(), 3);
    }

    #[test]
    fn test_find_with_predicate() {
        let mut visitor = FindVisitor::new();
        
        // 创建一个包含常量的表达式: 1 + 2 * 3
        let expr = Expression::BinaryOp(
            Box::new(Expression::Constant(Value::Int(1))),
            crate::graph::expression::BinaryOperator::Add,
            Box::new(Expression::BinaryOp(
                Box::new(Expression::Constant(Value::Int(2))),
                crate::graph::expression::BinaryOperator::Mul,
                Box::new(Expression::Constant(Value::Int(3))),
            )),
        );

        let constants = visitor.find_if(&expr, |e| {
            matches!(e, Expression::Constant(Value::Int(_)))
        });

        // 应该找到3个整数常量
        assert_eq!(constants.len(), 3);
    }
}