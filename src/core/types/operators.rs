//! 操作符类型定义
//!
//! 定义图数据库中使用的各种操作符类型和接口

use serde::{Deserialize, Serialize};

/// 操作符特征定义
pub trait Operator {
    /// 获取操作符的名称
    fn name(&self) -> &str;

    /// 获取操作符的优先级
    fn precedence(&self) -> u8;

    /// 检查操作符是否是左结合的
    fn is_left_associative(&self) -> bool;

    /// 获取操作符的元数（操作数数量）
    fn arity(&self) -> usize;
}

/// 操作符注册表
#[derive(Debug)]
pub struct OperatorRegistry {
    operators: Vec<OperatorInstance>,
}

/// 操作符实例，使用枚举避免动态分发
#[derive(Debug, Clone)]
pub enum OperatorInstance {
    Binary(BinaryOperator),
    Unary(UnaryOperator),
    Aggregate(AggregateFunction),
}

impl Operator for OperatorInstance {
    fn name(&self) -> &str {
        match self {
            OperatorInstance::Binary(op) => op.name(),
            OperatorInstance::Unary(op) => op.name(),
            OperatorInstance::Aggregate(op) => op.name(),
        }
    }

    fn precedence(&self) -> u8 {
        match self {
            OperatorInstance::Binary(op) => op.precedence(),
            OperatorInstance::Unary(op) => op.precedence(),
            OperatorInstance::Aggregate(op) => op.precedence(),
        }
    }

    fn is_left_associative(&self) -> bool {
        match self {
            OperatorInstance::Binary(op) => op.is_left_associative(),
            OperatorInstance::Unary(op) => op.is_left_associative(),
            OperatorInstance::Aggregate(op) => op.is_left_associative(),
        }
    }

    fn arity(&self) -> usize {
        match self {
            OperatorInstance::Binary(op) => op.arity(),
            OperatorInstance::Unary(op) => op.arity(),
            OperatorInstance::Aggregate(op) => op.arity(),
        }
    }
}

impl OperatorRegistry {
    /// 创建新的操作符注册表
    pub fn new() -> Self {
        Self {
            operators: Vec::new(),
        }
    }

    /// 注册二元操作符
    pub fn register_binary(&mut self, operator: BinaryOperator) {
        self.operators.push(OperatorInstance::Binary(operator));
    }

    /// 注册一元操作符
    pub fn register_unary(&mut self, operator: UnaryOperator) {
        self.operators.push(OperatorInstance::Unary(operator));
    }

    /// 注册聚合函数
    pub fn register_aggregate(&mut self, operator: AggregateFunction) {
        self.operators.push(OperatorInstance::Aggregate(operator));
    }

    /// 注册操作符实例
    pub fn register(&mut self, operator: OperatorInstance) {
        self.operators.push(operator);
    }

    /// 根据名称查找操作符
    pub fn find_by_name(&self, name: &str) -> Option<&OperatorInstance> {
        self.operators.iter().find(|op| op.name() == name)
    }

    /// 获取所有操作符
    pub fn get_all(&self) -> &[OperatorInstance] {
        &self.operators
    }

    /// 获取所有二元操作符
    pub fn get_binary_operators(&self) -> Vec<&BinaryOperator> {
        self.operators
            .iter()
            .filter_map(|op| {
                if let OperatorInstance::Binary(binary_op) = op {
                    Some(binary_op)
                } else {
                    None
                }
            })
            .collect()
    }

    /// 获取所有一元操作符
    pub fn get_unary_operators(&self) -> Vec<&UnaryOperator> {
        self.operators
            .iter()
            .filter_map(|op| {
                if let OperatorInstance::Unary(unary_op) = op {
                    Some(unary_op)
                } else {
                    None
                }
            })
            .collect()
    }

    /// 获取所有聚合函数
    pub fn get_aggregate_functions(&self) -> Vec<&AggregateFunction> {
        self.operators
            .iter()
            .filter_map(|op| {
                if let OperatorInstance::Aggregate(agg_func) = op {
                    Some(agg_func)
                } else {
                    None
                }
            })
            .collect()
    }
}

