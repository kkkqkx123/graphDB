//! 表达式错误类型
//!
//! 包含错误类型、错误消息和可选的位置信息
//! 支持序列化/反序列化，用于跨模块传递

use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// 表达式错误（结构化设计）
#[derive(Error, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExpressionError {
    /// 错误类型
    pub error_type: ExpressionErrorType,
    /// 错误消息
    pub message: String,
    /// 错误位置
    pub position: Option<ExpressionPosition>,
}

/// 表达式错误类型枚举
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
    /// 不支持的操作
    UnsupportedOperation,
    /// 类型转换错误
    TypeConversionError,
    /// 操作符错误
    OperatorError,
    /// 标签未找到
    LabelNotFound,
    /// 边未找到
    EdgeNotFound,
    /// 路径错误
    PathError,
    /// 范围错误
    RangeError,
    /// 聚合函数错误
    AggregateError,
    /// 验证错误
    ValidationError,
}

impl std::fmt::Display for ExpressionErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExpressionErrorType::TypeError => write!(f, "Type error"),
            ExpressionErrorType::UndefinedVariable => write!(f, "Undefined variable"),
            ExpressionErrorType::UndefinedFunction => write!(f, "Undefined function"),
            ExpressionErrorType::UnknownFunction => write!(f, "Unknown function"),
            ExpressionErrorType::FunctionError => write!(f, "Function error"),
            ExpressionErrorType::ArgumentCountError => write!(f, "Argument count error"),
            ExpressionErrorType::InvalidArgumentCount => write!(f, "Invalid argument count"),
            ExpressionErrorType::Overflow => write!(f, "Overflow error"),
            ExpressionErrorType::IndexOutOfBounds => write!(f, "Index out of bounds"),
            ExpressionErrorType::NullError => write!(f, "Null error"),
            ExpressionErrorType::SyntaxError => write!(f, "Syntax error"),
            ExpressionErrorType::InvalidOperation => write!(f, "Invalid operation"),
            ExpressionErrorType::PropertyNotFound => write!(f, "Property not found"),
            ExpressionErrorType::RuntimeError => write!(f, "Runtime error"),
            ExpressionErrorType::UnsupportedOperation => write!(f, "Unsupported operation"),
            ExpressionErrorType::TypeConversionError => write!(f, "Type conversion error"),
            ExpressionErrorType::OperatorError => write!(f, "Operator error"),
            ExpressionErrorType::LabelNotFound => write!(f, "Label not found"),
            ExpressionErrorType::EdgeNotFound => write!(f, "Edge not found"),
            ExpressionErrorType::PathError => write!(f, "Path error"),
            ExpressionErrorType::RangeError => write!(f, "Range error"),
            ExpressionErrorType::AggregateError => write!(f, "Aggregate error"),
            ExpressionErrorType::ValidationError => write!(f, "Validation error"),
        }
    }
}

/// 表达式错误位置信息
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

    /// 创建类型错误
    pub fn type_error(message: impl Into<String>) -> Self {
        Self::new(ExpressionErrorType::TypeError, message)
    }

    /// 创建未定义变量错误
    pub fn undefined_variable(name: impl Into<String>) -> Self {
        Self::new(
            ExpressionErrorType::UndefinedVariable,
            format!("Undefined variable: {}", name.into()),
        )
    }

    /// 创建未定义函数错误
    pub fn undefined_function(name: impl Into<String>) -> Self {
        Self::new(
            ExpressionErrorType::UndefinedFunction,
            format!("Undefined function: {}", name.into()),
        )
    }

    /// 创建参数数量错误
    pub fn argument_count_error(expected: usize, actual: usize) -> Self {
        Self::new(
            ExpressionErrorType::ArgumentCountError,
            format!("Argument count error: expected {}, got {}", expected, actual),
        )
    }

    /// 创建溢出错误
    pub fn overflow(message: impl Into<String>) -> Self {
        Self::new(ExpressionErrorType::Overflow, message)
    }

    /// 创建索引越界错误
    pub fn index_out_of_bounds(index: isize, size: usize) -> Self {
        Self::new(
            ExpressionErrorType::IndexOutOfBounds,
            format!("Index out of bounds: index {}, size {}", index, size),
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
            format!("Unknown function: {}", name.into()),
        )
    }

    /// 创建无效参数数量错误
    pub fn invalid_argument_count(name: impl Into<String>) -> Self {
        Self::new(
            ExpressionErrorType::InvalidArgumentCount,
            format!("Invalid argument count: {}", name.into()),
        )
    }

    /// 创建不支持的操作错误
    pub fn unsupported_operation(
        operation: impl Into<String>,
        suggestion: impl Into<String>,
    ) -> Self {
        Self::new(
            ExpressionErrorType::UnsupportedOperation,
            format!(
                "Unsupported operation: {}, suggestion: {}",
                operation.into(),
                suggestion.into()
            ),
        )
    }

    /// 创建类型转换错误
    pub fn type_conversion_error(from_type: impl Into<String>, to_type: impl Into<String>) -> Self {
        Self::new(
            ExpressionErrorType::TypeConversionError,
            format!(
                "Type conversion error: cannot convert from {} to {}",
                from_type.into(),
                to_type.into()
            ),
        )
    }

    /// 创建操作符错误
    pub fn operator_error(operator: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(
            ExpressionErrorType::OperatorError,
            format!("Operator error: {}: {}", operator.into(), message.into()),
        )
    }

    /// 创建标签未找到错误
    pub fn label_not_found(label: impl Into<String>) -> Self {
        Self::new(
            ExpressionErrorType::LabelNotFound,
            format!("Label not found: {}", label.into()),
        )
    }

    /// 创建边未找到错误
    pub fn edge_not_found(edge: impl Into<String>) -> Self {
        Self::new(
            ExpressionErrorType::EdgeNotFound,
            format!("Edge not found: {}", edge.into()),
        )
    }

    /// 创建路径错误
    pub fn path_error(message: impl Into<String>) -> Self {
        Self::new(ExpressionErrorType::PathError, message)
    }

    /// 创建范围错误
    pub fn range_error(message: impl Into<String>) -> Self {
        Self::new(ExpressionErrorType::RangeError, message)
    }

    /// 创建聚合函数错误
    pub fn aggregate_error(message: impl Into<String>) -> Self {
        Self::new(ExpressionErrorType::AggregateError, message)
    }

    /// 创建验证错误
    pub fn validation_error(message: impl Into<String>) -> Self {
        Self::new(ExpressionErrorType::ValidationError, message)
    }
}

impl fmt::Display for ExpressionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {}", self.error_type, self.message)
    }
}
