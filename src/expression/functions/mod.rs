//! 表达式函数模块
//!
//! 提供表达式求值过程中的函数定义和实现，包括内置函数和自定义函数
//!
//! ## 模块结构
//!
//! - `signature.rs` - 类型签名系统
//! - `registry.rs` - 函数注册表
//! - `builtin/` - 内置函数实现
//!
//! ## 使用方式
//!
//! ```rust
//! use crate::expression::functions::{global_registry, FunctionRegistry};
//!
//! let registry = global_registry();
//! let result = registry.execute("abs", &[Value::Int(-5)]);
//! ```

pub mod signature;
pub mod registry;
pub mod builtin;

pub use signature::{FunctionSignature, ValueType, RegisteredFunction};
pub use registry::{FunctionRegistry, global_registry};

use crate::core::error::{ExpressionError, ExpressionErrorType};
use crate::core::types::operators::AggregateFunction;
use crate::core::Value;

/// 函数引用枚举，用于表达式中引用函数
#[derive(Debug, Clone)]
pub enum FunctionRef<'a> {
    /// 内置函数引用
    Builtin(&'a BuiltinFunction),
    /// 自定义函数引用
    Custom(&'a CustomFunction),
}

/// 拥有所有权的函数引用
#[derive(Debug, Clone)]
pub enum OwnedFunctionRef {
    /// 内置函数引用（拥有所有权）
    Builtin(BuiltinFunction),
    /// 自定义函数引用（拥有所有权）
    Custom(CustomFunction),
}

impl<'a> From<FunctionRef<'a>> for OwnedFunctionRef {
    fn from(func_ref: FunctionRef<'a>) -> Self {
        match func_ref {
            FunctionRef::Builtin(f) => OwnedFunctionRef::Builtin(f.clone()),
            FunctionRef::Custom(f) => OwnedFunctionRef::Custom(f.clone()),
        }
    }
}

impl OwnedFunctionRef {
    pub fn name(&self) -> &str {
        match self {
            OwnedFunctionRef::Builtin(f) => f.name(),
            OwnedFunctionRef::Custom(f) => f.name(),
        }
    }

    pub fn execute(&self, args: &[Value]) -> Result<Value, ExpressionError> {
        match self {
            OwnedFunctionRef::Builtin(f) => f.execute(args),
            OwnedFunctionRef::Custom(f) => f.execute(args),
        }
    }

    /// 执行函数（带缓存）
    pub fn execute_with_cache(&self, args: &[Value], cache: &mut crate::expression::context::CacheManager) -> Result<Value, ExpressionError> {
        match self {
            OwnedFunctionRef::Builtin(f) => {
                if let BuiltinFunction::Regex(regex_func) = f {
                    regex_func.execute_with_cache(args, cache)
                } else {
                    f.execute(args)
                }
            }
            OwnedFunctionRef::Custom(f) => f.execute(args),
        }
    }
}

/// 表达式函数特征
pub trait ExpressionFunction: Send + Sync {
    /// 获取函数名称
    fn name(&self) -> &str;

    /// 获取参数数量
    fn arity(&self) -> usize;

    /// 检查是否接受可变参数
    fn is_variadic(&self) -> bool;

    /// 执行函数
    fn execute(&self, args: &[Value]) -> Result<Value, ExpressionError>;

    /// 获取函数描述
    fn description(&self) -> &str;
}

/// 内置函数类型，避免动态分发
#[derive(Debug, Clone)]
pub enum BuiltinFunction {
    /// 数学函数
    Math(MathFunction),
    /// 字符串函数
    String(StringFunction),
    /// 正则表达式函数
    Regex(RegexFunction),
    /// 聚合函数
    Aggregate(AggregateFunction),
    /// 类型转换函数
    Conversion(ConversionFunction),
    /// 日期时间函数
    DateTime(DateTimeFunction),
}

/// 数学函数
#[derive(Debug, Clone, PartialEq)]
pub enum MathFunction {
    Abs,
    Sqrt,
    Pow,
    Log,
    Log10,
    Sin,
    Cos,
    Tan,
    Round,
    Ceil,
    Floor,
}

/// 字符串函数
#[derive(Debug, Clone, PartialEq)]
pub enum StringFunction {
    Length,
    Upper,
    Lower,
    Trim,
    Substring,
    Concat,
    Replace,
    Contains,
    StartsWith,
    EndsWith,
}

/// 正则表达式函数
#[derive(Debug, Clone, PartialEq)]
pub enum RegexFunction {
    RegexMatch,
    RegexReplace,
    RegexFind,
}

/// 类型转换函数
#[derive(Debug, Clone, PartialEq)]
pub enum ConversionFunction {
    ToString,
    ToInt,
    ToFloat,
    ToBool,
}

/// 日期时间函数
#[derive(Debug, Clone, PartialEq)]
pub enum DateTimeFunction {
    Now,
    Date,
    Time,
    Year,
    Month,
    Day,
    Hour,
    Minute,
    Second,
}

/// 自定义函数定义
#[derive(Debug, Clone)]
pub struct CustomFunction {
    /// 函数名称
    pub name: String,
    /// 参数数量
    pub arity: usize,
    /// 是否接受可变参数
    pub is_variadic: bool,
    /// 函数描述
    pub description: String,
    /// 函数ID（用于标识不同的函数实现）
    pub function_id: u64,
}

impl ExpressionFunction for BuiltinFunction {
    fn name(&self) -> &str {
        match self {
            BuiltinFunction::Math(f) => f.name(),
            BuiltinFunction::String(f) => f.name(),
            BuiltinFunction::Regex(f) => f.name(),
            BuiltinFunction::Aggregate(f) => f.name(),
            BuiltinFunction::Conversion(f) => f.name(),
            BuiltinFunction::DateTime(f) => f.name(),
        }
    }

    fn arity(&self) -> usize {
        match self {
            BuiltinFunction::Math(f) => f.arity(),
            BuiltinFunction::String(f) => f.arity(),
            BuiltinFunction::Regex(f) => f.arity(),
            BuiltinFunction::Aggregate(f) => f.arity(),
            BuiltinFunction::Conversion(f) => f.arity(),
            BuiltinFunction::DateTime(f) => f.arity(),
        }
    }

    fn is_variadic(&self) -> bool {
        match self {
            BuiltinFunction::Math(f) => f.is_variadic(),
            BuiltinFunction::String(f) => f.is_variadic(),
            BuiltinFunction::Regex(f) => f.is_variadic(),
            BuiltinFunction::Aggregate(f) => f.is_variadic(),
            BuiltinFunction::Conversion(f) => f.is_variadic(),
            BuiltinFunction::DateTime(f) => f.is_variadic(),
        }
    }

    fn execute(&self, args: &[Value]) -> Result<Value, ExpressionError> {
        let registry = global_registry();
        let name = self.name();
        
        // 使用注册表执行函数
        registry.execute(name, args)
    }

    fn description(&self) -> &str {
        match self {
            BuiltinFunction::Math(f) => f.description(),
            BuiltinFunction::String(f) => f.description(),
            BuiltinFunction::Regex(f) => f.description(),
            BuiltinFunction::Aggregate(f) => f.description(),
            BuiltinFunction::Conversion(f) => f.description(),
            BuiltinFunction::DateTime(f) => f.description(),
        }
    }
}

impl MathFunction {
    pub fn name(&self) -> &str {
        match self {
            MathFunction::Abs => "abs",
            MathFunction::Sqrt => "sqrt",
            MathFunction::Pow => "pow",
            MathFunction::Log => "log",
            MathFunction::Log10 => "log10",
            MathFunction::Sin => "sin",
            MathFunction::Cos => "cos",
            MathFunction::Tan => "tan",
            MathFunction::Round => "round",
            MathFunction::Ceil => "ceil",
            MathFunction::Floor => "floor",
        }
    }

    pub fn arity(&self) -> usize {
        match self {
            MathFunction::Abs
            | MathFunction::Sqrt
            | MathFunction::Log10
            | MathFunction::Sin
            | MathFunction::Cos
            | MathFunction::Tan
            | MathFunction::Round
            | MathFunction::Ceil
            | MathFunction::Floor => 1,
            MathFunction::Pow | MathFunction::Log => 2,
        }
    }

    pub fn is_variadic(&self) -> bool {
        false
    }

    pub fn description(&self) -> &str {
        match self {
            MathFunction::Abs => "计算绝对值",
            MathFunction::Sqrt => "计算平方根",
            MathFunction::Pow => "计算幂",
            MathFunction::Log => "计算对数",
            MathFunction::Log10 => "计算以10为底的对数",
            MathFunction::Sin => "计算正弦",
            MathFunction::Cos => "计算余弦",
            MathFunction::Tan => "计算正切",
            MathFunction::Round => "四舍五入",
            MathFunction::Ceil => "向上取整",
            MathFunction::Floor => "向下取整",
        }
    }
}

impl StringFunction {
    pub fn name(&self) -> &str {
        match self {
            StringFunction::Length => "length",
            StringFunction::Upper => "upper",
            StringFunction::Lower => "lower",
            StringFunction::Trim => "trim",
            StringFunction::Substring => "substring",
            StringFunction::Concat => "concat",
            StringFunction::Replace => "replace",
            StringFunction::Contains => "contains",
            StringFunction::StartsWith => "starts_with",
            StringFunction::EndsWith => "ends_with",
        }
    }

    pub fn arity(&self) -> usize {
        match self {
            StringFunction::Length
            | StringFunction::Upper
            | StringFunction::Lower
            | StringFunction::Trim => 1,
            StringFunction::Substring => 3,
            StringFunction::Concat
            | StringFunction::Replace
            | StringFunction::Contains
            | StringFunction::StartsWith
            | StringFunction::EndsWith => 2,
        }
    }

    pub fn is_variadic(&self) -> bool {
        matches!(self, StringFunction::Concat)
    }

    pub fn description(&self) -> &str {
        match self {
            StringFunction::Length => "计算字符串长度",
            StringFunction::Upper => "转换为大写",
            StringFunction::Lower => "转换为小写",
            StringFunction::Trim => "去除首尾空白",
            StringFunction::Substring => "获取子字符串",
            StringFunction::Concat => "连接字符串",
            StringFunction::Replace => "替换字符串",
            StringFunction::Contains => "检查是否包含子字符串",
            StringFunction::StartsWith => "检查是否以指定字符串开头",
            StringFunction::EndsWith => "检查是否以指定字符串结尾",
        }
    }
}

impl RegexFunction {
    pub fn name(&self) -> &str {
        match self {
            RegexFunction::RegexMatch => "regex_match",
            RegexFunction::RegexReplace => "regex_replace",
            RegexFunction::RegexFind => "regex_find",
        }
    }

    pub fn arity(&self) -> usize {
        match self {
            RegexFunction::RegexMatch => 2,
            RegexFunction::RegexReplace => 3,
            RegexFunction::RegexFind => 2,
        }
    }

    pub fn is_variadic(&self) -> bool {
        false
    }

    pub fn description(&self) -> &str {
        match self {
            RegexFunction::RegexMatch => "正则表达式匹配",
            RegexFunction::RegexReplace => "正则表达式替换",
            RegexFunction::RegexFind => "正则表达式查找",
        }
    }

    /// 执行函数（带缓存）
    pub fn execute_with_cache(&self, args: &[Value], cache: &mut crate::expression::context::CacheManager) -> Result<Value, ExpressionError> {
        match self {
            RegexFunction::RegexMatch => {
                match (&args[0], &args[1]) {
                    (Value::String(s), Value::String(pattern)) => {
                        if let Some(regex) = cache.get_regex_internal(pattern) {
                            Ok(Value::Bool(regex.is_match(s)))
                        } else {
                            Err(ExpressionError::new(
                                ExpressionErrorType::InvalidOperation,
                                format!("无效的正则表达式: {}", pattern),
                            ))
                        }
                    }
                    (Value::Null(_), _) | (_, Value::Null(_)) => {
                        Ok(Value::Null(crate::core::value::NullType::Null))
                    }
                    _ => Err(ExpressionError::type_error("regex_match函数需要字符串类型")),
                }
            }
            RegexFunction::RegexReplace => {
                match (&args[0], &args[1], &args[2]) {
                    (Value::String(s), Value::String(pattern), Value::String(replacement)) => {
                        if let Some(regex) = cache.get_regex_internal(pattern) {
                            Ok(Value::String(regex.replace_all(s, replacement.as_str()).to_string()))
                        } else {
                            Err(ExpressionError::new(
                                ExpressionErrorType::InvalidOperation,
                                format!("无效的正则表达式: {}", pattern),
                            ))
                        }
                    }
                    (Value::Null(_), _, _) | (_, Value::Null(_), _) | (_, _, Value::Null(_)) => {
                        Ok(Value::Null(crate::core::value::NullType::Null))
                    }
                    _ => Err(ExpressionError::type_error("regex_replace函数需要字符串类型")),
                }
            }
            RegexFunction::RegexFind => {
                match (&args[0], &args[1]) {
                    (Value::String(s), Value::String(pattern)) => {
                        if let Some(regex) = cache.get_regex_internal(pattern) {
                            if let Some(matched) = regex.find(s) {
                                Ok(Value::String(matched.as_str().to_string()))
                            } else {
                                Ok(Value::Null(crate::core::value::NullType::Null))
                            }
                        } else {
                            Err(ExpressionError::new(
                                ExpressionErrorType::InvalidOperation,
                                format!("无效的正则表达式: {}", pattern),
                            ))
                        }
                    }
                    (Value::Null(_), _) | (_, Value::Null(_)) => {
                        Ok(Value::Null(crate::core::value::NullType::Null))
                    }
                    _ => Err(ExpressionError::type_error("regex_find函数需要字符串类型")),
                }
            }
        }
    }
}

impl AggregateFunction {
    pub fn is_variadic(&self) -> bool {
        false
    }

    pub fn description(&self) -> &str {
        match self {
            AggregateFunction::Count(_) => "计数",
            AggregateFunction::Sum(_) => "求和",
            AggregateFunction::Avg(_) => "平均值",
            AggregateFunction::Min(_) => "最小值",
            AggregateFunction::Max(_) => "最大值",
            AggregateFunction::Collect(_) => "收集",
            AggregateFunction::CollectSet(_) => "收集去重",
            AggregateFunction::Distinct(_) => "去重",
            AggregateFunction::Percentile(_, _) => "百分位数",
            AggregateFunction::Std(_) => "标准差",
            AggregateFunction::BitAnd(_) => "按位与",
            AggregateFunction::BitOr(_) => "按位或",
            AggregateFunction::GroupConcat(_, _) => "分组连接",
        }
    }
}

impl ConversionFunction {
    pub fn name(&self) -> &str {
        match self {
            ConversionFunction::ToString => "to_string",
            ConversionFunction::ToInt => "to_int",
            ConversionFunction::ToFloat => "to_float",
            ConversionFunction::ToBool => "to_bool",
        }
    }

    pub fn arity(&self) -> usize {
        1
    }

    pub fn is_variadic(&self) -> bool {
        false
    }

    pub fn description(&self) -> &str {
        match self {
            ConversionFunction::ToString => "转换为字符串",
            ConversionFunction::ToInt => "转换为整数",
            ConversionFunction::ToFloat => "转换为浮点数",
            ConversionFunction::ToBool => "转换为布尔值",
        }
    }
}

impl DateTimeFunction {
    pub fn name(&self) -> &str {
        match self {
            DateTimeFunction::Now => "now",
            DateTimeFunction::Date => "date",
            DateTimeFunction::Time => "time",
            DateTimeFunction::Year => "year",
            DateTimeFunction::Month => "month",
            DateTimeFunction::Day => "day",
            DateTimeFunction::Hour => "hour",
            DateTimeFunction::Minute => "minute",
            DateTimeFunction::Second => "second",
        }
    }

    pub fn arity(&self) -> usize {
        match self {
            DateTimeFunction::Now => 0,
            DateTimeFunction::Date | DateTimeFunction::Time => 1,
            DateTimeFunction::Year
            | DateTimeFunction::Month
            | DateTimeFunction::Day
            | DateTimeFunction::Hour
            | DateTimeFunction::Minute
            | DateTimeFunction::Second => 1,
        }
    }

    pub fn is_variadic(&self) -> bool {
        false
    }

    pub fn description(&self) -> &str {
        match self {
            DateTimeFunction::Now => "当前时间",
            DateTimeFunction::Date => "日期",
            DateTimeFunction::Time => "时间",
            DateTimeFunction::Year => "年份",
            DateTimeFunction::Month => "月份",
            DateTimeFunction::Day => "日期",
            DateTimeFunction::Hour => "小时",
            DateTimeFunction::Minute => "分钟",
            DateTimeFunction::Second => "秒",
        }
    }
}

impl ExpressionFunction for CustomFunction {
    fn name(&self) -> &str {
        &self.name
    }

    fn arity(&self) -> usize {
        self.arity
    }

    fn is_variadic(&self) -> bool {
        self.is_variadic
    }

    fn execute(&self, args: &[Value]) -> Result<Value, ExpressionError> {
        let registry = global_registry();
        registry.execute(&self.name, args)
    }

    fn description(&self) -> &str {
        &self.description
    }
}

impl FunctionRef<'_> {
    /// 获取函数名称
    pub fn name(&self) -> &str {
        match self {
            FunctionRef::Builtin(f) => f.name(),
            FunctionRef::Custom(f) => f.name(),
        }
    }

    /// 获取参数数量
    pub fn arity(&self) -> usize {
        match self {
            FunctionRef::Builtin(f) => f.arity(),
            FunctionRef::Custom(f) => f.arity(),
        }
    }

    /// 检查是否接受可变参数
    pub fn is_variadic(&self) -> bool {
        match self {
            FunctionRef::Builtin(f) => f.is_variadic(),
            FunctionRef::Custom(f) => f.is_variadic(),
        }
    }

    /// 执行函数
    pub fn execute(&self, args: &[Value]) -> Result<Value, ExpressionError> {
        match self {
            FunctionRef::Builtin(f) => f.execute(args),
            FunctionRef::Custom(f) => f.execute(args),
        }
    }

    /// 获取函数描述
    pub fn description(&self) -> &str {
        match self {
            FunctionRef::Builtin(f) => f.description(),
            FunctionRef::Custom(f) => f.description(),
        }
    }

    /// 执行函数（带缓存）
    pub fn execute_with_cache(&self, args: &[Value], cache: &mut crate::expression::context::CacheManager) -> Result<Value, ExpressionError> {
        match self {
            FunctionRef::Builtin(f) => {
                if let BuiltinFunction::Regex(regex_func) = f {
                    regex_func.execute_with_cache(args, cache)
                } else {
                    f.execute(args)
                }
            }
            FunctionRef::Custom(f) => f.execute(args),
        }
    }
}
