//! 验证器统一 trait 定义
//! 定义所有语句验证器的标准接口
//! 这是新验证器体系的核心，替代原有的分散式设计
//!
//! 设计原则：
//! 1. 保留完整功能（验证生命周期、上下文管理、权限检查等）
//! 2. 使用 trait 统一接口，便于扩展
//! 3. 使用枚举管理不同类型的验证器，避免动态分发

use crate::core::error::ValidationError;
use crate::query::context::ast::AstContext;

/// 列定义
#[derive(Debug, Clone)]
pub struct ColumnDef {
    pub name: String,
    pub type_: ValueType,
}

/// 值类型枚举
#[derive(Debug, Clone, PartialEq)]
pub enum ValueType {
    Unknown,
    Bool,
    Int,
    Float,
    String,
    Date,
    Time,
    DateTime,
    Vertex,
    Edge,
    Path,
    List,
    Map,
    Set,
    Null,
}

/// 表达式属性
#[derive(Debug, Clone, Default)]
pub struct ExpressionProps {
    pub input_props: Vec<InputProperty>,
    pub var_props: Vec<VarProperty>,
    pub tag_props: Vec<TagProperty>,
    pub edge_props: Vec<EdgeProperty>,
}

#[derive(Debug, Clone)]
pub struct InputProperty {
    pub prop_name: String,
    pub type_: ValueType,
}

#[derive(Debug, Clone)]
pub struct VarProperty {
    pub var_name: String,
    pub prop_name: String,
    pub type_: ValueType,
}

#[derive(Debug, Clone)]
pub struct TagProperty {
    pub tag_name: String,
    pub prop_name: String,
    pub type_: ValueType,
}

#[derive(Debug, Clone)]
pub struct EdgeProperty {
    pub edge_type: i32,
    pub prop_name: String,
    pub type_: ValueType,
}

/// 语句类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StatementType {
    Match,
    Go,
    FetchVertices,
    FetchEdges,
    Lookup,
    FindPath,
    GetSubgraph,
    InsertVertices,
    InsertEdges,
    Update,
    Delete,
    Create,
    CreateSpace,
    CreateTag,
    CreateEdge,
    Drop,
    DropSpace,
    DropTag,
    DropEdge,
    Alter,
    AlterTag,
    AlterEdge,
    Use,
    Pipe,
    Yield,
    OrderBy,
    Limit,
    Unwind,
    Set,
    Sequential,
    ShowSpaces,
    ShowTags,
    ShowEdges,
    DescribeSpace,
    DescribeTag,
    DescribeEdge,
    GroupBy,
    Assignment,
    Explain,
}

