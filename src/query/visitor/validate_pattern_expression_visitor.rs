//! ValidatePatternExpressionVisitor - 用于验证模式表达式的访问器=
//!
//! 主要功能：
//! - 验证列表推导表达式的变量作用域
//! - 验证路径模式表达式的合法性
//! - 处理局部变量定义
//! - 检查变量冲突

use crate::core::{
    expression_visitor::{ExpressionVisitor, ExpressionVisitorState},
    BinaryOperator, DataType, Expression, UnaryOperator, Value,
};
use crate::core::types::operators::AggregateFunction;
use crate::expression::Expr;
use crate::query::parser::ast::expr::*;

/// 模式表达式验证访问器
///
/// 用于验证模式表达式的合法性，处理变量作用域
#[derive(Debug)]
pub struct ValidatePatternExpressionVisitor {
    /// 局部变量集合
    local_variables: Vec<String>,
    /// 错误状态
    error: Option<String>,
    /// 访问者状态
    state: ExpressionVisitorState,
}

impl ValidatePatternExpressionVisitor {
    /// 创建新的模式表达式验证访问器
    pub fn new() -> Self {
        Self {
            local_variables: Vec::new(),
            error: None,
            state: ExpressionVisitorState::new(),
        }
    }

    /// 验证表达式
    pub fn validate(&mut self, expr: &Expr) -> Result<(), String> {
        self.local_variables.clear();
        self.error = None;

        self.visit_expression(expr)?;

        if let Some(err) = &self.error {
            Err(err.clone())
        } else {
            Ok(())
        }
    }

    /// 获取错误信息
    pub fn get_error(&self) -> Option<&String> {
        self.error.as_ref()
    }

    /// 设置错误信息
    fn set_error(&mut self, error: String) {
        self.error = Some(error);
    }

    /// 添加局部变量
    fn add_local_variable(&mut self, var: &str) {
        if !self.local_variables.contains(&var.to_string()) {
            self.local_variables.push(var.to_string());
        }
    }

    /// 移除局部变量
    fn remove_local_variable(&mut self, var: &str) {
        if let Some(pos) = self.local_variables.iter().position(|v| v == var) {
            self.local_variables.remove(pos);
        }
    }

    /// 检查变量是否为局部变量
    fn is_local_variable(&self, var: &str) -> bool {
        self.local_variables.contains(&var.to_string())
    }

    /// 将多个表达式用 AND 连接
    fn and_all(&self, exprs: &[Expression]) -> Option<Expression> {
        if exprs.is_empty() {
            return None;
        }

        if exprs.len() == 1 {
            return Some(exprs[0].clone());
        }

        let mut result = exprs[0].clone();
        for expr in &exprs[1..] {
            result = Expr::Binary {
                left: Box::new(result),
                op: BinaryOperator::And,
                right: Box::new(expr.clone()),
            };
        }

        Some(result)
    }
}

impl Default for ValidatePatternExpressionVisitor {
    fn default() -> Self {
        Self::new()
    }
}

impl ExpressionVisitor for ValidatePatternExpressionVisitor {
    type Result = Result<(), String>;

    fn visit_literal(&mut self, _value: &Value) -> Self::Result {
        Ok(())
    }

    fn visit_variable(&mut self, _name: &str) -> Self::Result {
        Ok(())
    }

    fn visit_property(&mut self, object: &Expr, _property: &str) -> Self::Result {
        self.visit_expression(object)
    }

    fn visit_binary(
        &mut self,
        left: &Expr,
        _op: &BinaryOperator,
        right: &Expr,
    ) -> Self::Result {
        self.visit_expression(left)?;
        self.visit_expression(right)
    }

    fn visit_unary(&mut self, _op: &UnaryOperator, operand: &Expr) -> Self::Result {
        self.visit_expression(operand)
    }

    fn visit_function(&mut self, name: &str, args: &[Expression]) -> Self::Result {
        let name_upper = name.to_uppercase();

        match name_upper.as_str() {
            "HAS" | "HASLABEL" | "HASTAG" => {
                if args.len() >= 1 {
                    if let Expr::Variable(var) = &args[0] {
                        if self.is_local_variable(var) {
                            self.set_error(format!(
                                "函数 {} 的参数不能是局部变量: {}",
                                name, var
                            ));
                            return Err(self.error.clone().unwrap());
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
        arg: &Expr,
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
        conditions: &[(Expr, Expr)],
        default: &Option<Box<Expr>>,
    ) -> Self::Result {
        for (cond, expr) in conditions {
            self.visit_expression(cond)?;
            self.visit_expression(expr)?;
        }
        if let Some(default_expr) = default {
            self.visit_expression(default_expr)?;
        }
        Ok(())
    }

    fn visit_type_cast(&mut self, expr: &Expr, _target_type: &DataType) -> Self::Result {
        self.visit_expression(expr)
    }

    fn visit_subscript(&mut self, collection: &Expr, index: &Expr) -> Self::Result {
        self.visit_expression(collection)?;
        self.visit_expression(index)
    }

    fn visit_range(
        &mut self,
        collection: &Expr,
        start: &Option<Box<Expr>>,
        end: &Option<Box<Expr>>,
    ) -> Self::Result {
        self.visit_expression(collection)?;
        if let Some(start_expr) = start {
            self.visit_expression(start_expr)?;
        }
        if let Some(end_expr) = end {
            self.visit_expression(end_expr)?;
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
}
