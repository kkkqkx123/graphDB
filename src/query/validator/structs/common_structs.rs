//! 通用数据结构

use crate::query::validator::structs::{
    clause_structs::{
        MatchClauseContext, OrderByClauseContext, PaginationContext, ReturnClauseContext,
        UnwindClauseContext, WhereClauseContext, WithClauseContext, YieldClauseContext,
    },
    AliasType, QueryPart,
};
use crate::core::error::ValidationError;

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

impl CypherClauseKind {
    /// 获取子句类型的字符串表示
    pub fn as_str(&self) -> &'static str {
        match self {
            CypherClauseKind::Match => "MATCH",
            CypherClauseKind::Where => "WHERE",
            CypherClauseKind::Return => "RETURN",
            CypherClauseKind::With => "WITH",
            CypherClauseKind::Unwind => "UNWIND",
            CypherClauseKind::Yield => "YIELD",
            CypherClauseKind::OrderBy => "ORDER BY",
            CypherClauseKind::Pagination => "PAGINATION",
        }
    }
}

/// 验证状态
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationState {
    Pending,
    InProgress,
    Success,
    Failed,
}

/// 验证统计信息
#[derive(Debug, Clone, Default)]
pub struct ValidationStats {
    pub total_validated: usize,
    pub success_count: usize,
    pub failure_count: usize,
    pub warning_count: usize,
}

impl ValidationStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_success(&mut self) {
        self.total_validated += 1;
        self.success_count += 1;
    }

    pub fn record_failure(&mut self) {
        self.total_validated += 1;
        self.failure_count += 1;
    }

    pub fn record_warning(&mut self) {
        self.warning_count += 1;
    }
}
