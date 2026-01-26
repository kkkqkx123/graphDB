//! FoldConstantExprVisitor - 用于折叠常量表达式的访问器
//!
//! 主要功能：
//! - 在编译时识别并计算常量表达式
//! - 支持算术运算、逻辑运算、函数调用等
//! - 优化查询性能，减少运行时计算

use crate::core::types::expression::Expression;
use crate::core::types::expression::visitor::{ExpressionVisitor, ExpressionVisitorState};
use crate::core::{
    BinaryOperator, DataType, UnaryOperator, Value,
};
use crate::core::types::operators::AggregateFunction;

/// 常量表达式折叠访问器
///
/// 用于在编译时计算常量表达式，优化查询性能
#[derive(Debug)]
pub struct FoldConstantExprVisitor {
    /// 是否可以折叠
    can_be_folded: bool,
    /// 错误状态
    error: Option<String>,
    /// 折叠后的表达式
    folded_expression: Option<Expression>,
    /// 访问者状态
    state: ExpressionVisitorState,
}

impl FoldConstantExprVisitor {
    /// 创建新的常量表达式折叠访问器
    pub fn new() -> Self {
        Self {
            can_be_folded: false,
            error: None,
            folded_expression: None,
            state: ExpressionVisitorState::new(),
        }
    }

    /// 尝试折叠表达式
    ///
    /// 返回折叠后的表达式，如果表达式不可折叠则返回原表达式的克隆
    pub fn fold(&mut self, expression: &Expression) -> Result<Expression, String> {
        self.can_be_folded = true;
        self.error = None;
        self.folded_expression = None;

        self.visit_expression(expression)?;

        if let Some(folded) = self.folded_expression.take() {
            Ok(folded)
        } else if self.can_be_folded {
            Ok(expression.clone())
        } else {
            Err(self.error.clone().unwrap_or_else(|| "表达式不可折叠".to_string()))
        }
    }

    /// 检查表达式是否为常量
    pub fn is_constant(expression: &Expression) -> bool {
        matches!(expression, Expression::Literal(_))
    }

    /// 检查是否可以折叠
    pub fn can_be_folded(&self) -> bool {
        self.can_be_folded
    }

    /// 获取错误信息
    pub fn get_error(&self) -> Option<&String> {
        self.error.as_ref()
    }

    /// 设置折叠后的表达式
    fn set_folded(&mut self, expression: Expression) {
        self.folded_expression = Some(expression);
    }

    /// 尝试计算二元运算
    fn try_fold_binary(
        &mut self,
        left: &Expression,
        op: &BinaryOperator,
        right: &Expression,
    ) -> Option<Expression> {
        if !Self::is_constant(left) || !Self::is_constant(right) {
            return None;
        }

        let left_val = match left {
            Expression::Literal(v) => v,
            _ => return None,
        };

        let right_val = match right {
            Expression::Literal(v) => v,
            _ => return None,
        };

        let result = match op {
            BinaryOperator::Add => self.fold_add(left_val, right_val)?,
            BinaryOperator::Subtract => self.fold_subtract(left_val, right_val)?,
            BinaryOperator::Multiply => self.fold_multiply(left_val, right_val)?,
            BinaryOperator::Divide => self.fold_divide(left_val, right_val)?,
            BinaryOperator::Modulo => self.fold_modulo(left_val, right_val)?,
            BinaryOperator::Equal => self.fold_equal(left_val, right_val)?,
            BinaryOperator::NotEqual => self.fold_not_equal(left_val, right_val)?,
            BinaryOperator::LessThan => self.fold_less_than(left_val, right_val)?,
            BinaryOperator::LessThanOrEqual => self.fold_less_than_or_equal(left_val, right_val)?,
            BinaryOperator::GreaterThan => self.fold_greater_than(left_val, right_val)?,
            BinaryOperator::GreaterThanOrEqual => {
                self.fold_greater_than_or_equal(left_val, right_val)?
            }
            BinaryOperator::And => self.fold_and(left_val, right_val)?,
            BinaryOperator::Or => self.fold_or(left_val, right_val)?,
            BinaryOperator::In => self.fold_in(left_val, right_val)?,
            _ => return None,
        };

        Some(Expression::Literal(result))
    }

