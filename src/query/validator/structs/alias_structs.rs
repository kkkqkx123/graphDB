//! 别名相关数据结构

use crate::query::validator::{MatchClauseContext, Path, UnwindClauseContext, WithClauseContext};
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

/// 边界子句上下文（With或Unwind）
#[derive(Debug, Clone)]
pub enum BoundaryClauseContext {
    With(WithClauseContext),
    Unwind(UnwindClauseContext),
}
