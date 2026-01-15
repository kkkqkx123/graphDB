//! 语句 AST 定义 (v2)
//!
//! 基于枚举的简化语句定义，支持所有图数据库操作语句。

use super::expr::{Expr, ExprUtils};
use super::pattern::*;
use super::types::*;
use crate::core::Value;

/// 语句枚举 - 所有图数据库操作语句
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Query(QueryStmt),
    Create(CreateStmt),
    Match(MatchStmt),
    Delete(DeleteStmt),
    Update(UpdateStmt),
    Go(GoStmt),
    Fetch(FetchStmt),
    Use(UseStmt),
    Show(ShowStmt),
    Explain(ExplainStmt),
    Lookup(LookupStmt),
    Subgraph(SubgraphStmt),
    FindPath(FindPathStmt),
    Insert(InsertStmt),
    Merge(MergeStmt),
    Unwind(UnwindStmt),
    Return(ReturnStmt),
    With(WithStmt),
    Set(SetStmt),
    Remove(RemoveStmt),
    Pipe(PipeStmt),
}

impl Stmt {
    /// 获取语句的位置信息
    pub fn span(&self) -> Span {
        match self {
            Stmt::Query(s) => s.span,
            Stmt::Create(s) => s.span,
            Stmt::Match(s) => s.span,
            Stmt::Delete(s) => s.span,
            Stmt::Update(s) => s.span,
            Stmt::Go(s) => s.span,
            Stmt::Fetch(s) => s.span,
            Stmt::Use(s) => s.span,
            Stmt::Show(s) => s.span,
            Stmt::Explain(s) => s.span,
            Stmt::Lookup(s) => s.span,
            Stmt::Subgraph(s) => s.span,
            Stmt::FindPath(s) => s.span,
            Stmt::Insert(s) => s.span,
            Stmt::Merge(s) => s.span,
            Stmt::Unwind(s) => s.span,
            Stmt::Return(s) => s.span,
            Stmt::With(s) => s.span,
            Stmt::Set(s) => s.span,
            Stmt::Remove(s) => s.span,
            Stmt::Pipe(s) => s.span,
        }
    }
}

/// 查询语句
#[derive(Debug, Clone, PartialEq)]
pub struct QueryStmt {
    pub span: Span,
    pub statements: Vec<Stmt>,
}

impl QueryStmt {
    pub fn new(statements: Vec<Stmt>, span: Span) -> Self {
        Self { span, statements }
    }
}

/// CREATE 语句
#[derive(Debug, Clone, PartialEq)]
pub struct CreateStmt {
    pub span: Span,
    pub target: CreateTarget,
}

/// 创建目标
#[derive(Debug, Clone, PartialEq)]
pub enum CreateTarget {
    Node {
        variable: Option<String>,
        labels: Vec<String>,
        properties: Option<Expr>,
    },
    Edge {
        variable: Option<String>,
        edge_type: String,
        src: Expr,
        dst: Expr,
        properties: Option<Expr>,
        direction: EdgeDirection,
    },
    Tag {
        name: String,
        properties: Vec<PropertyDef>,
    },
    EdgeType {
        name: String,
        properties: Vec<PropertyDef>,
    },
    Space {
        name: String,
    },
    Index {
        name: String,
        on: String,
        properties: Vec<String>,
    },
}

/// 属性定义
#[derive(Debug, Clone, PartialEq)]
pub struct PropertyDef {
    pub name: String,
    pub data_type: DataType,
    pub nullable: bool,
    pub default: Option<Value>,
}

/// MATCH 语句
#[derive(Debug, Clone, PartialEq)]
pub struct MatchStmt {
    pub span: Span,
    pub patterns: Vec<Pattern>,
    pub where_clause: Option<Expr>,
    pub return_clause: Option<ReturnClause>,
    pub order_by: Option<OrderByClause>,
    pub limit: Option<usize>,
    pub skip: Option<usize>,
}

/// 返回子句
#[derive(Debug, Clone, PartialEq)]
pub struct ReturnClause {
    pub span: Span,
    pub items: Vec<ReturnItem>,
    pub distinct: bool,
}

/// 返回项
#[derive(Debug, Clone, PartialEq)]
pub enum ReturnItem {
    All,
    Expression { expr: Expr, alias: Option<String> },
}

/// 排序子句
#[derive(Debug, Clone, PartialEq)]
pub struct OrderByClause {
    pub span: Span,
    pub items: Vec<OrderByItem>,
}

