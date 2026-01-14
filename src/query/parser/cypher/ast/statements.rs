//! Cypher语句类型定义

use crate::query::parser::cypher::ast::clauses::*;

/// Cypher语句类型
#[derive(Debug, Clone)]
pub enum CypherStatement {
    // Cypher 语句
    Match(MatchClause),
    Where(WhereClause),
    Return(ReturnClause),
    Create(CreateClause),
    Delete(DeleteClause),
    Set(SetClause),
    Remove(RemoveClause),
    Merge(MergeClause),
    With(WithClause),
    Unwind(UnwindClause),
    Call(CallClause),
    
    // NGQL 语句
    Go(GoClause),
    Lookup(LookupClause),
    FetchVertices(FetchVerticesClause),
    FetchEdges(FetchEdgesClause),
    FindPath(FindPathClause),
    Yield(YieldClause),
    
    // 管道操作
    Pipe(Box<CypherStatement>, Box<CypherStatement>),
    
    // 集合操作
    Union(Box<CypherStatement>, Box<CypherStatement>, bool),
    Intersect(Box<CypherStatement>, Box<CypherStatement>),
    Minus(Box<CypherStatement>, Box<CypherStatement>),
    
    // 管理语句
    CreateSpace(CreateSpaceClause),
    DropSpace(DropSpaceClause),
    CreateTag(CreateTagClause),
    DropTag(DropTagClause),
    CreateEdge(CreateEdgeClause),
    DropEdge(DropEdgeClause),
    
    // 解释语句
    Explain(Box<CypherStatement>),
    Profile(Box<CypherStatement>),
    
    // 复合查询
    Query(QueryClause),
}

/// 复合查询语句
#[derive(Debug, Clone)]
pub struct QueryClause {
    pub match_clause: Option<MatchClause>,
    pub where_clause: Option<WhereClause>,
    pub return_clause: Option<ReturnClause>,
    pub with_clause: Option<WithClause>,
}

/// GO 子句
#[derive(Debug, Clone)]
pub struct GoClause {
    pub step_clause: StepClause,
    pub from_clause: FromClause,
    pub over_clause: OverClause,
    pub where_clause: Option<WhereClause>,
    pub truncate_clause: Option<TruncateClause>,
    pub yield_clause: Option<YieldClause>,
}

/// 步骤子句
#[derive(Debug, Clone)]
pub struct StepClause {
    pub steps: u32,
    pub upto: Option<u32>,
}

/// FROM 子句
#[derive(Debug, Clone)]
pub enum FromClause {
    VertexList(Vec<Expression>),
    Variable(String),
}

/// OVER 子句
#[derive(Debug, Clone)]
pub struct OverClause {
    pub edges: Vec<String>,
    pub direction: EdgeDirection,
    pub is_over_all: bool,
}

/// 边方向
#[derive(Debug, Clone, PartialEq)]
pub enum EdgeDirection {
    Out,
    In,
    Both,
}

/// Truncate 子句
#[derive(Debug, Clone)]
pub struct TruncateClause {
    pub expression: Expression,
    pub is_sample: bool,
}

/// LOOKUP 子句
#[derive(Debug, Clone)]
pub struct LookupClause {
    pub from: String,
    pub where_clause: WhereClause,
    pub yield_clause: Option<YieldClause>,
}

/// FETCH VERTICES 子句
#[derive(Debug, Clone)]
pub struct FetchVerticesClause {
    pub vertex_ids: Vec<Expression>,
    pub yield_clause: Option<YieldClause>,
}

/// FETCH EDGES 子句
#[derive(Debug, Clone)]
pub struct FetchEdgesClause {
    pub edge_keys: Vec<EdgeKey>,
    pub yield_clause: Option<YieldClause>,
}

/// 边键
#[derive(Debug, Clone)]
pub struct EdgeKey {
    pub src_id: Expression,
    pub edge_type: String,
    pub ranking: Option<Expression>,
    pub dst_id: Expression,
}

/// FIND PATH 子句
#[derive(Debug, Clone)]
pub struct FindPathClause {
    pub path_type: PathType,
    pub from: Expression,
    pub to: Expression,
    pub over: Vec<String>,
    pub where_clause: Option<WhereClause>,
    pub yield_clause: Option<YieldClause>,
}

