//! 表达式上下文定义
//!
//! 提供表达式求值过程中的上下文管理

pub mod default_context;

use crate::core::types::expression::Expression;
use crate::core::types::query::FieldValue;
use crate::cache::{
    CacheConfig, CacheFactory, StatsCacheType,
    Cache, StatsCache
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use super::base::{ContextBase, ContextType, MutableContext};

// 重新导出默认上下文类型
pub use default_context::{
    DefaultExpressionContext, ExpressionContext, QueryContextAdapter, ExpressionContextBuilder,
    ExpressionContextCore, StorageExpressionContextCore, with_variables, with_vertex, with_edge,
};

/// 函数引用枚举，避免动态分发
#[derive(Debug, Clone)]
pub enum FunctionRef<'a> {
    /// 内置函数引用
    Builtin(&'a BuiltinFunction),
    /// 自定义函数引用
    Custom(&'a CustomFunction),
}

/// 表达式缓存管理器
#[derive(Debug)]
pub struct ExpressionCacheManager {
    /// 函数执行结果缓存
    function_cache: StatsCacheType<String, FieldValue>,
    /// 表达式解析结果缓存
    expression_cache: StatsCacheType<String, Expression>,
    /// 变量查找缓存
    variable_cache: StatsCacheType<String, FieldValue>,
    /// 缓存配置
    config: CacheConfig,
}

impl ExpressionCacheManager {
    /// 创建新的表达式缓存管理器
    pub fn new(config: CacheConfig) -> Self {
        let function_cache = CacheFactory::create_stats_cache_by_policy(
            &config.default_policy,
            config.parser_cache.expression_cache_capacity,
        );
        
        let expression_cache = CacheFactory::create_stats_cache_by_policy(
            &config.default_policy,
            config.parser_cache.expression_cache_capacity,
        );
        
        let variable_cache = CacheFactory::create_stats_cache_by_policy(
            &config.default_policy,
            config.parser_cache.expression_cache_capacity,
        );
        
        Self {
            function_cache,
            expression_cache,
            variable_cache,
            config,
        }
    }
    
    /// 获取函数执行结果
    pub fn get_function_result(&self, key: &str) -> Option<FieldValue> {
        if self.config.enabled {
            self.function_cache.get(&key.to_string())
        } else {
            None
        }
    }
    
    /// 缓存函数执行结果
    pub fn cache_function_result(&self, key: &str, result: FieldValue) {
        if self.config.enabled {
            self.function_cache.put(key.to_string(), result);
        }
    }
    
    /// 获取表达式解析结果
    pub fn get_expression(&self, key: &str) -> Option<Expression> {
        if self.config.enabled {
            self.expression_cache.get(&key.to_string())
        } else {
            None
        }
    }
    
    /// 缓存表达式解析结果
    pub fn cache_expression(&self, key: &str, expression: Expression) {
        if self.config.enabled {
            self.expression_cache.put(key.to_string(), expression);
        }
    }
    
    /// 获取变量查找结果
    pub fn get_variable(&self, key: &str) -> Option<FieldValue> {
        if self.config.enabled {
            self.variable_cache.get(&key.to_string())
        } else {
            None
        }
    }
    
    /// 缓存变量查找结果
    pub fn cache_variable(&self, key: &str, value: FieldValue) {
        if self.config.enabled {
            self.variable_cache.put(key.to_string(), value);
        }
    }
    
    /// 获取缓存统计信息
    pub fn get_cache_stats(&self) -> ExpressionCacheStats {
        ExpressionCacheStats {
            function_cache_hits: self.function_cache.hits(),
            function_cache_misses: self.function_cache.misses(),
            function_cache_hit_rate: self.function_cache.hit_rate(),
            expression_cache_hits: self.expression_cache.hits(),
            expression_cache_misses: self.expression_cache.misses(),
            expression_cache_hit_rate: self.expression_cache.hit_rate(),
            variable_cache_hits: self.variable_cache.hits(),
            variable_cache_misses: self.variable_cache.misses(),
            variable_cache_hit_rate: self.variable_cache.hit_rate(),
        }
    }
    
    /// 清空所有缓存
    pub fn clear_all(&self) {
        self.function_cache.clear();
        self.expression_cache.clear();
        self.variable_cache.clear();
    }
    
    /// 重置统计信息
    pub fn reset_stats(&self) {
        self.function_cache.reset_stats();
        self.expression_cache.reset_stats();
        self.variable_cache.reset_stats();
    }
}

/// 表达式缓存统计信息
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExpressionCacheStats {
    /// 函数缓存命中次数
    pub function_cache_hits: u64,
    /// 函数缓存未命中次数
    pub function_cache_misses: u64,
    /// 函数缓存命中率
    pub function_cache_hit_rate: f64,
    /// 表达式缓存命中次数
    pub expression_cache_hits: u64,
    /// 表达式缓存未命中次数
    pub expression_cache_misses: u64,
    /// 表达式缓存命中率
    pub expression_cache_hit_rate: f64,
    /// 变量缓存命中次数
    pub variable_cache_hits: u64,
    /// 变量缓存未命中次数
    pub variable_cache_misses: u64,
    /// 变量缓存命中率
    pub variable_cache_hit_rate: f64,
}

/// 表达式上下文枚举，避免动态分发
#[derive(Debug, Clone)]
pub enum ExpressionContextType {
    /// 基础表达式上下文
    Basic(BasicExpressionContext),
}

/// 表达式上下文特征
pub trait ExpressionContextCore {
    /// 获取变量值
    fn get_variable(&self, name: &str) -> Option<&FieldValue>;

    /// 获取函数
    fn get_function(&self, name: &str) -> Option<FunctionRef>;

    /// 检查变量是否存在
    fn has_variable(&self, name: &str) -> bool;

    /// 获取所有变量名
    fn get_variable_names(&self) -> Vec<&str>;

    /// 获取上下文深度
    fn get_depth(&self) -> usize;

    /// 创建子上下文
    fn create_child_context(&self) -> ExpressionContextType;
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
    fn execute(&self, args: &[FieldValue]) -> Result<FieldValue, ExpressionError>;

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

/// 聚合函数
#[derive(Debug, Clone, PartialEq)]
pub enum AggregateFunction {
    Count,
    Sum,
    Avg,
    Min,
    Max,
    Collect,
    Distinct,
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

impl ExpressionFunction for BuiltinFunction {
    fn name(&self) -> &str {
        match self {
            BuiltinFunction::Math(f) => f.name(),
            BuiltinFunction::String(f) => f.name(),
            BuiltinFunction::Aggregate(f) => f.name(),
            BuiltinFunction::Conversion(f) => f.name(),
            BuiltinFunction::DateTime(f) => f.name(),
        }
    }

    fn arity(&self) -> usize {
        match self {
            BuiltinFunction::Math(f) => f.arity(),
            BuiltinFunction::String(f) => f.arity(),
            BuiltinFunction::Aggregate(f) => f.arity(),
            BuiltinFunction::Conversion(f) => f.arity(),
            BuiltinFunction::DateTime(f) => f.arity(),
        }
    }

    fn is_variadic(&self) -> bool {
        match self {
            BuiltinFunction::Math(f) => f.is_variadic(),
            BuiltinFunction::String(f) => f.is_variadic(),
            BuiltinFunction::Aggregate(f) => f.is_variadic(),
            BuiltinFunction::Conversion(f) => f.is_variadic(),
            BuiltinFunction::DateTime(f) => f.is_variadic(),
        }
    }

    fn execute(&self, args: &[FieldValue]) -> Result<FieldValue, ExpressionError> {
        match self {
            BuiltinFunction::Math(f) => f.execute(args),
            BuiltinFunction::String(f) => f.execute(args),
            BuiltinFunction::Aggregate(f) => f.execute(args),
            BuiltinFunction::Conversion(f) => f.execute(args),
            BuiltinFunction::DateTime(f) => f.execute(args),
        }
    }

    fn description(&self) -> &str {
        match self {
            BuiltinFunction::Math(f) => f.description(),
            BuiltinFunction::String(f) => f.description(),
            BuiltinFunction::Aggregate(f) => f.description(),
            BuiltinFunction::Conversion(f) => f.description(),
            BuiltinFunction::DateTime(f) => f.description(),
        }
    }
}

// 为每个函数类型实现ExpressionFunction trait
impl ExpressionFunction for MathFunction {
    fn name(&self) -> &str {
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

    fn arity(&self) -> usize {
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

    fn is_variadic(&self) -> bool {
        false
    }

    fn execute(&self, _args: &[FieldValue]) -> Result<FieldValue, ExpressionError> {
        // 实现数学函数的具体逻辑
        // 这里暂时返回错误，等待后续实现
        Err(ExpressionError::runtime_error(format!(
            "数学函数 {:?} 尚未实现",
            self
        )))
    }

    fn description(&self) -> &str {
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

impl ExpressionFunction for StringFunction {
    fn name(&self) -> &str {
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

    fn arity(&self) -> usize {
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

    fn is_variadic(&self) -> bool {
        matches!(self, StringFunction::Concat)
    }

    fn execute(&self, _args: &[FieldValue]) -> Result<FieldValue, ExpressionError> {
        Err(ExpressionError::runtime_error(format!(
            "字符串函数 {:?} 尚未实现",
            self
        )))
    }

    fn description(&self) -> &str {
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

impl ExpressionFunction for AggregateFunction {
    fn name(&self) -> &str {
        match self {
            AggregateFunction::Count => "count",
            AggregateFunction::Sum => "sum",
            AggregateFunction::Avg => "avg",
            AggregateFunction::Min => "min",
            AggregateFunction::Max => "max",
            AggregateFunction::Collect => "collect",
            AggregateFunction::Distinct => "distinct",
        }
    }

    fn arity(&self) -> usize {
        1
    }

    fn is_variadic(&self) -> bool {
        false
    }

    fn execute(&self, _args: &[FieldValue]) -> Result<FieldValue, ExpressionError> {
        Err(ExpressionError::runtime_error(format!(
            "聚合函数 {:?} 尚未实现",
            self
        )))
    }

    fn description(&self) -> &str {
        match self {
            AggregateFunction::Count => "计数",
            AggregateFunction::Sum => "求和",
            AggregateFunction::Avg => "平均值",
            AggregateFunction::Min => "最小值",
            AggregateFunction::Max => "最大值",
            AggregateFunction::Collect => "收集",
            AggregateFunction::Distinct => "去重",
        }
    }
}

impl ExpressionFunction for ConversionFunction {
    fn name(&self) -> &str {
        match self {
            ConversionFunction::ToString => "to_string",
            ConversionFunction::ToInt => "to_int",
            ConversionFunction::ToFloat => "to_float",
            ConversionFunction::ToBool => "to_bool",
        }
    }

    fn arity(&self) -> usize {
        1
    }

    fn is_variadic(&self) -> bool {
        false
    }

    fn execute(&self, _args: &[FieldValue]) -> Result<FieldValue, ExpressionError> {
        Err(ExpressionError::runtime_error(format!(
            "类型转换函数 {:?} 尚未实现",
            self
        )))
    }

    fn description(&self) -> &str {
        match self {
            ConversionFunction::ToString => "转换为字符串",
            ConversionFunction::ToInt => "转换为整数",
            ConversionFunction::ToFloat => "转换为浮点数",
            ConversionFunction::ToBool => "转换为布尔值",
        }
    }
}

impl ExpressionFunction for DateTimeFunction {
    fn name(&self) -> &str {
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

    fn arity(&self) -> usize {
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

    fn is_variadic(&self) -> bool {
        false
    }

    fn execute(&self, _args: &[FieldValue]) -> Result<FieldValue, ExpressionError> {
        Err(ExpressionError::runtime_error(format!(
            "日期时间函数 {:?} 尚未实现",
            self
        )))
    }

    fn description(&self) -> &str {
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

/// 基础表达式上下文
#[derive(Debug)]
pub struct BasicExpressionContext {
    /// 变量绑定
    pub variables: HashMap<String, FieldValue>,
    /// 函数注册表
    pub functions: HashMap<String, BuiltinFunction>,
    /// 自定义函数注册表
    pub custom_functions: HashMap<String, CustomFunction>,
    /// 父上下文
    pub parent: Option<Box<BasicExpressionContext>>,
    /// 上下文深度
    pub depth: usize,
    /// 缓存管理器
    pub cache_manager: Option<Arc<ExpressionCacheManager>>,
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

    fn execute(&self, _args: &[FieldValue]) -> Result<FieldValue, ExpressionError> {
        // 这里应该根据function_id调用具体的函数实现
        // 暂时返回错误，等待后续实现
        Err(ExpressionError::runtime_error(format!(
            "自定义函数 {} 尚未实现",
            self.name
        )))
    }

    fn description(&self) -> &str {
        &self.description
    }
}

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
    /// 参数数量错误
    ArgumentCountError,
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

/// 表达式求值选项
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvaluationOptions {
    /// 是否启用严格模式
    pub strict_mode: bool,
    /// 是否允许隐式类型转换
    pub allow_implicit_conversion: bool,
    /// 最大递归深度
    pub max_recursion_depth: usize,
    /// 超时时间（毫秒）
    pub timeout_ms: Option<u64>,
    /// 缓存配置
    pub cache_config: CacheConfig,
}

impl ExpressionContextCore for BasicExpressionContext {
    fn get_variable(&self, name: &str) -> Option<&FieldValue> {
        // 在当前上下文中查找
        if let Some(value) = self.variables.get(name) {
            // 缓存查找结果
            if let Some(cache_manager) = &self.cache_manager {
                let cache_key = format!("var:{}:{}", name, self.depth);
                cache_manager.cache_variable(&cache_key, value.clone());
            }
            return Some(value);
        }

        // 如果在当前上下文中找不到，则在父上下文中查找
        if let Some(parent) = &self.parent {
            parent.get_variable(name)
        } else {
            None
        }
    }

    fn get_function(&self, name: &str) -> Option<FunctionRef> {
        // 在当前上下文中查找内置函数
        if let Some(function) = self.functions.get(name) {
            return Some(FunctionRef::Builtin(function));
        }

        // 然后查找自定义函数
        if let Some(function) = self.custom_functions.get(name) {
            return Some(FunctionRef::Custom(function));
        }

        // 如果在当前上下文中找不到，则在父上下文中查找
        if let Some(parent) = &self.parent {
            parent.get_function(name)
        } else {
            None
        }
    }

    fn has_variable(&self, name: &str) -> bool {
        self.get_variable(name).is_some()
    }

    fn get_variable_names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.variables.keys().map(|k| k.as_str()).collect();

        // 添加父上下文中的变量名（去重）
        if let Some(parent) = &self.parent {
            let parent_names = parent.get_variable_names();
            for name in parent_names {
                if !names.contains(&name) {
                    names.push(name);
                }
            }
        }

        names
    }

    fn get_depth(&self) -> usize {
        self.depth
    }

    fn create_child_context(&self) -> ExpressionContextType {
        ExpressionContextType::Basic(BasicExpressionContext {
            variables: HashMap::new(),
            functions: HashMap::new(),
            custom_functions: HashMap::new(),
            parent: Some(Box::new(self.clone())),
            depth: self.get_depth() + 1,
            cache_manager: self.cache_manager.clone(),
        })
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
    pub fn execute(&self, args: &[FieldValue]) -> Result<FieldValue, ExpressionError> {
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
}

impl ExpressionContextCore for ExpressionContextType {
    fn get_variable(&self, name: &str) -> Option<&FieldValue> {
        match self {
            ExpressionContextType::Basic(ctx) => ctx.get_variable(name),
        }
    }

    fn get_function(&self, name: &str) -> Option<FunctionRef> {
        match self {
            ExpressionContextType::Basic(ctx) => ctx.get_function(name),
        }
    }

    fn has_variable(&self, name: &str) -> bool {
        match self {
            ExpressionContextType::Basic(ctx) => ctx.has_variable(name),
        }
    }

    fn get_variable_names(&self) -> Vec<&str> {
        match self {
            ExpressionContextType::Basic(ctx) => ctx.get_variable_names(),
        }
    }

    fn get_depth(&self) -> usize {
        match self {
            ExpressionContextType::Basic(ctx) => ctx.get_depth(),
        }
    }

    fn create_child_context(&self) -> ExpressionContextType {
        match self {
            ExpressionContextType::Basic(ctx) => ctx.create_child_context(),
        }
    }
}

impl BasicExpressionContext {
    /// 创建新的基础表达式上下文
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
            custom_functions: HashMap::new(),
            parent: None,
            depth: 0,
            cache_manager: None,
        }
    }

    /// 创建带缓存管理器的基础表达式上下文
    pub fn with_cache(cache_config: CacheConfig) -> Self {
        Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
            custom_functions: HashMap::new(),
            parent: None,
            depth: 0,
            cache_manager: Some(Arc::new(ExpressionCacheManager::new(cache_config))),
        }
    }

    /// 创建带父上下文的基础表达式上下文
    pub fn with_parent(parent: BasicExpressionContext) -> Self {
        let parent_depth = parent.get_depth();
        Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
            custom_functions: HashMap::new(),
            parent: Some(Box::new(parent)),
            depth: parent_depth + 1,
            cache_manager: None,
        }
    }

    /// 创建带父上下文和缓存管理器的基础表达式上下文
    pub fn with_parent_and_cache(parent: BasicExpressionContext, cache_config: CacheConfig) -> Self {
        let parent_depth = parent.get_depth();
        Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
            custom_functions: HashMap::new(),
            parent: Some(Box::new(parent)),
            depth: parent_depth + 1,
            cache_manager: Some(Arc::new(ExpressionCacheManager::new(cache_config))),
        }
    }

    /// 设置变量
    pub fn set_variable(&mut self, name: impl Into<String>, value: FieldValue) {
        self.variables.insert(name.into(), value);
    }

    /// 批量设置变量
    pub fn set_variables(&mut self, variables: HashMap<String, FieldValue>) {
        self.variables = variables;
    }

    /// 注册内置函数
    pub fn register_builtin_function(&mut self, function: BuiltinFunction) {
        self.functions.insert(function.name().to_string(), function);
    }

    /// 注册自定义函数
    pub fn register_custom_function(&mut self, function: CustomFunction) {
        self.custom_functions
            .insert(function.name.clone(), function);
    }

    /// 获取内置函数
    pub fn get_builtin_function(&self, name: &str) -> Option<&BuiltinFunction> {
        self.functions.get(name)
    }

    /// 获取自定义函数
    pub fn get_custom_function(&self, name: &str) -> Option<&CustomFunction> {
        self.custom_functions.get(name)
    }

    /// 移除变量
    pub fn remove_variable(&mut self, name: &str) -> Option<FieldValue> {
        self.variables.remove(name)
    }

    /// 清空所有变量
    pub fn clear_variables(&mut self) {
        self.variables.clear();
    }

    /// 检查变量是否在当前上下文中定义
    pub fn is_local_variable(&self, name: &str) -> bool {
        self.variables.contains_key(name)
    }

    /// 获取当前上下文中的变量名
    pub fn get_local_variable_names(&self) -> Vec<&str> {
        self.variables.keys().map(|k| k.as_str()).collect()
    }

    /// 获取缓存统计信息
    pub fn get_cache_stats(&self) -> Option<ExpressionCacheStats> {
        self.cache_manager.as_ref().map(|cm| cm.get_cache_stats())
    }

    /// 清空所有缓存
    pub fn clear_cache(&self) {
        if let Some(cache_manager) = &self.cache_manager {
            cache_manager.clear_all();
        }
    }

    /// 重置缓存统计信息
    pub fn reset_cache_stats(&self) {
        if let Some(cache_manager) = &self.cache_manager {
            cache_manager.reset_stats();
        }
    }

    /// 执行函数并缓存结果
    pub fn execute_function_with_cache(
        &self,
        function_ref: &FunctionRef,
        args: &[FieldValue],
    ) -> Result<FieldValue, ExpressionError> {
        // 缓存功能暂时禁用，因为需要修复生命周期问题

        // 执行函数
        let result = function_ref.execute(args);

        result
    }

    /// 将参数转换为哈希值用于缓存键
    fn args_to_hash(&self, args: &[FieldValue]) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        for arg in args {
            arg.hash(&mut hasher);
        }
        format!("{:x}", hasher.finish())
    }
}

impl Default for BasicExpressionContext {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for BasicExpressionContext {
    fn clone(&self) -> Self {
        Self {
            variables: self.variables.clone(),
            functions: self.functions.clone(),
            custom_functions: self.custom_functions.clone(),
            parent: self.parent.as_ref().map(|p| Box::new(p.as_ref().clone())),
            depth: self.get_depth(),
            cache_manager: self.cache_manager.clone(),
        }
    }
}

impl ContextBase for BasicExpressionContext {
    fn id(&self) -> &str {
        // 使用深度作为ID的一部分，但需要返回一个引用
        // 这里使用一个静态字符串作为ID
        "expression_context"
    }

    fn context_type(&self) -> ContextType {
        ContextType::Expression
    }

    fn created_at(&self) -> std::time::SystemTime {
        std::time::SystemTime::now() // 使用当前时间作为创建时间
    }

    fn updated_at(&self) -> std::time::SystemTime {
        std::time::SystemTime::now() // 使用当前时间作为更新时间
    }

    fn is_valid(&self) -> bool {
        true // 表达式上下文总是有效的
    }
}

impl MutableContext for BasicExpressionContext {
    fn touch(&mut self) {
        // 更新时间戳
    }

    fn invalidate(&mut self) {
        // 表达式上下文不支持无效化
    }

    fn revalidate(&mut self) -> bool {
        true // 表达式上下文总是有效的
    }
}

impl super::base::HierarchicalContext for BasicExpressionContext {
    fn parent_id(&self) -> Option<&str> {
        self.parent.as_ref().map(|_| "parent_expression")
    }

    fn depth(&self) -> usize {
        self.depth
    }
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
}

impl Default for EvaluationOptions {
    fn default() -> Self {
        Self {
            strict_mode: false,
            allow_implicit_conversion: true,
            max_recursion_depth: 1000,
            timeout_ms: Some(30000), // 30秒
            cache_config: CacheConfig::default(),
        }
    }
}

/// 表达式求值统计
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvaluationStatistics {
    /// 求值的表达式数量
    pub expressions_evaluated: usize,
    /// 函数调用次数
    pub function_calls: usize,
    /// 变量访问次数
    pub variable_accesses: usize,
    /// 总求值时间（微秒）
    pub total_evaluation_time_us: u64,
    /// 平均求值时间（微秒）
    pub average_evaluation_time_us: f64,
    /// 最大递归深度
    pub max_recursion_depth: usize,
    /// 详细的缓存统计信息
    pub cache_stats: Option<ExpressionCacheStats>,
}

impl EvaluationStatistics {
    /// 创建新的求值统计
    pub fn new() -> Self {
        Self {
            expressions_evaluated: 0,
            function_calls: 0,
            variable_accesses: 0,
            total_evaluation_time_us: 0,
            average_evaluation_time_us: 0.0,
            max_recursion_depth: 0,
            cache_stats: None,
        }
    }

    /// 创建带缓存统计的求值统计
    pub fn with_cache_stats(cache_stats: ExpressionCacheStats) -> Self {
        Self {
            expressions_evaluated: 0,
            function_calls: 0,
            variable_accesses: 0,
            total_evaluation_time_us: 0,
            average_evaluation_time_us: 0.0,
            max_recursion_depth: 0,
            cache_stats: Some(cache_stats),
        }
    }

    /// 记录表达式求值
    pub fn record_expression_evaluation(&mut self, evaluation_time_us: u64) {
        self.expressions_evaluated += 1;
        self.total_evaluation_time_us += evaluation_time_us;
        self.average_evaluation_time_us =
            self.total_evaluation_time_us as f64 / self.expressions_evaluated as f64;
    }

    /// 记录函数调用
    pub fn record_function_call(&mut self) {
        self.function_calls += 1;
    }

    /// 记录变量访问
    pub fn record_variable_access(&mut self) {
        self.variable_accesses += 1;
    }

    /// 更新缓存统计信息
    pub fn update_cache_stats(&mut self, cache_stats: Option<ExpressionCacheStats>) {
        self.cache_stats = cache_stats;
    }

    /// 更新最大递归深度
    pub fn update_max_recursion_depth(&mut self, depth: usize) {
        if depth > self.max_recursion_depth {
            self.max_recursion_depth = depth;
        }
    }

    /// 获取总体缓存命中率
    pub fn overall_cache_hit_rate(&self) -> f64 {
        if let Some(ref cache_stats) = self.cache_stats {
            let total_hits = cache_stats.function_cache_hits +
                           cache_stats.expression_cache_hits +
                           cache_stats.variable_cache_hits;
            let total_misses = cache_stats.function_cache_misses +
                             cache_stats.expression_cache_misses +
                             cache_stats.variable_cache_misses;
            let total_requests = total_hits + total_misses;
            
            if total_requests == 0 {
                0.0
            } else {
                total_hits as f64 / total_requests as f64
            }
        } else {
            0.0
        }
    }

    /// 获取函数缓存命中率
    pub fn function_cache_hit_rate(&self) -> f64 {
        self.cache_stats
            .as_ref()
            .map(|stats| stats.function_cache_hit_rate)
            .unwrap_or_else(|| 0.0)
    }

    /// 获取表达式缓存命中率
    pub fn expression_cache_hit_rate(&self) -> f64 {
        self.cache_stats
            .as_ref()
            .map(|stats| stats.expression_cache_hit_rate)
            .unwrap_or_else(|| 0.0)
    }

    /// 获取变量缓存命中率
    pub fn variable_cache_hit_rate(&self) -> f64 {
        self.cache_stats
            .as_ref()
            .map(|stats| stats.variable_cache_hit_rate)
            .unwrap_or_else(|| 0.0)
    }
}

impl Default for EvaluationStatistics {
    fn default() -> Self {
        Self::new()
    }
}

// ExpressionContextCore和StorageExpressionContextCore已移动到default_context模块
