//! 查询类型基础定义

/// 查询类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QueryType {
    #[default]
    ReadQuery,
    WriteQuery,
    AdminQuery,
    SchemaQuery,
}
