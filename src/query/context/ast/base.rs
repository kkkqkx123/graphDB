//! 基础AST上下文定义

use crate::core::context::query::QueryContext;
use crate::query::context::validate::types::SpaceInfo;
use crate::query::parser::ast::Stmt;
use std::sync::Arc;

/// 查询类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QueryType {
    /// 读查询
    ReadQuery,
    /// 写查询
    WriteQuery,
    /// 管理查询
    AdminQuery,
    /// 模式查询
    SchemaQuery,
}

impl Default for QueryType {
    fn default() -> Self {
        QueryType::ReadQuery
    }
}

/// 变量信息
#[derive(Debug, Clone)]
pub struct VariableInfo {
    pub variable_name: String,
    pub variable_type: String,
    pub source_clause: String,
    pub is_aggregated: bool,
    pub properties: Vec<String>,
}

impl VariableInfo {
    pub fn new(variable_name: String, variable_type: String) -> Self {
        Self {
            variable_name,
            variable_type,
            source_clause: String::new(),
            is_aggregated: false,
            properties: Vec::new(),
        }
    }
}

/// 变量作用域管理器
/// 
/// 负责管理查询中的变量作用域层级结构，支持嵌套作用域和变量查找。
/// 与 `VariableVisibility` 枚举不同，此结构体用于管理复杂的变量作用域关系。
#[derive(Debug, Clone)]
pub struct VariableScope {
    pub current_scope: std::collections::HashMap<String, VariableInfo>,
    pub parent_scope: Option<Arc<VariableScope>>,
}

impl VariableScope {
    pub fn new() -> Self {
        Self {
            current_scope: std::collections::HashMap::new(),
            parent_scope: None,
        }
    }

    pub fn with_parent(parent: Arc<VariableScope>) -> Self {
        Self {
            current_scope: std::collections::HashMap::new(),
            parent_scope: Some(parent),
        }
    }

    pub fn add_variable(&mut self, name: String, info: VariableInfo) -> Result<(), ScopeError> {
        if self.current_scope.contains_key(&name) {
            return Err(ScopeError::VariableAlreadyExists(name));
        }
        self.current_scope.insert(name, info);
        Ok(())
    }

    pub fn lookup(&self, name: &str) -> Option<VariableInfo> {
        if let Some(info) = self.current_scope.get(name) {
            return Some(info.clone());
        }

        if let Some(parent) = &self.parent_scope {
            return parent.lookup(name);
        }

        None
    }
}

impl Default for VariableScope {
    fn default() -> Self {
        Self::new()
    }
}

/// 作用域错误类型
#[derive(Debug, Clone)]
pub enum ScopeError {
    VariableAlreadyExists(String),
    VariableNotFound(String),
}

impl std::fmt::Display for ScopeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScopeError::VariableAlreadyExists(name) => {
                write!(f, "变量 '{}' 已存在于当前作用域中", name)
            }
            ScopeError::VariableNotFound(name) => {
                write!(f, "变量 '{}' 未找到", name)
            }
        }
    }
}

impl std::error::Error for ScopeError {}

/// 验证错误类型
#[derive(Debug, Clone)]
pub enum ValidationError {
    MissingSentence,
    InvalidSpaceName,
    VariableScopeInconsistent,
    QueryTypeMismatch,
    Custom(String),
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::MissingSentence => {
                write!(f, "缺少查询语句")
            }
            ValidationError::InvalidSpaceName => {
                write!(f, "无效的空间名称")
            }
            ValidationError::VariableScopeInconsistent => {
                write!(f, "变量作用域不一致")
            }
            ValidationError::QueryTypeMismatch => {
                write!(f, "查询类型不匹配")
            }
            ValidationError::Custom(msg) => {
                write!(f, "{}", msg)
            }
        }
    }
}

impl std::error::Error for ValidationError {}

/// AST上下文接口
pub trait AstContextTrait {
    /// 获取查询上下文
    fn get_query_context(&self) -> Option<Arc<QueryContext>>;

    /// 获取查询语句
    fn get_sentence(&self) -> Option<Stmt>;

    /// 获取命名空间信息
    fn get_space_info(&self) -> SpaceInfo;

    /// 变量查找
    fn lookup_variable(&self, name: &str) -> Option<VariableInfo>;

    /// 上下文验证（基础验证）
    fn validate(&self) -> Result<(), ValidationError> {
        if self.get_sentence().is_none() {
            return Err(ValidationError::MissingSentence);
        }

        if self.get_space_info().space_name.is_empty() {
            return Err(ValidationError::InvalidSpaceName);
        }

        Ok(())
    }

    /// 完整的上下文验证
    fn validate_full(&self) -> Result<(), ValidationError> {
        self.validate()?;
        Ok(())
    }
}

