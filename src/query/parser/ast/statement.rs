//! Statement AST definitions for the query parser

use super::{expression::*, pattern::*, types::*};

#[derive(Debug, Clone, PartialEq)]
pub struct Query {
    pub stmt: Statement,
    pub semicolon: bool, // Whether the query ends with a semicolon
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    CreateNode(CreateNodeStatement),
    CreateEdge(CreateEdgeStatement),
    Match(MatchStatement),
    Delete(DeleteStatement),
    Update(UpdateStatement),
    Use(UseStatement),
    Show(ShowStatement),
    Explain(ExplainStatement),
    Go(GoStatement),
    FetchVertices(FetchVerticesStatement),
    FetchEdges(FetchEdgesStatement),
    Lookup(LookupStatement),
    FindPath(FindPathStatement),
    // Add more statement types as needed
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreateNodeStatement {
    pub if_not_exists: bool,
    pub tags: Vec<TagIdentifier>,
    pub properties: Vec<Property>,
    pub yield_clause: Option<YieldClause>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreateEdgeStatement {
    pub if_not_exists: bool,
    pub edge_type: Identifier,
    pub src: Expression,
    pub dst: Expression,
    pub ranking: Option<Expression>,
    pub properties: Vec<Property>,
    pub yield_clause: Option<YieldClause>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchStatement {
    pub clauses: Vec<MatchClause>,
    pub return_clause: Option<ReturnClause>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MatchClause {
    Match(MatchClauseDetail),
    Where(WhereClause),
    Return(ReturnClause),
    OrderBy(OrderByClause),
    Limit(LimitClause),
    Skip(SkipClause),
    Unwind(UnwindClause),
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchClauseDetail {
    pub patterns: Vec<MatchPath>,
    pub where_clause: Option<WhereClause>,
    pub with_clause: Option<WithClause>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WhereClause {
    pub condition: Expression,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReturnClause {
    pub distinct: bool,
    pub items: Vec<ReturnItem>,
    pub order_by: Option<OrderByClause>,
    pub limit: Option<LimitClause>,
    pub skip: Option<SkipClause>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WithClause {
    pub items: Vec<ReturnItem>,
    pub where_clause: Option<WhereClause>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OrderByClause {
    pub items: Vec<OrderByItem>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OrderByItem {
    pub expr: Expression,
    pub order: OrderType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OrderType {
    Asc,
    Desc,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitClause {
    pub expr: Expression,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SkipClause {
    pub expr: Expression,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnwindClause {
    pub expr: Expression,
    pub alias: Identifier,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReturnItem {
    Asterisk,
    Expression(Expression, Option<Identifier>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeleteStatement {
    pub delete_vertices: bool, // true for DELETE VERTEX, false for DELETE EDGE
    pub vertex_exprs: Vec<Expression>,
    pub edge_exprs: Option<EdgeDeleteCondition>,
    pub where_clause: Option<WhereClause>,
    pub yield_clause: Option<YieldClause>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EdgeDeleteCondition {
    pub src: Option<Expression>,
    pub dst: Option<Expression>,
    pub rank: Option<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UpdateStatement {
    pub update_vertices: bool, // true for UPDATE VERTEX, false for UPDATE EDGE
    pub vertex_ref: Option<Expression>,
    pub edge_ref: Option<EdgeUpdateRef>,
    pub update_items: Vec<Assignment>,
    pub condition: Option<WhereClause>,
    pub yield_clause: Option<YieldClause>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EdgeUpdateRef {
    pub src: Expression,
    pub dst: Expression,
    pub rank: Option<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UseStatement {
    pub space: Identifier,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ShowStatement {
    ShowSpaces,
    ShowTags,
    ShowEdges,
    ShowTagIndex(Identifier),
    ShowEdgeIndex(Identifier),
    ShowUsers,
    ShowRoles(Option<Identifier>),
    ShowHosts,
    ShowParts(Option<Identifier>),
    ShowCharset,
    ShowCollation,
    ShowConfigs(Option<ConfigModule>),
    ShowStats,
    ShowServiceLogs(Option<LogType>),
    ShowSessions(Option<SessionOptions>),
    ShowQueries,
    ShowMutation(Identifier),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConfigModule {
    Graph,
    Meta,
    Storage,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LogType {
    Run,
    Slow,
    Audit(Option<String>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct SessionOptions {
    pub session_id: Option<i64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExplainStatement {
    pub stmt: Box<crate::query::parser::ast::Statement>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GoStatement {
    pub steps: GoSteps,
    pub over: OverClause,
    pub from: Vec<Expression>,
    pub where_clause: Option<Expression>,
    pub yield_clause: YieldClause,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GoSteps {
    Exact(Expression),
    Range(Option<Expression>, Option<Expression>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverClause {
    pub edge_types: Vec<Identifier>,
    pub direction: EdgeDirection,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FetchVerticesStatement {
    pub from: Vec<Expression>,
    pub properties: Vec<PropertyRef>,
    pub where_clause: Option<Expression>,
    pub yield_clause: Option<YieldClause>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FetchEdgesStatement {
    pub from: Vec<Expression>,
    pub properties: Vec<PropertyRef>,
    pub where_clause: Option<Expression>,
    pub yield_clause: Option<YieldClause>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LookupStatement {
    pub schema_name: Identifier,
    pub where_clause: Option<Expression>,
    pub yield_clause: YieldClause,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PathType {
    Shortest,
    AllShortest,
    Single,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FindPathStatement {
    pub path_type: PathType,
    pub src: Expression,
    pub dst: Expression,
    pub min_hop: Option<u32>,
    pub max_hop: Option<u32>,
    pub edge_types: Vec<Identifier>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct YieldClause {
    pub items: Vec<YieldExpression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct YieldExpression {
    pub expr: Expression,
    pub alias: Option<Identifier>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_statement_structures() {
        // Test creating a simple match statement
        let match_stmt = MatchStatement {
            clauses: vec![MatchClause::Match(MatchClauseDetail {
                patterns: vec![MatchPath {
                    path: vec![MatchPathSegment::Node(MatchNode {
                        identifier: Some("n".to_string()),
                        labels: vec![Label {
                            name: "Person".to_string(),
                        }],
                        properties: None,
                        predicates: vec![],
                    })],
                }],
                where_clause: None,
                with_clause: None,
            })],
            return_clause: Some(ReturnClause {
                distinct: false,
                items: vec![ReturnItem::Expression(
                    Expression::PropertyAccess(
                        Box::new(Expression::Variable("n".to_string())),
                        "name".to_string(),
                    ),
                    None,
                )],
                order_by: None,
                limit: None,
                skip: None,
            }),
        };

        assert!(matches!(match_stmt.clauses[0], MatchClause::Match(_)));
    }
}