    fn fold_add(&self, left: &Value, right: &Value) -> Option<Value> {
        match (left, right) {
            (Value::Int(l), Value::Int(r)) => Some(Value::Int(l + r)),
            (Value::Float(l), Value::Float(r)) => Some(Value::Float(l + r)),
            (Value::Int(l), Value::Float(r)) => Some(Value::Float(*l as f64 + r)),
            (Value::Float(l), Value::Int(r)) => Some(Value::Float(l + *r as f64)),
            (Value::String(l), Value::String(r)) => Some(Value::String(format!("{}{}", l, r))),
            _ => None,
        }
    }

    fn fold_subtract(&self, left: &Value, right: &Value) -> Option<Value> {
        match (left, right) {
            (Value::Int(l), Value::Int(r)) => Some(Value::Int(l - r)),
            (Value::Float(l), Value::Float(r)) => Some(Value::Float(l - r)),
            (Value::Int(l), Value::Float(r)) => Some(Value::Float(*l as f64 - r)),
            (Value::Float(l), Value::Int(r)) => Some(Value::Float(l - *r as f64)),
            _ => None,
        }
    }

    fn fold_multiply(&self, left: &Value, right: &Value) -> Option<Value> {
        match (left, right) {
            (Value::Int(l), Value::Int(r)) => Some(Value::Int(l * r)),
            (Value::Float(l), Value::Float(r)) => Some(Value::Float(l * r)),
            (Value::Int(l), Value::Float(r)) => Some(Value::Float(*l as f64 * r)),
            (Value::Float(l), Value::Int(r)) => Some(Value::Float(l * *r as f64)),
            _ => None,
        }
    }

    fn fold_divide(&self, left: &Value, right: &Value) -> Option<Value> {
        match (left, right) {
            (Value::Int(l), Value::Int(r)) => {
                if *r == 0 {
                    None
                } else {
                    Some(Value::Int(l / r))
                }
            }
            (Value::Float(l), Value::Float(r)) => {
                if *r == 0.0 {
                    None
                } else {
                    Some(Value::Float(l / r))
                }
            }
            (Value::Int(l), Value::Float(r)) => {
                if *r == 0.0 {
                    None
                } else {
                    Some(Value::Float(*l as f64 / r))
                }
            }
            (Value::Float(l), Value::Int(r)) => {
                if *r == 0 {
                    None
                } else {
                    Some(Value::Float(l / *r as f64))
                }
            }
            _ => None,
        }
    }

    fn fold_modulo(&self, left: &Value, right: &Value) -> Option<Value> {
        match (left, right) {
            (Value::Int(l), Value::Int(r)) => {
                if *r == 0 {
                    None
                } else {
                    Some(Value::Int(l % r))
                }
            }
            _ => None,
        }
    }

    fn fold_equal(&self, left: &Value, right: &Value) -> Option<Value> {
        Some(Value::Bool(left == right))
    }

    fn fold_not_equal(&self, left: &Value, right: &Value) -> Option<Value> {
        Some(Value::Bool(left != right))
    }

    fn fold_less_than(&self, left: &Value, right: &Value) -> Option<Value> {
        match (left, right) {
            (Value::Int(l), Value::Int(r)) => Some(Value::Bool(l < r)),
            (Value::Float(l), Value::Float(r)) => Some(Value::Bool(l < r)),
            (Value::Int(l), Value::Float(r)) => Some(Value::Bool((*l as f64) < *r)),
            (Value::Float(l), Value::Int(r)) => Some(Value::Bool(*l < (*r as f64))),
            (Value::String(l), Value::String(r)) => Some(Value::Bool(l < r)),
            _ => None,
        }
    }

    fn fold_less_than_or_equal(&self, left: &Value, right: &Value) -> Option<Value> {
        match (left, right) {
            (Value::Int(l), Value::Int(r)) => Some(Value::Bool(l <= r)),
            (Value::Float(l), Value::Float(r)) => Some(Value::Bool(l <= r)),
            (Value::Int(l), Value::Float(r)) => Some(Value::Bool((*l as f64) <= *r)),
            (Value::Float(l), Value::Int(r)) => Some(Value::Bool(*l <= (*r as f64))),
            (Value::String(l), Value::String(r)) => Some(Value::Bool(l <= r)),
            _ => None,
        }
    }

    fn fold_greater_than(&self, left: &Value, right: &Value) -> Option<Value> {
        match (left, right) {
            (Value::Int(l), Value::Int(r)) => Some(Value::Bool(l > r)),
            (Value::Float(l), Value::Float(r)) => Some(Value::Bool(l > r)),
            (Value::Int(l), Value::Float(r)) => Some(Value::Bool((*l as f64) > *r)),
            (Value::Float(l), Value::Int(r)) => Some(Value::Bool(*l > (*r as f64))),
            (Value::String(l), Value::String(r)) => Some(Value::Bool(l > r)),
            _ => None,
        }
    }