/// 排序项
#[derive(Debug, Clone, PartialEq)]
pub struct OrderByItem {
    pub expr: Expr,
    pub direction: OrderDirection,
}

/// DELETE 语句
#[derive(Debug, Clone, PartialEq)]
pub struct DeleteStmt {
    pub span: Span,
    pub target: DeleteTarget,
    pub where_clause: Option<Expr>,
}

/// 删除目标
#[derive(Debug, Clone, PartialEq)]
pub enum DeleteTarget {
    Vertices(Vec<Expr>),
    Edges {
        src: Expr,
        dst: Expr,
        edge_type: Option<String>,
        rank: Option<Expr>,
    },
    Tag(String),
    Index(String),
}

/// UPDATE 语句
#[derive(Debug, Clone, PartialEq)]
pub struct UpdateStmt {
    pub span: Span,
    pub target: UpdateTarget,
    pub set_clause: SetClause,
    pub where_clause: Option<Expr>,
}

/// 更新目标
#[derive(Debug, Clone, PartialEq)]
pub enum UpdateTarget {
    Vertex(Expr),
    Edge {
        src: Expr,
        dst: Expr,
        edge_type: Option<String>,
        rank: Option<Expr>,
    },
    Tag(String),
}

/// SET 子句
#[derive(Debug, Clone, PartialEq)]
pub struct SetClause {
    pub span: Span,
    pub assignments: Vec<Assignment>,
}

/// 赋值操作
#[derive(Debug, Clone, PartialEq)]
pub struct Assignment {
    pub property: String,
    pub value: Expr,
}

/// GO 语句
#[derive(Debug, Clone, PartialEq)]
pub struct GoStmt {
    pub span: Span,
    pub steps: Steps,
    pub from: FromClause,
    pub over: Option<OverClause>,
    pub where_clause: Option<Expr>,
    pub yield_clause: Option<YieldClause>,
}

/// 步数定义
#[derive(Debug, Clone, PartialEq)]
pub enum Steps {
    Fixed(usize),
    Range { min: usize, max: usize },
    Variable(String),
}

/// FROM 子句
#[derive(Debug, Clone, PartialEq)]
pub struct FromClause {
    pub span: Span,
    pub vertices: Vec<Expr>,
}

/// OVER 子句
#[derive(Debug, Clone, PartialEq)]
pub struct OverClause {
    pub span: Span,
    pub edge_types: Vec<String>,
    pub direction: EdgeDirection,
}

/// YIELD 子句
#[derive(Debug, Clone, PartialEq)]
pub struct YieldClause {
    pub span: Span,
    pub items: Vec<YieldItem>,
}

/// YIELD 项
#[derive(Debug, Clone, PartialEq)]
pub struct YieldItem {
    pub expr: Expr,
    pub alias: Option<String>,
}

/// FETCH 语句
#[derive(Debug, Clone, PartialEq)]
pub struct FetchStmt {
    pub span: Span,
    pub target: FetchTarget,
}

/// 获取目标
#[derive(Debug, Clone, PartialEq)]
pub enum FetchTarget {
    Vertices {
        ids: Vec<Expr>,
        properties: Option<Vec<String>>,
    },
    Edges {
        src: Expr,
        dst: Expr,
        edge_type: String,
        rank: Option<Expr>,
        properties: Option<Vec<String>>,
    },
}

/// USE 语句
#[derive(Debug, Clone, PartialEq)]
pub struct UseStmt {
    pub span: Span,
    pub space: String,
}

/// SHOW 语句
#[derive(Debug, Clone, PartialEq)]
pub struct ShowStmt {
    pub span: Span,
    pub target: ShowTarget,
}

/// 显示目标
#[derive(Debug, Clone, PartialEq)]
pub enum ShowTarget {
    Spaces,
    Tags,
    Edges,
    Tag(String),
    Edge(String),
    Indexes,
    Index(String),
    Users,
    Roles,
}

/// EXPLAIN 语句
#[derive(Debug, Clone, PartialEq)]
pub struct ExplainStmt {
    pub span: Span,
    pub statement: Box<Stmt>,
}

/// LOOKUP 语句（新增）
#[derive(Debug, Clone, PartialEq)]
pub struct LookupStmt {
    pub span: Span,
    pub target: LookupTarget,
    pub where_clause: Option<Expr>,
    pub yield_clause: Option<YieldClause>,
}

