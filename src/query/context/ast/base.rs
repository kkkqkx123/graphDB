//! 基础AST上下文定义

use crate::core::error::ValidationError;
use crate::query::context::ast::common::VariableInfo;
use crate::query::context::execution::QueryContext;
use crate::query::context::request_context::RequestContext;
use crate::query::context::symbol::SymbolTable;
use crate::query::context::validate::types::SpaceInfo;
use crate::query::parser::ast::Stmt;
use crate::query::validator::ColumnDef;
use once_cell::sync::Lazy;
use std::sync::Arc;

static EMPTY_SYMBOL_TABLE: Lazy<SymbolTable> = Lazy::new(SymbolTable::new);

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

/// AST上下文接口
pub trait AstContextTrait {
    fn get_query_context(&self) -> Option<Arc<QueryContext>>;
    fn get_sentence(&self) -> Option<Stmt>;
    fn get_space_info(&self) -> SpaceInfo;
    fn lookup_variable(&self, name: &str) -> Option<VariableInfo>;
}

/// 基础AST上下文
///
/// 提供查询执行过程中的AST节点上下文信息，包括：
/// - QueryContext: 访问运行时资源
/// - Sentence: 关联原始语法树节点
/// - SpaceInfo: 支持多空间场景
/// - QueryType: 查询类型标识
/// - 验证结果: outputs, inputs, validation_errors
///
/// 注意：符号表由 QueryContext 持有，AstContext 通过 qctx.sym_table() 访问
#[derive(Debug, Clone)]
pub struct AstContext {
    pub qctx: Option<Arc<QueryContext>>,
    pub sentence: Option<Stmt>,
    pub space: SpaceInfo,
    pub query_type: QueryType,
    pub(crate) outputs: Vec<ColumnDef>,
    pub(crate) inputs: Vec<ColumnDef>,
    pub(crate) validation_errors: Vec<ValidationError>,
}

impl AstContext {
    pub fn new(qctx: Option<Arc<QueryContext>>, sentence: Option<Stmt>) -> Self {
        Self {
            qctx,
            sentence,
            space: SpaceInfo::default(),
            query_type: QueryType::default(),
            outputs: Vec::new(),
            inputs: Vec::new(),
            validation_errors: Vec::new(),
        }
    }

    pub fn from_strings(query_type: &str, query_text: &str) -> Self {
        let request_params = crate::query::context::request_context::RequestParams::new(query_text.to_string());
        let request_context = std::sync::Arc::new(RequestContext::new(None, request_params));
        
        let mut qctx = QueryContext::new();
        qctx.set_rctx(request_context);
        
        let ctx = Self {
            qctx: Some(std::sync::Arc::new(qctx)),
            sentence: None,
            space: SpaceInfo::default(),
            query_type: QueryType::default(),
            outputs: Vec::new(),
            inputs: Vec::new(),
            validation_errors: Vec::new(),
        };

        if query_type == "CYPHER" {}
        else if query_type == "MATCH" {}

        ctx
    }