    fn fold_greater_than_or_equal(&self, left: &Value, right: &Value) -> Option<Value> {
        match (left, right) {
            (Value::Int(l), Value::Int(r)) => Some(Value::Bool(l >= r)),
            (Value::Float(l), Value::Float(r)) => Some(Value::Bool(l >= r)),
            (Value::Int(l), Value::Float(r)) => Some(Value::Bool((*l as f64) >= *r)),
            (Value::Float(l), Value::Int(r)) => Some(Value::Bool(*l >= (*r as f64))),
            (Value::String(l), Value::String(r)) => Some(Value::Bool(l >= r)),
            _ => None,
        }
    }

    fn fold_and(&self, left: &Value, right: &Value) -> Option<Value> {
        match (left, right) {
            (Value::Bool(l), Value::Bool(r)) => Some(Value::Bool(*l && *r)),
            _ => None,
        }
    }

    fn fold_or(&self, left: &Value, right: &Value) -> Option<Value> {
        match (left, right) {
            (Value::Bool(l), Value::Bool(r)) => Some(Value::Bool(*l || *r)),
            _ => None,
        }
    }

    fn fold_in(&self, left: &Value, right: &Value) -> Option<Value> {
        match right {
            Value::List(items) => Some(Value::Bool(items.contains(left))),
            _ => None,
        }
    }

    /// 尝试计算一元运算
    fn try_fold_unary(&mut self, op: &UnaryOperator, operand: &Expression) -> Option<Expression> {
        if !Self::is_constant(operand) {
            return None;
        }

        let val = match operand {
            Expression::Literal(v) => v,
            _ => return None,
        };

        let result = match op {
            UnaryOperator::Plus => self.fold_unary_plus(val)?,
            UnaryOperator::Minus => self.fold_unary_minus(val)?,
            UnaryOperator::Not => self.fold_unary_not(val)?,
            _ => return None,
        };

        Some(Expression::Literal(result))
    }

    fn fold_unary_plus(&self, val: &Value) -> Option<Value> {
        Some(val.clone())
    }

    fn fold_unary_minus(&self, val: &Value) -> Option<Value> {
        match val {
            Value::Int(i) => Some(Value::Int(-i)),
            Value::Float(f) => Some(Value::Float(-f)),
            _ => None,
        }
    }

    fn fold_unary_not(&self, val: &Value) -> Option<Value> {
        match val {
            Value::Bool(b) => Some(Value::Bool(!b)),
            _ => None,
        }
    }

    fn fold_math_function_internal(&self, name: &str, val: &Value) -> Option<Value> {
        let num = match val {
            Value::Int(i) => *i as f64,
            Value::Float(f) => *f,
            _ => return None,
        };

        let result = match name.to_uppercase().as_str() {
            "ABS" => num.abs(),
            "CEIL" => num.ceil(),
            "FLOOR" => num.floor(),
            "SQRT" => num.sqrt(),
            "POW" => return None,
            "EXP" => num.exp(),
            "LOG" => num.ln(),
            "LOG10" => num.log10(),
            "SIN" => num.sin(),
            "COS" => num.cos(),
            "TAN" => num.tan(),
            "ASIN" => num.asin(),
            "ACOS" => num.acos(),
            "ATAN" => num.atan(),
            "ROUND" => num.round(),
            _ => return None,
        };

        Some(Value::Float(result))
    }

    fn fold_string_function_internal(&self, name: &str, val: &Value) -> Option<Value> {
        let s = match val {
            Value::String(s) => s,
            _ => return None,
        };

        let result = match name.to_uppercase().as_str() {
            "LOWER" => s.to_lowercase(),
            "UPPER" => s.to_uppercase(),
            "TRIM" => s.trim().to_string(),
            "LTRIM" => s.trim_start().to_string(),
            "RTRIM" => s.trim_end().to_string(),
            "REVERSE" => s.chars().rev().collect::<String>(),
            _ => return None,
        };

        Some(Value::String(result))
    }
}

impl Default for FoldConstantExprVisitor {
    fn default() -> Self {
        Self::new()
    }
}

impl ExpressionVisitor for FoldConstantExprVisitor {
    type Result = Result<(), String>;

