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
//! use crate::expression::functions::BuiltinFunction;
//!
//! let func = BuiltinFunction::Math(MathFunction::Abs);
//! let result = func.execute(&[Value::Int(-5)]);
//! ```

pub mod signature;
pub mod registry;
pub mod builtin;

pub use signature::{FunctionSignature, RegisteredFunction, ValueType};
pub use registry::{global_registry, global_registry_ref, FunctionRegistry};

// 从 builtin 子模块重新导出函数类型
pub use builtin::container::ContainerFunction;
pub use builtin::conversion::ConversionFunction;
pub use builtin::datetime::DateTimeFunction;
pub use builtin::geography::GeographyFunction;
pub use builtin::graph::GraphFunction;
pub use builtin::math::MathFunction;
pub use builtin::path::PathFunction;
pub use builtin::regex::RegexFunction;
pub use builtin::string::StringFunction;
pub use builtin::utility::UtilityFunction;

use crate::core::error::{ExpressionError, ExpressionErrorType};
use crate::core::types::operators::AggregateFunction;
use crate::core::Value;
use crate::expression::context::CacheManager;

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
    pub fn execute_with_cache(
        &self,
        args: &[Value],
        cache: &mut CacheManager,
    ) -> Result<Value, ExpressionError> {
        match self {
            OwnedFunctionRef::Builtin(f) => f.execute_with_cache(args, cache),
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
    /// 地理空间函数
    Geography(GeographyFunction),
    /// 实用函数
    Utility(UtilityFunction),
    /// 图相关函数
    Graph(GraphFunction),
    /// 容器操作函数
    Container(ContainerFunction),
    /// 路径函数
    Path(PathFunction),
}

impl BuiltinFunction {
    /// 获取函数名称
    pub fn name(&self) -> &str {
        match self {
            BuiltinFunction::Math(f) => f.name(),
            BuiltinFunction::String(f) => f.name(),
            BuiltinFunction::Regex(f) => f.name(),
            BuiltinFunction::Aggregate(f) => f.name(),
            BuiltinFunction::Conversion(f) => f.name(),
            BuiltinFunction::DateTime(f) => f.name(),
            BuiltinFunction::Geography(f) => f.name(),
            BuiltinFunction::Utility(f) => f.name(),
            BuiltinFunction::Graph(f) => f.name(),
            BuiltinFunction::Container(f) => f.name(),
            BuiltinFunction::Path(f) => f.name(),
        }
    }

    /// 获取参数数量
    pub fn arity(&self) -> usize {
        match self {
            BuiltinFunction::Math(f) => f.arity(),
            BuiltinFunction::String(f) => f.arity(),
            BuiltinFunction::Regex(f) => f.arity(),
            BuiltinFunction::Aggregate(f) => f.arity(),
            BuiltinFunction::Conversion(f) => f.arity(),
            BuiltinFunction::DateTime(f) => f.arity(),
            BuiltinFunction::Geography(f) => f.arity(),
            BuiltinFunction::Utility(f) => f.arity(),
            BuiltinFunction::Graph(f) => f.arity(),
            BuiltinFunction::Container(f) => f.arity(),
            BuiltinFunction::Path(f) => f.arity(),
        }
    }

    /// 检查是否接受可变参数
    pub fn is_variadic(&self) -> bool {
        match self {
            BuiltinFunction::Math(f) => f.is_variadic(),
            BuiltinFunction::String(f) => f.is_variadic(),
            BuiltinFunction::Regex(f) => f.is_variadic(),
            BuiltinFunction::Aggregate(f) => f.is_variadic(),
            BuiltinFunction::Conversion(f) => f.is_variadic(),
            BuiltinFunction::DateTime(f) => f.is_variadic(),
            BuiltinFunction::Geography(f) => f.is_variadic(),
            BuiltinFunction::Utility(f) => f.is_variadic(),
            BuiltinFunction::Graph(f) => f.is_variadic(),
            BuiltinFunction::Container(f) => f.is_variadic(),
            BuiltinFunction::Path(f) => f.is_variadic(),
        }
    }

    /// 获取函数描述
    pub fn description(&self) -> &str {
        match self {
            BuiltinFunction::Math(f) => f.description(),
            BuiltinFunction::String(f) => f.description(),
            BuiltinFunction::Regex(f) => f.description(),
            BuiltinFunction::Aggregate(f) => f.description(),
            BuiltinFunction::Conversion(f) => f.description(),
            BuiltinFunction::DateTime(f) => f.description(),
            BuiltinFunction::Geography(f) => f.description(),
            BuiltinFunction::Utility(f) => f.description(),
            BuiltinFunction::Graph(f) => f.description(),
            BuiltinFunction::Container(f) => f.description(),
            BuiltinFunction::Path(f) => f.description(),
        }
    }

    /// 执行函数
    pub fn execute(&self, args: &[Value]) -> Result<Value, ExpressionError> {
        match self {
            BuiltinFunction::Math(f) => f.execute(args),
            BuiltinFunction::String(f) => f.execute(args),
            BuiltinFunction::Regex(f) => f.execute(args),
            BuiltinFunction::Aggregate(_) => Err(ExpressionError::new(
                ExpressionErrorType::InvalidOperation,
                "聚合函数需要在聚合上下文中执行".to_string(),
            )),
            BuiltinFunction::Conversion(f) => f.execute(args),
            BuiltinFunction::DateTime(f) => f.execute(args),
            BuiltinFunction::Geography(f) => f.execute(args),
            BuiltinFunction::Utility(f) => f.execute(args),
            BuiltinFunction::Graph(f) => f.execute(args),
            BuiltinFunction::Container(f) => f.execute(args),
            BuiltinFunction::Path(f) => f.execute(args),
        }
    }

    /// 执行函数（带缓存）
    pub fn execute_with_cache(
        &self,
        args: &[Value],
        cache: &mut CacheManager,
    ) -> Result<Value, ExpressionError> {
        match self {
            BuiltinFunction::Regex(f) => f.execute_with_cache(args, cache),
            BuiltinFunction::DateTime(f) => f.execute_with_cache(args, cache),
            _ => self.execute(args),
        }
    }
}

impl ExpressionFunction for BuiltinFunction {
    fn name(&self) -> &str {
        self.name()
    }

    fn arity(&self) -> usize {
        self.arity()
    }

    fn is_variadic(&self) -> bool {
        self.is_variadic()
    }

    fn execute(&self, args: &[Value]) -> Result<Value, ExpressionError> {
        self.execute(args)
    }

    fn description(&self) -> &str {
        self.description()
    }
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

impl CustomFunction {
    /// 创建新的自定义函数
    pub fn new(
        name: impl Into<String>,
        arity: usize,
        is_variadic: bool,
        description: impl Into<String>,
        function_id: u64,
    ) -> Self {
        Self {
            name: name.into(),
            arity,
            is_variadic,
            description: description.into(),
            function_id,
        }
    }

    /// 执行函数
    pub fn execute(&self, _args: &[Value]) -> Result<Value, ExpressionError> {
        Err(ExpressionError::new(
            ExpressionErrorType::InvalidOperation,
            format!("自定义函数 '{}' 需要在自定义函数上下文中执行", self.name),
        ))
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
        self.execute(args)
    }

    fn description(&self) -> &str {
        &self.description
    }
}