    pub fn with_space(
        qctx: Option<Arc<QueryContext>>,
        sentence: Option<Stmt>,
        space: SpaceInfo,
    ) -> Self {
        Self {
            qctx,
            sentence,
            space,
            query_type: QueryType::default(),
            outputs: Vec::new(),
            inputs: Vec::new(),
            validation_errors: Vec::new(),
        }
    }

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
            query_type,
            outputs: Vec::new(),
            inputs: Vec::new(),
            validation_errors: Vec::new(),
        }
    }

    pub fn query_context(&self) -> Option<&QueryContext> {
        self.qctx.as_deref()
    }

    pub fn sentence(&self) -> Option<&Stmt> {
        self.sentence.as_ref()
    }

    pub fn set_statement(&mut self, stmt: Stmt) {
        self.sentence = Some(stmt);
    }

    pub fn space(&self) -> &SpaceInfo {
        &self.space
    }

    pub fn set_space(&mut self, space: SpaceInfo) {
        self.space = space;
    }

    pub fn symbol_table(&self) -> &SymbolTable {
        self.qctx.as_ref()
            .map_or(&*EMPTY_SYMBOL_TABLE, |v| v.sym_table())
    }

    pub fn query_type(&self) -> QueryType {
        self.query_type
    }

    pub fn set_query_type(&mut self, query_type: QueryType) {
        self.query_type = query_type;
    }

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
                Stmt::Yield(_) => "YIELD",
                Stmt::Set(_) => "SET",
                Stmt::Remove(_) => "REMOVE",
                Stmt::Pipe(_) => "PIPE",
                Stmt::Drop(_) => "DROP",
                Stmt::Desc(_) => "DESC",
                Stmt::Alter(_) => "ALTER",
                Stmt::CreateUser(_) => "CREATE_USER",
                Stmt::AlterUser(_) => "ALTER_USER",
                Stmt::DropUser(_) => "DROP_USER",
                Stmt::ChangePassword(_) => "CHANGE_PASSWORD",
                Stmt::Grant(_) => "GRANT",
                Stmt::Revoke(_) => "REVOKE",
                Stmt::DescribeUser(_) => "DESCRIBE_USER",
                Stmt::ShowUsers(_) => "SHOW_USERS",
                Stmt::ShowRoles(_) => "SHOW_ROLES",
            },
            None => {
                if let Some(ref qctx) = self.qctx {
                    let query_text = qctx.rctx()
                        .map(|rctx| rctx.request_params().query.clone())
                        .unwrap_or_default();
                    let upper_query = query_text.to_uppercase();
                    let trimmed_query = upper_query.trim_start();

                    if trimmed_query.starts_with("MATCH") {
                        let after_match = &trimmed_query[5..].trim_start();
                        if after_match.starts_with('(') || after_match.starts_with('{') {
                            if after_match.ends_with(')') || after_match.ends_with('}') {
                                let content = &after_match[..after_match.len()-1];
                                if !content.contains("RETURN")
                                    && !content.contains("WHERE")
                                    && !content.contains("WITH")
                                    && !content.contains("ORDER")
                                    && !content.contains("LIMIT")
                                    && !content.contains("SKIP") {
                                    return "MATCH";
                                }
                            }
                        }
                        return "CYPHER";
                    }

                    if trimmed_query.starts_with("CREATE")
                        || trimmed_query.starts_with("RETURN")
                        || trimmed_query.starts_with("WHERE") {
                        return "CYPHER";
                    }

                    if trimmed_query.starts_with("GO")
                        || trimmed_query.starts_with("LOOKUP")
                        || trimmed_query.starts_with("FETCH")
                        || trimmed_query.starts_with("FIND")
                        || trimmed_query.starts_with("SUBGRAPH") {
                        return "QUERY";
                    }
                }
                "UNKNOWN"
            }
        }
    }

    pub fn set_query_type_from_statement(&mut self) {
        let stmt_type = self.statement_type().to_uppercase();
        self.query_type = match stmt_type.as_str() {
            "MATCH" | "CYPHER" => QueryType::ReadQuery,
            "CREATE" | "RETURN" | "WHERE" => QueryType::WriteQuery,
            "GO" | "LOOKUP" | "FETCH" | "FIND" | "SUBGRAPH" => QueryType::ReadQuery,
            _ => QueryType::default(),
        };
    }

    pub fn contains_path_query(&self) -> bool {
        matches!(
            &self.sentence,
            Some(Stmt::FindPath(_)) | Some(Stmt::Subgraph(_))
        )
    }

    pub fn query_text(&self) -> String {
        self.qctx
            .as_ref()
            .and_then(|ctx| ctx.rctx().map(|rctx| rctx.request_params().query.clone()))
            .unwrap_or_default()
    }

    pub fn add_output(&mut self, name: String, type_: crate::query::validator::ValueType) {
        self.outputs.push(ColumnDef { name, type_ });
    }

    pub fn outputs(&self) -> &[ColumnDef] {
        &self.outputs
    }

    pub fn add_input(&mut self, name: String, type_: crate::query::validator::ValueType) {
        self.inputs.push(ColumnDef { name, type_ });
    }

    pub fn inputs(&self) -> &[ColumnDef] {
        &self.inputs
    }

    pub fn add_validation_error(&mut self, error: ValidationError) {
        self.validation_errors.push(error);
    }

    pub fn validation_errors(&self) -> &[ValidationError] {
        &self.validation_errors
    }

    pub fn has_validation_errors(&self) -> bool {
        !self.validation_errors.is_empty()
    }

    pub fn clear_validation_errors(&mut self) {
        self.validation_errors.clear();
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
        self.symbol_table().get_variable_info(name)
    }
}

impl Default for AstContext {
    fn default() -> Self {
        Self {
            qctx: Some(Arc::new(QueryContext::new())),
            sentence: None,
            space: SpaceInfo::default(),
            query_type: QueryType::default(),
            outputs: Vec::new(),
            inputs: Vec::new(),
            validation_errors: Vec::new(),
        }
    }
}

impl From<(&str, &str)> for AstContext {
    fn from((_query_type, _query_text): (&str, &str)) -> Self {
        Self {
            qctx: Some(Arc::new(QueryContext::new())),
            sentence: None,
            space: SpaceInfo::default(),
            query_type: QueryType::default(),
            outputs: Vec::new(),
            inputs: Vec::new(),
            validation_errors: Vec::new(),
        }
    }
}
