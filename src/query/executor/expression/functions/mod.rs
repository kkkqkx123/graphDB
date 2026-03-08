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
//! use crate::query::executor::expression::functions::BuiltinFunction;
//!
//! let func = BuiltinFunction::Math(MathFunction::Abs);
//! let result = func.execute(&[Value::Int(-5)]);
//! ```

pub mod builtin;
pub mod registry;
pub mod signature;

pub use registry::{global_registry, global_registry_ref, FunctionRegistry};
pub use signature::ValueType;

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
use std::ffi::c_void;

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
    /// 
    /// 注意：缓存功能已移除，直接调用execute
    pub fn execute_with_cache(
        &self,
        args: &[Value],
        _cache: &mut (),
    ) -> Result<Value, ExpressionError> {
        self.execute(args)
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
    /// 
    /// 注意：缓存功能已移除，此方法直接调用execute
    pub fn execute_with_cache(
        &self,
        args: &[Value],
        _cache: &mut (),
    ) -> Result<Value, ExpressionError> {
        self.execute(args)
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

/// C 函数上下文结构（不透明指针）
pub struct CFunctionContext {
    /// 结果值
    pub result: Option<Value>,
    /// 错误消息
    pub error: Option<String>,
    /// 聚合状态（用于聚合函数）
    pub aggregate_state: Option<Box<dyn std::any::Any + Send>>,
    /// 用户数据指针
    pub user_data: usize,
}

impl CFunctionContext {
    pub fn new() -> Self {
        Self {
            result: None,
            error: None,
            aggregate_state: None,
            user_data: 0,
        }
    }

    pub fn with_user_data(user_data: usize) -> Self {
        Self {
            result: None,
            error: None,
            aggregate_state: None,
            user_data,
        }
    }

    pub fn set_result(&mut self, value: Value) {
        self.result = Some(value);
    }

    pub fn set_error(&mut self, error: String) {
        self.error = Some(error);
    }

    /// 设置聚合状态
    pub fn set_aggregate_state<T: std::any::Any + Send + 'static>(&mut self, state: T) {
        self.aggregate_state = Some(Box::new(state));
    }

    /// 获取聚合状态
    pub fn get_aggregate_state<T: std::any::Any + Send + 'static>(&self) -> Option<&T> {
        self.aggregate_state.as_ref()?.downcast_ref::<T>()
    }

    /// 获取聚合状态的可变引用
    pub fn get_aggregate_state_mut<T: std::any::Any + Send + 'static>(&mut self) -> Option<&mut T> {
        self.aggregate_state.as_mut()?.downcast_mut::<T>()
    }
}

/// 标量函数回调类型
pub type ScalarFunctionCallback = extern "C" fn(*mut CFunctionContext, i32, *const Value);

/// 聚合步骤回调类型
pub type AggregateStepCallback = extern "C" fn(*mut CFunctionContext, i32, *const Value);

/// 聚合最终回调类型
pub type AggregateFinalCallback = extern "C" fn(*mut CFunctionContext);

/// 自定义函数实现类型
#[derive(Clone, Copy)]
pub enum CustomFunctionImpl {
    /// Rust 实现的自定义函数
    Rust(fn(&[Value]) -> Result<Value, ExpressionError>),
    /// C 回调实现的标量函数
    C {
        /// 标量函数回调（存储函数指针地址）
        scalar_callback: usize,
        /// 用户数据（存储指针地址）
        user_data: usize,
    },
    /// C 回调实现的聚合函数
    Aggregate {
        /// 聚合步骤回调（存储函数指针地址）
        step_callback: usize,
        /// 聚合最终回调（存储函数指针地址）
        final_callback: usize,
        /// 用户数据（存储指针地址）
        user_data: usize,
    },
}

impl std::fmt::Debug for CustomFunctionImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CustomFunctionImpl::Rust(_) => write!(f, "Rust closure"),
            CustomFunctionImpl::C { .. } => write!(f, "C scalar callback"),
            CustomFunctionImpl::Aggregate { .. } => write!(f, "C aggregate callback"),
        }
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
    /// 函数实现
    pub implementation: CustomFunctionImpl,
}

impl CustomFunction {
    /// 创建新的 Rust 自定义函数
    pub fn new_rust(
        name: impl Into<String>,
        arity: usize,
        is_variadic: bool,
        description: impl Into<String>,
        implementation: fn(&[Value]) -> Result<Value, ExpressionError>,
    ) -> Self {
        Self {
            name: name.into(),
            arity,
            is_variadic,
            description: description.into(),
            implementation: CustomFunctionImpl::Rust(implementation),
        }
    }

    /// 创建新的 C 回调自定义函数
    pub fn new_c(
        name: impl Into<String>,
        arity: usize,
        is_variadic: bool,
        description: impl Into<String>,
        scalar_callback: ScalarFunctionCallback,
        user_data: *mut c_void,
    ) -> Self {
        Self {
            name: name.into(),
            arity,
            is_variadic,
            description: description.into(),
            implementation: CustomFunctionImpl::C {
                scalar_callback: scalar_callback as usize,
                user_data: user_data as usize,
            },
        }
    }

    /// 创建新的 C 回调聚合函数
    pub fn new_c_aggregate(
        name: impl Into<String>,
        arity: usize,
        is_variadic: bool,
        description: impl Into<String>,
        step_callback: AggregateStepCallback,
        final_callback: AggregateFinalCallback,
        user_data: *mut c_void,
    ) -> Self {
        Self {
            name: name.into(),
            arity,
            is_variadic,
            description: description.into(),
            implementation: CustomFunctionImpl::Aggregate {
                step_callback: step_callback as usize,
                final_callback: final_callback as usize,
                user_data: user_data as usize,
            },
        }
    }

    /// 检查是否为聚合函数
    pub fn is_aggregate(&self) -> bool {
        matches!(self.implementation, CustomFunctionImpl::Aggregate { .. })
    }

    /// 执行函数
    pub fn execute(&self, args: &[Value]) -> Result<Value, ExpressionError> {
        match &self.implementation {
            CustomFunctionImpl::Rust(func) => func(args),
            CustomFunctionImpl::C {
                scalar_callback,
                user_data: _,
            } => {
                // 创建 C 函数上下文
                let mut ctx = CFunctionContext::new();
                let ctx_ptr = &mut ctx as *mut CFunctionContext;

                // 将 usize 转换回函数指针
                let callback: ScalarFunctionCallback = unsafe {
                    std::mem::transmute(*scalar_callback)
                };

                // 调用 C 回调
                callback(ctx_ptr, args.len() as i32, args.as_ptr());

                // 检查错误
                if let Some(error) = ctx.error {
                    return Err(ExpressionError::new(
                        ExpressionErrorType::FunctionExecutionError,
                        error,
                    ));
                }

                // 返回结果
                ctx.result.ok_or_else(|| {
                    ExpressionError::new(
                        ExpressionErrorType::FunctionExecutionError,
                        format!("函数 '{}' 未设置返回值", self.name),
                    )
                })
            }
            CustomFunctionImpl::Aggregate { .. } => {
                Err(ExpressionError::new(
                    ExpressionErrorType::InvalidOperation,
                    "聚合函数需要在聚合上下文中执行".to_string(),
                ))
            }
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
        self.execute(args)
    }

    fn description(&self) -> &str {
        &self.description
    }
}
