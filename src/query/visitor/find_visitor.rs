//! FindVisitor - 用于查找表达式中特定类型子表达式的访问器
//! 对应 NebulaGraph FindVisitor.h/.cpp 的功能

use crate::core::expression_visitor::{ExpressionVisitor, ExpressionVisitorState};
use crate::core::types::expression::DataType;
use crate::core::types::operators::{AggregateFunction, BinaryOperator, UnaryOperator};
use crate::core::Value;
use crate::expression::Expr;
use crate::query::parser::ast::expr::*;
use std::collections::HashSet;

#[derive(Debug)]
pub struct FindVisitor {
    /// 找到的表达式列表
    found_exprs: Vec<Expr>,
    /// 访问者状态
    state: ExpressionVisitorState,
}

impl FindVisitor {
    pub fn new() -> Self {
        Self {
            found_exprs: Vec::new(),
            state: ExpressionVisitorState::new(),
        }
    }

    /// 搜索表达式中匹配类型的所有子表达式
    pub fn find(&mut self, expr: &Expr) -> Vec<Expr> {
        self.found_exprs.clear();
        let _ = self.visit_expression(expr);
        self.found_exprs.clone()
    }

    /// 检查表达式中是否存在匹配类型的子表达式
    pub fn exist(&mut self, expr: &Expr) -> bool {
        self.found_exprs.clear();
        let _ = self.visit_expression(expr);
        !self.found_exprs.is_empty()
    }

    /// 搜索表达式中匹配特定条件的子表达式
    pub fn find_if<F>(&mut self, expr: &Expr, predicate: F) -> Vec<Expr>
    where
        F: Fn(&Expr) -> bool,
    {
        let mut results = Vec::new();
        self.visit_with_predicate(expr, &predicate, &mut results);
        results
    }

    fn visit_with_predicate<F>(
        &self,
        expr: &Expr,
        predicate: &F,
        results: &mut Vec<Expr>,
    ) where
        F: Fn(&Expr) -> bool,
    {
        if predicate(expr) {
            results.push(expr.clone());
        }

        for child in expr.children() {
            self.visit_with_predicate(child, predicate, results);
        }
    }
}

impl ExpressionVisitor for FindVisitor {
    type Result = ();

    fn state(&self) -> &ExpressionVisitorState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut ExpressionVisitorState {
        &mut self.state
    }

    fn visit_literal(&mut self, value: &Value) -> Self::Result {
        self.found_exprs.push(Expr::Literal(value.clone()));
    }

    fn visit_variable(&mut self, name: &str) -> Self::Result {
        self.found_exprs.push(Expr::Variable(name.to_string()));
    }

    fn visit_property(&mut self, object: &Expr, property: &str) -> Self::Result {
        self.found_exprs.push(Expr::Property {
            object: Box::new(object.clone()),
            property: property.to_string(),
        });
        self.visit_expression(object);
    }

    fn visit_binary(
        &mut self,
        left: &Expr,
        _op: &BinaryOperator,
        right: &Expr,
    ) -> Self::Result {
        self.found_exprs.push(Expr::Binary {
            left: Box::new(left.clone()),
            op: *_op,
            right: Box::new(right.clone()),
        });
        self.visit_expression(left);
        self.visit_expression(right);
    }

    fn visit_unary(
        &mut self,
        op: &UnaryOperator,
        operand: &Expr,
    ) -> Self::Result {
        self.found_exprs.push(Expr::Unary {
            op: *op,
            operand: Box::new(operand.clone()),
        });
        self.visit_expression(operand);
    }

    fn visit_function(&mut self, name: &str, args: &[Expr]) -> Self::Result {
        self.found_exprs.push(Expr::Function {
            name: name.to_string(),
            args: args.to_vec(),
        });
        for arg in args {
            self.visit_expression(arg);
        }
    }

    fn visit_aggregate(
        &mut self,
        func: &AggregateFunction,
        arg: &Expr,
        distinct: bool,
    ) -> Self::Result {
        self.found_exprs.push(Expr::Aggregate {
            func: func.clone(),
            arg: Box::new(arg.clone()),
            distinct,
        });
        self.visit_expression(arg);
    }

    fn visit_list(&mut self, items: &[Expr]) -> Self::Result {
        self.found_exprs.push(Expr::List(items.to_vec()));
        for item in items {
            self.visit_expression(item);
        }
    }

    fn visit_map(&mut self, pairs: &[(String, Expr)]) -> Self::Result {
        self.found_exprs.push(Expr::Map(pairs.to_vec()));
        for (_, value) in pairs {
            self.visit_expression(value);
        }
    }

    fn visit_case(
        &mut self,
        conditions: &[(Expr, Expr)],
        default: &Option<Box<Expr>>,
    ) -> Self::Result {
        self.found_exprs.push(Expr::Case {
            conditions: conditions.to_vec(),
            default: default.clone(),
        });
        for (cond, expr) in conditions {
            self.visit_expression(cond);
            self.visit_expression(expr);
        }
        if let Some(default_expr) = default {
            self.visit_expression(default_expr);
        }
    }

    fn visit_type_cast(&mut self, expr: &Expr, target_type: &DataType) -> Self::Result {
        self.found_exprs.push(Expr::TypeCast {
            expr: Box::new(expr.clone()),
            target_type: target_type.clone(),
        });
        self.visit_expression(expr);
    }

    fn visit_subscript(&mut self, collection: &Expr, index: &Expr) -> Self::Result {
        self.found_exprs.push(Expr::Subscript {
            collection: Box::new(collection.clone()),
            index: Box::new(index.clone()),
        });
        self.visit_expression(collection);
        self.visit_expression(index);
    }

    fn visit_range(
        &mut self,
        collection: &Expr,
        start: &Option<Box<Expr>>,
        end: &Option<Box<Expr>>,
    ) -> Self::Result {
        self.found_exprs.push(Expr::Range {
            collection: Box::new(collection.clone()),
            start: start.clone(),
            end: end.clone(),
        });
        self.visit_expression(collection);
        if let Some(start_expr) = start {
            self.visit_expression(start_expr);
        }
        if let Some(end_expr) = end {
            self.visit_expression(end_expr);
        }
    }

    fn visit_path(&mut self, items: &[Expr]) -> Self::Result {
        self.found_exprs.push(Expr::Path(items.to_vec()));
        for item in items {
            self.visit_expression(item);
        }
    }

    fn visit_label(&mut self, name: &str) -> Self::Result {
        self.found_exprs.push(Expr::Label(name.to_string()));
    }
}
