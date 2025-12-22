//! 扩展操作符定义
//!
//! 提供对Core操作符的扩展，支持图数据库特有的操作符

use serde::{Deserialize, Serialize};
use crate::core::types::operators::BinaryOperator as CoreBinaryOperator;
use crate::core::types::operators::UnaryOperator as CoreUnaryOperator;
use crate::core::types::operators::AggregateFunction as CoreAggregateFunction;

/// 扩展二元操作符
/// 
/// 包含Core操作符和Expression模块特有的操作符
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExtendedBinaryOperator {
    /// Core基础操作符
    Core(CoreBinaryOperator),
    
    /// Expression模块扩展操作符
    Xor,                    // 异或操作
    NotIn,                  // 不在集合中
    Subscript,              // 下标访问
    Attribute,              // 属性访问
    Contains,               // 包含检查
    StartsWith,             // 前缀匹配
    EndsWith,               // 后缀匹配
}

/// 扩展一元操作符
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExtendedUnaryOperator {
    /// Core基础操作符
    Core(CoreUnaryOperator),
}

/// 扩展聚合函数
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExtendedAggregateFunction {
    /// Core基础聚合函数
    Core(CoreAggregateFunction),
}

impl ExtendedBinaryOperator {
    /// 获取操作符名称
    pub fn name(&self) -> &str {
        match self {
            ExtendedBinaryOperator::Core(core_op) => {
                use crate::core::types::operators::Operator;
                core_op.name()
            },
            ExtendedBinaryOperator::Xor => "XOR",
            ExtendedBinaryOperator::NotIn => "NOT IN",
            ExtendedBinaryOperator::Subscript => "[]",
            ExtendedBinaryOperator::Attribute => ".",
            ExtendedBinaryOperator::Contains => "CONTAINS",
            ExtendedBinaryOperator::StartsWith => "STARTS WITH",
            ExtendedBinaryOperator::EndsWith => "ENDS WITH",
        }
    }
    
    /// 获取操作符优先级
    pub fn precedence(&self) -> u8 {
        match self {
            ExtendedBinaryOperator::Core(core_op) => {
                use crate::core::types::operators::Operator;
                core_op.precedence()
            },
            ExtendedBinaryOperator::Xor => 2,        // 逻辑异或，与AND同级
            ExtendedBinaryOperator::NotIn => 4,      // 包含操作，与IN同级
            ExtendedBinaryOperator::Subscript => 9,  // 下标访问，高优先级
            ExtendedBinaryOperator::Attribute => 9,  // 属性访问，高优先级
            ExtendedBinaryOperator::Contains => 4,   // 包含检查
            ExtendedBinaryOperator::StartsWith => 4, // 前缀匹配
            ExtendedBinaryOperator::EndsWith => 4,   // 后缀匹配
        }
    }
    
    /// 检查是否是左结合的
    pub fn is_left_associative(&self) -> bool {
        match self {
            ExtendedBinaryOperator::Core(core_op) => {
                use crate::core::types::operators::Operator;
                core_op.is_left_associative()
            },
            ExtendedBinaryOperator::Xor => true,
            ExtendedBinaryOperator::NotIn => true,
            ExtendedBinaryOperator::Subscript => true,
            ExtendedBinaryOperator::Attribute => true,
            ExtendedBinaryOperator::Contains => true,
            ExtendedBinaryOperator::StartsWith => true,
            ExtendedBinaryOperator::EndsWith => true,
        }
    }
    
    /// 检查是否是算术操作符
    pub fn is_arithmetic(&self) -> bool {
        match self {
            ExtendedBinaryOperator::Core(core_op) => core_op.is_arithmetic(),
            _ => false,
        }
    }
    
    /// 检查是否是比较操作符
    pub fn is_comparison(&self) -> bool {
        match self {
            ExtendedBinaryOperator::Core(core_op) => core_op.is_comparison(),
            ExtendedBinaryOperator::NotIn => true,
            ExtendedBinaryOperator::Contains => true,
            ExtendedBinaryOperator::StartsWith => true,
            ExtendedBinaryOperator::EndsWith => true,
            _ => false,
        }
    }
    