/// 基础AST上下文
///
/// 提供查询执行过程中的AST节点上下文信息，包括：
/// - QueryContext: 访问运行时资源（元数据管理器、存储客户端等）
/// - Sentence: 关联原始语法树节点，用于错误定位和调试
/// - SpaceInfo: 支持多空间场景
/// - VariableScope: 变量作用域管理
/// - QueryType: 查询类型标识
#[derive(Debug, Clone)]
pub struct AstContext {
    /// 查询上下文引用，提供运行时资源访问
    pub qctx: Option<Arc<QueryContext>>,

    /// 语法树节点引用，用于错误定位和调试
    pub sentence: Option<Stmt>,

    /// 空间信息，支持多空间查询
    pub space: SpaceInfo,

    /// 变量作用域管理
    pub variable_scope: VariableScope,

    /// 查询类型标识
    pub query_type: QueryType,
}

impl AstContext {
    /// 创建新的AST上下文
    pub fn new(qctx: Option<Arc<QueryContext>>, sentence: Option<Stmt>) -> Self {
        Self {
            qctx,
            sentence,
            space: SpaceInfo::default(),
            variable_scope: VariableScope::new(),
            query_type: QueryType::default(),
        }
    }

    /// 创建新的AST上下文（便捷方法）
    pub fn from_strings(query_type: &str, query_text: &str) -> Self {
        let mut ctx = Self {
            qctx: None,
            sentence: None,
            space: SpaceInfo::default(),
            variable_scope: VariableScope::new(),
            query_type: QueryType::default(),
        };

        // 设置查询上下文，以便query_text()方法可以返回正确的查询文本
        ctx.qctx = Some(std::sync::Arc::new(crate::core::context::query::QueryContext::new(
            "temp_id".to_string(),
            crate::core::context::query::QueryType::DataQuery,
            query_text.to_string(),
            crate::core::context::session::SessionInfo::new(
                "temp_session".to_string(),
                "temp_user".to_string(),
                vec!["temp_role".to_string()],
                "127.0.0.1".to_string(),
                8080,
                "temp_client".to_string(),
                "temp_connection".to_string(),
            ),
        )));

        // 根据query_type参数设置一个虚拟的语句，以便statement_type()方法返回正确的值
        if query_type == "CYPHER" {
            // 对于Cypher查询，我们不设置具体的语句，而是需要一种方式来让statement_type()返回"CYPHER"
            // 我们将通过扩展AstContext来实现这一点
        }

        ctx
    }

    /// 创建带有空间信息的AST上下文
    pub fn with_space(
        qctx: Option<Arc<QueryContext>>,
        sentence: Option<Stmt>,
        space: SpaceInfo,
    ) -> Self {
        Self {
            qctx,
            sentence,
            space,
            variable_scope: VariableScope::new(),
            query_type: QueryType::default(),
        }
    }

    /// 创建带有查询类型的AST上下文
    pub fn with_query_type(
        qctx: Option<Arc<QueryContext>>,
        sentence: Option<Stmt>,
        space: SpaceInfo,
        query_type: QueryType,
    ) -> Self {
        Self {
            qctx,
            sentence,
            space,
            variable_scope: VariableScope::new(),
            query_type,
        }
    }

    /// 获取查询上下文
    pub fn query_context(&self) -> Option<&Arc<QueryContext>> {
        self.qctx.as_ref()
    }

    /// 获取语句引用
    pub fn sentence(&self) -> Option<&Stmt> {
        self.sentence.as_ref()
    }

    /// 获取空间信息
    pub fn space(&self) -> &SpaceInfo {
        &self.space
    }

    /// 设置空间信息
    pub fn set_space(&mut self, space: SpaceInfo) {
        self.space = space;
    }

    /// 获取变量作用域
    pub fn variable_scope(&self) -> &VariableScope {
        &self.variable_scope
    }

    /// 获取可变变量作用域
    pub fn variable_scope_mut(&mut self) -> &mut VariableScope {
        &mut self.variable_scope
    }

    /// 获取查询类型
    pub fn query_type(&self) -> QueryType {
        self.query_type
    }

    /// 设置查询类型
    pub fn set_query_type(&mut self, query_type: QueryType) {
        self.query_type = query_type;
    }

    /// 验证变量作用域一致性
    fn validate_variable_scope(&self) -> Result<(), ValidationError> {
        for (name, info) in &self.variable_scope.current_scope {
            if info.variable_name != *name {
                return Err(ValidationError::VariableScopeInconsistent);
            }
        }
        Ok(())
    }

