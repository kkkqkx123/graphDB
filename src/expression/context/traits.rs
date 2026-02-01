//! 表达式上下文特征定义（分解版本）
//!
//! 提供表达式上下文的分解特征定义，便于按需实现

use crate::core::Value;
use crate::expression::functions::FunctionRef;

pub trait VariableContext {
    fn get_variable(&self, name: &str) -> Option<Value>;
    fn set_variable(&mut self, name: String, value: Value);
    fn get_variable_names(&self) -> Vec<&str>;
    fn has_variable(&self, name: &str) -> bool {
        self.get_variable(name).is_some()
    }
    fn variable_count(&self) -> usize;
    fn get_all_variables(&self) -> Option<std::collections::HashMap<String, Value>>;
    fn clear_variables(&mut self);
}

pub trait FunctionContext {
    fn get_function(&self, name: &str) -> Option<FunctionRef>;
    fn has_function(&self, name: &str) -> bool {
        self.get_function(name).is_some()
    }
    fn get_function_names(&self) -> Vec<&str>;
}

pub trait CacheContext {
    fn get_regex(&mut self, pattern: &str) -> Option<&regex::Regex>;
}

pub trait GraphContext {
    fn get_vertex(&self) -> Option<&crate::core::Vertex>;
    fn get_edge(&self) -> Option<&crate::core::Edge>;
    fn get_path(&self, name: &str) -> Option<&crate::core::vertex_edge_path::Path>;
    fn set_vertex(&mut self, vertex: crate::core::Vertex);
    fn set_edge(&mut self, edge: crate::core::Edge);
    fn add_path(&mut self, name: String, path: crate::core::vertex_edge_path::Path);
}

pub trait ScopedContext {
    fn get_depth(&self) -> usize;
    fn create_child_context(&self) -> Box<dyn crate::expression::evaluator::traits::ExpressionContext>;
}

pub use crate::expression::evaluator::traits::ExpressionContext;