    /// 检查是否是逻辑操作符
    pub fn is_logical(&self) -> bool {
        match self {
            ExtendedBinaryOperator::Core(core_op) => core_op.is_logical(),
            ExtendedBinaryOperator::Xor => true,
            _ => false,
        }
    }
}

impl From<CoreBinaryOperator> for ExtendedBinaryOperator {
    fn from(core_op: CoreBinaryOperator) -> Self {
        ExtendedBinaryOperator::Core(core_op)
    }
}

impl From<CoreUnaryOperator> for ExtendedUnaryOperator {
    fn from(core_op: CoreUnaryOperator) -> Self {
        ExtendedUnaryOperator::Core(core_op)
    }
}

impl From<CoreAggregateFunction> for ExtendedAggregateFunction {
    fn from(core_op: CoreAggregateFunction) -> Self {
        ExtendedAggregateFunction::Core(core_op)
    }
}

/// 为了向后兼容，提供类型别名
pub type BinaryOperator = ExtendedBinaryOperator;
pub type UnaryOperator = ExtendedUnaryOperator;
pub type AggregateFunction = ExtendedAggregateFunction;

/// 操作符转换工具
pub mod conversion {
    use super::*;
    use crate::core::types::operators::BinaryOperator as CoreBinOp;
    
    /// 从旧的expression::binary::BinaryOperator转换
    pub fn from_legacy_binary_operator(
        legacy_op: &crate::expression::binary::LegacyBinaryOperator,
    ) -> ExtendedBinaryOperator {
        use crate::expression::binary::LegacyBinaryOperator as Legacy;
        
        match legacy_op {
            Legacy::Add => ExtendedBinaryOperator::Core(CoreBinOp::Add),
            Legacy::Sub => ExtendedBinaryOperator::Core(CoreBinOp::Subtract),
            Legacy::Mul => ExtendedBinaryOperator::Core(CoreBinOp::Multiply),
            Legacy::Div => ExtendedBinaryOperator::Core(CoreBinOp::Divide),
            Legacy::Mod => ExtendedBinaryOperator::Core(CoreBinOp::Modulo),
            Legacy::Eq => ExtendedBinaryOperator::Core(CoreBinOp::Equal),
            Legacy::Ne => ExtendedBinaryOperator::Core(CoreBinOp::NotEqual),
            Legacy::Lt => ExtendedBinaryOperator::Core(CoreBinOp::LessThan),
            Legacy::Le => ExtendedBinaryOperator::Core(CoreBinOp::LessThanOrEqual),
            Legacy::Gt => ExtendedBinaryOperator::Core(CoreBinOp::GreaterThan),
            Legacy::Ge => ExtendedBinaryOperator::Core(CoreBinOp::GreaterThanOrEqual),
            Legacy::And => ExtendedBinaryOperator::Core(CoreBinOp::And),
            Legacy::Or => ExtendedBinaryOperator::Core(CoreBinOp::Or),
            Legacy::In => ExtendedBinaryOperator::Core(CoreBinOp::In),
            Legacy::Xor => ExtendedBinaryOperator::Xor,
            Legacy::NotIn => ExtendedBinaryOperator::NotIn,
            Legacy::Subscript => ExtendedBinaryOperator::Subscript,
            Legacy::Attribute => ExtendedBinaryOperator::Attribute,
            Legacy::Contains => ExtendedBinaryOperator::Contains,
            Legacy::StartsWith => ExtendedBinaryOperator::StartsWith,
            Legacy::EndsWith => ExtendedBinaryOperator::EndsWith,
        }
    }
    
    /// 转换为Core操作符（如果可能）
    pub fn to_core_binary_operator(
        extended_op: &ExtendedBinaryOperator,
    ) -> Option<CoreBinaryOperator> {
        match extended_op {
            ExtendedBinaryOperator::Core(core_op) => Some(core_op.clone()),
            _ => None,
        }
    }
}