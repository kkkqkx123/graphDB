//! 基础AST上下文定义

use crate::query::context::execution::QueryContext;
use crate::query::context::request_context::RequestContext;
use crate::query::context::symbol::SymbolTable;
use crate::query::context::validate::types::SpaceInfo;
use crate::query::parser::ast::Stmt;
use std::sync::Arc;

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

/// 变量信息
///
/// 统一变量信息结构，用于存储查询中的变量元数据
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

    pub fn with_source_clause(mut self, source_clause: String) -> Self {
        self.source_clause = source_clause;
        self
    }

    pub fn with_properties(mut self, properties: Vec<String>) -> Self {
        self.properties = properties;
        self
    }

    pub fn with_aggregated(mut self, is_aggregated: bool) -> Self {
        self.is_aggregated = is_aggregated;
        self
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
/// - SymbolTable: 符号表管理
/// - QueryType: 查询类型标识
#[derive(Debug, Clone)]
pub struct AstContext {
    pub qctx: Option<Arc<QueryContext>>,
    pub sentence: Option<Stmt>,
    pub space: SpaceInfo,
    pub symbol_table: SymbolTable,
    pub query_type: QueryType,
}

impl AstContext {
    pub fn new(qctx: Option<Arc<QueryContext>>, sentence: Option<Stmt>) -> Self {
        Self {
            qctx,
            sentence,
            space: SpaceInfo::default(),
            symbol_table: SymbolTable::new(),
            query_type: QueryType::default(),
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
            symbol_table: SymbolTable::new(),
            query_type: QueryType::default(),
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
            symbol_table: SymbolTable::new(),
            query_type: QueryType::default(),
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
            symbol_table: SymbolTable::new(),
            query_type,
        }
    }

    pub fn query_context(&self) -> Option<&Arc<QueryContext>> {
        self.qctx.as_ref()
    }

    pub fn sentence(&self) -> Option<&Stmt> {
        self.sentence.as_ref()
    }

    pub fn space(&self) -> &SpaceInfo {
        &self.space
    }

    pub fn set_space(&mut self, space: SpaceInfo) {
        self.space = space;
    }

    pub fn symbol_table(&self) -> &SymbolTable {
        &self.symbol_table
    }

    pub fn symbol_table_mut(&mut self) -> &mut SymbolTable {
        &mut self.symbol_table
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
                Stmt::Set(_) => "SET",
                Stmt::Remove(_) => "REMOVE",
                Stmt::Pipe(_) => "PIPE",
                Stmt::Drop(_) => "DROP",
                Stmt::Desc(_) => "DESC",
                Stmt::Alter(_) => "ALTER",
                Stmt::ChangePassword(_) => "CHANGE_PASSWORD",
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
        self.symbol_table.get_variable_info(name)
    }
}

impl Default for AstContext {
    fn default() -> Self {
        Self {
            qctx: None,
            sentence: None,
            space: SpaceInfo::default(),
            symbol_table: SymbolTable::new(),
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
            symbol_table: SymbolTable::new(),
            query_type: QueryType::default(),
        }
    }
}
