//! 基础表达式上下文模块
//!
//! 提供表达式求值过程中的基础上下文实现

use crate::cache::CacheConfig;
use crate::core::context_traits::{ContextBase, ContextType, HierarchicalContext, MutableContext};
use crate::core::types::query::FieldValue;
use crate::expression::cache::{ExpressionCacheManager, ExpressionCacheStats};
use crate::expression::functions::{BuiltinFunction, CustomFunction, ExpressionFunction, FunctionRef};
use std::collections::HashMap;
use std::sync::Arc;

/// 表达式上下文枚举，避免动态分发
#[derive(Debug, Clone)]
pub enum ExpressionContextType {
    /// 基础表达式上下文
    Basic(BasicExpressionContext),
}

/// 表达式上下文特征
pub trait ExpressionContextCoreExtended {
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

impl ExpressionContextCoreExtended for BasicExpressionContext {
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
            ExpressionContextCoreExtended::get_variable(parent, name)
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
        ExpressionContextCoreExtended::get_variable(self, name).is_some()
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

impl ExpressionContextCoreExtended for Box<BasicExpressionContext> {
    fn get_variable(&self, name: &str) -> Option<&FieldValue> {
        (**self).get_variable(name)
    }

    fn get_function(&self, name: &str) -> Option<FunctionRef> {
        (**self).get_function(name)
    }

    fn has_variable(&self, name: &str) -> bool {
        (**self).has_variable(name)
    }

    fn get_variable_names(&self) -> Vec<&str> {
        (**self).get_variable_names()
    }

    fn get_depth(&self) -> usize {
        (**self).get_depth()
    }