impl Default for OperatorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for OperatorRegistry {
    fn clone(&self) -> Self {
        Self {
            operators: self.operators.clone(),
        }
    }
}

/// 二元操作符实现
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinaryOperator {
    // 算术操作
    Add,
    Subtract, // Sub
    Multiply, // Mul
    Divide,   // Div
    Modulo,   // Mod
    Exponent, // Exp

    // 比较操作
    Equal,              // Eq
    NotEqual,           // Ne
    LessThan,           // Lt
    LessThanOrEqual,    // Le
    GreaterThan,        // Gt
    GreaterThanOrEqual, // Ge

    // 逻辑操作
    And,
    Or,
    Xor, // 异或操作

    // 字符串操作
    StringConcat,
    Like, // Regex
    In,
    NotIn,      // 不在集合中
    Contains,   // 包含检查
    StartsWith, // 前缀匹配
    EndsWith,   // 后缀匹配

    // 访问操作
    Subscript, // 下标访问
    Attribute, // 属性访问

    // 集合操作
    Union,
    Intersect,
    Except,
}

impl Operator for BinaryOperator {
    fn name(&self) -> &str {
        match self {
            BinaryOperator::Add => "+",
            BinaryOperator::Subtract => "-",
            BinaryOperator::Multiply => "*",
            BinaryOperator::Divide => "/",
            BinaryOperator::Modulo => "%",
            BinaryOperator::Exponent => "**",
            BinaryOperator::Equal => "==",
            BinaryOperator::NotEqual => "!=",
            BinaryOperator::LessThan => "<",
            BinaryOperator::LessThanOrEqual => "<=",
            BinaryOperator::GreaterThan => ">",
            BinaryOperator::GreaterThanOrEqual => ">=",
            BinaryOperator::And => "AND",
            BinaryOperator::Or => "OR",
            BinaryOperator::Xor => "XOR",
            BinaryOperator::StringConcat => "||",
            BinaryOperator::Like => "=~", // Regex
            BinaryOperator::In => "IN",
            BinaryOperator::NotIn => "NOT IN",
            BinaryOperator::Contains => "CONTAINS",
            BinaryOperator::StartsWith => "STARTS WITH",
            BinaryOperator::EndsWith => "ENDS WITH",
            BinaryOperator::Subscript => "[]",
            BinaryOperator::Attribute => ".",
            BinaryOperator::Union => "UNION",
            BinaryOperator::Intersect => "INTERSECT",
            BinaryOperator::Except => "EXCEPT",
        }
    }

    fn precedence(&self) -> u8 {
        match self {
            // 优先级 1: 逻辑或
            BinaryOperator::Or => 1,

            // 优先级 2: 逻辑与和异或
            BinaryOperator::And | BinaryOperator::Xor => 2,

            // 优先级 3: 比较操作
            BinaryOperator::Equal
            | BinaryOperator::NotEqual
            | BinaryOperator::LessThan
            | BinaryOperator::LessThanOrEqual
            | BinaryOperator::GreaterThan
            | BinaryOperator::GreaterThanOrEqual => 3,

            // 优先级 4: 包含和匹配
            BinaryOperator::In
            | BinaryOperator::NotIn
            | BinaryOperator::Like
            | BinaryOperator::Contains
            | BinaryOperator::StartsWith
            | BinaryOperator::EndsWith => 4,

            // 优先级 5: 集合操作
            BinaryOperator::Union | BinaryOperator::Intersect | BinaryOperator::Except => 5,

            // 优先级 6: 加减法
            BinaryOperator::Add | BinaryOperator::Subtract => 6,

            // 优先级 7: 乘除法、取模和指数
            BinaryOperator::Multiply | BinaryOperator::Divide | BinaryOperator::Modulo => 7,
            BinaryOperator::Exponent => 8, // 指数运算优先级更高

            // 优先级 8: 字符串连接 (调整为优先级9)
            BinaryOperator::StringConcat => 9,

            // 优先级 9: 访问操作 (调整为优先级10)
            BinaryOperator::Subscript | BinaryOperator::Attribute => 10,
        }
    }

