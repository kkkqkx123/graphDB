//! 函数注册表
//!
//! 提供函数的注册、查找和执行功能
//! 具体函数实现位于 builtin/ 子模块

use crate::core::error::{ExpressionError, ExpressionErrorType};
use crate::core::Value;
use std::collections::HashMap;
use std::sync::Arc;
use super::BuiltinFunction;
use super::CustomFunction;

/// 函数注册表
/// 
/// 使用静态分发机制，通过 BuiltinFunction 和 CustomFunction 枚举直接调用函数
/// 避免了动态分发（dyn）的开销
#[derive(Debug)]
pub struct FunctionRegistry {
    /// 内置函数映射（函数名 -> BuiltinFunction 枚举）
    builtin_functions: HashMap<String, BuiltinFunction>,
    /// 自定义函数映射（函数名 -> CustomFunction）
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
            builtin_functions: HashMap::new(),
            custom_functions: HashMap::new(),
        };
        registry.register_all_builtin_functions();
        registry
    }

    /// 检查函数是否存在
    pub fn contains(&self, name: &str) -> bool {
        self.builtin_functions.contains_key(name) || self.custom_functions.contains_key(name)
    }

    /// 获取所有函数名称
    pub fn function_names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.builtin_functions.keys()
            .map(|s| s.as_str())
            .collect();
        names.extend(self.custom_functions.keys().map(|s| s.as_str()));
        names
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

    /// 执行函数（根据名称）
    pub fn execute(&self, name: &str, args: &[Value]) -> Result<Value, ExpressionError> {
        // 先尝试查找内置函数
        if let Some(func) = self.builtin_functions.get(name) {
            return func.execute(args);
        }
        
        // 再尝试查找自定义函数
        if let Some(func) = self.custom_functions.get(name) {
            return func.execute(args);
        }
        
        Err(ExpressionError::new(
            ExpressionErrorType::UndefinedFunction,
            format!("未定义的函数: {}", name),
        ))
    }

    /// 注册所有内置函数
    fn register_all_builtin_functions(&mut self) {
        use super::MathFunction;
        use super::StringFunction;
        use super::RegexFunction;
        use super::ConversionFunction;
        use super::DateTimeFunction;

        // 注册数学函数
        self.register_builtin(BuiltinFunction::Math(MathFunction::Abs));
        self.register_builtin(BuiltinFunction::Math(MathFunction::Sqrt));
        self.register_builtin(BuiltinFunction::Math(MathFunction::Pow));
        self.register_builtin(BuiltinFunction::Math(MathFunction::Log));
        self.register_builtin(BuiltinFunction::Math(MathFunction::Log10));
        self.register_builtin(BuiltinFunction::Math(MathFunction::Sin));
        self.register_builtin(BuiltinFunction::Math(MathFunction::Cos));
        self.register_builtin(BuiltinFunction::Math(MathFunction::Tan));
        self.register_builtin(BuiltinFunction::Math(MathFunction::Round));
        self.register_builtin(BuiltinFunction::Math(MathFunction::Ceil));
        self.register_builtin(BuiltinFunction::Math(MathFunction::Floor));
        self.register_builtin(BuiltinFunction::Math(MathFunction::Asin));
        self.register_builtin(BuiltinFunction::Math(MathFunction::Acos));
        self.register_builtin(BuiltinFunction::Math(MathFunction::Atan));
        self.register_builtin(BuiltinFunction::Math(MathFunction::Cbrt));
        self.register_builtin(BuiltinFunction::Math(MathFunction::Hypot));
        self.register_builtin(BuiltinFunction::Math(MathFunction::Sign));
        self.register_builtin(BuiltinFunction::Math(MathFunction::Rand));
        self.register_builtin(BuiltinFunction::Math(MathFunction::Rand32));
        self.register_builtin(BuiltinFunction::Math(MathFunction::Rand64));
        self.register_builtin(BuiltinFunction::Math(MathFunction::E));
        self.register_builtin(BuiltinFunction::Math(MathFunction::Pi));
        self.register_builtin(BuiltinFunction::Math(MathFunction::Exp2));
        self.register_builtin(BuiltinFunction::Math(MathFunction::Log2));
        self.register_builtin(BuiltinFunction::Math(MathFunction::Radians));
        self.register_builtin(BuiltinFunction::Math(MathFunction::BitAnd));
        self.register_builtin(BuiltinFunction::Math(MathFunction::BitOr));
        self.register_builtin(BuiltinFunction::Math(MathFunction::BitXor));

        // 注册字符串函数
        self.register_builtin(BuiltinFunction::String(StringFunction::Length));
        self.register_builtin(BuiltinFunction::String(StringFunction::Upper));
        self.register_builtin(BuiltinFunction::String(StringFunction::Lower));
        self.register_builtin(BuiltinFunction::String(StringFunction::Trim));
        self.register_builtin(BuiltinFunction::String(StringFunction::Substring));
        self.register_builtin(BuiltinFunction::String(StringFunction::Concat));
        self.register_builtin(BuiltinFunction::String(StringFunction::Replace));
        self.register_builtin(BuiltinFunction::String(StringFunction::Contains));
        self.register_builtin(BuiltinFunction::String(StringFunction::StartsWith));
        self.register_builtin(BuiltinFunction::String(StringFunction::EndsWith));
        self.register_builtin(BuiltinFunction::String(StringFunction::Split));
        self.register_builtin(BuiltinFunction::String(StringFunction::Lpad));
        self.register_builtin(BuiltinFunction::String(StringFunction::Rpad));
        self.register_builtin(BuiltinFunction::String(StringFunction::ConcatWs));
        self.register_builtin(BuiltinFunction::String(StringFunction::Strcasecmp));

        // 注册正则表达式函数
        self.register_builtin(BuiltinFunction::Regex(RegexFunction::RegexMatch));
        self.register_builtin(BuiltinFunction::Regex(RegexFunction::RegexReplace));
        self.register_builtin(BuiltinFunction::Regex(RegexFunction::RegexFind));

        // 注册类型转换函数
        self.register_builtin(BuiltinFunction::Conversion(ConversionFunction::ToString));
        self.register_builtin(BuiltinFunction::Conversion(ConversionFunction::ToInt));
        self.register_builtin(BuiltinFunction::Conversion(ConversionFunction::ToFloat));
        self.register_builtin(BuiltinFunction::Conversion(ConversionFunction::ToBool));

        // 注册日期时间函数
        self.register_builtin(BuiltinFunction::DateTime(DateTimeFunction::Now));
        self.register_builtin(BuiltinFunction::DateTime(DateTimeFunction::Date));
        self.register_builtin(BuiltinFunction::DateTime(DateTimeFunction::Time));
        self.register_builtin(BuiltinFunction::DateTime(DateTimeFunction::DateTime));
        self.register_builtin(BuiltinFunction::DateTime(DateTimeFunction::Year));
        self.register_builtin(BuiltinFunction::DateTime(DateTimeFunction::Month));
        self.register_builtin(BuiltinFunction::DateTime(DateTimeFunction::Day));
        self.register_builtin(BuiltinFunction::DateTime(DateTimeFunction::Hour));
        self.register_builtin(BuiltinFunction::DateTime(DateTimeFunction::Minute));
        self.register_builtin(BuiltinFunction::DateTime(DateTimeFunction::Second));
        self.register_builtin(BuiltinFunction::DateTime(DateTimeFunction::TimeStamp));

        // 注册地理空间函数
        use super::GeographyFunction;
        self.register_builtin(BuiltinFunction::Geography(GeographyFunction::StPoint));
        self.register_builtin(BuiltinFunction::Geography(GeographyFunction::StGeogFromText));
        self.register_builtin(BuiltinFunction::Geography(GeographyFunction::StAsText));
        self.register_builtin(BuiltinFunction::Geography(GeographyFunction::StCentroid));
        self.register_builtin(BuiltinFunction::Geography(GeographyFunction::StIsValid));
        self.register_builtin(BuiltinFunction::Geography(GeographyFunction::StIntersects));
        self.register_builtin(BuiltinFunction::Geography(GeographyFunction::StCovers));
        self.register_builtin(BuiltinFunction::Geography(GeographyFunction::StCoveredBy));
        self.register_builtin(BuiltinFunction::Geography(GeographyFunction::StDWithin));
        self.register_builtin(BuiltinFunction::Geography(GeographyFunction::StDistance));

        // 注册实用函数
        use super::UtilityFunction;
        self.register_builtin(BuiltinFunction::Utility(UtilityFunction::Coalesce));
        self.register_builtin(BuiltinFunction::Utility(UtilityFunction::Hash));
        self.register_builtin(BuiltinFunction::Utility(UtilityFunction::JsonExtract));

        // 注册图相关函数
        use super::GraphFunction;
        self.register_builtin(BuiltinFunction::Graph(GraphFunction::Id));
        self.register_builtin(BuiltinFunction::Graph(GraphFunction::Tags));
        self.register_builtin(BuiltinFunction::Graph(GraphFunction::Labels));
        self.register_builtin(BuiltinFunction::Graph(GraphFunction::Properties));
        self.register_builtin(BuiltinFunction::Graph(GraphFunction::EdgeType));
        self.register_builtin(BuiltinFunction::Graph(GraphFunction::Src));
        self.register_builtin(BuiltinFunction::Graph(GraphFunction::Dst));
        self.register_builtin(BuiltinFunction::Graph(GraphFunction::Rank));
        self.register_builtin(BuiltinFunction::Graph(GraphFunction::StartNode));
        self.register_builtin(BuiltinFunction::Graph(GraphFunction::EndNode));

        // 注册容器操作函数
        use super::ContainerFunction;
        self.register_builtin(BuiltinFunction::Container(ContainerFunction::Head));
        self.register_builtin(BuiltinFunction::Container(ContainerFunction::Last));
        self.register_builtin(BuiltinFunction::Container(ContainerFunction::Tail));
        self.register_builtin(BuiltinFunction::Container(ContainerFunction::Size));
        self.register_builtin(BuiltinFunction::Container(ContainerFunction::Range));
        self.register_builtin(BuiltinFunction::Container(ContainerFunction::Keys));
        self.register_builtin(BuiltinFunction::Container(ContainerFunction::ReverseList));
        self.register_builtin(BuiltinFunction::Container(ContainerFunction::ToSet));

        // 注册路径函数
        use super::PathFunction;
        self.register_builtin(BuiltinFunction::Path(PathFunction::Nodes));
        self.register_builtin(BuiltinFunction::Path(PathFunction::Relationships));
    }
}

/// 全局函数注册表实例
pub fn global_registry() -> Arc<FunctionRegistry> {
    use std::sync::OnceLock;
    static REGISTRY: OnceLock<Arc<FunctionRegistry>> = OnceLock::new();
    REGISTRY.get_or_init(|| Arc::new(FunctionRegistry::new())).clone()
}
