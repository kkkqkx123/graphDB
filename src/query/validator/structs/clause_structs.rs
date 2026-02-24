//! 子句相关数据结构

use super::alias_structs::AliasType;
use super::path_structs::Path;
use crate::core::Expression;
use crate::core::YieldColumn;
use crate::core::DataType;
use crate::core::error::ValidationError;
use crate::core::types::OrderDirection;
use crate::query::validator::QueryPart;
use crate::query::validator::strategies::type_inference::ExpressionValidationContext;
use std::collections::HashMap;

/// Match子句上下文
#[derive(Debug, Clone)]
pub struct MatchClauseContext {
    pub paths: Vec<Path>,
    pub aliases_available: HashMap<String, AliasType>,
    pub aliases_generated: HashMap<String, AliasType>,
    pub where_clause: Option<WhereClauseContext>,
    pub is_optional: bool,
    pub skip: Option<Expression>,
    pub limit: Option<Expression>,
    pub query_parts: Vec<QueryPart>,
    pub errors: Vec<ValidationError>,
}

/// WHERE子句上下文
#[derive(Debug, Clone)]
pub struct WhereClauseContext {
    pub filter: Option<Expression>,
    pub aliases_available: HashMap<String, AliasType>,
    pub aliases_generated: HashMap<String, AliasType>,
    pub paths: Vec<Path>, // WHERE子句中可能包含的路径
    pub query_parts: Vec<QueryPart>,
    pub errors: Vec<ValidationError>,
}

/// RETURN子句上下文
#[derive(Debug, Clone)]
pub struct ReturnClauseContext {
    pub yield_clause: YieldClauseContext,
    pub aliases_available: HashMap<String, AliasType>,
    pub aliases_generated: HashMap<String, AliasType>,
    pub pagination: Option<PaginationContext>,
    pub order_by: Option<OrderByClauseContext>,
    pub distinct: bool,
    pub query_parts: Vec<QueryPart>,
    pub errors: Vec<ValidationError>,
}

/// WITH子句上下文
#[derive(Debug, Clone)]
pub struct WithClauseContext {
    pub yield_clause: YieldClauseContext,
    pub aliases_available: HashMap<String, AliasType>,
    pub aliases_generated: HashMap<String, AliasType>,
    pub where_clause: Option<WhereClauseContext>,
    pub pagination: Option<PaginationContext>,
    pub order_by: Option<OrderByClauseContext>,
    pub distinct: bool,
    pub query_parts: Vec<QueryPart>,
    pub errors: Vec<ValidationError>,
}

/// UNWIND子句上下文
#[derive(Debug, Clone)]
pub struct UnwindClauseContext {
    pub alias: String,
    pub unwind_expression: Expression,
    pub aliases_available: HashMap<String, AliasType>,
    pub aliases_generated: HashMap<String, AliasType>,
    pub paths: Vec<Path>, // Unwind子句中可能包含的路径
    pub query_parts: Vec<QueryPart>,
    pub errors: Vec<ValidationError>,
}

/// Yield子句上下文
#[derive(Debug, Clone)]
pub struct YieldClauseContext {
    pub yield_columns: Vec<YieldColumn>,
    pub aliases_available: HashMap<String, AliasType>,
    pub aliases_generated: HashMap<String, AliasType>,
    pub distinct: bool,
    pub has_agg: bool,
    pub group_keys: Vec<Expression>,
    pub group_items: Vec<Expression>,
    pub need_gen_project: bool,
    pub agg_output_column_names: Vec<String>,
    pub proj_output_column_names: Vec<String>,
    pub proj_cols: Vec<YieldColumn>,
    pub paths: Vec<Path>,
    pub query_parts: Vec<QueryPart>,
    pub errors: Vec<ValidationError>,
    pub filter_condition: Option<Expression>,
    pub skip: Option<usize>,
    pub limit: Option<usize>,
}

/// 分页上下文
#[derive(Debug, Clone)]
pub struct PaginationContext {
    pub skip: i64,
    pub limit: i64,
}

/// 排序子句上下文
#[derive(Debug, Clone)]
pub struct OrderByClauseContext {
    pub indexed_order_factors: Vec<(usize, OrderDirection)>,
}

// 为各上下文类型实现 ExpressionValidationContext trait
impl ExpressionValidationContext for MatchClauseContext {
    fn get_aliases(&self) -> &HashMap<String, AliasType> {
        &self.aliases_available
    }

    fn get_variable_types(&self) -> Option<&HashMap<String, DataType>> {
        None
    }
}

impl ExpressionValidationContext for WhereClauseContext {
    fn get_aliases(&self) -> &HashMap<String, AliasType> {
        &self.aliases_available
    }

    fn get_variable_types(&self) -> Option<&HashMap<String, DataType>> {
        None
    }
}

impl ExpressionValidationContext for ReturnClauseContext {
    fn get_aliases(&self) -> &HashMap<String, AliasType> {
        &self.aliases_available
    }

    fn get_variable_types(&self) -> Option<&HashMap<String, DataType>> {
        None
    }
}

impl ExpressionValidationContext for WithClauseContext {
    fn get_aliases(&self) -> &HashMap<String, AliasType> {
        &self.aliases_available
    }

    fn get_variable_types(&self) -> Option<&HashMap<String, DataType>> {
        None
    }
}

impl ExpressionValidationContext for UnwindClauseContext {
    fn get_aliases(&self) -> &HashMap<String, AliasType> {
        &self.aliases_available
    }

    fn get_variable_types(&self) -> Option<&HashMap<String, DataType>> {
        None
    }
}

impl ExpressionValidationContext for YieldClauseContext {
    fn get_aliases(&self) -> &HashMap<String, AliasType> {
        &self.aliases_available
    }

    fn get_variable_types(&self) -> Option<&HashMap<String, DataType>> {
        None
    }
}
