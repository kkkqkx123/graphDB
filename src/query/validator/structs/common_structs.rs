//! 通用数据结构

use crate::query::validator::structs::{
    clause_structs::{
        MatchClauseContext, OrderByClauseContext, PaginationContext, ReturnClauseContext,
        UnwindClauseContext, WhereClauseContext, WithClauseContext, YieldClauseContext,
    },
    AliasType, QueryPart,
};
use crate::query::validator::validation_interface::{ValidationContext, ValidationError};

/// 验证上下文实现
#[derive(Debug, Clone)]
pub struct ValidationContextImpl {
    pub query_parts: Vec<QueryPart>,
    pub errors: Vec<ValidationError>,
    pub aliases: std::collections::HashMap<String, AliasType>,
    pub current_clause: Option<ClauseType>,
}

impl ValidationContextImpl {
    pub fn new() -> Self {
        Self {
            query_parts: Vec::new(),
            errors: Vec::new(),
            aliases: std::collections::HashMap::new(),
            current_clause: None,
        }
    }

    pub fn add_error(&mut self, error: ValidationError) {
        self.errors.push(error);
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn get_errors(&self) -> &[ValidationError] {
        &self.errors
    }
}

impl ValidationContext for ValidationContextImpl {
    fn get_query_parts(&self) -> &[QueryPart] {
        &self.query_parts
    }

    fn get_aliases(&self) -> &std::collections::HashMap<String, AliasType> {
        &self.aliases
    }

    fn add_error(&mut self, error: ValidationError) {
        self.errors.push(error);
    }

    fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    fn get_errors(&self) -> &[ValidationError] {
        &self.errors
    }
}

/// 子句类型枚举
#[derive(Debug, Clone, PartialEq)]
pub enum ClauseType {
    Match,
    Return,
    With,
    Unwind,
    Where,
    Yield,
}

/// Cypher子句上下文枚举
/// 表示不同类型的Cypher子句上下文
#[derive(Debug, Clone)]
pub enum CypherClauseContext {
    Match(MatchClauseContext),
    Where(WhereClauseContext),
    Return(ReturnClauseContext),
    With(WithClauseContext),
    Unwind(UnwindClauseContext),
    Yield(YieldClauseContext),
    OrderBy(OrderByClauseContext),
    Pagination(PaginationContext),
}

/// Cypher子句类型
#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash)]
pub enum CypherClauseKind {
    Match,
    Where,
    Return,
    With,
    Unwind,
    Yield,
    OrderBy,
    Pagination,
}

impl CypherClauseContext {
    /// 获取子句类型
    pub fn kind(&self) -> CypherClauseKind {
        match self {
            CypherClauseContext::Match(_) => CypherClauseKind::Match,
            CypherClauseContext::Where(_) => CypherClauseKind::Where,
            CypherClauseContext::Return(_) => CypherClauseKind::Return,
            CypherClauseContext::With(_) => CypherClauseKind::With,
            CypherClauseContext::Unwind(_) => CypherClauseKind::Unwind,
            CypherClauseContext::Yield(_) => CypherClauseKind::Yield,
            CypherClauseContext::OrderBy(_) => CypherClauseKind::OrderBy,
            CypherClauseContext::Pagination(_) => CypherClauseKind::Pagination,
        }
    }

    /// 获取 Yield 子句上下文
    pub fn yield_clause(&self) -> Option<&super::clause_structs::YieldClauseContext> {
        match self {
            CypherClauseContext::Yield(ctx) => Some(ctx),
            _ => None,
        }
    }
}

use crate::core::DataType;
use std::collections::HashMap;

/// 验证结果
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn add_error(&mut self, error: ValidationError) {
        self.errors.push(error);
        self.is_valid = false;
    }

    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    pub fn merge(&mut self, other: ValidationResult) {
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
        if !other.is_valid {
            self.is_valid = false;
        }
    }
}

/// LOOKUP 索引类型
#[derive(Debug, Clone, PartialEq)]
pub enum LookupIndexType {
    None,
    Single(String),
    Composite(Vec<String>),
}

/// LOOKUP 目标定义
#[derive(Debug, Clone)]
pub struct LookupTarget {
    pub label: String,
    pub index_type: LookupIndexType,
    pub properties: HashMap<String, DataType>,
}
