//! ExtractFilterExprVisitor - 用于提取过滤表达式的访问器
//! 对应 NebulaGraph ExtractFilterExprVisitor.h/.cpp 的功能

use crate::core::expression_visitor::{ExpressionVisitor, ExpressionVisitorState};
use crate::core::Value;
use crate::core::{AggregateFunction, BinaryOperator, DataType, Expression, UnaryOperator};

#[derive(Debug)]
pub struct ExtractFilterExprVisitor {
    /// 提取到的过滤表达式
    filter_exprs: Vec<Expression>,
    /// 是否只提取顶层的过滤条件
    top_level_only: bool,
    /// 当前是否在顶层
    is_top_level: bool,
    /// 访问者状态
    state: ExpressionVisitorState,
}

impl Clone for ExtractFilterExprVisitor {
    fn clone(&self) -> Self {
        Self {
            filter_exprs: self.filter_exprs.clone(),
            top_level_only: self.top_level_only,
            is_top_level: self.is_top_level,
            state: self.state.clone(),
        }
    }
}

impl ExtractFilterExprVisitor {
    pub fn new(top_level_only: bool) -> Self {
        Self {
            filter_exprs: Vec::new(),
            top_level_only,
            is_top_level: true,
            state: ExpressionVisitorState::new(),
        }
    }

    pub fn extract(&mut self, expr: &Expression) -> Result<Vec<Expression>, String> {
        self.filter_exprs.clear();
        self.is_top_level = true;
        let result = self.visit_expression(expr);
        result?;
        Ok(self.filter_exprs.clone())
    }

    fn visit_with_updated_level(&mut self, expr: &Expression) -> Result<(), String> {
        let old_top_level = self.is_top_level;
        self.is_top_level = false;
        let result = self.visit_expression(expr);
        self.is_top_level = old_top_level;
        result
    }

    pub fn get_filter_exprs(&self) -> &Vec<Expression> {
        &self.filter_exprs
    }
}

fn is_filter_function(func_name: &str) -> bool {
    matches!(
        func_name.to_lowercase().as_str(),
        "isempty"
            | "isnull"
            | "isnotnull"
            | "isnullorempty"
            | "has"
            | "haslabel"
            | "hastag"
            | "contains"
    )
}

impl ExpressionVisitor for ExtractFilterExprVisitor {
    type Result = Result<(), String>;

    fn visit_literal(&mut self, _value: &Value) -> Self::Result {
        Ok(())
    }

    fn visit_variable(&mut self, _name: &str) -> Self::Result {
        Ok(())
    }

    fn visit_property(&mut self, object: &Expression, _property: &str) -> Self::Result {
        self.visit_expression(object)
    }

    fn visit_binary(
        &mut self,
        left: &Expression,
        op: &BinaryOperator,
        right: &Expression,
    ) -> Self::Result {
        if self.is_top_level || !self.top_level_only {
            self.visit_with_updated_level(left)?;
            self.visit_with_updated_level(right)?;
        } else {
            self.filter_exprs.push(Expression::Binary {
                left: Box::new(left.clone()),
                op: op.clone(),
                right: Box::new(right.clone()),
            });
        }
        Ok(())
    }

    fn visit_unary(&mut self, _op: &UnaryOperator, operand: &Expression) -> Self::Result {
        self.visit_expression(operand)
    }

    fn visit_function(&mut self, name: &str, args: &[Expression]) -> Self::Result {
        if is_filter_function(name) {
            if self.is_top_level || !self.top_level_only {
                self.filter_exprs.push(Expression::Function {
                    name: name.to_string(),
                    args: args.to_vec(),
                });
            }
        }
        for arg in args {
            self.visit_expression(arg)?;
        }
        Ok(())
    }

    fn visit_aggregate(
        &mut self,
        _func: &AggregateFunction,
        arg: &Expression,
        _distinct: bool,
    ) -> Self::Result {
        self.visit_expression(arg)
    }

    fn visit_list(&mut self, items: &[Expression]) -> Self::Result {
        for item in items {
            self.visit_expression(item)?;
        }
        Ok(())
    }

    fn visit_map(&mut self, pairs: &[(String, Expression)]) -> Self::Result {
        for (_, expr) in pairs {
            self.visit_expression(expr)?;
        }
        Ok(())
    }

    fn visit_case(
        &mut self,
        conditions: &[(Expression, Expression)],
        default: &Option<Box<Expression>>,
    ) -> Self::Result {
        for (cond, expr) in conditions {
            self.visit_expression(cond)?;
            self.visit_expression(expr)?;
        }
        if let Some(d) = default {
            self.visit_expression(d)?;
        }
        Ok(())
    }

