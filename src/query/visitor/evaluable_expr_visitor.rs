//! EvaluableExprVisitor - 用于判断表达式是否可求值的访问器
//! 对应 NebulaGraph EvaluableExprVisitor.h/.cpp 的功能

use crate::core::{
    AggregateFunction, BinaryOperator, DataType, Expression, LiteralValue, UnaryOperator,
};
use crate::expression::ExpressionVisitor;
use crate::query::visitor::QueryVisitor;

#[derive(Debug)]
pub struct EvaluableExprVisitor {
    /// 表达式是否可求值
    evaluable: bool,
    /// 错误信息
    error: Option<String>,
}

impl EvaluableExprVisitor {
    pub fn new() -> Self {
        Self {
            evaluable: true,
            error: None,
        }
    }

    pub fn is_evaluable(&mut self, expr: &Expression) -> bool {
        self.evaluable = true;
        self.error = None;

        if let Err(e) = self.visit(expr) {
            self.evaluable = false;
            self.error = Some(e);
        }

        self.evaluable
    }

    pub fn get_error(&self) -> Option<&String> {
        self.error.as_ref()
    }
}

impl ExpressionVisitor for EvaluableExprVisitor {
    type Result = Result<(), String>;

    fn visit_literal(&mut self, _value: &LiteralValue) -> Self::Result {
        Ok(())
    }

    fn visit_variable(&mut self, _name: &str) -> Self::Result {
        self.evaluable = false;
        Ok(())
    }

    fn visit_property(&mut self, _object: &Expression, _property: &str) -> Self::Result {
        self.evaluable = false;
        Ok(())
    }

    fn visit_binary(
        &mut self,
        left: &Expression,
        _op: &BinaryOperator,
        right: &Expression,
    ) -> Self::Result {
        self.visit(left)?;
        self.visit(right)?;
        Ok(())
    }

    fn visit_unary(&mut self, _op: &UnaryOperator, operand: &Expression) -> Self::Result {
        self.visit(operand)?;
        Ok(())
    }

    fn visit_function(&mut self, _name: &str, args: &[Expression]) -> Self::Result {
        for arg in args {
            self.visit(arg)?;
        }
        Ok(())
    }

    fn visit_aggregate(
        &mut self,
        _func: &AggregateFunction,
        arg: &Expression,
        _distinct: bool,
    ) -> Self::Result {
        self.visit(arg)?;
        Ok(())
    }

    fn visit_list(&mut self, items: &[Expression]) -> Self::Result {
        for item in items {
            self.visit(item)?;
        }
        Ok(())
    }

    fn visit_map(&mut self, pairs: &[(String, Expression)]) -> Self::Result {
        for (_, value) in pairs {
            self.visit(value)?;
        }
        Ok(())
    }

    fn visit_case(
        &mut self,
        conditions: &[(Expression, Expression)],
        default: &Option<Box<Expression>>,
    ) -> Self::Result {
        for (condition, value) in conditions {
            self.visit(condition)?;
            self.visit(value)?;
        }
        if let Some(default_expr) = default {
            self.visit(default_expr)?;
        }
        Ok(())
    }

    fn visit_type_cast(&mut self, expr: &Expression, _target_type: &DataType) -> Self::Result {
        self.visit(expr)?;
        Ok(())
    }

    fn visit_subscript(&mut self, collection: &Expression, index: &Expression) -> Self::Result {
        self.visit(collection)?;
        self.visit(index)?;
        Ok(())
    }

    fn visit_range(
        &mut self,
        collection: &Expression,
        start: &Option<Box<Expression>>,
        end: &Option<Box<Expression>>,
    ) -> Self::Result {
        self.visit(collection)?;
        if let Some(start_expr) = start {
            self.visit(start_expr)?;
        }
        if let Some(end_expr) = end {
            self.visit(end_expr)?;
        }
        Ok(())
    }

    fn visit_path(&mut self, items: &[Expression]) -> Self::Result {
        for item in items {
            self.visit(item)?;
        }
        Ok(())
    }

    fn visit_label(&mut self, _name: &str) -> Self::Result {
        Ok(())
    }

    fn visit_tag_property(&mut self, _tag: &str, _prop: &str) -> Self::Result {
        self.evaluable = false;
        Ok(())
    }

    fn visit_edge_property(&mut self, _edge: &str, _prop: &str) -> Self::Result {
        self.evaluable = false;
        Ok(())
    }

    fn visit_input_property(&mut self, _prop: &str) -> Self::Result {
        self.evaluable = false;
        Ok(())
    }

    fn visit_variable_property(&mut self, _var: &str, _prop: &str) -> Self::Result {
        self.evaluable = false;
        Ok(())
    }

    fn visit_source_property(&mut self, _tag: &str, _prop: &str) -> Self::Result {
        self.evaluable = false;
        Ok(())
    }

    fn visit_destination_property(&mut self, _tag: &str, _prop: &str) -> Self::Result {
        self.evaluable = false;
        Ok(())
    }
}

impl QueryVisitor for EvaluableExprVisitor {
    type QueryResult = bool;

    fn get_result(&self) -> Self::QueryResult {
        self.evaluable
    }

    fn reset(&mut self) {
        self.evaluable = true;
        self.error = None;
    }

    fn is_success(&self) -> bool {
        self.error.is_none()
    }
}
