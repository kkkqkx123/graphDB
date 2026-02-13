//! 函数注册表
//!
//! 提供内置函数的注册、查找和执行功能

use crate::core::error::{ExpressionError, ExpressionErrorType};
use crate::core::Value;
use chrono::{Datelike, NaiveDate, Utc};
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
        self.register_math_functions();
        self.register_string_functions();
        self.register_regex_functions();
        self.register_conversion_functions();
        self.register_datetime_functions();
    }

    fn register_math_functions(&mut self) {
        let registry = self;

        // abs - INT 版本
        registry.register(
            "abs",
            FunctionSignature::new(
                "abs",
                vec![ValueType::Int],
                ValueType::Int,
                1,
                1,
                true,
                "计算绝对值",
            ),
            |args| {
                match &args[0] {
                    Value::Int(i) => Ok(Value::Int(i.abs())),
                    Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                    _ => Err(ExpressionError::type_error("abs函数需要整数类型")),
                }
            },
        );

        // abs - FLOAT 版本
        registry.register(
            "abs",
            FunctionSignature::new(
                "abs",
                vec![ValueType::Float],
                ValueType::Float,
                1,
                1,
                true,
                "计算绝对值",
            ),
            |args| {
                match &args[0] {
                    Value::Float(f) => Ok(Value::Float(f.abs())),
                    Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                    _ => Err(ExpressionError::type_error("abs函数需要浮点数类型")),
                }
            },
        );

        // ceil - INT 版本
        registry.register(
            "ceil",
            FunctionSignature::new(
                "ceil",
                vec![ValueType::Int],
                ValueType::Float,
                1,
                1,
                true,
                "向上取整",
            ),
            |args| {
                match &args[0] {
                    Value::Int(i) => Ok(Value::Float(*i as f64)),
                    Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                    _ => Err(ExpressionError::type_error("ceil函数需要整数类型")),
                }
            },
        );

        // ceil - FLOAT 版本
        registry.register(
            "ceil",
            FunctionSignature::new(
                "ceil",
                vec![ValueType::Float],
                ValueType::Float,
                1,
                1,
                true,
                "向上取整",
            ),
            |args| {
                match &args[0] {
                    Value::Float(f) => Ok(Value::Float(f.ceil())),
                    Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                    _ => Err(ExpressionError::type_error("ceil函数需要浮点数类型")),
                }
            },
        );

        // floor - INT 版本
        registry.register(
            "floor",
            FunctionSignature::new(
                "floor",
                vec![ValueType::Int],
                ValueType::Float,
                1,
                1,
                true,
                "向下取整",
            ),
            |args| {
                match &args[0] {
                    Value::Int(i) => Ok(Value::Float(*i as f64)),
                    Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                    _ => Err(ExpressionError::type_error("floor函数需要整数类型")),
                }
            },
        );

        // floor - FLOAT 版本
        registry.register(
            "floor",
            FunctionSignature::new(
                "floor",
                vec![ValueType::Float],
                ValueType::Float,
                1,
                1,
                true,
                "向下取整",
            ),
            |args| {
                match &args[0] {
                    Value::Float(f) => Ok(Value::Float(f.floor())),
                    Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                    _ => Err(ExpressionError::type_error("floor函数需要浮点数类型")),
                }
            },
        );

        // round - INT 版本
        registry.register(
            "round",
            FunctionSignature::new(
                "round",
                vec![ValueType::Int],
                ValueType::Int,
                1,
                1,
                true,
                "四舍五入",
            ),
            |args| {
                match &args[0] {
                    Value::Int(i) => Ok(Value::Int(*i)),
                    Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                    _ => Err(ExpressionError::type_error("round函数需要整数类型")),
                }
            },
        );

        // round - FLOAT 版本
        registry.register(
            "round",
            FunctionSignature::new(
                "round",
                vec![ValueType::Float],
                ValueType::Float,
                1,
                1,
                true,
                "四舍五入",
            ),
            |args| {
                match &args[0] {
                    Value::Float(f) => Ok(Value::Float(f.round())),
                    Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                    _ => Err(ExpressionError::type_error("round函数需要浮点数类型")),
                }
            },
        );

        // sqrt - INT 版本
        registry.register(
            "sqrt",
            FunctionSignature::new(
                "sqrt",
                vec![ValueType::Int],
                ValueType::Float,
                1,
                1,
                true,
                "计算平方根",
            ),
            |args| {
                match &args[0] {
                    Value::Int(i) if *i >= 0 => Ok(Value::Float(((*i) as f64).sqrt())),
                    Value::Int(i) if *i < 0 => Err(ExpressionError::new(
                        ExpressionErrorType::InvalidOperation,
                        "sqrt of negative number".to_string(),
                    )),
                    Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                    _ => Err(ExpressionError::type_error("sqrt函数需要整数类型")),
                }
            },
        );

        // sqrt - FLOAT 版本
        registry.register(
            "sqrt",
            FunctionSignature::new(
                "sqrt",
                vec![ValueType::Float],
                ValueType::Float,
                1,
                1,
                true,
                "计算平方根",
            ),
            |args| {
                match &args[0] {
                    Value::Float(f) if *f >= 0.0 => Ok(Value::Float(f.sqrt())),
                    Value::Float(f) if *f < 0.0 => Err(ExpressionError::new(
                        ExpressionErrorType::InvalidOperation,
                        "sqrt of negative number".to_string(),
                    )),
                    Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                    _ => Err(ExpressionError::type_error("sqrt函数需要浮点数类型")),
                }
            },
        );

        // pow - INT, INT 版本
        registry.register(
            "pow",
            FunctionSignature::new(
                "pow",
                vec![ValueType::Int, ValueType::Int],
                ValueType::Float,
                2,
                2,
                true,
                "计算幂",
            ),
            |args| {
                match (&args[0], &args[1]) {
                    (Value::Int(base), Value::Int(exp)) => {
                        Ok(Value::Float(((*base) as f64).powf(*exp as f64)))
                    }
                    (Value::Null(_), _) | (_, Value::Null(_)) => {
                        Ok(Value::Null(crate::core::value::NullType::Null))
                    }
                    _ => Err(ExpressionError::type_error("pow函数需要整数类型")),
                }
            },
        );

        // pow - FLOAT, FLOAT 版本
        registry.register(
            "pow",
            FunctionSignature::new(
                "pow",
                vec![ValueType::Float, ValueType::Float],
                ValueType::Float,
                2,
                2,
                true,
                "计算幂",
            ),
            |args| {
                match (&args[0], &args[1]) {
                    (Value::Float(base), Value::Float(exp)) => {
                        Ok(Value::Float(base.powf(*exp)))
                    }
                    (Value::Null(_), _) | (_, Value::Null(_)) => {
                        Ok(Value::Null(crate::core::value::NullType::Null))
                    }
                    _ => Err(ExpressionError::type_error("pow函数需要浮点数类型")),
                }
            },
        );

        // pow - INT, FLOAT 版本
        registry.register(
            "pow",
            FunctionSignature::new(
                "pow",
                vec![ValueType::Int, ValueType::Float],
                ValueType::Float,
                2,
                2,
                true,
                "计算幂",
            ),
            |args| {
                match (&args[0], &args[1]) {
                    (Value::Int(base), Value::Float(exp)) => {
                        Ok(Value::Float(((*base) as f64).powf(*exp)))
                    }
                    (Value::Null(_), _) | (_, Value::Null(_)) => {
                        Ok(Value::Null(crate::core::value::NullType::Null))
                    }
                    _ => Err(ExpressionError::type_error("pow函数需要整数和浮点数类型")),
                }
            },
        );

        // pow - FLOAT, INT 版本
        registry.register(
            "pow",
            FunctionSignature::new(
                "pow",
                vec![ValueType::Float, ValueType::Int],
                ValueType::Float,
                2,
                2,
                true,
                "计算幂",
            ),
            |args| {
                match (&args[0], &args[1]) {
                    (Value::Float(base), Value::Int(exp)) => {
                        Ok(Value::Float(base.powf(*exp as f64)))
                    }
                    (Value::Null(_), _) | (_, Value::Null(_)) => {
                        Ok(Value::Null(crate::core::value::NullType::Null))
                    }
                    _ => Err(ExpressionError::type_error("pow函数需要浮点数和整数类型")),
                }
            },
        );

        // exp - INT 版本
        registry.register(
            "exp",
            FunctionSignature::new(
                "exp",
                vec![ValueType::Int],
                ValueType::Float,
                1,
                1,
                true,
                "计算指数",
            ),
            |args| {
                match &args[0] {
                    Value::Int(i) => Ok(Value::Float(((*i) as f64).exp())),
                    Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                    _ => Err(ExpressionError::type_error("exp函数需要整数类型")),
                }
            },
        );

        // exp - FLOAT 版本
        registry.register(
            "exp",
            FunctionSignature::new(
                "exp",
                vec![ValueType::Float],
                ValueType::Float,
                1,
                1,
                true,
                "计算指数",
            ),
            |args| {
                match &args[0] {
                    Value::Float(f) => Ok(Value::Float(f.exp())),
                    Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                    _ => Err(ExpressionError::type_error("exp函数需要浮点数类型")),
                }
            },
        );

        // log - INT 版本
        registry.register(
            "log",
            FunctionSignature::new(
                "log",
                vec![ValueType::Int],
                ValueType::Float,
                1,
                1,
                true,
                "计算自然对数",
            ),
            |args| {
                match &args[0] {
                    Value::Int(i) if *i > 0 => Ok(Value::Float(((*i) as f64).ln())),
                    Value::Int(i) if *i <= 0 => Err(ExpressionError::new(
                        ExpressionErrorType::InvalidOperation,
                        "log of non-positive number".to_string(),
                    )),
                    Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                    _ => Err(ExpressionError::type_error("log函数需要整数类型")),
                }
            },
        );

        // log - FLOAT 版本
        registry.register(
            "log",
            FunctionSignature::new(
                "log",
                vec![ValueType::Float],
                ValueType::Float,
                1,
                1,
                true,
                "计算自然对数",
            ),
            |args| {
                match &args[0] {
                    Value::Float(f) if *f > 0.0 => Ok(Value::Float(f.ln())),
                    Value::Float(f) if *f <= 0.0 => Err(ExpressionError::new(
                        ExpressionErrorType::InvalidOperation,
                        "log of non-positive number".to_string(),
                    )),
                    Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                    _ => Err(ExpressionError::type_error("log函数需要浮点数类型")),
                }
            },
        );

        // log10 - INT 版本
        registry.register(
            "log10",
            FunctionSignature::new(
                "log10",
                vec![ValueType::Int],
                ValueType::Float,
                1,
                1,
                true,
                "计算以10为底的对数",
            ),
            |args| {
                match &args[0] {
                    Value::Int(i) if *i > 0 => Ok(Value::Float(((*i) as f64).log10())),
                    Value::Int(i) if *i <= 0 => Err(ExpressionError::new(
                        ExpressionErrorType::InvalidOperation,
                        "log10 of non-positive number".to_string(),
                    )),
                    Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                    _ => Err(ExpressionError::type_error("log10函数需要整数类型")),
                }
            },
        );

        // log10 - FLOAT 版本
        registry.register(
            "log10",
            FunctionSignature::new(
                "log10",
                vec![ValueType::Float],
                ValueType::Float,
                1,
                1,
                true,
                "计算以10为底的对数",
            ),
            |args| {
                match &args[0] {
                    Value::Float(f) if *f > 0.0 => Ok(Value::Float(f.log10())),
                    Value::Float(f) if *f <= 0.0 => Err(ExpressionError::new(
                        ExpressionErrorType::InvalidOperation,
                        "log10 of non-positive number".to_string(),
                    )),
                    Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                    _ => Err(ExpressionError::type_error("log10函数需要浮点数类型")),
                }
            },
        );

        // sin - INT 版本
        registry.register(
            "sin",
            FunctionSignature::new(
                "sin",
                vec![ValueType::Int],
                ValueType::Float,
                1,
                1,
                true,
                "计算正弦",
            ),
            |args| {
                match &args[0] {
                    Value::Int(i) => Ok(Value::Float(((*i) as f64).sin())),
                    Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                    _ => Err(ExpressionError::type_error("sin函数需要整数类型")),
                }
            },
        );

        // sin - FLOAT 版本
        registry.register(
            "sin",
            FunctionSignature::new(
                "sin",
                vec![ValueType::Float],
                ValueType::Float,
                1,
                1,
                true,
                "计算正弦",
            ),
            |args| {
                match &args[0] {
                    Value::Float(f) => Ok(Value::Float(f.sin())),
                    Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                    _ => Err(ExpressionError::type_error("sin函数需要浮点数类型")),
                }
            },
        );

        // cos - INT 版本
        registry.register(
            "cos",
            FunctionSignature::new(
                "cos",
                vec![ValueType::Int],
                ValueType::Float,
                1,
                1,
                true,
                "计算余弦",
            ),
            |args| {
                match &args[0] {
                    Value::Int(i) => Ok(Value::Float(((*i) as f64).cos())),
                    Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                    _ => Err(ExpressionError::type_error("cos函数需要整数类型")),
                }
            },
        );

        // cos - FLOAT 版本
        registry.register(
            "cos",
            FunctionSignature::new(
                "cos",
                vec![ValueType::Float],
                ValueType::Float,
                1,
                1,
                true,
                "计算余弦",
            ),
            |args| {
                match &args[0] {
                    Value::Float(f) => Ok(Value::Float(f.cos())),
                    Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                    _ => Err(ExpressionError::type_error("cos函数需要浮点数类型")),
                }
            },
        );

        // tan - INT 版本
        registry.register(
            "tan",
            FunctionSignature::new(
                "tan",
                vec![ValueType::Int],
                ValueType::Float,
                1,
                1,
                true,
                "计算正切",
            ),
            |args| {
                match &args[0] {
                    Value::Int(i) => Ok(Value::Float(((*i) as f64).tan())),
                    Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                    _ => Err(ExpressionError::type_error("tan函数需要整数类型")),
                }
            },
        );

        // tan - FLOAT 版本
        registry.register(
            "tan",
            FunctionSignature::new(
                "tan",
                vec![ValueType::Float],
                ValueType::Float,
                1,
                1,
                true,
                "计算正切",
            ),
            |args| {
                match &args[0] {
                    Value::Float(f) => Ok(Value::Float(f.tan())),
                    Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                    _ => Err(ExpressionError::type_error("tan函数需要浮点数类型")),
                }
            },
        );
    }

    fn register_string_functions(&mut self) {
        let registry = self;

        // length
        registry.register(
            "length",
            FunctionSignature::new(
                "length",
                vec![ValueType::String],
                ValueType::Int,
                1,
                1,
                true,
                "计算字符串长度",
            ),
            |args| {
                match &args[0] {
                    Value::String(s) => Ok(Value::Int(s.len() as i64)),
                    Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                    _ => Err(ExpressionError::type_error("length函数需要字符串类型")),
                }
            },
        );

        // upper / toupper
        for name in ["upper", "toupper"] {
            registry.register(
                name,
                FunctionSignature::new(
                    name,
                    vec![ValueType::String],
                    ValueType::String,
                    1,
                    1,
                    true,
                    "转换为大写",
                ),
                |args| {
                    match &args[0] {
                        Value::String(s) => Ok(Value::String(s.to_uppercase())),
                        Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                        _ => Err(ExpressionError::type_error("upper函数需要字符串类型")),
                    }
                },
            );
        }

        // lower / tolower
        for name in ["lower", "tolower"] {
            registry.register(
                name,
                FunctionSignature::new(
                    name,
                    vec![ValueType::String],
                    ValueType::String,
                    1,
                    1,
                    true,
                    "转换为小写",
                ),
                |args| {
                    match &args[0] {
                        Value::String(s) => Ok(Value::String(s.to_lowercase())),
                        Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                        _ => Err(ExpressionError::type_error("lower函数需要字符串类型")),
                    }
                },
            );
        }

        // trim
        registry.register(
            "trim",
            FunctionSignature::new(
                "trim",
                vec![ValueType::String],
                ValueType::String,
                1,
                1,
                true,
                "去除首尾空白",
            ),
            |args| {
                match &args[0] {
                    Value::String(s) => Ok(Value::String(s.trim().to_string())),
                    Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                    _ => Err(ExpressionError::type_error("trim函数需要字符串类型")),
                }
            },
        );

        // ltrim
        registry.register(
            "ltrim",
            FunctionSignature::new(
                "ltrim",
                vec![ValueType::String],
                ValueType::String,
                1,
                1,
                true,
                "去除左侧空白",
            ),
            |args| {
                match &args[0] {
                    Value::String(s) => Ok(Value::String(s.trim_start().to_string())),
                    Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                    _ => Err(ExpressionError::type_error("ltrim函数需要字符串类型")),
                }
            },
        );

        // rtrim
        registry.register(
            "rtrim",
            FunctionSignature::new(
                "rtrim",
                vec![ValueType::String],
                ValueType::String,
                1,
                1,
                true,
                "去除右侧空白",
            ),
            |args| {
                match &args[0] {
                    Value::String(s) => Ok(Value::String(s.trim_end().to_string())),
                    Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                    _ => Err(ExpressionError::type_error("rtrim函数需要字符串类型")),
                }
            },
        );

        // concat
        registry.register(
            "concat",
            FunctionSignature::new(
                "concat",
                vec![ValueType::String],
                ValueType::String,
                2,
                usize::MAX,
                true,
                "连接字符串",
            ),
            |args| {
                let mut result = String::new();
                for arg in args {
                    match arg {
                        Value::String(s) => result.push_str(s),
                        Value::Null(_) => return Ok(Value::Null(crate::core::value::NullType::Null)),
                        _ => return Err(ExpressionError::type_error("concat函数需要字符串类型")),
                    }
                }
                Ok(Value::String(result))
            },
        );

        // replace
        registry.register(
            "replace",
            FunctionSignature::new(
                "replace",
                vec![ValueType::String, ValueType::String, ValueType::String],
                ValueType::String,
                3,
                3,
                true,
                "替换字符串",
            ),
            |args| {
                match (&args[0], &args[1], &args[2]) {
                    (Value::String(s), Value::String(from), Value::String(to)) => {
                        Ok(Value::String(s.replace(from, to)))
                    }
                    (Value::String(_), Value::String(_), Value::Null(_))
                    | (Value::String(_), Value::Null(_), _) => {
                        Ok(Value::Null(crate::core::value::NullType::Null))
                    }
                    (Value::Null(_), _, _) => Ok(Value::Null(crate::core::value::NullType::Null)),
                    _ => Err(ExpressionError::type_error("replace函数需要字符串类型")),
                }
            },
        );

        // substring / substr
        for name in ["substring", "substr"] {
            registry.register(
                name,
                FunctionSignature::new(
                    name,
                    vec![ValueType::String, ValueType::Int],
                    ValueType::String,
                    2,
                    3,
                    true,
                    "获取子字符串",
                ),
                |args| {
                    match (&args[0], &args[1]) {
                        (Value::String(s), Value::Int(start)) => {
                            let start = *start as usize;
                            if args.len() == 3 {
                                if let Value::Int(len) = &args[2] {
                                    let len = *len as usize;
                                    if start <= s.len() && len <= s.len() - start {
                                        Ok(Value::String(s[start..start + len].to_string()))
                                    } else {
                                        Ok(Value::String(s[start..].to_string()))
                                    }
                                } else {
                                    Err(ExpressionError::type_error("substring函数需要整数类型参数"))
                                }
                            } else {
                                if start <= s.len() {
                                    Ok(Value::String(s[start..].to_string()))
                                } else {
                                    Ok(Value::String(String::new()))
                                }
                            }
                        }
                        (Value::Null(_), _) | (_, Value::Null(_)) => {
                            Ok(Value::Null(crate::core::value::NullType::Null))
                        }
                        _ => Err(ExpressionError::type_error("substring函数需要字符串和整数类型")),
                    }
                },
            );
        }

        // contains
        registry.register(
            "contains",
            FunctionSignature::new(
                "contains",
                vec![ValueType::String, ValueType::String],
                ValueType::Bool,
                2,
                2,
                true,
                "检查是否包含子字符串",
            ),
            |args| {
                match (&args[0], &args[1]) {
                    (Value::String(s), Value::String(sub)) => {
                        Ok(Value::Bool(s.contains(sub)))
                    }
                    (Value::Null(_), _) | (_, Value::Null(_)) => {
                        Ok(Value::Null(crate::core::value::NullType::Null))
                    }
                    _ => Err(ExpressionError::type_error("contains函数需要字符串类型")),
                }
            },
        );

        // starts_with
        registry.register(
            "starts_with",
            FunctionSignature::new(
                "starts_with",
                vec![ValueType::String, ValueType::String],
                ValueType::Bool,
                2,
                2,
                true,
                "检查是否以指定字符串开头",
            ),
            |args| {
                match (&args[0], &args[1]) {
                    (Value::String(s), Value::String(prefix)) => {
                        Ok(Value::Bool(s.starts_with(prefix)))
                    }
                    (Value::Null(_), _) | (_, Value::Null(_)) => {
                        Ok(Value::Null(crate::core::value::NullType::Null))
                    }
                    _ => Err(ExpressionError::type_error("starts_with函数需要字符串类型")),
                }
            },
        );

        // ends_with
        registry.register(
            "ends_with",
            FunctionSignature::new(
                "ends_with",
                vec![ValueType::String, ValueType::String],
                ValueType::Bool,
                2,
                2,
                true,
                "检查是否以指定字符串结尾",
            ),
            |args| {
                match (&args[0], &args[1]) {
                    (Value::String(s), Value::String(suffix)) => {
                        Ok(Value::Bool(s.ends_with(suffix)))
                    }
                    (Value::Null(_), _) | (_, Value::Null(_)) => {
                        Ok(Value::Null(crate::core::value::NullType::Null))
                    }
                    _ => Err(ExpressionError::type_error("ends_with函数需要字符串类型")),
                }
            },
        );

        // left
        registry.register(
            "left",
            FunctionSignature::new(
                "left",
                vec![ValueType::String, ValueType::Int],
                ValueType::String,
                2,
                2,
                true,
                "获取左侧子字符串",
            ),
            |args| {
                match (&args[0], &args[1]) {
                    (Value::String(s), Value::Int(len)) => {
                        let len = *len as usize;
                        if len <= s.len() {
                            Ok(Value::String(s[..len].to_string()))
                        } else {
                            Ok(Value::String(s.clone()))
                        }
                    }
                    (Value::Null(_), _) | (_, Value::Null(_)) => {
                        Ok(Value::Null(crate::core::value::NullType::Null))
                    }
                    _ => Err(ExpressionError::type_error("left函数需要字符串和整数类型")),
                }
            },
        );

        // right
        registry.register(
            "right",
            FunctionSignature::new(
                "right",
                vec![ValueType::String, ValueType::Int],
                ValueType::String,
                2,
                2,
                true,
                "获取右侧子字符串",
            ),
            |args| {
                match (&args[0], &args[1]) {
                    (Value::String(s), Value::Int(len)) => {
                        let len = *len as usize;
                        if len <= s.len() {
                            Ok(Value::String(s[s.len() - len..].to_string()))
                        } else {
                            Ok(Value::String(s.clone()))
                        }
                    }
                    (Value::Null(_), _) | (_, Value::Null(_)) => {
                        Ok(Value::Null(crate::core::value::NullType::Null))
                    }
                    _ => Err(ExpressionError::type_error("right函数需要字符串和整数类型")),
                }
            },
        );

        // reverse
        registry.register(
            "reverse",
            FunctionSignature::new(
                "reverse",
                vec![ValueType::String],
                ValueType::String,
                1,
                1,
                true,
                "反转字符串",
            ),
            |args| {
                match &args[0] {
                    Value::String(s) => {
                        let reversed: String = s.chars().rev().collect();
                        Ok(Value::String(reversed))
                    }
                    Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                    _ => Err(ExpressionError::type_error("reverse函数需要字符串类型")),
                }
            },
        );
    }

    fn register_conversion_functions(&mut self) {
        let registry = self;

        // to_string - STRING 版本
        registry.register(
            "to_string",
            FunctionSignature::new(
                "to_string",
                vec![ValueType::String],
                ValueType::String,
                1,
                1,
                true,
                "转换为字符串",
            ),
            |args| {
                match &args[0] {
                    Value::String(s) => Ok(Value::String(s.clone())),
                    Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                    _ => Err(ExpressionError::type_error("to_string函数需要字符串类型")),
                }
            },
        );

        // to_string - INT 版本
        registry.register(
            "to_string",
            FunctionSignature::new(
                "to_string",
                vec![ValueType::Int],
                ValueType::String,
                1,
                1,
                true,
                "转换为字符串",
            ),
            |args| {
                match &args[0] {
                    Value::Int(i) => Ok(Value::String(i.to_string())),
                    Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                    _ => Err(ExpressionError::type_error("to_string函数需要整数类型")),
                }
            },
        );

        // to_string - FLOAT 版本
        registry.register(
            "to_string",
            FunctionSignature::new(
                "to_string",
                vec![ValueType::Float],
                ValueType::String,
                1,
                1,
                true,
                "转换为字符串",
            ),
            |args| {
                match &args[0] {
                    Value::Float(f) => Ok(Value::String(f.to_string())),
                    Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                    _ => Err(ExpressionError::type_error("to_string函数需要浮点数类型")),
                }
            },
        );

        // to_string - BOOL 版本
        registry.register(
            "to_string",
            FunctionSignature::new(
                "to_string",
                vec![ValueType::Bool],
                ValueType::String,
                1,
                1,
                true,
                "转换为字符串",
            ),
            |args| {
                match &args[0] {
                    Value::Bool(b) => Ok(Value::String(b.to_string())),
                    Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                    _ => Err(ExpressionError::type_error("to_string函数需要布尔类型")),
                }
            },
        );

        // to_int - INT 版本
        for name in ["to_int", "to_integer"] {
            registry.register(
                name,
                FunctionSignature::new(
                    name,
                    vec![ValueType::Int],
                    ValueType::Int,
                    1,
                    1,
                    true,
                    "转换为整数",
                ),
                |args| {
                    match &args[0] {
                        Value::Int(i) => Ok(Value::Int(*i)),
                        Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                        _ => Err(ExpressionError::type_error("to_int函数需要整数类型")),
                    }
                },
            );
        }

        // to_int - FLOAT 版本
        for name in ["to_int", "to_integer"] {
            registry.register(
                name,
                FunctionSignature::new(
                    name,
                    vec![ValueType::Float],
                    ValueType::Int,
                    1,
                    1,
                    true,
                    "转换为整数",
                ),
                |args| {
                    match &args[0] {
                        Value::Float(f) => Ok(Value::Int(*f as i64)),
                        Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                        _ => Err(ExpressionError::type_error("to_int函数需要浮点数类型")),
                    }
                },
            );
        }

        // to_int - STRING 版本
        for name in ["to_int", "to_integer"] {
            registry.register(
                name,
                FunctionSignature::new(
                    name,
                    vec![ValueType::String],
                    ValueType::Int,
                    1,
                    1,
                    true,
                    "转换为整数",
                ),
                |args| {
                    match &args[0] {
                        Value::String(s) => {
                            s.parse::<i64>()
                                .map(Value::Int)
                                .map_err(|_| ExpressionError::type_error("无法将字符串转换为整数"))
                        }
                        Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                        _ => Err(ExpressionError::type_error("to_int函数需要字符串类型")),
                    }
                },
            );
        }

        // to_int - BOOL 版本
        for name in ["to_int", "to_integer"] {
            registry.register(
                name,
                FunctionSignature::new(
                    name,
                    vec![ValueType::Bool],
                    ValueType::Int,
                    1,
                    1,
                    true,
                    "转换为整数",
                ),
                |args| {
                    match &args[0] {
                        Value::Bool(b) => Ok(Value::Int(if *b { 1 } else { 0 })),
                        Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                        _ => Err(ExpressionError::type_error("to_int函数需要布尔类型")),
                    }
                },
            );
        }

        // to_float - FLOAT 版本
        registry.register(
            "to_float",
            FunctionSignature::new(
                "to_float",
                vec![ValueType::Float],
                ValueType::Float,
                1,
                1,
                true,
                "转换为浮点数",
            ),
            |args| {
                match &args[0] {
                    Value::Float(f) => Ok(Value::Float(*f)),
                    Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                    _ => Err(ExpressionError::type_error("to_float函数需要浮点数类型")),
                }
            },
        );

        // to_float - INT 版本
        registry.register(
            "to_float",
            FunctionSignature::new(
                "to_float",
                vec![ValueType::Int],
                ValueType::Float,
                1,
                1,
                true,
                "转换为浮点数",
            ),
            |args| {
                match &args[0] {
                    Value::Int(i) => Ok(Value::Float(*i as f64)),
                    Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                    _ => Err(ExpressionError::type_error("to_float函数需要整数类型")),
                }
            },
        );

        // to_float - STRING 版本
        registry.register(
            "to_float",
            FunctionSignature::new(
                "to_float",
                vec![ValueType::String],
                ValueType::Float,
                1,
                1,
                true,
                "转换为浮点数",
            ),
            |args| {
                match &args[0] {
                    Value::String(s) => {
                        s.parse::<f64>()
                            .map(Value::Float)
                            .map_err(|_| ExpressionError::type_error("无法将字符串转换为浮点数"))
                    }
                    Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                    _ => Err(ExpressionError::type_error("to_float函数需要字符串类型")),
                }
            },
        );

        // to_bool - BOOL 版本
        for name in ["to_bool", "toboolean"] {
            registry.register(
                name,
                FunctionSignature::new(
                    name,
                    vec![ValueType::Bool],
                    ValueType::Bool,
                    1,
                    1,
                    true,
                    "转换为布尔值",
                ),
                |args| {
                    match &args[0] {
                        Value::Bool(b) => Ok(Value::Bool(*b)),
                        Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                        _ => Err(ExpressionError::type_error("to_bool函数需要布尔类型")),
                    }
                },
            );
        }

        // to_bool - INT 版本
        for name in ["to_bool", "toboolean"] {
            registry.register(
                name,
                FunctionSignature::new(
                    name,
                    vec![ValueType::Int],
                    ValueType::Bool,
                    1,
                    1,
                    true,
                    "转换为布尔值",
                ),
                |args| {
                    match &args[0] {
                        Value::Int(i) => Ok(Value::Bool(*i != 0)),
                        Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                        _ => Err(ExpressionError::type_error("to_bool函数需要整数类型")),
                    }
                },
            );
        }

        // to_bool - FLOAT 版本
        for name in ["to_bool", "toboolean"] {
            registry.register(
                name,
                FunctionSignature::new(
                    name,
                    vec![ValueType::Float],
                    ValueType::Bool,
                    1,
                    1,
                    true,
                    "转换为布尔值",
                ),
                |args| {
                    match &args[0] {
                        Value::Float(f) => Ok(Value::Bool(*f != 0.0)),
                        Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                        _ => Err(ExpressionError::type_error("to_bool函数需要浮点数类型")),
                    }
                },
            );
        }

        // to_bool - STRING 版本
        for name in ["to_bool", "toboolean"] {
            registry.register(
                name,
                FunctionSignature::new(
                    name,
                    vec![ValueType::String],
                    ValueType::Bool,
                    1,
                    1,
                    true,
                    "转换为布尔值",
                ),
                |args| {
                    match &args[0] {
                        Value::String(s) => {
                            let lower = s.to_lowercase();
                            if lower == "true" || lower == "1" {
                                Ok(Value::Bool(true))
                            } else if lower == "false" || lower == "0" {
                                Ok(Value::Bool(false))
                            } else {
                                Ok(Value::Null(crate::core::value::NullType::Null))
                            }
                        }
                        Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                        _ => Err(ExpressionError::type_error("to_bool函数需要字符串类型")),
                    }
                },
            );
        }
    }

    fn register_regex_functions(&mut self) {
        let registry = self;

        // regex_match
        registry.register(
            "regex_match",
            FunctionSignature::new(
                "regex_match",
                vec![ValueType::String, ValueType::String],
                ValueType::Bool,
                2,
                2,
                true,
                "正则表达式匹配",
            ),
            |args| {
                match (&args[0], &args[1]) {
                    (Value::String(s), Value::String(pattern)) => {
                        let regex = regex::Regex::new(pattern)
                            .map_err(|e| ExpressionError::new(
                                ExpressionErrorType::InvalidOperation,
                                format!("无效的正则表达式: {}", e),
                            ))?;
                        Ok(Value::Bool(regex.is_match(s)))
                    }
                    (Value::Null(_), _) | (_, Value::Null(_)) => {
                        Ok(Value::Null(crate::core::value::NullType::Null))
                    }
                    _ => Err(ExpressionError::type_error("regex_match函数需要字符串类型")),
                }
            },
        );

        // regex_replace
        registry.register(
            "regex_replace",
            FunctionSignature::new(
                "regex_replace",
                vec![ValueType::String, ValueType::String, ValueType::String],
                ValueType::String,
                3,
                3,
                true,
                "正则表达式替换",
            ),
            |args| {
                match (&args[0], &args[1], &args[2]) {
                    (Value::String(s), Value::String(pattern), Value::String(replacement)) => {
                        let regex = regex::Regex::new(pattern)
                            .map_err(|e| ExpressionError::new(
                                ExpressionErrorType::InvalidOperation,
                                format!("无效的正则表达式: {}", e),
                            ))?;
                        Ok(Value::String(regex.replace_all(s, replacement.as_str()).to_string()))
                    }
                    (Value::Null(_), _, _) | (_, Value::Null(_), _) | (_, _, Value::Null(_)) => {
                        Ok(Value::Null(crate::core::value::NullType::Null))
                    }
                    _ => Err(ExpressionError::type_error("regex_replace函数需要字符串类型")),
                }
            },
        );

        // regex_find
        registry.register(
            "regex_find",
            FunctionSignature::new(
                "regex_find",
                vec![ValueType::String, ValueType::String],
                ValueType::String,
                2,
                2,
                true,
                "正则表达式查找",
            ),
            |args| {
                match (&args[0], &args[1]) {
                    (Value::String(s), Value::String(pattern)) => {
                        let regex = regex::Regex::new(pattern)
                            .map_err(|e| ExpressionError::new(
                                ExpressionErrorType::InvalidOperation,
                                format!("无效的正则表达式: {}", e),
                            ))?;
                        if let Some(matched) = regex.find(s) {
                            Ok(Value::String(matched.as_str().to_string()))
                        } else {
                            Ok(Value::Null(crate::core::value::NullType::Null))
                        }
                    }
                    (Value::Null(_), _) | (_, Value::Null(_)) => {
                        Ok(Value::Null(crate::core::value::NullType::Null))
                    }
                    _ => Err(ExpressionError::type_error("regex_find函数需要字符串类型")),
                }
            },
        );
    }

    fn register_datetime_functions(&mut self) {
        let registry = self;

        // now
        registry.register(
            "now",
            FunctionSignature::new(
                "now",
                vec![],
                ValueType::Int,
                0,
                0,
                false,
                "获取当前时间戳",
            ),
            |_args| {
                use std::time::{SystemTime, UNIX_EPOCH};
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Failed to create function registry")
                    .as_millis();
                Ok(Value::Int(now as i64))
            },
        );

        // date
        registry.register(
            "date",
            FunctionSignature::new(
                "date",
                vec![ValueType::String],
                ValueType::Date,
                0,
                1,
                true,
                "创建日期",
            ),
            |args| {
                if args.is_empty() {
                    let now = Utc::now();
                    Ok(Value::Date(crate::core::value::DateValue {
                        year: now.year(),
                        month: now.month() as u32,
                        day: now.day() as u32,
                    }))
                } else {
                    match &args[0] {
                        Value::String(s) => {
                            let naivedate = NaiveDate::parse_from_str(s, "%Y-%m-%d")
                                .map_err(|_| ExpressionError::type_error("无法解析日期字符串"))?;
                            Ok(Value::Date(crate::core::value::DateValue {
                                year: naivedate.year(),
                                month: naivedate.month() as u32,
                                day: naivedate.day() as u32,
                            }))
                        }
                        Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                        _ => Err(ExpressionError::type_error("date函数需要字符串类型")),
                    }
                }
            },
        );

        // year
        registry.register(
            "year",
            FunctionSignature::new(
                "year",
                vec![ValueType::Any],
                ValueType::Int,
                1,
                1,
                true,
                "提取年份",
            ),
            |args| {
                match &args[0] {
                    Value::Date(d) => Ok(Value::Int(d.year as i64)),
                    Value::DateTime(dt) => Ok(Value::Int(dt.year as i64)),
                    Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                    _ => Err(ExpressionError::type_error("year函数需要日期或日期时间类型")),
                }
            },
        );

        // month
        registry.register(
            "month",
            FunctionSignature::new(
                "month",
                vec![ValueType::Any],
                ValueType::Int,
                1,
                1,
                true,
                "提取月份",
            ),
            |args| {
                match &args[0] {
                    Value::Date(d) => Ok(Value::Int(d.month as i64)),
                    Value::DateTime(dt) => Ok(Value::Int(dt.month as i64)),
                    Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                    _ => Err(ExpressionError::type_error("month函数需要日期或日期时间类型")),
                }
            },
        );

        // day
        registry.register(
            "day",
            FunctionSignature::new(
                "day",
                vec![ValueType::Any],
                ValueType::Int,
                1,
                1,
                true,
                "提取日",
            ),
            |args| {
                match &args[0] {
                    Value::Date(d) => Ok(Value::Int(d.day as i64)),
                    Value::DateTime(dt) => Ok(Value::Int(dt.day as i64)),
                    Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                    _ => Err(ExpressionError::type_error("day函数需要日期或日期时间类型")),
                }
            },
        );

        // hour
        registry.register(
            "hour",
            FunctionSignature::new(
                "hour",
                vec![ValueType::Any],
                ValueType::Int,
                1,
                1,
                true,
                "提取小时",
            ),
            |args| {
                match &args[0] {
                    Value::Time(t) => Ok(Value::Int(t.hour as i64)),
                    Value::DateTime(dt) => Ok(Value::Int(dt.hour as i64)),
                    Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                    _ => Err(ExpressionError::type_error("hour函数需要时间或日期时间类型")),
                }
            },
        );

        // minute
        registry.register(
            "minute",
            FunctionSignature::new(
                "minute",
                vec![ValueType::Any],
                ValueType::Int,
                1,
                1,
                true,
                "提取分钟",
            ),
            |args| {
                match &args[0] {
                    Value::Time(t) => Ok(Value::Int(t.minute as i64)),
                    Value::DateTime(dt) => Ok(Value::Int(dt.minute as i64)),
                    Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                    _ => Err(ExpressionError::type_error("minute函数需要时间或日期时间类型")),
                }
            },
        );

        // second
        registry.register(
            "second",
            FunctionSignature::new(
                "second",
                vec![ValueType::Any],
                ValueType::Int,
                1,
                1,
                true,
                "提取秒",
            ),
            |args| {
                match &args[0] {
                    Value::Time(t) => Ok(Value::Int(t.sec as i64)),
                    Value::DateTime(dt) => Ok(Value::Int(dt.sec as i64)),
                    Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                    _ => Err(ExpressionError::type_error("second函数需要时间或日期时间类型")),
                }
            },
        );
    }
}

/// 全局函数注册表实例
pub fn global_registry() -> Arc<FunctionRegistry> {
    use std::sync::OnceLock;
    static REGISTRY: OnceLock<Arc<FunctionRegistry>> = OnceLock::new();
    REGISTRY.get_or_init(|| Arc::new(FunctionRegistry::new())).clone()
}
