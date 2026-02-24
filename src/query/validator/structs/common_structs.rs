//! 通用数据结构

use crate::query::validator::structs::{AliasType, QueryPart};
use crate::core::error::ValidationError;
use crate::core::DataType;
use crate::query::validator::validator_trait::ColumnDef;
use crate::query::validator::strategies::type_inference::ExpressionValidationContext;
use std::collections::HashMap;

/// 验证上下文实现
#[derive(Debug, Clone)]
pub struct ValidationContextImpl {
    pub query_parts: Vec<QueryPart>,
    pub errors: Vec<ValidationError>,
    pub aliases: std::collections::HashMap<String, AliasType>,
    /// 变量定义：变量名 -> 列定义
    pub variables: std::collections::HashMap<String, Vec<ColumnDef>>,
}

impl ValidationContextImpl {
    pub fn new() -> Self {
        Self {
            query_parts: Vec::new(),
            errors: Vec::new(),
            aliases: std::collections::HashMap::new(),
            variables: std::collections::HashMap::new(),
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

    /// 检查变量是否存在
    pub fn exists_var(&self, name: &str) -> bool {
        self.variables.contains_key(name)
    }

    /// 获取变量的列定义
    pub fn get_var(&self, name: &str) -> Vec<ColumnDef> {
        self.variables.get(name).cloned().unwrap_or_default()
    }

    /// 注册变量
    pub fn register_variable(&mut self, name: String, cols: Vec<ColumnDef>) {
        self.variables.insert(name, cols);
    }
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

// 为 ValidationContextImpl 实现 ExpressionValidationContext trait
impl ExpressionValidationContext for ValidationContextImpl {
    fn get_aliases(&self) -> &HashMap<String, AliasType> {
        &self.aliases
    }

    fn get_variable_types(&self) -> Option<&HashMap<String, DataType>> {
        None
    }
}
