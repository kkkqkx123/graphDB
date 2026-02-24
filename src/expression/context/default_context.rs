//! 默认表达式上下文实现
//!
//! 提供表达式求值过程中的上下文管理

use crate::core::Value;
use crate::expression::context::cache_manager::CacheManager;
use crate::expression::functions::registry::FunctionRegistry;
use std::collections::HashMap;

/// 表达式上下文
///
/// 提供表达式求值所需的上下文环境，包括：
/// - 变量存储
/// - 函数注册
/// - 正则缓存
#[derive(Debug)]
pub struct DefaultExpressionContext {
    /// 变量存储
    variables: HashMap<String, Value>,
    /// 函数注册表
    function_registry: FunctionRegistry,
    /// 缓存管理器
    cache_manager: CacheManager,
}

impl DefaultExpressionContext {
    /// 创建新的上下文
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            function_registry: FunctionRegistry::new(),
            cache_manager: CacheManager::new(),
        }
    }

    /// 创建带有全局函数的上下文
    pub fn with_global_functions() -> Self {
        let mut context = Self::new();
        context.register_global_functions();
        context
    }

    /// 注册全局函数
    fn register_global_functions(&mut self) {
        use crate::expression::functions::{BuiltinFunction, MathFunction, StringFunction, RegexFunction, ConversionFunction, DateTimeFunction};
        
        // 数学函数
        self.function_registry.register_builtin(BuiltinFunction::Math(MathFunction::Abs));
        self.function_registry.register_builtin(BuiltinFunction::Math(MathFunction::Sqrt));
        self.function_registry.register_builtin(BuiltinFunction::Math(MathFunction::Pow));
        self.function_registry.register_builtin(BuiltinFunction::Math(MathFunction::Log));
        self.function_registry.register_builtin(BuiltinFunction::Math(MathFunction::Log10));
        self.function_registry.register_builtin(BuiltinFunction::Math(MathFunction::Sin));
        self.function_registry.register_builtin(BuiltinFunction::Math(MathFunction::Cos));
        self.function_registry.register_builtin(BuiltinFunction::Math(MathFunction::Tan));
        self.function_registry.register_builtin(BuiltinFunction::Math(MathFunction::Round));
        self.function_registry.register_builtin(BuiltinFunction::Math(MathFunction::Ceil));
        self.function_registry.register_builtin(BuiltinFunction::Math(MathFunction::Floor));

        // 字符串函数
        self.function_registry.register_builtin(BuiltinFunction::String(StringFunction::Length));
        self.function_registry.register_builtin(BuiltinFunction::String(StringFunction::Upper));
        self.function_registry.register_builtin(BuiltinFunction::String(StringFunction::Lower));
        self.function_registry.register_builtin(BuiltinFunction::String(StringFunction::Trim));
        self.function_registry.register_builtin(BuiltinFunction::String(StringFunction::Substring));
        self.function_registry.register_builtin(BuiltinFunction::String(StringFunction::Concat));
        self.function_registry.register_builtin(BuiltinFunction::String(StringFunction::Replace));
        self.function_registry.register_builtin(BuiltinFunction::String(StringFunction::Contains));
        self.function_registry.register_builtin(BuiltinFunction::String(StringFunction::StartsWith));
        self.function_registry.register_builtin(BuiltinFunction::String(StringFunction::EndsWith));

        // 正则函数
        self.function_registry.register_builtin(BuiltinFunction::Regex(RegexFunction::RegexMatch));
        self.function_registry.register_builtin(BuiltinFunction::Regex(RegexFunction::RegexReplace));
        self.function_registry.register_builtin(BuiltinFunction::Regex(RegexFunction::RegexFind));

        // 转换函数
        self.function_registry.register_builtin(BuiltinFunction::Conversion(ConversionFunction::ToString));
        self.function_registry.register_builtin(BuiltinFunction::Conversion(ConversionFunction::ToInt));
        self.function_registry.register_builtin(BuiltinFunction::Conversion(ConversionFunction::ToFloat));
        self.function_registry.register_builtin(BuiltinFunction::Conversion(ConversionFunction::ToBool));

        // 日期时间函数
        self.function_registry.register_builtin(BuiltinFunction::DateTime(DateTimeFunction::Now));
        self.function_registry.register_builtin(BuiltinFunction::DateTime(DateTimeFunction::Date));
        self.function_registry.register_builtin(BuiltinFunction::DateTime(DateTimeFunction::Time));
        self.function_registry.register_builtin(BuiltinFunction::DateTime(DateTimeFunction::Year));
        self.function_registry.register_builtin(BuiltinFunction::DateTime(DateTimeFunction::Month));
        self.function_registry.register_builtin(BuiltinFunction::DateTime(DateTimeFunction::Day));
        self.function_registry.register_builtin(BuiltinFunction::DateTime(DateTimeFunction::Hour));
        self.function_registry.register_builtin(BuiltinFunction::DateTime(DateTimeFunction::Minute));
        self.function_registry.register_builtin(BuiltinFunction::DateTime(DateTimeFunction::Second));
    }

    /// 注册自定义函数
    pub fn register_function(&mut self, function: crate::expression::functions::CustomFunction) {
        self.function_registry.register_custom_full(function);
    }

    /// 添加变量
    pub fn add_variable(mut self, name: String, value: Value) -> Self {
        self.variables.insert(name, value);
        self
    }

    /// 批量添加变量
    pub fn with_variables<I>(mut self, variables: I) -> Self
    where
        I: IntoIterator<Item = (String, Value)>,
    {
        for (name, value) in variables {
            self.variables.insert(name, value);
        }
        self
    }
}

impl Default for DefaultExpressionContext {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::expression::evaluator::traits::ExpressionContext for DefaultExpressionContext {
    fn get_variable(&self, name: &str) -> Option<Value> {
        self.variables.get(name).cloned()
    }

    fn set_variable(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }

    fn get_function(&self, name: &str) -> Option<crate::expression::functions::FunctionRef> {
        self.function_registry.get_builtin(name).map(|f| crate::expression::functions::FunctionRef::Builtin(f))
            .or_else(|| self.function_registry.get_custom(name).map(|f| crate::expression::functions::FunctionRef::Custom(f)))
    }

    fn supports_cache(&self) -> bool {
        true
    }

    fn get_cache(&mut self) -> Option<&mut CacheManager> {
        Some(&mut self.cache_manager)
    }
}
