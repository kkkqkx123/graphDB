//! 索引管理器接口 - 定义索引管理的基本操作

/// 索引信息 - 表示数据库索引
#[derive(Debug, Clone)]
pub struct Index {
    pub name: String,
    pub schema_name: String,
    pub fields: Vec<String>,
    pub is_unique: bool,
}

/// 索引管理器接口 - 定义索引管理的基本操作
pub trait IndexManager: Send + Sync + std::fmt::Debug {
    /// 获取指定名称的索引
    fn get_index(&self, name: &str) -> Option<Index>;
    /// 列出所有索引名称
    fn list_indexes(&self) -> Vec<String>;
    /// 检查索引是否存在
    fn has_index(&self, name: &str) -> bool;
}