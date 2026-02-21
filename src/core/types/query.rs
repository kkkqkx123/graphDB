//! 查询类型基础定义

/// 查询类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QueryType {
    ReadQuery,
    WriteQuery,
    AdminQuery,
    SchemaQuery,
}

impl Default for QueryType {
    fn default() -> Self {
        QueryType::ReadQuery
    }
}