    fn is_left_associative(&self) -> bool {
        match self {
            // 大多数二元操作符都是左结合的
            BinaryOperator::Add | BinaryOperator::Subtract |
            BinaryOperator::Multiply | BinaryOperator::Divide | BinaryOperator::Modulo |
            BinaryOperator::Exponent |  // 指数运算通常是右结合的，但这里按左结合处理
            BinaryOperator::Equal | BinaryOperator::NotEqual |
            BinaryOperator::LessThan | BinaryOperator::LessThanOrEqual |
            BinaryOperator::GreaterThan | BinaryOperator::GreaterThanOrEqual |
            BinaryOperator::And | BinaryOperator::Or | BinaryOperator::Xor |
            BinaryOperator::StringConcat | BinaryOperator::Like | BinaryOperator::In |
            BinaryOperator::NotIn | BinaryOperator::Contains |
            BinaryOperator::StartsWith | BinaryOperator::EndsWith |
            BinaryOperator::Subscript | BinaryOperator::Attribute |
            BinaryOperator::Union | BinaryOperator::Intersect | BinaryOperator::Except => true,
        }
    }

    fn arity(&self) -> usize {
        2 // 二元操作符总是需要两个操作数
    }
}

/// 一元操作符实现
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnaryOperator {
    // 算术操作
    Plus,
    Minus,

    // 逻辑操作
    Not,

    // 存在性检查
    IsNull,
    IsNotNull,
    IsEmpty,
    IsNotEmpty,

    // 增减操作
    Increment,
    Decrement,
}

impl Operator for UnaryOperator {
    fn name(&self) -> &str {
        match self {
            UnaryOperator::Plus => "+",
            UnaryOperator::Minus => "-",
            UnaryOperator::Not => "NOT",
            UnaryOperator::IsNull => "IS NULL",
            UnaryOperator::IsNotNull => "IS NOT NULL",
            UnaryOperator::IsEmpty => "IS EMPTY",
            UnaryOperator::IsNotEmpty => "IS NOT EMPTY",
            UnaryOperator::Increment => "++",
            UnaryOperator::Decrement => "--",
        }
    }

    fn precedence(&self) -> u8 {
        match self {
            // 一元操作符具有高优先级
            UnaryOperator::Plus
            | UnaryOperator::Minus
            | UnaryOperator::Not
            | UnaryOperator::Increment
            | UnaryOperator::Decrement => 9,

            // 存在性检查操作符
            UnaryOperator::IsNull
            | UnaryOperator::IsNotNull
            | UnaryOperator::IsEmpty
            | UnaryOperator::IsNotEmpty => 3,
        }
    }

    fn is_left_associative(&self) -> bool {
        match self {
            // 前缀一元操作符是右结合的
            UnaryOperator::Plus
            | UnaryOperator::Minus
            | UnaryOperator::Not
            | UnaryOperator::IsNull
            | UnaryOperator::IsNotNull
            | UnaryOperator::IsEmpty
            | UnaryOperator::IsNotEmpty => false,

            // 后缀一元操作符是左结合的
            UnaryOperator::Increment | UnaryOperator::Decrement => true,
        }
    }

    fn arity(&self) -> usize {
        1 // 一元操作符总是需要一个操作数
    }
}

/// 聚合函数操作符
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AggregateFunction {
    Count,
    Sum,
    Avg,
    Min,
    Max,
    Collect,
    Distinct,
    Percentile,
}

impl Operator for AggregateFunction {
    fn name(&self) -> &str {
        match self {
            AggregateFunction::Count => "COUNT",
            AggregateFunction::Sum => "SUM",
            AggregateFunction::Avg => "AVG",
            AggregateFunction::Min => "MIN",
            AggregateFunction::Max => "MAX",
            AggregateFunction::Collect => "COLLECT",
            AggregateFunction::Distinct => "DISTINCT",
            AggregateFunction::Percentile => "PERCENTILE",
        }
    }

    fn precedence(&self) -> u8 {
        // 聚合函数具有最高优先级
        10
    }

    fn is_left_associative(&self) -> bool {
        // 聚合函数是函数调用，不考虑结合性
        true
    }

