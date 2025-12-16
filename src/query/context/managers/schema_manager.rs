//! Schema管理器接口 - 定义Schema管理的基本操作

/// Schema信息 - 表示数据库Schema
#[derive(Debug, Clone)]
pub struct Schema {
    pub name: String,
    pub fields: std::collections::HashMap<String, String>,
    pub is_vertex: bool,
}

/// Schema管理器接口 - 定义Schema管理的基本操作
pub trait SchemaManager: Send + Sync + std::fmt::Debug {
    /// 获取指定名称的Schema
    fn get_schema(&self, name: &str) -> Option<Schema>;
    /// 列出所有Schema名称
    fn list_schemas(&self) -> Vec<String>;
    /// 检查Schema是否存在
    fn has_schema(&self, name: &str) -> bool;
}

/// 字符集信息 - 管理字符集和排序规则
#[derive(Debug, Clone)]
pub struct CharsetInfo {
    pub charset: String,
    pub collation: String,
}

impl Default for CharsetInfo {
    fn default() -> Self {
        Self {
            charset: "utf8mb4".to_string(),
            collation: "utf8mb4_general_ci".to_string(),
        }
    }
}