//! 别名相关数据结构

use crate::query::validator::{MatchClauseContext, Path};
use std::collections::HashMap;

/// Cypher查询中的别名类型
#[derive(Debug, Clone, PartialEq)]
pub enum AliasType {
    Node,
    Edge,
    NodeList,
    EdgeList,
    Path,
    Variable,
    Runtime,
    CTE,
    Expression,
}

/// 边界子句类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoundaryClauseType {
    With,
    Unwind,
}

/// 边界子句的公共数据
#[derive(Debug, Clone)]
pub struct BoundaryClauseContext {
    pub clause_type: BoundaryClauseType,
    pub aliases_available: HashMap<String, AliasType>,
    pub aliases_generated: HashMap<String, AliasType>,
    pub query_parts: Vec<QueryPart>,
    pub errors: Vec<ValidationError>,
}

/// 查询部分结构
#[derive(Debug, Clone)]
pub struct QueryPart {
    pub matchs: Vec<MatchClauseContext>,
    pub boundary: Option<BoundaryClauseContext>,
    pub aliases_available: HashMap<String, AliasType>,
    pub aliases_generated: HashMap<String, AliasType>,
    pub paths: Vec<Path>,
}