impl StatementType {
    pub fn as_str(&self) -> &'static str {
        match self {
            StatementType::Match => "MATCH",
            StatementType::Go => "GO",
            StatementType::FetchVertices => "FETCH_VERTICES",
            StatementType::FetchEdges => "FETCH_EDGES",
            StatementType::Lookup => "LOOKUP",
            StatementType::FindPath => "FIND_PATH",
            StatementType::GetSubgraph => "GET_SUBGRAPH",
            StatementType::InsertVertices => "INSERT_VERTICES",
            StatementType::InsertEdges => "INSERT_EDGES",
            StatementType::Update => "UPDATE",
            StatementType::Delete => "DELETE",
            StatementType::Create => "CREATE",
            StatementType::CreateSpace => "CREATE_SPACE",
            StatementType::CreateTag => "CREATE_TAG",
            StatementType::CreateEdge => "CREATE_EDGE",
            StatementType::Drop => "DROP",
            StatementType::DropSpace => "DROP_SPACE",
            StatementType::DropTag => "DROP_TAG",
            StatementType::DropEdge => "DROP_EDGE",
            StatementType::Alter => "ALTER",
            StatementType::AlterTag => "ALTER_TAG",
            StatementType::AlterEdge => "ALTER_EDGE",
            StatementType::Use => "USE",
            StatementType::Pipe => "PIPE",
            StatementType::Yield => "YIELD",
            StatementType::OrderBy => "ORDER_BY",
            StatementType::Limit => "LIMIT",
            StatementType::Unwind => "UNWIND",
            StatementType::Set => "SET",
            StatementType::Sequential => "SEQUENTIAL",
            StatementType::ShowSpaces => "SHOW_SPACES",
            StatementType::ShowTags => "SHOW_TAGS",
            StatementType::ShowEdges => "SHOW_EDGES",
            StatementType::DescribeSpace => "DESCRIBE_SPACE",
            StatementType::DescribeTag => "DESCRIBE_TAG",
            StatementType::DescribeEdge => "DESCRIBE_EDGE",
            StatementType::GroupBy => "GROUP_BY",
            StatementType::Assignment => "ASSIGNMENT",
            StatementType::Explain => "EXPLAIN",
        }
    }

    pub fn is_ddl(&self) -> bool {
        matches!(
            self,
            StatementType::Create
                | StatementType::CreateSpace
                | StatementType::CreateTag
                | StatementType::CreateEdge
                | StatementType::Drop
                | StatementType::DropSpace
                | StatementType::DropTag
                | StatementType::DropEdge
                | StatementType::Alter
                | StatementType::AlterTag
                | StatementType::AlterEdge
        )
    }

    pub fn is_dml(&self) -> bool {
        matches!(
            self,
            StatementType::InsertVertices
                | StatementType::InsertEdges
                | StatementType::Update
                | StatementType::Delete
        )
    }
}

/// 验证结果
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub success: bool,
    pub errors: Vec<ValidationError>,
    pub inputs: Vec<ColumnDef>,
    pub outputs: Vec<ColumnDef>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    pub fn success(inputs: Vec<ColumnDef>, outputs: Vec<ColumnDef>) -> Self {
        Self {
            success: true,
            errors: Vec::new(),
            inputs,
            outputs,
            warnings: Vec::new(),
        }
    }

    pub fn failure(errors: Vec<ValidationError>) -> Self {
        Self {
            success: false,
            errors,
            inputs: Vec::new(),
            outputs: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    pub fn merge(&mut self, other: ValidationResult) {
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
        if !other.success {
            self.success = false;
        }
    }
}

/// 所有语句验证器的统一接口
///
/// 设计原则：
/// 1. 简化接口，只保留核心方法
/// 2. 验证生命周期由 Validator 枚举统一管理
/// 3. 使用静态分发替代动态分发
/// 4. 直接使用 AstContext 作为验证上下文（AstContext 已包含 QueryContext）
pub trait StatementValidator {
    /// 执行验证逻辑
    /// 返回验证结果，包含输入/输出列定义
    /// 直接使用 &mut AstContext 作为上下文，避免不必要的包装
    fn validate(&mut self, ast: &mut AstContext) -> Result<ValidationResult, ValidationError>;

    /// 获取语句类型
    fn statement_type(&self) -> StatementType;

    /// 获取输入列定义
    fn inputs(&self) -> &[ColumnDef];

    /// 获取输出列定义
    fn outputs(&self) -> &[ColumnDef];

    /// 判断是否为全局语句（不需要预先选择空间）
    fn is_global_statement(&self) -> bool;

    /// 获取验证器名称
    fn validator_name(&self) -> String {
        format!("{}Validator", self.statement_type().as_str())
    }

    /// 获取表达式属性
    fn expression_props(&self) -> &ExpressionProps;

    /// 获取用户定义变量列表
    fn user_defined_vars(&self) -> &[String];
}

/// 判断语句类型是否为全局语句
pub fn is_global_statement_type(stmt_type: StatementType) -> bool {
    matches!(
        stmt_type,
        StatementType::CreateSpace
            | StatementType::DropSpace
            | StatementType::ShowSpaces
            | StatementType::DescribeSpace
            | StatementType::Use
    )
}