    fn visit_literal(&mut self, _value: &Value) -> Self::Result {
        Ok(())
    }

    fn visit_variable(&mut self, _name: &str) -> Self::Result {
        self.can_be_folded = false;
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
        if let Some(folded) = self.try_fold_binary(left, op, right) {
            self.set_folded(folded);
        } else {
            self.visit_expression(left)?;
            self.visit_expression(right)?;
        }
        Ok(())
    }

    fn visit_unary(&mut self, op: &UnaryOperator, operand: &Expression) -> Self::Result {
        if let Some(folded) = self.try_fold_unary(op, operand) {
            self.set_folded(folded);
        } else {
            self.visit_expression(operand)?;
        }
        Ok(())
    }

    fn visit_function(&mut self, name: &str, args: &[Expression]) -> Self::Result {
        let name_upper = name.to_uppercase();

        match name_upper.as_str() {
            "ABS" | "CEIL" | "FLOOR" | "SQRT" | "POW" | "EXP" | "LOG" | "LOG10" | "SIN" | "COS"
            | "TAN" | "ASIN" | "ACOS" | "ATAN" | "ROUND" => {
                if args.len() == 1 && Self::is_constant(&args[0]) {
                    if let Expression::Literal(val) = &args[0] {
                        if let Some(folded) = self.fold_math_function_internal(name, val) {
                            self.set_folded(Expression::Literal(folded));
                            return Ok(());
                        }
                    }
                }
            }
            "LOWER" | "UPPER" | "TRIM" | "LTRIM" | "RTRIM" | "REVERSE" => {
                if args.len() == 1 && Self::is_constant(&args[0]) {
                    if let Expression::Literal(val) = &args[0] {
                        if let Some(folded) = self.fold_string_function_internal(name, val) {
                            self.set_folded(Expression::Literal(folded));
                            return Ok(());
                        }
                    }
                }
            }
            _ => {}
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
        let mut all_constant = true;
        let mut folded_items = Vec::new();

        for item in items {
            if Self::is_constant(item) {
                folded_items.push(item.clone());
            } else {
                all_constant = false;
                break;
            }
        }

        if all_constant {
            self.set_folded(Expression::List(folded_items));
        } else {
            for item in items {
                self.visit_expression(item)?;
            }
        }
        Ok(())
    }

    fn visit_map(&mut self, pairs: &[(String, Expression)]) -> Self::Result {
        let mut all_constant = true;
        let mut folded_pairs = Vec::new();

        for (key, value) in pairs {
            if Self::is_constant(value) {
                folded_pairs.push((key.clone(), value.clone()));
            } else {
                all_constant = false;
                break;
            }
        }

        if all_constant {
            self.set_folded(Expression::Map(folded_pairs));
        } else {
            for (_, value) in pairs {
                self.visit_expression(value)?;
            }
        }
        Ok(())
    }

    fn visit_case(
        &mut self,
        conditions: &[(Expression, Expression)],
        default: Option<&Expression>,
    ) -> Self::Result {
        for (cond, expression) in conditions {
            self.visit_expression(cond)?;
            self.visit_expression(expression)?;
        }
        if let Some(default_expression) = default {
            self.visit_expression(default_expression)?;
        }
        Ok(())
    }

    fn visit_type_cast(&mut self, expression: &Expression, _target_type: &DataType) -> Self::Result {
        self.visit_expression(expression)
    }

    fn visit_subscript(&mut self, collection: &Expression, index: &Expression) -> Self::Result {
        self.visit_expression(collection)?;
        self.visit_expression(index)
    }

    fn visit_range(
        &mut self,
        collection: &Expression,
        start: Option<&Expression>,
        end: Option<&Expression>,
    ) -> Self::Result {
        self.visit_expression(collection)?;
        if let Some(start_expression) = start {
            self.visit_expression(start_expression)?;
        }
        if let Some(end_expression) = end {
            self.visit_expression(end_expression)?;
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

    fn visit_list_comprehension(
        &mut self,
        _variable: &str,
        source: &Expression,
        filter: Option<&Expression>,
        map: Option<&Expression>,
    ) -> Self::Result {
        self.visit_expression(source)?;
        if let Some(f) = filter {
            self.visit_expression(f)?;
        }
        if let Some(m) = map {
            self.visit_expression(m)?;
        }
        Ok(())
    }

    fn state(&self) -> &ExpressionVisitorState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut ExpressionVisitorState {
        &mut self.state
    }
}
