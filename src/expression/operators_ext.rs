//! 操作符扩展模块
//! 
//! 直接重新导出Core操作符，提供便利函数

// 直接重新导出Core操作符
pub use crate::core::types::operators::{
    BinaryOperator, UnaryOperator, AggregateFunction
};

// 便利函数用于创建操作符
impl BinaryOperator {
    /// 创建加法操作符
    pub fn add() -> Self {
        BinaryOperator::Add
    }
    
    /// 创建减法操作符
    pub fn subtract() -> Self {
        BinaryOperator::Subtract
    }
    
    /// 创建乘法操作符
    pub fn multiply() -> Self {
        BinaryOperator::Multiply
    }
    
    /// 创建除法操作符
    pub fn divide() -> Self {
        BinaryOperator::Divide
    }
    
    /// 创建等于操作符
    pub fn equal() -> Self {
        BinaryOperator::Equal
    }
    
    /// 创建不等于操作符
    pub fn not_equal() -> Self {
        BinaryOperator::NotEqual
    }
    
    /// 创建小于操作符
    pub fn less_than() -> Self {
        BinaryOperator::LessThan
    }
    
    /// 创建小于等于操作符
    pub fn less_than_or_equal() -> Self {
        BinaryOperator::LessThanOrEqual
    }
    
    /// 创建大于操作符
    pub fn greater_than() -> Self {
        BinaryOperator::GreaterThan
    }
    
    /// 创建大于等于操作符
    pub fn greater_than_or_equal() -> Self {
        BinaryOperator::GreaterThanOrEqual
    }
    
    /// 创建逻辑与操作符
    pub fn and() -> Self {
        BinaryOperator::And
    }
    
    /// 创建逻辑或操作符
    pub fn or() -> Self {
        BinaryOperator::Or
    }
    
    /// 创建字符串连接操作符
    pub fn string_concat() -> Self {
        BinaryOperator::StringConcat
    }
    
    /// 创建Like操作符
    pub fn like() -> Self {
        BinaryOperator::Like
    }
    
    /// 创建In操作符
    pub fn in_op() -> Self {
        BinaryOperator::In
    }
    
    /// 创建Xor操作符
    pub fn xor() -> Self {
        BinaryOperator::Xor
    }
    
    /// 创建NotIn操作符
    pub fn not_in() -> Self {
        BinaryOperator::NotIn
    }
    
    /// 创建Contains操作符
    pub fn contains() -> Self {
        BinaryOperator::Contains
    }
    
    /// 创建StartsWith操作符
    pub fn starts_with() -> Self {
        BinaryOperator::StartsWith
    }
    
    /// 创建EndsWith操作符
    pub fn ends_with() -> Self {
        BinaryOperator::EndsWith
    }
    
    /// 创建下标操作符
    pub fn subscript() -> Self {
        BinaryOperator::Subscript
    }
    
    /// 创建属性操作符
    pub fn attribute() -> Self {
        BinaryOperator::Attribute
    }
}

impl UnaryOperator {
    /// 创建正号操作符
    pub fn plus() -> Self {
        UnaryOperator::Plus
    }
    
    /// 创建负号操作符
    pub fn minus() -> Self {
        UnaryOperator::Minus
    }
    
    /// 创建逻辑非操作符
    pub fn not() -> Self {
        UnaryOperator::Not
    }
    
    /// 创建IsNull操作符
    pub fn is_null() -> Self {
        UnaryOperator::IsNull
    }
    
    /// 创建IsNotNull操作符
    pub fn is_not_null() -> Self {
        UnaryOperator::IsNotNull
    }
    
    /// 创建IsEmpty操作符
    pub fn is_empty() -> Self {
        UnaryOperator::IsEmpty
    }
    
    /// 创建IsNotEmpty操作符
    pub fn is_not_empty() -> Self {
        UnaryOperator::IsNotEmpty
    }
    
    /// 创建自增操作符
    pub fn increment() -> Self {
        UnaryOperator::Increment
    }
    
    /// 创建自减操作符
    pub fn decrement() -> Self {
        UnaryOperator::Decrement
    }
}

impl AggregateFunction {
    /// 创建Count聚合函数
    pub fn count() -> Self {
        AggregateFunction::Count
    }
    
    /// 创建Sum聚合函数
    pub fn sum() -> Self {
        AggregateFunction::Sum
    }
    
    /// 创建Avg聚合函数
    pub fn avg() -> Self {
        AggregateFunction::Avg
    }
    
    /// 创建Min聚合函数
    pub fn min() -> Self {
        AggregateFunction::Min
    }
    
    /// 创建Max聚合函数
    pub fn max() -> Self {
        AggregateFunction::Max
    }
    
    /// 创建Collect聚合函数
    pub fn collect() -> Self {
        AggregateFunction::Collect
    }
    
    /// 创建Distinct聚合函数
    pub fn distinct() -> Self {
        AggregateFunction::Distinct
    }
}