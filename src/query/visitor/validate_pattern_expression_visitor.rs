//! ValidatePatternExpressionVisitor - 用于验证模式表达式的访问器=
//!
//! 主要功能：
//! - 验证列表推导表达式的变量作用域
//! - 验证路径模式表达式的合法性
//! - 处理局部变量定义
//! - 检查变量冲突

use crate::core::types::expression::Expression;
use crate::core::{
    expression_visitor::{ExpressionVisitor, ExpressionVisitorState},
    BinaryOperator, DataType, UnaryOperator, Value,
};
use crate::core::types::operators::AggregateFunction;

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
    pub fn validate(&mut self, expression: &Expression) -> Result<(), String> {
        self.local_variables.clear();
        self.error = None;

        self.visit_expression(expression)?;

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

    /// 检查变量是否为局部变量
    fn is_local_variable(&self, var: &str) -> bool {
        self.local_variables.contains(&var.to_string())
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

    fn visit_property(&mut self, object: &Expression, _property: &str) -> Self::Result {
        self.visit_expression(object)
    }

    fn visit_binary(
        &mut self,
        left: &Expression,
        _op: &BinaryOperator,
        right: &Expression,
    ) -> Self::Result {
        self.visit_expression(left)?;
        self.visit_expression(right)
    }

    fn visit_unary(&mut self, _op: &UnaryOperator, operand: &Expression) -> Self::Result {
        self.visit_expression(operand)
    }

    fn visit_function(&mut self, name: &str, args: &[Expression]) -> Self::Result {
        let name_upper = name.to_uppercase();

        match name_upper.as_str() {
            "HAS" | "HASLABEL" | "HASTAG" => {
                if args.len() >= 1 {
                    if let Expression::Variable(var) = &args[0] {
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
        for (_, expression) in pairs {
            self.visit_expression(expression)?;
        }
        Ok(())
    }

    fn visit_case(
        &mut self,
        conditions: &[(Expression, Expression)],
        default: &Option<Box<Expression>>,
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
        start: &Option<Box<Expression>>,
        end: &Option<Box<Expression>>,
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

    fn state(&self) -> &ExpressionVisitorState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut ExpressionVisitorState {
        &mut self.state
    }
}
