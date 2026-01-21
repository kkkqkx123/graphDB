//! FindVisitor - 用于查找表达式中特定类型子表达式的访问器
//! 对应 NebulaGraph FindVisitor.h/.cpp 的功能

use crate::core::expression_visitor::{ExpressionVisitor, ExpressionVisitorState};
use crate::core::types::expression::DataType;
use crate::core::Value;
use crate::expression::{Expression, ExpressionType};
use std::collections::HashSet;

#[derive(Debug)]
pub struct FindVisitor {
    /// 要查找的表达式类型集合
    target_types: HashSet<ExpressionType>,
    /// 找到的表达式列表
    found_exprs: Vec<Expression>,
    /// 访问者状态
    state: ExpressionVisitorState,
}

impl FindVisitor {
    pub fn new() -> Self {
        Self {
            target_types: HashSet::new(),
            found_exprs: Vec::new(),
            state: ExpressionVisitorState::new(),
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
        let _ = self.visit_expression(expr);
        self.found_exprs.clone()
    }

    /// 检查表达式中是否存在匹配类型的子表达式
    pub fn exist(&mut self, expr: &Expression) -> bool {
        self.found_exprs.clear();
        let _ = self.visit_expression(expr);
        !self.found_exprs.is_empty()
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

        for child in expr.children() {
            self.visit_with_predicate(child, predicate, results);
        }
    }

    /// 检查表达式类型是否匹配目标类型
    fn should_collect(&self, expr_type: ExpressionType) -> bool {
        self.target_types.contains(&expr_type)
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
        if self.should_collect(ExpressionType::Literal) {
            self.found_exprs.push(Expression::Literal(value.clone()));
        }
    }

    fn visit_variable(&mut self, name: &str) -> Self::Result {
        if self.should_collect(ExpressionType::Variable) {
            self.found_exprs.push(Expression::Variable(name.to_string()));
        }
    }

    fn visit_property(&mut self, object: &Expression, property: &str) -> Self::Result {
        if self.should_collect(ExpressionType::Property) {
            self.found_exprs.push(Expression::Property {
                object: Box::new(object.clone()),
                property: property.to_string(),
            });
        }
        self.visit_expression(object);
    }

    fn visit_binary(
        &mut self,
        left: &Expression,
        op: &crate::core::types::operators::BinaryOperator,
        right: &Expression,
    ) -> Self::Result {
        if self.should_collect(ExpressionType::Binary) {
            self.found_exprs.push(Expression::Binary {
                left: Box::new(left.clone()),
                op: op.clone(),
                right: Box::new(right.clone()),
            });
        }
        self.visit_expression(left);
        self.visit_expression(right);
    }

    fn visit_unary(
        &mut self,
        op: &crate::core::types::operators::UnaryOperator,
        operand: &Expression,
    ) -> Self::Result {
        if self.should_collect(ExpressionType::Unary) {
            self.found_exprs.push(Expression::Unary {
                op: op.clone(),
                operand: Box::new(operand.clone()),
            });
        }
        self.visit_expression(operand);
    }

    fn visit_function(&mut self, name: &str, args: &[Expression]) -> Self::Result {
        if self.should_collect(ExpressionType::Function) {
            self.found_exprs.push(Expression::Function {
                name: name.to_string(),
                args: args.to_vec(),
            });
        }
        for arg in args {
            self.visit_expression(arg);
        }
    }

    fn visit_aggregate(
        &mut self,
        func: &crate::core::types::operators::AggregateFunction,
        arg: &Expression,
        distinct: bool,
    ) -> Self::Result {
        if self.should_collect(ExpressionType::Aggregate) {
            self.found_exprs.push(Expression::Aggregate {
                func: func.clone(),
                arg: Box::new(arg.clone()),
                distinct,
            });
        }
        self.visit_expression(arg);
    }

    fn visit_list(&mut self, items: &[Expression]) -> Self::Result {
        if self.should_collect(ExpressionType::List) {
            self.found_exprs.push(Expression::List(items.to_vec()));
        }
        for item in items {
            self.visit_expression(item);
        }
    }

    fn visit_map(&mut self, pairs: &[(String, Expression)]) -> Self::Result {
        if self.should_collect(ExpressionType::Map) {
            self.found_exprs.push(Expression::Map(pairs.to_vec()));
        }
        for (_, value) in pairs {
            self.visit_expression(value);
        }
    }

    fn visit_case(
        &mut self,
        conditions: &[(Expression, Expression)],
        default: &Option<Box<Expression>>,
    ) -> Self::Result {
        if self.should_collect(ExpressionType::Case) {
            self.found_exprs.push(Expression::Case {
                conditions: conditions.to_vec(),
                default: default.clone(),
            });
        }
        for (condition, value) in conditions {
            self.visit_expression(condition);
            self.visit_expression(value);
        }
        if let Some(default_expr) = default {
            self.visit_expression(default_expr);
        }
    }

    fn visit_type_cast(&mut self, expr: &Expression, target_type: &DataType) -> Self::Result {
        if self.should_collect(ExpressionType::TypeCast) {
            self.found_exprs.push(Expression::TypeCast {
                expr: Box::new(expr.clone()),
                target_type: target_type.clone(),
            });
        }
        self.visit_expression(expr);
    }

    fn visit_subscript(&mut self, collection: &Expression, index: &Expression) -> Self::Result {
        if self.should_collect(ExpressionType::Subscript) {
            self.found_exprs.push(Expression::Subscript {
                collection: Box::new(collection.clone()),
                index: Box::new(index.clone()),
            });
        }
        self.visit_expression(collection);
        self.visit_expression(index);
    }

    fn visit_range(
        &mut self,
        collection: &Expression,
        start: &Option<Box<Expression>>,
        end: &Option<Box<Expression>>,
    ) -> Self::Result {
        if self.should_collect(ExpressionType::Range) {
            self.found_exprs.push(Expression::Range {
                collection: Box::new(collection.clone()),
                start: start.clone(),
                end: end.clone(),
            });
        }
        self.visit_expression(collection);
        if let Some(start_expr) = start {
            self.visit_expression(start_expr);
        }
        if let Some(end_expr) = end {
            self.visit_expression(end_expr);
        }
    }

    fn visit_path(&mut self, items: &[Expression]) -> Self::Result {
        if self.should_collect(ExpressionType::Path) {
            self.found_exprs.push(Expression::Path(items.to_vec()));
        }
        for item in items {
            self.visit_expression(item);
        }
    }

    fn visit_label(&mut self, name: &str) -> Self::Result {
        if self.should_collect(ExpressionType::Label) {
            self.found_exprs.push(Expression::Label(name.to_string()));
        }
    }

    fn visit_constant_expr(&mut self, _e: &crate::query::parser::ast::expr::ConstantExpr) -> Self::Result {}

    fn visit_variable_expr(&mut self, e: &crate::query::parser::ast::expr::VariableExpr) -> Self::Result {
        self.visit_variable(&e.name);
    }

    fn visit_binary_expr(&mut self, e: &crate::query::parser::ast::expr::BinaryExpr) -> Self::Result {
        self.visit_expr(&e.left);
        self.visit_expr(&e.right);
    }

    fn visit_unary_expr(&mut self, e: &crate::query::parser::ast::expr::UnaryExpr) -> Self::Result {
        self.visit_expr(&e.operand);
    }

    fn visit_function_call_expr(
        &mut self,
        e: &crate::query::parser::ast::expr::FunctionCallExpr,
    ) -> Self::Result {
        for arg in &e.args {
            self.visit_expr(arg);
        }
    }

    fn visit_property_access_expr(
        &mut self,
        e: &crate::query::parser::ast::expr::PropertyAccessExpr,
    ) -> Self::Result {
        self.visit_expr(&e.object);
    }

    fn visit_list_expr(&mut self, e: &crate::query::parser::ast::expr::ListExpr) -> Self::Result {
        for item in &e.elements {
            self.visit_expr(item);
        }
    }

    fn visit_map_expr(&mut self, e: &crate::query::parser::ast::expr::MapExpr) -> Self::Result {
        for (_, value) in &e.pairs {
            self.visit_expr(value);
        }
    }

    fn visit_case_expr(&mut self, e: &crate::query::parser::ast::expr::CaseExpr) -> Self::Result {
        for (condition, value) in &e.when_then_pairs {
            self.visit_expr(condition);
            self.visit_expr(value);
        }
        if let Some(expr) = &e.default {
            self.visit_expr(expr);
        }
    }

    fn visit_subscript_expr(
        &mut self,
        e: &crate::query::parser::ast::expr::SubscriptExpr,
    ) -> Self::Result {
        self.visit_expr(&e.collection);
        self.visit_expr(&e.index);
    }

    fn visit_type_cast_expr(
        &mut self,
        e: &crate::query::parser::ast::expr::TypeCastExpr,
    ) -> Self::Result {
        self.visit_expr(&e.expr);
    }

    fn visit_range_expr(&mut self, e: &crate::query::parser::ast::expr::RangeExpr) -> Self::Result {
        self.visit_expr(&e.collection);
        if let Some(expr) = &e.start {
            self.visit_expr(expr);
        }
        if let Some(expr) = &e.end {
            self.visit_expr(expr);
        }
    }

    fn visit_path_expr(&mut self, e: &crate::query::parser::ast::expr::PathExpr) -> Self::Result {
        for item in &e.elements {
            self.visit_expr(item);
        }
    }

    fn visit_label_expr(&mut self, _e: &crate::query::parser::ast::expr::LabelExpr) -> Self::Result {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::operators::BinaryOperator;

    #[test]
    fn test_find_literals() {
        let mut visitor = FindVisitor::new();

        let expr = Expression::Binary {
            left: Box::new(Expression::Literal(Value::Int(1))),
            op: BinaryOperator::Add,
            right: Box::new(Expression::Binary {
                left: Box::new(Expression::Literal(Value::Int(2))),
                op: BinaryOperator::Multiply,
                right: Box::new(Expression::Literal(Value::Int(3))),
            }),
        };

        let literals = visitor.add_target_type(ExpressionType::Literal).find(&expr);

        assert_eq!(literals.len(), 3);
    }

    #[test]
    fn test_find_with_predicate() {
        let mut visitor = FindVisitor::new();

        let expr = Expression::Binary {
            left: Box::new(Expression::Literal(Value::Int(1))),
            op: BinaryOperator::Add,
            right: Box::new(Expression::Binary {
                left: Box::new(Expression::Literal(Value::Int(2))),
                op: BinaryOperator::Multiply,
                right: Box::new(Expression::Literal(Value::Int(3))),
            }),
        };

        let literals = visitor.find_if(&expr, |e| matches!(e, Expression::Literal(Value::Int(_))));

        assert_eq!(literals.len(), 3);
    }
}
