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
            ExpressionErrorType::TypeError => write!(f, "类型错误"),
            ExpressionErrorType::UndefinedVariable => write!(f, "未定义变量"),
            ExpressionErrorType::UndefinedFunction => write!(f, "未定义函数"),
            ExpressionErrorType::UnknownFunction => write!(f, "未知函数"),
            ExpressionErrorType::FunctionError => write!(f, "函数错误"),
            ExpressionErrorType::ArgumentCountError => write!(f, "参数数量错误"),
            ExpressionErrorType::InvalidArgumentCount => write!(f, "无效参数数量"),
            ExpressionErrorType::Overflow => write!(f, "溢出错误"),
            ExpressionErrorType::IndexOutOfBounds => write!(f, "索引越界"),
            ExpressionErrorType::NullError => write!(f, "空值错误"),
            ExpressionErrorType::SyntaxError => write!(f, "语法错误"),
            ExpressionErrorType::InvalidOperation => write!(f, "无效操作"),
            ExpressionErrorType::PropertyNotFound => write!(f, "属性未找到"),
            ExpressionErrorType::RuntimeError => write!(f, "运行时错误"),
            ExpressionErrorType::UnsupportedOperation => write!(f, "不支持的操作"),
            ExpressionErrorType::TypeConversionError => write!(f, "类型转换错误"),
            ExpressionErrorType::OperatorError => write!(f, "操作符错误"),
            ExpressionErrorType::LabelNotFound => write!(f, "标签未找到"),
            ExpressionErrorType::EdgeNotFound => write!(f, "边未找到"),
            ExpressionErrorType::PathError => write!(f, "路径错误"),
            ExpressionErrorType::RangeError => write!(f, "范围错误"),
            ExpressionErrorType::AggregateError => write!(f, "聚合函数错误"),
            ExpressionErrorType::ValidationError => write!(f, "验证错误"),
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

    /// 创建不支持的操作错误
    pub fn unsupported_operation(
        operation: impl Into<String>,
        suggestion: impl Into<String>,
    ) -> Self {
        Self::new(
            ExpressionErrorType::UnsupportedOperation,
            format!(
                "不支持的操作: {}, 建议: {}",
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
                "类型转换错误: 无法从 {} 转换为 {}",
                from_type.into(),
                to_type.into()
            ),
        )
    }

    /// 创建操作符错误
    pub fn operator_error(operator: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(
            ExpressionErrorType::OperatorError,
            format!("操作符错误: {}: {}", operator.into(), message.into()),
        )
    }

    /// 创建标签未找到错误
    pub fn label_not_found(label: impl Into<String>) -> Self {
        Self::new(
            ExpressionErrorType::LabelNotFound,
            format!("标签未找到: {}", label.into()),
        )
    }

    /// 创建边未找到错误
    pub fn edge_not_found(edge: impl Into<String>) -> Self {
        Self::new(
            ExpressionErrorType::EdgeNotFound,
            format!("边未找到: {}", edge.into()),
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