    fn arity(&self) -> usize {
        match self {
            AggregateFunction::Count => 1, // COUNT(*) 是特殊情况，但通常有一个参数
            AggregateFunction::Sum => 1,
            AggregateFunction::Avg => 1,
            AggregateFunction::Min => 1,
            AggregateFunction::Max => 1,
            AggregateFunction::Collect => 1,
            AggregateFunction::Distinct => 1,
            AggregateFunction::Percentile => 2, // 需要字段和百分位数两个参数
        }
    }
}

/// 操作符类别
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OperatorCategory {
    Arithmetic, // 算术操作符
    Comparison, // 比较操作符
    Logical,    // 逻辑操作符
    String,     // 字符串操作符
    Collection, // 集合操作符
    Aggregate,  // 聚合函数
    Unary,      // 一元操作符
}

impl BinaryOperator {
    /// 获取操作符类别
    pub fn category(&self) -> OperatorCategory {
        match self {
            BinaryOperator::Add
            | BinaryOperator::Subtract
            | BinaryOperator::Multiply
            | BinaryOperator::Divide
            | BinaryOperator::Modulo
            | BinaryOperator::Exponent => OperatorCategory::Arithmetic,
            BinaryOperator::Equal
            | BinaryOperator::NotEqual
            | BinaryOperator::LessThan
            | BinaryOperator::LessThanOrEqual
            | BinaryOperator::GreaterThan
            | BinaryOperator::GreaterThanOrEqual => OperatorCategory::Comparison,
            BinaryOperator::And | BinaryOperator::Or | BinaryOperator::Xor => {
                OperatorCategory::Logical
            }
            BinaryOperator::StringConcat
            | BinaryOperator::Like
            | BinaryOperator::Contains
            | BinaryOperator::StartsWith
            | BinaryOperator::EndsWith => OperatorCategory::String,
            BinaryOperator::In | BinaryOperator::NotIn => OperatorCategory::Collection,
            BinaryOperator::Subscript | BinaryOperator::Attribute => OperatorCategory::Unary, // 访问操作符
            BinaryOperator::Union | BinaryOperator::Intersect | BinaryOperator::Except => {
                OperatorCategory::Collection
            }
        }
    }

    /// 检查是否是算术操作符
    pub fn is_arithmetic(&self) -> bool {
        matches!(self.category(), OperatorCategory::Arithmetic)
    }

    /// 检查是否是比较操作符
    pub fn is_comparison(&self) -> bool {
        matches!(self.category(), OperatorCategory::Comparison)
    }

    /// 检查是否是逻辑操作符
    pub fn is_logical(&self) -> bool {
        matches!(self.category(), OperatorCategory::Logical)
    }
}

impl UnaryOperator {
    /// 获取操作符类别
    pub fn category(&self) -> OperatorCategory {
        match self {
            UnaryOperator::Plus | UnaryOperator::Minus => OperatorCategory::Arithmetic,
            UnaryOperator::Not => OperatorCategory::Logical,
            UnaryOperator::IsNull
            | UnaryOperator::IsNotNull
            | UnaryOperator::IsEmpty
            | UnaryOperator::IsNotEmpty => OperatorCategory::Comparison,
            UnaryOperator::Increment | UnaryOperator::Decrement => OperatorCategory::Arithmetic,
        }
    }

    /// 检查是否是前缀操作符
    pub fn is_prefix(&self) -> bool {
        !matches!(self, UnaryOperator::Increment | UnaryOperator::Decrement)
    }

    /// 检查是否是后缀操作符
    pub fn is_postfix(&self) -> bool {
        matches!(self, UnaryOperator::Increment | UnaryOperator::Decrement)
    }
}

impl AggregateFunction {
    /// 获取操作符类别
    pub fn category(&self) -> OperatorCategory {
        OperatorCategory::Aggregate
    }

    /// 检查是否是数值聚合函数
    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            AggregateFunction::Sum
                | AggregateFunction::Avg
                | AggregateFunction::Min
                | AggregateFunction::Max
        )
    }

    /// 检查是否是集合聚合函数
    pub fn is_collection(&self) -> bool {
        matches!(
            self,
            AggregateFunction::Count | AggregateFunction::Collect | AggregateFunction::Distinct
        )
    }
}
