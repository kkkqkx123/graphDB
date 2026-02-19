//! 验证器核心类型定义
//!
//! 定义验证器模块使用的基础类型，包括语句类型、列定义等

use crate::core::DataType;

/// 语句类型枚举
///
/// 标识不同类型的查询语句，用于验证器分发和错误报告
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StatementType {
    /// MATCH 语句
    Match,
    /// GO 语句
    Go,
    /// FETCH VERTICES 语句
    FetchVertices,
    /// FETCH EDGES 语句
    FetchEdges,
    /// LOOKUP 语句
    Lookup,
    /// FIND PATH 语句
    FindPath,
    /// GET SUBGRAPH 语句
    GetSubgraph,
    /// INSERT 语句
    Insert,
    /// UPDATE 语句
    Update,
    /// DELETE 语句
    Delete,
    /// CREATE 语句（SPACE/TAG/EDGE）
    Create,
    /// DROP 语句
    Drop,
    /// ALTER 语句
    Alter,
    /// USE 语句
    Use,
    /// PIPE 语句（|）
    Pipe,
    /// YIELD 语句
    Yield,
    /// UNWIND 语句
    Unwind,
    /// SET 语句
    Set,
    /// SEQUENTIAL 语句（多语句）
    Sequential,
    /// SHOW 语句
    Show,
    /// DESCRIBE 语句
    Describe,
}

impl StatementType {
    /// 获取语句类型的名称
    pub fn name(&self) -> &'static str {
        match self {
            StatementType::Match => "MATCH",
            StatementType::Go => "GO",
            StatementType::FetchVertices => "FETCH VERTICES",
            StatementType::FetchEdges => "FETCH EDGES",
            StatementType::Lookup => "LOOKUP",
            StatementType::FindPath => "FIND PATH",
            StatementType::GetSubgraph => "GET SUBGRAPH",
            StatementType::Insert => "INSERT",
            StatementType::Update => "UPDATE",
            StatementType::Delete => "DELETE",
            StatementType::Create => "CREATE",
            StatementType::Drop => "DROP",
            StatementType::Alter => "ALTER",
            StatementType::Use => "USE",
            StatementType::Pipe => "PIPE",
            StatementType::Yield => "YIELD",
            StatementType::Unwind => "UNWIND",
            StatementType::Set => "SET",
            StatementType::Sequential => "SEQUENTIAL",
            StatementType::Show => "SHOW",
            StatementType::Describe => "DESCRIBE",
        }
    }

    /// 检查是否需要预先选择图空间
    pub fn requires_space(&self) -> bool {
        !matches!(
            self,
            StatementType::Create
                | StatementType::Drop
                | StatementType::Use
                | StatementType::Show
                | StatementType::Describe
        )
    }
}

impl std::fmt::Display for StatementType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// 列定义
///
/// 描述验证器输入或输出的列结构
#[derive(Debug, Clone, PartialEq)]
pub struct ColumnDef {
    /// 列名称
    pub name: String,
    /// 数据类型
    pub data_type: DataType,
    /// 是否可为空
    pub nullable: bool,
}

impl ColumnDef {
    /// 创建新的列定义
    pub fn new(name: impl Into<String>, data_type: DataType) -> Self {
        Self {
            name: name.into(),
            data_type,
            nullable: true,
        }
    }

    /// 创建非空列定义
    pub fn new_non_null(name: impl Into<String>, data_type: DataType) -> Self {
        Self {
            name: name.into(),
            data_type,
            nullable: false,
        }
    }

    /// 设置是否可为空
    pub fn with_nullable(mut self, nullable: bool) -> Self {
        self.nullable = nullable;
        self
    }
}

/// 表达式属性
///
/// 记录在验证过程中发现的表达式属性
#[derive(Debug, Clone, Default)]
pub struct ExpressionProps {
    /// 输入属性
    pub input_props: Vec<InputProperty>,
    /// 变量属性
    pub var_props: Vec<VarProperty>,
    /// 标签属性
    pub tag_props: Vec<TagProperty>,
    /// 边属性
    pub edge_props: Vec<EdgeProperty>,
}

/// 输入属性
#[derive(Debug, Clone)]
pub struct InputProperty {
    pub prop_name: String,
    pub data_type: DataType,
}

/// 变量属性
#[derive(Debug, Clone)]
pub struct VarProperty {
    pub var_name: String,
    pub prop_name: String,
    pub data_type: DataType,
}

/// 标签属性
#[derive(Debug, Clone)]
pub struct TagProperty {
    pub tag_name: String,
    pub prop_name: String,
    pub data_type: DataType,
}

/// 边属性
#[derive(Debug, Clone)]
pub struct EdgeProperty {
    pub edge_type: i32,
    pub prop_name: String,
    pub data_type: DataType,
}
