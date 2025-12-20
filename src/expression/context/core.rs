//! 表达式上下文核心trait定义
//!
//! 定义所有表达式上下文必须实现的核心接口

use crate::core::{Edge, Value, Vertex};

/// 表达式上下文核心trait
///
/// 所有表达式上下文实现都必须实现此trait
pub trait ExpressionContextCore {
    /// 获取变量值
    fn get_variable(&self, name: &str) -> Option<Value>;
    
    /// 设置变量值
    fn set_variable(&mut self, name: String, value: Value);
    
    /// 获取顶点引用
    fn get_vertex(&self) -> Option<&Vertex>;
    
    /// 获取边引用
    fn get_edge(&self) -> Option<&Edge>;
    
    /// 获取路径
    fn get_path(&self, name: &str) -> Option<&crate::core::vertex_edge_path::Path>;
    
    /// 设置顶点
    fn set_vertex(&mut self, vertex: Vertex);
    
    /// 设置边
    fn set_edge(&mut self, edge: Edge);
    
    /// 添加路径
    fn add_path(&mut self, name: String, path: crate::core::vertex_edge_path::Path);
    
    /// 检查是否为空上下文
    fn is_empty(&self) -> bool;
    
    /// 获取变量数量
    fn variable_count(&self) -> usize;
    
    /// 获取所有变量名
    fn variable_names(&self) -> Vec<String>;
    
    /// 获取所有变量
    fn get_all_variables(&self) -> Option<std::collections::HashMap<String, Value>>;
    
    /// 清空所有数据
    fn clear(&mut self);
}

/// 存储层表达式上下文trait
///
/// 为存储层特定的表达式上下文提供额外接口
pub trait StorageExpressionContextTrait: ExpressionContextCore {
    /// 获取变量值（最新版本）
    fn get_var(&self, name: &str) -> Result<Value, String>;
    
    /// 获取指定版本的变量值
    fn get_versioned_var(&self, name: &str, version: i64) -> Result<Value, String>;
    
    /// 设置变量值
    fn set_var(&mut self, name: &str, value: Value) -> Result<(), String>;
    
    /// 设置表达式内部变量
    fn set_inner_var(&mut self, var: &str, value: Value);
    
    /// 获取表达式内部变量
    fn get_inner_var(&self, var: &str) -> Option<Value>;
    
    /// 获取变量属性值
    fn get_var_prop(&self, var: &str, prop: &str) -> Result<Value, String>;
    
    /// 获取目标顶点属性值
    fn get_dst_prop(&self, tag: &str, prop: &str) -> Result<Value, String>;
    
    /// 获取输入属性值
    fn get_input_prop(&self, prop: &str) -> Result<Value, String>;
    
    /// 获取输入属性索引
    fn get_input_prop_index(&self, prop: &str) -> Result<usize, String>;
    
    /// 按列索引获取值
    fn get_column(&self, index: i32) -> Result<Value, String>;
    
    /// 获取标签属性值
    fn get_tag_prop(&self, tag: &str, prop: &str) -> Result<Value, String>;
    
    /// 获取边属性值
    fn get_edge_prop(&self, edge: &str, prop: &str) -> Result<Value, String>;
    
    /// 获取源顶点属性值
    fn get_src_prop(&self, tag: &str, prop: &str) -> Result<Value, String>;
    
    /// 获取顶点
    fn get_vertex(&self, name: &str) -> Result<Value, String>;
    
    /// 获取边
    fn get_edge(&self) -> Result<Value, String>;
}