/// 路径类型
#[derive(Debug, Clone, PartialEq)]
pub enum PathType {
    Shortest,
    AllShortest,
    AllPaths,
}

/// YIELD 子句
#[derive(Debug, Clone)]
pub struct YieldClause {
    pub columns: Vec<YieldColumn>,
    pub distinct: bool,
}

/// YIELD 列
#[derive(Debug, Clone)]
pub struct YieldColumn {
    pub expression: Expression,
    pub alias: Option<String>,
}

/// CREATE SPACE 子句
#[derive(Debug, Clone)]
pub struct CreateSpaceClause {
    pub space_name: String,
    pub if_not_exists: bool,
    pub options: Vec<SpaceOption>,
}

/// SPACE 选项
#[derive(Debug, Clone)]
pub enum SpaceOption {
    PartitionNum(u32),
    ReplicaFactor(u32),
    VidType(String),
}

/// DROP SPACE 子句
#[derive(Debug, Clone)]
pub struct DropSpaceClause {
    pub space_name: String,
    pub if_exists: bool,
}

/// CREATE TAG 子句
#[derive(Debug, Clone)]
pub struct CreateTagClause {
    pub tag_name: String,
    pub if_not_exists: bool,
    pub properties: Vec<PropertyDefinition>,
}

/// 属性定义
#[derive(Debug, Clone)]
pub struct PropertyDefinition {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub default: Option<Expression>,
}

/// DROP TAG 子句
#[derive(Debug, Clone)]
pub struct DropTagClause {
    pub tag_name: String,
    pub if_exists: bool,
}

/// CREATE EDGE 子句
#[derive(Debug, Clone)]
pub struct CreateEdgeClause {
    pub edge_name: String,
    pub if_not_exists: bool,
    pub properties: Vec<PropertyDefinition>,
}

/// DROP EDGE 子句
#[derive(Debug, Clone)]
pub struct DropEdgeClause {
    pub edge_name: String,
    pub if_exists: bool,
}

impl CypherStatement {
    /// 获取语句类型
    pub fn statement_type(&self) -> &str {
        match self {
            CypherStatement::Match(_) => "MATCH",
            CypherStatement::Where(_) => "WHERE",
            CypherStatement::Return(_) => "RETURN",
            CypherStatement::Create(_) => "CREATE",
            CypherStatement::Delete(_) => "DELETE",
            CypherStatement::Set(_) => "SET",
            CypherStatement::Remove(_) => "REMOVE",
            CypherStatement::Merge(_) => "MERGE",
            CypherStatement::With(_) => "WITH",
            CypherStatement::Unwind(_) => "UNWIND",
            CypherStatement::Call(_) => "CALL",
            CypherStatement::Go(_) => "GO",
            CypherStatement::Lookup(_) => "LOOKUP",
            CypherStatement::FetchVertices(_) => "FETCH VERTICES",
            CypherStatement::FetchEdges(_) => "FETCH EDGES",
            CypherStatement::FindPath(_) => "FIND PATH",
            CypherStatement::Yield(_) => "YIELD",
            CypherStatement::Pipe(_, _) => "PIPE",
            CypherStatement::Union(_, _, _) => "UNION",
            CypherStatement::Intersect(_, _) => "INTERSECT",
            CypherStatement::Minus(_, _) => "MINUS",
            CypherStatement::CreateSpace(_) => "CREATE SPACE",
            CypherStatement::DropSpace(_) => "DROP SPACE",
            CypherStatement::CreateTag(_) => "CREATE TAG",
            CypherStatement::DropTag(_) => "DROP TAG",
            CypherStatement::CreateEdge(_) => "CREATE EDGE",
            CypherStatement::DropEdge(_) => "DROP EDGE",
            CypherStatement::Explain(_) => "EXPLAIN",
            CypherStatement::Profile(_) => "PROFILE",
            CypherStatement::Query(_) => "QUERY",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cypher_statement_type() {
        let match_stmt = CypherStatement::Match(MatchClause {
            patterns: Vec::new(),
            where_clause: None,
            optional: false,
        });

        assert_eq!(match_stmt.statement_type(), "MATCH");

        let return_stmt = CypherStatement::Return(ReturnClause {
            return_items: Vec::new(),
            distinct: false,
            order_by: None,
            skip: None,
            limit: None,
        });

        assert_eq!(return_stmt.statement_type(), "RETURN");
    }
}
