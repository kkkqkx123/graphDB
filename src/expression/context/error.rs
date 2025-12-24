//! 表达式错误处理模块
//!
//! 提供表达式求值过程中的错误定义和处理

use crate::core::types::expression::Expression;
use serde::{Deserialize, Serialize};
use std::fmt;

/// 表达式错误
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExpressionError {
    /// 错误类型
    pub error_type: ExpressionErrorType,
    /// 错误消息
    pub message: String,
    /// 错误位置
    pub position: Option<ExpressionPosition>,
    /// 相关表达式
    pub expression: Option<Expression>,
}

/// 表达式错误类型
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExpressionErrorType {
    /// 类型错误
    TypeError,
    /// 未定义变量
    UndefinedVariable,
    /// 未定义函数
    UndefinedFunction,
    /// 未知函数
    UnknownFunction,
    /// 函数错误
    FunctionError,
    /// 参数数量错误
    ArgumentCountError,
    /// 无效参数数量
    InvalidArgumentCount,
    /// 除零错误
    DivisionByZero,
    /// 溢出错误
    Overflow,
    /// 索引越界
    IndexOutOfBounds,
    /// 空值错误
    NullError,
    /// 语法错误
    SyntaxError,
    /// 无效操作
    InvalidOperation,
    /// 属性未找到
    PropertyNotFound,
    /// 运行时错误
    RuntimeError,
}

/// 表达式位置
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExpressionPosition {
    /// 行号
    pub line: usize,
    /// 列号
    pub column: usize,
    /// 偏移量
    pub offset: usize,
    /// 长度
    pub length: usize,
}

impl ExpressionError {
    /// 创建新的表达式错误
    pub fn new(error_type: ExpressionErrorType, message: impl Into<String>) -> Self {
        Self {
            error_type,
            message: message.into(),
            position: None,
            expression: None,
        }
    }

    /// 设置错误位置
    pub fn with_position(
        mut self,
        line: usize,
        column: usize,
        offset: usize,
        length: usize,
    ) -> Self {
        self.position = Some(ExpressionPosition {
            line,
            column,
            offset,
            length,
        });
        self
    }

    /// 设置相关表达式
    pub fn with_expression(mut self, expression: Expression) -> Self {
        self.expression = Some(expression);
        self
    }

    /// 创建类型错误
    pub fn type_error(message: impl Into<String>) -> Self {
        Self::new(ExpressionErrorType::TypeError, message)
    }

    /// 创建未定义变量错误
    pub fn undefined_variable(name: impl Into<String>) -> Self {
        Self::new(
            ExpressionErrorType::UndefinedVariable,
            format!("未定义的变量: {}", name.into()),
        )
    }

    /// 创建未定义函数错误
    pub fn undefined_function(name: impl Into<String>) -> Self {
        Self::new(
            ExpressionErrorType::UndefinedFunction,
            format!("未定义的函数: {}", name.into()),
        )
    }

    /// 创建参数数量错误
    pub fn argument_count_error(expected: usize, actual: usize) -> Self {
        Self::new(
            ExpressionErrorType::ArgumentCountError,
            format!("参数数量错误: 期望 {}, 实际 {}", expected, actual),
        )
    }

    /// 创建除零错误
    pub fn division_by_zero() -> Self {
        Self::new(ExpressionErrorType::DivisionByZero, "除零错误".to_string())
    }

    /// 创建溢出错误
    pub fn overflow(message: impl Into<String>) -> Self {
        Self::new(ExpressionErrorType::Overflow, message)
    }

    /// 创建索引越界错误
    pub fn index_out_of_bounds(index: isize, size: usize) -> Self {
        Self::new(
            ExpressionErrorType::IndexOutOfBounds,
            format!("索引越界: 索引 {}, 大小 {}", index, size),
        )
    }

    /// 创建空值错误
    pub fn null_error(message: impl Into<String>) -> Self {
        Self::new(ExpressionErrorType::NullError, message)
    }

    /// 创建语法错误
    pub fn syntax_error(message: impl Into<String>) -> Self {
        Self::new(ExpressionErrorType::SyntaxError, message)
    }

    /// 创建运行时错误
    pub fn runtime_error(message: impl Into<String>) -> Self {
        Self::new(ExpressionErrorType::RuntimeError, message)
    }

    /// 创建函数错误
    pub fn function_error(message: impl Into<String>) -> Self {
        Self::new(ExpressionErrorType::FunctionError, message)
    }

    /// 创建无效操作错误
    pub fn invalid_operation(message: impl Into<String>) -> Self {
        Self::new(ExpressionErrorType::InvalidOperation, message)
    }

    /// 创建属性未找到错误
    pub fn property_not_found(message: impl Into<String>) -> Self {
        Self::new(ExpressionErrorType::PropertyNotFound, message)
    }

    /// 创建未知函数错误
    pub fn unknown_function(name: impl Into<String>) -> Self {
        Self::new(
            ExpressionErrorType::UnknownFunction,
            format!("未知函数: {}", name.into()),
        )
    }

    /// 创建无效参数数量错误
    pub fn invalid_argument_count(name: impl Into<String>) -> Self {
        Self::new(
            ExpressionErrorType::InvalidArgumentCount,
            format!("无效参数数量: {}", name.into()),
        )
    }
}

impl fmt::Display for ExpressionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {}", self.error_type, self.message)
    }
}