    fn visit_type_cast(&mut self, expr: &Expression, _target_type: &DataType) -> Self::Result {
        self.visit_expression(expr)
    }

    fn visit_subscript(&mut self, collection: &Expression, index: &Expression) -> Self::Result {
        self.visit_expression(collection)?;
        self.visit_expression(index)
    }

    fn visit_range(
        &mut self,
        collection: &Expression,
        start: &Option<Box<Expression>>,
        end: &Option<Box<Expression>>,
    ) -> Self::Result {
        self.visit_expression(collection)?;
        if let Some(s) = start {
            self.visit_expression(s)?;
        }
        if let Some(e) = end {
            self.visit_expression(e)?;
        }
        Ok(())
    }

    fn visit_path(&mut self, items: &[Expression]) -> Self::Result {
        for item in items {
            self.visit_expression(item)?;
        }
        Ok(())
    }

    fn visit_label(&mut self, _name: &str) -> Self::Result {
        Ok(())
    }

    fn state(&self) -> &ExpressionVisitorState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut ExpressionVisitorState {
        &mut self.state
    }

    fn visit_constant_expr(&mut self, _expr: &crate::query::parser::ast::expr::ConstantExpr) -> Self::Result {
        Ok(())
    }

    fn visit_variable_expr(&mut self, _expr: &crate::query::parser::ast::expr::VariableExpr) -> Self::Result {
        Ok(())
    }

    fn visit_binary_expr(&mut self, expr: &crate::query::parser::ast::expr::BinaryExpr) -> Self::Result {
        self.visit_expr(&expr.left)?;
        self.visit_expr(&expr.right)?;
        Ok(())
    }

    fn visit_unary_expr(&mut self, expr: &crate::query::parser::ast::expr::UnaryExpr) -> Self::Result {
        self.visit_expr(&expr.operand)?;
        Ok(())
    }

    fn visit_function_call_expr(&mut self, expr: &crate::query::parser::ast::expr::FunctionCallExpr) -> Self::Result {
        for arg in &expr.args {
            self.visit_expr(arg)?;
        }
        Ok(())
    }

    fn visit_property_access_expr(&mut self, expr: &crate::query::parser::ast::expr::PropertyAccessExpr) -> Self::Result {
        self.visit_expr(&expr.object)?;
        Ok(())
    }

    fn visit_list_expr(&mut self, expr: &crate::query::parser::ast::expr::ListExpr) -> Self::Result {
        for item in &expr.elements {
            self.visit_expr(item)?;
        }
        Ok(())
    }

    fn visit_map_expr(&mut self, expr: &crate::query::parser::ast::expr::MapExpr) -> Self::Result {
        for (_, value) in &expr.pairs {
            self.visit_expr(value)?;
        }
        Ok(())
    }

    fn visit_case_expr(&mut self, expr: &crate::query::parser::ast::expr::CaseExpr) -> Self::Result {
        for (cond, val) in &expr.when_then_pairs {
            self.visit_expr(cond)?;
            self.visit_expr(val)?;
        }
        if let Some(d) = &expr.default {
            self.visit_expr(d)?;
        }
        Ok(())
    }

    fn visit_subscript_expr(&mut self, expr: &crate::query::parser::ast::expr::SubscriptExpr) -> Self::Result {
        self.visit_expr(&expr.collection)?;
        self.visit_expr(&expr.index)?;
        Ok(())
    }

    fn visit_type_cast_expr(&mut self, expr: &crate::query::parser::ast::expr::TypeCastExpr) -> Self::Result {
        self.visit_expr(&expr.expr)?;
        Ok(())
    }

    fn visit_range_expr(&mut self, expr: &crate::query::parser::ast::expr::RangeExpr) -> Self::Result {
        self.visit_expr(&expr.collection)?;
        if let Some(s) = &expr.start {
            self.visit_expr(s)?;
        }
        if let Some(e) = &expr.end {
            self.visit_expr(e)?;
        }
        Ok(())
    }

    fn visit_path_expr(&mut self, expr: &crate::query::parser::ast::expr::PathExpr) -> Self::Result {
        for item in &expr.elements {
            self.visit_expr(item)?;
        }
        Ok(())
    }

    fn visit_label_expr(&mut self, _expr: &crate::query::parser::ast::expr::LabelExpr) -> Self::Result {
        Ok(())
    }
}

fn is_filter_expression(expr: &Expression) -> bool {
    // 检查表达式是否为过滤表达式
    // 通常关系表达式和函数调用是过滤表达式
    matches!(
        expr,
        Expression::Binary { .. } | Expression::Function { .. }
    )
}
