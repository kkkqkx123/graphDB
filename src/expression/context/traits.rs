//! 表达式上下文特征定义（简化版本）
//!
//! 提供表达式上下文的基础特征定义

use crate::core::Value;
use crate::expression::functions::FunctionRef;

/// 变量上下文特征
///
/// 提供基本的变量访问和管理功能
pub trait VariableContext {
    /// 获取变量值
    fn get_variable(&self, name: &str) -> Option<Value>;

    /// 设置变量值
    fn set_variable(&mut self, name: String, value: Value);

    /// 获取所有变量名
    fn get_variable_names(&self) -> Vec<&str>;

    /// 检查变量是否存在
    fn has_variable(&self, name: &str) -> bool {
        self.get_variable(name).is_some()
    }

    /// 获取变量数量
    fn variable_count(&self) -> usize;

    /// 获取所有变量
    fn get_all_variables(&self) -> Option<std::collections::HashMap<String, Value>>;

    /// 清空所有变量
    fn clear_variables(&mut self);
}

/// 函数上下文特征
///
/// 提供函数注册和调用功能
pub trait FunctionContext {
    /// 获取函数
    fn get_function(&self, name: &str) -> Option<FunctionRef>;

    /// 检查函数是否存在
    fn has_function(&self, name: &str) -> bool {
        self.get_function(name).is_some()
    }

    /// 获取所有函数名
    fn get_function_names(&self) -> Vec<&str>;
}

/// 缓存上下文特征
///
/// 提供缓存管理功能
pub trait CacheContext {
    /// 获取或编译正则表达式
    fn get_regex(&mut self, pattern: &str) -> Option<&regex::Regex>;
}

/// 图数据库上下文特征
///
/// 提供图数据库特有的顶点、边、路径访问功能
pub trait GraphContext {
    /// 获取顶点引用
    fn get_vertex(&self) -> Option<&crate::core::Vertex>;

    /// 获取边引用
    fn get_edge(&self) -> Option<&crate::core::Edge>;

    /// 获取路径
    fn get_path(&self, name: &str) -> Option<&crate::core::vertex_edge_path::Path>;

    /// 设置顶点
    fn set_vertex(&mut self, vertex: crate::core::Vertex);

    /// 设置边
    fn set_edge(&mut self, edge: crate::core::Edge);

    /// 添加路径
    fn add_path(&mut self, name: String, path: crate::core::vertex_edge_path::Path);
}

/// 作用域上下文特征
///
/// 提供嵌套作用域支持
pub trait ScopedContext {
    /// 获取上下文深度
    fn get_depth(&self) -> usize;

    /// 创建子上下文
    fn create_child_context(&self) -> Box<dyn crate::expression::evaluator::traits::ExpressionContext>;
}

// 重新导出 ExpressionContext 以保持向后兼容
pub use crate::expression::evaluator::traits::ExpressionContext;