    fn create_child_context(&self) -> ExpressionContextType {
        (**self).create_child_context()
    }
}

impl ExpressionContextCoreExtended for ExpressionContextType {
    fn get_variable(&self, name: &str) -> Option<&FieldValue> {
        match self {
            ExpressionContextType::Basic(ctx) => {
                ExpressionContextCoreExtended::get_variable(ctx, name)
            }
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
    pub fn with_parent_and_cache(
        parent: BasicExpressionContext,
        cache_config: CacheConfig,
    ) -> Self {
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
    ) -> Result<FieldValue, crate::core::ExpressionError> {
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
            parent: self.parent.clone(),
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

impl HierarchicalContext for BasicExpressionContext {
    fn parent_id(&self) -> Option<&str> {
        self.parent.as_ref().map(|_| "parent_expression")
    }

    fn depth(&self) -> usize {
        self.depth
    }
}

// 为BasicExpressionContext实现统一的ExpressionContext trait
impl crate::expression::context::default_context::ExpressionContext for BasicExpressionContext {
    fn get_variable(&self, name: &str) -> Option<crate::core::Value> {
        // 将FieldValue转换为Value
        ExpressionContextCoreExtended::get_variable(self, name).map(|fv| {
            // 这里需要实现FieldValue到Value的转换
            // 暂时返回一个简单的值，实际实现需要完整的转换逻辑
            match fv {
                crate::core::types::query::FieldValue::Scalar(scalar) => match scalar {
                    crate::core::types::query::ScalarValue::Bool(b) => crate::core::Value::Bool(*b),
                    crate::core::types::query::ScalarValue::Int(i) => crate::core::Value::Int(*i),
                    crate::core::types::query::ScalarValue::Float(f) => {
                        crate::core::Value::Float(*f)
                    }
                    crate::core::types::query::ScalarValue::String(s) => {
                        crate::core::Value::String(s.clone())
                    }
                    crate::core::types::query::ScalarValue::Null => {
                        crate::core::Value::Null(crate::core::NullType::Null)
                    }
                },
                _ => {
                    // 对于复杂类型，暂时返回空值
                    crate::core::Value::Null(crate::core::NullType::Null)
                }
            }
        })
    }

    fn set_variable(&mut self, name: String, value: crate::core::Value) {
        // 将Value转换为FieldValue
        let field_value = match value {
            crate::core::Value::Bool(b) => crate::core::types::query::FieldValue::Scalar(
                crate::core::types::query::ScalarValue::Bool(b),
            ),
            crate::core::Value::Int(i) => crate::core::types::query::FieldValue::Scalar(
                crate::core::types::query::ScalarValue::Int(i),
            ),
            crate::core::Value::Float(f) => crate::core::types::query::FieldValue::Scalar(
                crate::core::types::query::ScalarValue::Float(f),
            ),
            crate::core::Value::String(s) => crate::core::types::query::FieldValue::Scalar(
                crate::core::types::query::ScalarValue::String(s),
            ),
            crate::core::Value::Null(_) => crate::core::types::query::FieldValue::Scalar(
                crate::core::types::query::ScalarValue::Null,
            ),
            _ => {
                // 对于复杂类型，暂时返回空值
                crate::core::types::query::FieldValue::Scalar(
                    crate::core::types::query::ScalarValue::Null,
                )
            }
        };

        self.set_variable(name, field_value);
    }

    fn get_vertex(&self) -> Option<&crate::core::Vertex> {
        // BasicExpressionContext没有直接的vertex字段，需要从变量中获取
        // 这里暂时返回None，实际实现可能需要从特定变量中获取
        None
    }

    fn get_edge(&self) -> Option<&crate::core::Edge> {
        // BasicExpressionContext没有直接的edge字段，需要从变量中获取
        // 这里暂时返回None，实际实现可能需要从特定变量中获取
        None
    }

    fn get_path(&self, _name: &str) -> Option<&crate::core::vertex_edge_path::Path> {
        // BasicExpressionContext没有path字段，需要从变量中获取
        // 这里暂时返回None，实际实现可能需要从特定变量中获取
        None
    }

    fn set_vertex(&mut self, vertex: crate::core::Vertex) {
        // 将vertex存储为变量
        // 将vertex_edge_path::Vertex转换为types::query::Vertex
        let query_vertex = crate::core::types::query::Vertex {
            id: vertex.vid.to_string(), // 将 Box<Value> 转换为 String
            tags: vertex.tags.iter().map(|tag| tag.name.clone()).collect(),
            properties: vertex
                .properties
                .iter()
                .map(|(k, v)| {
                    (
                        k.clone(),
                        match v {
                            crate::core::value::Value::Bool(b) => {
                                crate::core::types::query::ScalarValue::Bool(*b)
                            }
                            crate::core::value::Value::Int(i) => {
                                crate::core::types::query::ScalarValue::Int(*i)
                            }
                            crate::core::value::Value::Float(f) => {
                                crate::core::types::query::ScalarValue::Float(*f)
                            }
                            crate::core::value::Value::String(s) => {
                                crate::core::types::query::ScalarValue::String(s.clone())
                            }
                            crate::core::value::Value::Null(_) => {
                                crate::core::types::query::ScalarValue::Null
                            }
                            _ => crate::core::types::query::ScalarValue::Null,
                        },
                    )
                })
                .collect(),
        };
        let field_value = crate::core::types::query::FieldValue::Vertex(query_vertex);
        self.set_variable("_vertex".to_string(), field_value);
    }

    fn set_edge(&mut self, edge: crate::core::Edge) {
        // 将edge存储为变量
        // 将vertex_edge_path::Edge转换为types::query::Edge
        let src_str = match &*edge.src {
            crate::core::value::Value::String(s) => s.clone(),
            v => v.to_string(),
        };
        let dst_str = match &*edge.dst {
            crate::core::value::Value::String(s) => s.clone(),
            v => v.to_string(),
        };
        let query_edge = crate::core::types::query::Edge {
            id: format!("{}_{}", src_str, dst_str),
            edge_type: edge.edge_type.clone(),
            src: src_str,
            dst: dst_str,
            properties: edge
                .properties()
                .iter()
                .map(|(k, v)| {
                    (
                        k.clone(),
                        match v {
                            crate::core::value::Value::Bool(b) => {
                                crate::core::types::query::ScalarValue::Bool(*b)
                            }
                            crate::core::value::Value::Int(i) => {
                                crate::core::types::query::ScalarValue::Int(*i)
                            }
                            crate::core::value::Value::Float(f) => {
                                crate::core::types::query::ScalarValue::Float(*f)
                            }
                            crate::core::value::Value::String(s) => {
                                crate::core::types::query::ScalarValue::String(s.clone())
                            }
                            crate::core::value::Value::Null(_) => {
                                crate::core::types::query::ScalarValue::Null
                            }
                            _ => crate::core::types::query::ScalarValue::Null,
                        },
                    )
                })
                .collect(),
            ranking: Some(edge.ranking),
        };
        let field_value = crate::core::types::query::FieldValue::Edge(query_edge);
        self.set_variable("_edge".to_string(), field_value);
    }

    fn add_path(&mut self, name: String, _path: crate::core::vertex_edge_path::Path) {
        // 将path存储为变量
        // 简化实现：跳过复杂的Path转换
        // TODO: 实现完整的vertex_edge_path::Path到types::query::Path的转换
        let field_value =
            crate::core::types::query::FieldValue::Path(crate::core::types::query::Path {
                segments: Vec::new(),
            });
        self.set_variable(name, field_value);
    }

    fn is_empty(&self) -> bool {
        self.variables.is_empty()
    }

    fn variable_count(&self) -> usize {
        self.variables.len()
    }

    fn variable_names(&self) -> Vec<String> {
        self.variables.keys().cloned().collect()
    }

    fn get_all_variables(&self) -> Option<std::collections::HashMap<String, crate::core::Value>> {
        // 将FieldValue转换为Value
        let mut value_map = std::collections::HashMap::new();
        for (name, field_value) in &self.variables {
            let value = match field_value {
                crate::core::types::query::FieldValue::Scalar(scalar) => match scalar {
                    crate::core::types::query::ScalarValue::Bool(b) => crate::core::Value::Bool(*b),
                    crate::core::types::query::ScalarValue::Int(i) => crate::core::Value::Int(*i),
                    crate::core::types::query::ScalarValue::Float(f) => {
                        crate::core::Value::Float(*f)
                    }
                    crate::core::types::query::ScalarValue::String(s) => {
                        crate::core::Value::String(s.clone())
                    }
                    crate::core::types::query::ScalarValue::Null => {
                        crate::core::Value::Null(crate::core::NullType::Null)
                    }
                },
                _ => {
                    // 对于复杂类型，暂时返回空值
                    crate::core::Value::Null(crate::core::NullType::Null)
                }
            };
            value_map.insert(name.clone(), value);
        }
        Some(value_map)
    }

    fn clear(&mut self) {
        self.variables.clear();
    }

    fn get_variable_names(&self) -> Vec<&str> {
        self.variables.keys().map(|k| k.as_str()).collect()
    }
}