    /// 验证查询类型与语句匹配
    fn validate_query_type(&self) -> Result<(), ValidationError> {
        match &self.sentence {
            Some(stmt) => {
                let expected_type = match stmt {
                    Stmt::Query(_) | Stmt::Match(_) | Stmt::Go(_) | Stmt::Fetch(_) 
                    | Stmt::Lookup(_) | Stmt::Subgraph(_) | Stmt::FindPath(_) => QueryType::ReadQuery,
                    Stmt::Create(_) | Stmt::Delete(_) | Stmt::Update(_) 
                    | Stmt::Insert(_) | Stmt::Merge(_) | Stmt::Set(_) 
                    | Stmt::Remove(_) => QueryType::WriteQuery,
                    Stmt::Use(_) | Stmt::Show(_) => QueryType::AdminQuery,
                    Stmt::Explain(_) => QueryType::SchemaQuery,
                    Stmt::Unwind(_) | Stmt::Return(_) | Stmt::With(_) 
                    | Stmt::Pipe(_) => QueryType::ReadQuery,
                };

                if self.query_type != expected_type {
                    return Err(ValidationError::QueryTypeMismatch);
                }
                Ok(())
            }
            None => Err(ValidationError::MissingSentence),
        }
    }

    /// 完整的上下文验证
    pub fn validate_full(&self) -> Result<(), ValidationError> {
        self.validate_variable_scope()?;
        self.validate_query_type()?;
        Ok(())
    }

    /// 获取语句类型
    pub fn statement_type(&self) -> &str {
        match &self.sentence {
            Some(stmt) => match stmt {
                Stmt::Query(_) => "QUERY",
                Stmt::Create(_) => "CREATE",
                Stmt::Match(_) => "MATCH",
                Stmt::Delete(_) => "DELETE",
                Stmt::Update(_) => "UPDATE",
                Stmt::Go(_) => "GO",
                Stmt::Fetch(_) => "FETCH",
                Stmt::Use(_) => "USE",
                Stmt::Show(_) => "SHOW",
                Stmt::Explain(_) => "EXPLAIN",
                Stmt::Lookup(_) => "LOOKUP",
                Stmt::Subgraph(_) => "SUBGRAPH",
                Stmt::FindPath(_) => "FIND_PATH",
                Stmt::Insert(_) => "INSERT",
                Stmt::Merge(_) => "MERGE",
                Stmt::Unwind(_) => "UNWIND",
                Stmt::Return(_) => "RETURN",
                Stmt::With(_) => "WITH",
                Stmt::Set(_) => "SET",
                Stmt::Remove(_) => "REMOVE",
                Stmt::Pipe(_) => "PIPE",
            },
            None => {
                // 检查查询上下文中的查询文本，如果包含"CYPHER"相关信息，返回"CYPHER"
                if let Some(ref qctx) = self.qctx {
                    if qctx.query_text.to_uppercase().starts_with("MATCH")
                        || qctx.query_text.to_uppercase().starts_with("CREATE")
                        || qctx.query_text.to_uppercase().starts_with("RETURN")
                        || qctx.query_text.to_uppercase().starts_with("WHERE") {
                        return "CYPHER";
                    }

                    // 如果是NGQL查询，返回"QUERY"
                    if qctx.query_text.to_uppercase().starts_with("GO")
                        || qctx.query_text.to_uppercase().starts_with("LOOKUP")
                        || qctx.query_text.to_uppercase().starts_with("FETCH")
                        || qctx.query_text.to_uppercase().starts_with("FIND")
                        || qctx.query_text.to_uppercase().starts_with("SUBGRAPH") {
                        return "QUERY";
                    }
                }
                "UNKNOWN"
            }
        }
    }

    /// 检查是否为路径查询
    pub fn contains_path_query(&self) -> bool {
        matches!(
            &self.sentence,
            Some(Stmt::FindPath(_)) | Some(Stmt::Subgraph(_))
        )
    }

    /// 获取查询文本
    pub fn query_text(&self) -> &str {
        self.qctx
            .as_ref()
            .map(|ctx| ctx.query_text.as_str())
            .unwrap_or("")
    }
}

impl AstContextTrait for AstContext {
    fn get_query_context(&self) -> Option<Arc<QueryContext>> {
        self.qctx.clone()
    }

    fn get_sentence(&self) -> Option<Stmt> {
        self.sentence.clone()
    }

    fn get_space_info(&self) -> SpaceInfo {
        self.space.clone()
    }

    fn lookup_variable(&self, name: &str) -> Option<VariableInfo> {
        self.variable_scope.lookup(name)
    }

    fn validate(&self) -> Result<(), ValidationError> {
        self.validate_full()
    }
}

impl Default for AstContext {
    fn default() -> Self {
        Self {
            qctx: None,
            sentence: None,
            space: SpaceInfo::default(),
            variable_scope: VariableScope::default(),
            query_type: QueryType::default(),
        }
    }
}

impl From<(&str, &str)> for AstContext {
    fn from((_query_type, _query_text): (&str, &str)) -> Self {
        Self {
            qctx: None,
            sentence: None,
            space: SpaceInfo::default(),
            variable_scope: VariableScope::default(),
            query_type: QueryType::default(),
        }
    }
}
