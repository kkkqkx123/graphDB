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
use crate::query::context::execution::QueryContext;
use crate::query::parser::ast::Stmt;

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
    Drop,
    Alter,
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
            StatementType::Drop => "DROP",
            StatementType::Alter => "ALTER",
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
        }
    }

    pub fn is_ddl(&self) -> bool {
        matches!(
            self,
            StatementType::Create | StatementType::Drop | StatementType::Alter
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
/// 1. 保留完整验证生命周期
/// 2. 提供统一的接口便于管理和扩展
/// 3. 支持完整的上下文管理和错误收集
pub trait StatementValidator {
    /// 执行完整的验证生命周期
    ///
    /// 验证生命周期：
    /// 1. 检查是否需要空间（is_global_statement）
    /// 2. 执行具体验证逻辑（validate_impl）
    /// 3. 权限检查（check_permission）
    /// 4. 生成执行计划（to_plan）
    /// 5. 同步输入/输出到 AstContext
    fn validate(
        &mut self,
        query_context: Option<&QueryContext>,
        ast: &mut AstContext,
    ) -> Result<ValidationResult, ValidationError>;

    /// 获取语句类型
    fn statement_type(&self) -> StatementType;

    /// 获取输入列定义
    fn inputs(&self) -> &[ColumnDef];

    /// 获取输出列定义
    fn outputs(&self) -> &[ColumnDef];

    /// 判断是否为全局语句（不需要预先选择空间）
    /// 默认实现根据语句类型判断
    fn is_global_statement(&self, ast: &AstContext) -> bool {
        let stmt_type = ast.statement_type();

        // 基础全局语句类型
        if matches!(
            stmt_type,
            "CREATE_USER" | "ALTER_USER" | "DROP_USER" | "CHANGE_PASSWORD"
                | "SHOW_SPACES" | "DESC_SPACE"
                | "SHOW_USERS" | "DESC_USER"
                | "USE"
        ) {
            return true;
        }

        // 检查 CREATE 语句是否是 CREATE SPACE
        if stmt_type == "CREATE" {
            if let Some(ref stmt) = ast.sentence() {
                if let Stmt::Create(create_stmt) = stmt {
                    if let crate::query::parser::ast::stmt::CreateTarget::Space { .. } = create_stmt.target {
                        return true;
                    }
                }
            }
        }

        // 检查 DROP 语句是否是 DROP SPACE
        if stmt_type == "DROP" {
            if let Some(ref stmt) = ast.sentence() {
                if let Stmt::Drop(drop_stmt) = stmt {
                    if let crate::query::parser::ast::stmt::DropTarget::Space(_) = drop_stmt.target {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// 权限检查
    /// 默认实现返回成功，子类可以覆盖
    fn check_permission(&self) -> Result<(), ValidationError> {
        Ok(())
    }

    /// 生成执行计划
    /// 默认实现返回成功，子类可以覆盖
    fn to_plan(&mut self, _ast: &mut AstContext) -> Result<(), ValidationError> {
        Ok(())
    }

    /// 获取验证器名称
    fn validator_name(&self) -> String {
        format!("{}Validator", self.statement_type().as_str())
    }

    /// 获取表达式属性
    fn expression_props(&self) -> &ExpressionProps;

    /// 获取用户定义变量列表
    fn user_defined_vars(&self) -> &[String];
}

/// 验证器构建器 trait
/// 用于构建特定类型的验证器
pub trait ValidatorBuilder<V: StatementValidator> {
    fn build(self) -> V;
}

/// 验证器注册表
/// 用于管理和查找验证器
pub struct ValidatorRegistry {
    validators: std::collections::HashMap<StatementType, Box<dyn Fn() -> Box<dyn StatementValidator>>>,
}

impl ValidatorRegistry {
    pub fn new() -> Self {
        Self {
            validators: std::collections::HashMap::new(),
        }
    }

    pub fn register<F>(&mut self, stmt_type: StatementType, factory: F)
    where
        F: Fn() -> Box<dyn StatementValidator> + 'static,
    {
        self.validators.insert(stmt_type, Box::new(factory));
    }

    pub fn get(&self, stmt_type: StatementType) -> Option<&Box<dyn Fn() -> Box<dyn StatementValidator>>> {
        self.validators.get(&stmt_type)
    }

    pub fn contains(&self, stmt_type: StatementType) -> bool {
        self.validators.contains_key(&stmt_type)
    }
}

impl Default for ValidatorRegistry {
    fn default() -> Self {
        Self::new()
    }
}