/// LOOKUP 目标
#[derive(Debug, Clone, PartialEq)]
pub enum LookupTarget {
    Tag(String),
    Edge(String),
}

/// SUBGRAPH 语句（新增）
#[derive(Debug, Clone, PartialEq)]
pub struct SubgraphStmt {
    pub span: Span,
    pub steps: Steps,
    pub from: FromClause,
    pub over: Option<OverClause>,
    pub where_clause: Option<Expr>,
    pub yield_clause: Option<YieldClause>,
}

/// FIND PATH 语句（新增）
#[derive(Debug, Clone, PartialEq)]
pub struct FindPathStmt {
    pub span: Span,
    pub from: FromClause,
    pub to: Expr,
    pub over: Option<OverClause>,
    pub where_clause: Option<Expr>,
    pub shortest: bool,
    pub yield_clause: Option<YieldClause>,
}

/// INSERT 语句
#[derive(Debug, Clone, PartialEq)]
pub struct InsertStmt {
    pub span: Span,
    pub target: InsertTarget,
}

/// INSERT 目标
#[derive(Debug, Clone, PartialEq)]
pub enum InsertTarget {
    Vertices { ids: Vec<Expr> },
    Edge { src: Expr, dst: Expr },
}

/// MERGE 语句
#[derive(Debug, Clone, PartialEq)]
pub struct MergeStmt {
    pub span: Span,
    pub pattern: Pattern,
}

/// UNWIND 语句
#[derive(Debug, Clone, PartialEq)]
pub struct UnwindStmt {
    pub span: Span,
    pub expression: Expr,
    pub variable: String,
}

/// RETURN 语句
#[derive(Debug, Clone, PartialEq)]
pub struct ReturnStmt {
    pub span: Span,
    pub items: Vec<ReturnItem>,
    pub distinct: bool,
}

/// WITH 语句
#[derive(Debug, Clone, PartialEq)]
pub struct WithStmt {
    pub span: Span,
    pub items: Vec<ReturnItem>,
    pub where_clause: Option<Expr>,
}

/// SET 语句
#[derive(Debug, Clone, PartialEq)]
pub struct SetStmt {
    pub span: Span,
    pub assignments: Vec<Assignment>,
}

/// REMOVE 语句
#[derive(Debug, Clone, PartialEq)]
pub struct RemoveStmt {
    pub span: Span,
    pub items: Vec<Expr>,
}

/// PIPE 语句
#[derive(Debug, Clone, PartialEq)]
pub struct PipeStmt {
    pub span: Span,
    pub expression: Expr,
}

// 语句工具函数
pub struct StmtUtils;

impl StmtUtils {
    /// 获取语句中使用的所有变量
    pub fn find_variables(stmt: &Stmt) -> Vec<String> {
        let mut variables = Vec::new();
        Self::find_variables_recursive(stmt, &mut variables);
        variables
    }

