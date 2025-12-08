//! FindVisitor - 用于查找表达式中特定类型子表达式的访问器
//! 对应 NebulaGraph FindVisitor.h/.cpp 的功能

use std::collections::HashSet;
use crate::graph::expression::{Expression, ExpressionKind};

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
            // 对于常量、属性等没有子表达式的类型
            Expression::Constant(_) |
            Expression::Property(_) => {
                // 没有子表达式，无需处理
            }
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
            // 对于常量、属性等没有子表达式的类型
            Expression::Constant(_) |
            Expression::Property(_) => {
                // 没有子表达式，无需处理
            }
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