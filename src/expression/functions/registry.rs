//! 函数注册表
//!
//! 提供函数的注册、查找和执行功能
//! 具体函数实现位于 builtin/ 子模块

use crate::core::error::{ExpressionError, ExpressionErrorType};
use crate::core::Value;
use std::collections::HashMap;
use std::sync::Arc;
use super::signature::{FunctionSignature, RegisteredFunction, ValueType};
use super::BuiltinFunction;
use super::CustomFunction;
use super::ExpressionFunction;

/// 函数注册表
#[derive(Debug)]
pub struct FunctionRegistry {
    functions: HashMap<String, Vec<RegisteredFunction>>,
    builtin_functions: HashMap<String, BuiltinFunction>,
    custom_functions: HashMap<String, CustomFunction>,
}

impl Default for FunctionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl FunctionRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            functions: HashMap::new(),
            builtin_functions: HashMap::new(),
            custom_functions: HashMap::new(),
        };
        registry.register_all_builtin_functions();
        registry
    }

    /// 注册函数
    pub fn register<F>(&mut self, name: &str, signature: FunctionSignature, func: F)
    where
        F: Fn(&[Value]) -> Result<Value, ExpressionError> + 'static + Send + Sync,
    {
        let registered = RegisteredFunction::new(
            signature,
            Box::new(func),
        );
        self.functions
            .entry(name.to_string())
            .or_insert_with(Vec::new)
            .push(registered);
    }

    /// 查找函数（根据参数数量）
    pub fn find(&self, name: &str, arity: usize) -> Option<&Vec<RegisteredFunction>> {
        self.functions.get(name).filter(|funcs| {
            funcs.iter().any(|f| f.signature.check_arity(arity))
        })
    }

    /// 执行函数（支持函数重载）
    pub fn execute(&self, name: &str, args: &[Value]) -> Result<Value, ExpressionError> {
        let funcs = self.functions.get(name).ok_or_else(|| {
            ExpressionError::new(
                ExpressionErrorType::UndefinedFunction,
                format!("未定义的函数: {}", name),
            )
        })?;

        let mut best_match: Option<&RegisteredFunction> = None;
        let mut best_score = i32::MIN;

        for registered in funcs {
            let score = registered.signature.type_matching_score(args);
            if score > best_score {
                best_score = score;
                best_match = Some(registered);
            }
        }

        if let Some(registered) = best_match {
            if best_score > i32::MIN {
                return (registered.body)(args);
            }
        }

        let signatures: Vec<_> = funcs.iter()
            .map(|f| format!("{}", f.signature.arg_types.iter().map(|t| t.to_string()).collect::<Vec<_>>().join(", ")))
            .collect();

        Err(ExpressionError::new(
            ExpressionErrorType::TypeError,
            format!(
                "函数 {} 参数类型不匹配。期望: {}，实际: {}",
                name,
                signatures.join(" | "),
                args.iter()
                    .map(|v| format!("{}", ValueType::from_value(v)))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        ))
    }

    /// 获取函数签名
    pub fn get_signatures(&self, name: &str) -> Option<Vec<FunctionSignature>> {
        self.functions.get(name).map(|funcs| {
            funcs.iter().map(|f| f.signature.clone()).collect()
        })
    }

    /// 检查函数是否存在
    pub fn contains(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }

    /// 获取所有函数名称
    pub fn function_names(&self) -> Vec<&str> {
        self.functions.keys().map(|s| s.as_str()).collect()
    }

    /// 获取函数（根据名称）
    pub fn get(&self, name: &str) -> Option<&Vec<RegisteredFunction>> {
        self.functions.get(name)
    }

    /// 重新注册所有内置函数
    pub fn reregister_all_builtins(&mut self) {
        self.register_all_builtin_functions();
    }

    /// 注册自定义函数
    pub fn register_custom<F>(&mut self, name: &str, signature: FunctionSignature, func: F)
    where
        F: Fn(&[Value]) -> Result<Value, ExpressionError> + 'static + Send + Sync,
    {
        self.register(name, signature, func);
    }

    /// 注册内置函数
    pub fn register_builtin(&mut self, function: BuiltinFunction) {
        self.builtin_functions.insert(function.name().to_string(), function);
    }

    /// 获取内置函数
    pub fn get_builtin(&self, name: &str) -> Option<&BuiltinFunction> {
        self.builtin_functions.get(name)
    }

    /// 注册自定义函数（完整形式）
    pub fn register_custom_full(&mut self, function: CustomFunction) {
        self.custom_functions.insert(function.name.clone(), function);
    }

    /// 获取自定义函数
    pub fn get_custom(&self, name: &str) -> Option<&CustomFunction> {
        self.custom_functions.get(name)
    }

    /// 注册所有内置函数
    fn register_all_builtin_functions(&mut self) {
        super::builtin::register_all(self);
    }
}

/// 全局函数注册表实例
pub fn global_registry() -> Arc<FunctionRegistry> {
    use std::sync::OnceLock;
    static REGISTRY: OnceLock<Arc<FunctionRegistry>> = OnceLock::new();
    REGISTRY.get_or_init(|| Arc::new(FunctionRegistry::new())).clone()
}