    fn find_variables_recursive(stmt: &Stmt, variables: &mut Vec<String>) {
        match stmt {
            Stmt::Match(s) => {
                for pattern in &s.patterns {
                    variables.extend(PatternUtils::find_variables(pattern));
                }
                if let Some(ref where_clause) = s.where_clause {
                    variables.extend(ExprUtils::find_variables(where_clause));
                }
            }
            Stmt::Create(s) => match &s.target {
                CreateTarget::Node { properties, .. } => {
                    if let Some(props) = properties {
                        variables.extend(ExprUtils::find_variables(props));
                    }
                }
                CreateTarget::Edge {
                    src,
                    dst,
                    properties,
                    ..
                } => {
                    variables.extend(ExprUtils::find_variables(src));
                    variables.extend(ExprUtils::find_variables(dst));
                    if let Some(props) = properties {
                        variables.extend(ExprUtils::find_variables(props));
                    }
                }
                _ => {}
            },
            Stmt::Delete(s) => {
                match &s.target {
                    DeleteTarget::Vertices(vertices) => {
                        for vertex in vertices {
                            variables.extend(ExprUtils::find_variables(vertex));
                        }
                    }
                    DeleteTarget::Edges { src, dst, rank, .. } => {
                        variables.extend(ExprUtils::find_variables(src));
                        variables.extend(ExprUtils::find_variables(dst));
                        if let Some(ref rank) = rank {
                            variables.extend(ExprUtils::find_variables(rank));
                        }
                    }
                    _ => {}
                }
                if let Some(ref where_clause) = s.where_clause {
                    variables.extend(ExprUtils::find_variables(where_clause));
                }
            }
            Stmt::Update(s) => {
                match &s.target {
                    UpdateTarget::Vertex(vertex) => {
                        variables.extend(ExprUtils::find_variables(vertex));
                    }
                    UpdateTarget::Edge { src, dst, rank, .. } => {
                        variables.extend(ExprUtils::find_variables(src));
                        variables.extend(ExprUtils::find_variables(dst));
                        if let Some(ref rank) = rank {
                            variables.extend(ExprUtils::find_variables(rank));
                        }
                    }
                    _ => {}
                }
                for assignment in &s.set_clause.assignments {
                    variables.extend(ExprUtils::find_variables(&assignment.value));
                }
                if let Some(ref where_clause) = s.where_clause {
                    variables.extend(ExprUtils::find_variables(where_clause));
                }
            }
            Stmt::Go(s) => {
                for vertex in &s.from.vertices {
                    variables.extend(ExprUtils::find_variables(vertex));
                }
                if let Some(ref where_clause) = s.where_clause {
                    variables.extend(ExprUtils::find_variables(where_clause));
                }
            }
            Stmt::Fetch(s) => match &s.target {
                FetchTarget::Vertices { ids, .. } => {
                    for id in ids {
                        variables.extend(ExprUtils::find_variables(id));
                    }
                }
                FetchTarget::Edges { src, dst, rank, .. } => {
                    variables.extend(ExprUtils::find_variables(src));
                    variables.extend(ExprUtils::find_variables(dst));
                    if let Some(ref rank) = rank {
                        variables.extend(ExprUtils::find_variables(rank));
                    }
                }
            },
            Stmt::Lookup(s) => {
                if let Some(ref where_clause) = s.where_clause {
                    variables.extend(ExprUtils::find_variables(where_clause));
                }
            }
            Stmt::Subgraph(s) => {
                for vertex in &s.from.vertices {
                    variables.extend(ExprUtils::find_variables(vertex));
                }
                if let Some(ref where_clause) = s.where_clause {
                    variables.extend(ExprUtils::find_variables(where_clause));
                }
            }
            Stmt::FindPath(s) => {
                for vertex in &s.from.vertices {
                    variables.extend(ExprUtils::find_variables(vertex));
                }
                variables.extend(ExprUtils::find_variables(&s.to));
                if let Some(ref where_clause) = s.where_clause {
                    variables.extend(ExprUtils::find_variables(where_clause));
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::query::parser::ast::VariableExpr;

    use super::*;

    #[test]
    fn test_create_stmt() {
        let stmt = Stmt::Create(CreateStmt {
            span: Span::default(),
            target: CreateTarget::Node {
                variable: Some("n".to_string()),
                labels: vec!["Person".to_string()],
                properties: None,
            },
        });

        assert!(matches!(stmt, Stmt::Create(_)));
    }

    #[test]
    fn test_match_stmt() {
        let stmt = Stmt::Match(MatchStmt {
            span: Span::default(),
            patterns: vec![],
            where_clause: None,
            return_clause: None,
            order_by: None,
            limit: None,
            skip: None,
        });

        assert!(matches!(stmt, Stmt::Match(_)));
    }

    #[test]
    fn test_lookup_stmt() {
        let stmt = Stmt::Lookup(LookupStmt {
            span: Span::default(),
            target: LookupTarget::Tag("Person".to_string()),
            where_clause: None,
            yield_clause: None,
        });

        assert!(matches!(stmt, Stmt::Lookup(_)));
    }

    #[test]
    fn test_subgraph_stmt() {
        let stmt = Stmt::Subgraph(SubgraphStmt {
            span: Span::default(),
            steps: Steps::Fixed(1),
            from: FromClause {
                span: Span::default(),
                vertices: vec![],
            },
            over: None,
            where_clause: None,
            yield_clause: None,
        });

        assert!(matches!(stmt, Stmt::Subgraph(_)));
    }

    #[test]
    fn test_find_path_stmt() {
        let stmt = Stmt::FindPath(FindPathStmt {
            span: Span::default(),
            from: FromClause {
                span: Span::default(),
                vertices: vec![],
            },
            to: Expr::Variable(VariableExpr::new("target".to_string(), Span::default())),
            over: None,
            where_clause: None,
            shortest: true,
            yield_clause: None,
        });

        assert!(matches!(stmt, Stmt::FindPath(_)));
    }
}
