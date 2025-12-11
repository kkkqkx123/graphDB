//! Abstract Syntax Tree (AST) definitions for the query parser
//!
//! This module defines the AST nodes that represent parsed queries.

use crate::core::Value;
use std::collections::HashMap;

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
    pub stmt: Box<Statement>,
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

// MatchPath represents a pattern in a MATCH clause
#[derive(Debug, Clone, PartialEq)]
pub struct MatchPath {
    pub path: Vec<MatchPathSegment>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MatchPathSegment {
    Node(MatchNode),
    Edge(MatchEdge),
    // For more complex path patterns
    PathPattern(MatchPathPattern),
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchNode {
    pub identifier: Option<Identifier>,
    pub labels: Vec<Label>,
    pub properties: Option<Expression>,
    pub predicates: Vec<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchEdge {
    pub direction: EdgeDirection,
    pub identifier: Option<Identifier>,
    pub types: Vec<Identifier>,
    pub relationship: Option<Identifier>,
    pub properties: Option<Expression>,
    pub predicates: Vec<Expression>,
    pub range: Option<StepRange>, // For variable length paths
}

#[derive(Debug, Clone, PartialEq)]
pub enum EdgeDirection {
    Outbound,      // ->
    Inbound,       // <-
    Bidirectional, // -
}

#[derive(Debug, Clone, PartialEq)]
pub struct StepRange {
    pub min: Option<u32>,
    pub max: Option<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchPathPattern {
    // Complex path patterns can be nested structures
    pub path: Vec<MatchPathSegment>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Label {
    pub name: Identifier,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TagIdentifier {
    pub name: Identifier,
    pub properties: Option<HashMap<String, Expression>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Property {
    pub name: Identifier,
    pub value: Expression,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Assignment {
    pub prop: PropertyRef,
    pub value: Expression,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PropertyRef {
    Prop(Identifier, Identifier), // tagName.propName
    InlineProp(Identifier),       // propName without tagName
}

// Expression AST nodes
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Constant(Value),
    Variable(Identifier),
    FunctionCall(FunctionCall),
    PropertyAccess(Box<Expression>, Identifier),
    AttributeAccess(Box<Expression>, Identifier), // e.g., tagName.propertyName
    Arithmetic(Box<Expression>, ArithmeticOp, Box<Expression>),
    Logical(Box<Expression>, LogicalOp, Box<Expression>),
    Relational(Box<Expression>, RelationalOp, Box<Expression>),
    Unary(UnaryOp, Box<Expression>),
    List(Vec<Expression>),
    Map(Vec<(Identifier, Expression)>),
    Subscript(Box<Expression>, Box<Expression>), // expr[index] or expr.key
    Case(CaseExpression),
    InList(Box<Expression>, Vec<Expression>),
    NotInList(Box<Expression>, Vec<Expression>),
    Contains(Box<Expression>, Box<Expression>),
    StartsWith(Box<Expression>, Box<Expression>),
    EndsWith(Box<Expression>, Box<Expression>),
    IsNull(Box<Expression>),
    IsNotNull(Box<Expression>),
    All(Box<Expression>, Box<Expression>), // For list predicates
    Single(Box<Expression>, Box<Expression>),
    Any(Box<Expression>, Box<Expression>),
    None(Box<Expression>, Box<Expression>),
    // Add more expression types as needed
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArithmeticOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LogicalOp {
    And,
    Or,
    Xor,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RelationalOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Regex,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Not,
    Plus,
    Minus,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionCall {
    pub name: Identifier,
    pub args: Vec<Expression>,
    pub distinct: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CaseExpression {
    pub match_expr: Option<Box<Expression>>,
    pub when_then_pairs: Vec<(Expression, Expression)>,
    pub default: Option<Box<Expression>>,
}

// Type aliases for common structures
pub type Identifier = String;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ast_structures() {
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

        let stmt = Statement::Match(match_stmt);
        assert!(matches!(stmt, Statement::Match(_)));
    }
}
