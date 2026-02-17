//! 语句 AST 定义 (v2)
//!
//! 基于枚举的简化语句定义，支持所有图数据库操作语句。

use super::pattern::*;
use super::types::*;
use crate::core::types::PropertyDef;
use crate::core::types::expression::Expression;
use crate::core::types::expression::utils::CoreExprUtils;

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
    Profile(ProfileStmt),
    GroupBy(GroupByStmt),
    Lookup(LookupStmt),
    Subgraph(SubgraphStmt),
    FindPath(FindPathStmt),
    Insert(InsertStmt),
    Merge(MergeStmt),
    Unwind(UnwindStmt),
    Return(ReturnStmt),
    With(WithStmt),
    Yield(YieldStmt),
    Set(SetStmt),
    Remove(RemoveStmt),
    Pipe(PipeStmt),
    Drop(DropStmt),
    Desc(DescStmt),
    Alter(AlterStmt),
    CreateUser(CreateUserStmt),
    AlterUser(AlterUserStmt),
    DropUser(DropUserStmt),
    ChangePassword(ChangePasswordStmt),
    Grant(GrantStmt),
    Revoke(RevokeStmt),
    DescribeUser(DescribeUserStmt),
    ShowUsers(ShowUsersStmt),
    ShowRoles(ShowRolesStmt),
    ShowCreate(ShowCreateStmt),
    ShowSessions(ShowSessionsStmt),
    ShowQueries(ShowQueriesStmt),
    KillQuery(KillQueryStmt),
    ShowConfigs(ShowConfigsStmt),
    UpdateConfigs(UpdateConfigsStmt),
    Assignment(AssignmentStmt),
    SetOperation(SetOperationStmt),
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
            Stmt::Profile(s) => s.span,
            Stmt::GroupBy(s) => s.span,
            Stmt::Lookup(s) => s.span,
            Stmt::Subgraph(s) => s.span,
            Stmt::FindPath(s) => s.span,
            Stmt::Insert(s) => s.span,
            Stmt::Merge(s) => s.span,
            Stmt::Unwind(s) => s.span,
            Stmt::Return(s) => s.span,
            Stmt::With(s) => s.span,
            Stmt::Yield(s) => s.span,
            Stmt::Set(s) => s.span,
            Stmt::Remove(s) => s.span,
            Stmt::Pipe(s) => s.span,
            Stmt::Drop(s) => s.span,
            Stmt::Desc(s) => s.span,
            Stmt::Alter(s) => s.span,
            Stmt::CreateUser(s) => s.span,
            Stmt::AlterUser(s) => s.span,
            Stmt::DropUser(s) => s.span,
            Stmt::ChangePassword(s) => s.span,
            Stmt::Grant(s) => s.span,
            Stmt::Revoke(s) => s.span,
            Stmt::DescribeUser(s) => s.span,
            Stmt::ShowUsers(s) => s.span,
            Stmt::ShowRoles(s) => s.span,
            Stmt::ShowCreate(s) => s.span,
            Stmt::ShowSessions(s) => s.span,
            Stmt::ShowQueries(s) => s.span,
            Stmt::KillQuery(s) => s.span,
            Stmt::ShowConfigs(s) => s.span,
            Stmt::UpdateConfigs(s) => s.span,
            Stmt::Assignment(s) => s.span,
            Stmt::SetOperation(s) => s.span,
        }
    }

    /// 获取语句类型名称
    pub fn kind(&self) -> &'static str {
        match self {
            Stmt::Query(_) => "QUERY",
            Stmt::Create(_) => "CREATE",
            Stmt::Match(_) => "MATCH",
            Stmt::Delete(_) => "DELETE",
            Stmt::Update(_) => "UPDATE",
            Stmt::Go(_) => "GO",
            Stmt::Fetch(_) => "FETCH",
            Stmt::Use(_) => "USE",
            Stmt::Show(_) => "SHOW",
            Stmt::Explain(_) => "EXPLAIN",
            Stmt::Profile(_) => "PROFILE",
            Stmt::GroupBy(_) => "GROUP BY",
            Stmt::Lookup(_) => "LOOKUP",
            Stmt::Subgraph(_) => "SUBGRAPH",
            Stmt::FindPath(_) => "FIND PATH",
            Stmt::Insert(_) => "INSERT",
            Stmt::Merge(_) => "MERGE",
            Stmt::Unwind(_) => "UNWIND",
            Stmt::Return(_) => "RETURN",
            Stmt::With(_) => "WITH",
            Stmt::Yield(_) => "YIELD",
            Stmt::Set(_) => "SET",
            Stmt::Remove(_) => "REMOVE",
            Stmt::Pipe(_) => "PIPE",
            Stmt::Drop(_) => "DROP",
            Stmt::Desc(_) => "DESC",
            Stmt::Alter(_) => "ALTER",
            Stmt::CreateUser(_) => "CREATE USER",
            Stmt::AlterUser(_) => "ALTER USER",
            Stmt::DropUser(_) => "DROP USER",
            Stmt::ChangePassword(_) => "CHANGE PASSWORD",
            Stmt::Grant(_) => "GRANT",
            Stmt::Revoke(_) => "REVOKE",
            Stmt::DescribeUser(_) => "DESCRIBE USER",
            Stmt::ShowUsers(_) => "SHOW USERS",
            Stmt::ShowRoles(_) => "SHOW ROLES",
            Stmt::ShowCreate(_) => "SHOW CREATE",
            Stmt::ShowSessions(_) => "SHOW SESSIONS",
            Stmt::ShowQueries(_) => "SHOW QUERIES",
            Stmt::KillQuery(_) => "KILL QUERY",
            Stmt::ShowConfigs(_) => "SHOW CONFIGS",
            Stmt::UpdateConfigs(_) => "UPDATE CONFIGS",
            Stmt::Assignment(_) => "ASSIGNMENT",
            Stmt::SetOperation(_) => "SET OPERATION",
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
    pub if_not_exists: bool,
}

/// 创建目标
#[derive(Debug, Clone, PartialEq)]
pub enum CreateTarget {
    Node {
        variable: Option<String>,
        labels: Vec<String>,
        properties: Option<Expression>,
    },
    Edge {
        variable: Option<String>,
        edge_type: String,
        src: Expression,
        dst: Expression,
        properties: Option<Expression>,
        direction: EdgeDirection,
    },
    Tag {
        name: String,
        properties: Vec<PropertyDef>,
        ttl_duration: Option<i64>,
        ttl_col: Option<String>,
    },
    EdgeType {
        name: String,
        properties: Vec<PropertyDef>,
        ttl_duration: Option<i64>,
        ttl_col: Option<String>,
    },
    Space {
        name: String,
        vid_type: String,
        partition_num: i64,
        replica_factor: i64,
        comment: Option<String>,
    },
    Index {
        name: String,
        on: String,
        properties: Vec<String>,
    },
}

/// MATCH 语句
#[derive(Debug, Clone, PartialEq)]
pub struct MatchStmt {
    pub span: Span,
    pub patterns: Vec<Pattern>,
    pub where_clause: Option<Expression>,
    pub return_clause: Option<ReturnClause>,
    pub order_by: Option<OrderByClause>,
    pub limit: Option<usize>,
    pub skip: Option<usize>,
    pub optional: bool,
}

/// 返回子句
#[derive(Debug, Clone, PartialEq)]
pub struct ReturnClause {
    pub span: Span,
    pub items: Vec<ReturnItem>,
    pub distinct: bool,
    pub limit: Option<super::types::LimitClause>,
    pub skip: Option<super::types::SkipClause>,
    pub sample: Option<super::types::SampleClause>,
}

/// 返回项
#[derive(Debug, Clone, PartialEq)]
pub enum ReturnItem {
    All,
    Expression { expression: Expression, alias: Option<String> },
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
    pub expression: Expression,
    pub direction: OrderDirection,
}

/// DELETE 语句
#[derive(Debug, Clone, PartialEq)]
pub struct DeleteStmt {
    pub span: Span,
    pub target: DeleteTarget,
    pub where_clause: Option<Expression>,
    pub with_edge: bool, // 是否同时删除关联边
}

impl DeleteStmt {
    /// 创建新的DELETE语句
    pub fn new(target: DeleteTarget, span: Span) -> Self {
        Self {
            span,
            target,
            where_clause: None,
            with_edge: false,
        }
    }

    /// 设置是否删除关联边
    pub fn with_edge(mut self, with_edge: bool) -> Self {
        self.with_edge = with_edge;
        self
    }
}

/// 删除目标
#[derive(Debug, Clone, PartialEq)]
pub enum DeleteTarget {
    Vertices(Vec<Expression>),
    Edges {
        edge_type: Option<String>,
        edges: Vec<(Expression, Expression, Option<Expression>)>,
    },
    /// 删除标签 - 包含标签名列表和顶点ID列表
    Tags {
        tag_names: Vec<String>,
        vertex_ids: Vec<Expression>,
        is_all_tags: bool,
    },
    Index(String),
}

/// UPDATE 语句
#[derive(Debug, Clone, PartialEq)]
pub struct UpdateStmt {
    pub span: Span,
    pub target: UpdateTarget,
    pub set_clause: SetClause,
    pub where_clause: Option<Expression>,
    pub is_upsert: bool,
    pub yield_clause: Option<YieldClause>,
}

/// 更新目标
#[derive(Debug, Clone, PartialEq)]
pub enum UpdateTarget {
    Vertex(Expression),
    Edge {
        src: Expression,
        dst: Expression,
        edge_type: Option<String>,
        rank: Option<Expression>,
    },
    Tag(String),
    /// 指定 Tag 的顶点更新: UPDATE VERTEX <vid> ON <tag> SET ...
    TagOnVertex {
        vid: Box<Expression>,
        tag_name: String,
    },
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
    pub value: Expression,
}

/// GO 语句
#[derive(Debug, Clone, PartialEq)]
pub struct GoStmt {
    pub span: Span,
    pub steps: Steps,
    pub from: FromClause,
    pub over: Option<OverClause>,
    pub where_clause: Option<Expression>,
    pub yield_clause: Option<YieldClause>,
}

/// 步数定义
#[derive(Debug, Clone, PartialEq)]
pub enum Steps {
    Fixed(usize),
    Range { min: usize, max: usize },
    Variable(String),
}

/// STEP 子句
#[derive(Debug, Clone, PartialEq)]
pub struct StepClause {
    pub span: Span,
    pub steps: Steps,
}

/// WHERE 子句
#[derive(Debug, Clone, PartialEq)]
pub struct WhereClause {
    pub span: Span,
    pub condition: Expression,
}

/// FROM 子句
#[derive(Debug, Clone, PartialEq)]
pub struct FromClause {
    pub span: Span,
    pub vertices: Vec<Expression>,
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
    pub limit: Option<super::types::LimitClause>,
    pub skip: Option<super::types::SkipClause>,
    pub sample: Option<super::types::SampleClause>,
}

/// YIELD 项
#[derive(Debug, Clone, PartialEq)]
pub struct YieldItem {
    pub expression: Expression,
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
        ids: Vec<Expression>,
        properties: Option<Vec<String>>,
    },
    Edges {
        src: Expression,
        dst: Expression,
        edge_type: String,
        rank: Option<Expression>,
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

/// EXPLAIN 格式类型
#[derive(Debug, Clone, PartialEq)]
pub enum ExplainFormat {
    Table,
    Dot,
}

impl Default for ExplainFormat {
    fn default() -> Self {
        ExplainFormat::Table
    }
}

/// EXPLAIN 语句
#[derive(Debug, Clone, PartialEq)]
pub struct ExplainStmt {
    pub span: Span,
    pub statement: Box<Stmt>,
    pub format: ExplainFormat,
}

/// PROFILE 语句
#[derive(Debug, Clone, PartialEq)]
pub struct ProfileStmt {
    pub span: Span,
    pub statement: Box<Stmt>,
    pub format: ExplainFormat,
}

/// GROUP BY 语句
#[derive(Debug, Clone, PartialEq)]
pub struct GroupByStmt {
    pub span: Span,
    pub group_items: Vec<Expression>,
    pub yield_clause: YieldClause,
    pub having_clause: Option<Expression>,
}

/// LOOKUP 语句（新增）
#[derive(Debug, Clone, PartialEq)]
pub struct LookupStmt {
    pub span: Span,
    pub target: LookupTarget,
    pub where_clause: Option<Expression>,
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
    pub where_clause: Option<Expression>,
    pub yield_clause: Option<YieldClause>,
}

/// FIND PATH 语句（新增）
#[derive(Debug, Clone, PartialEq)]
pub struct FindPathStmt {
    pub span: Span,
    pub from: FromClause,
    pub to: Expression,
    pub over: Option<OverClause>,
    pub where_clause: Option<Expression>,
    pub shortest: bool,
    pub max_steps: Option<usize>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub yield_clause: Option<YieldClause>,
    pub weight_expression: Option<String>,
    pub heuristic_expression: Option<String>,
    pub with_loop: bool,   // 是否允许自环
    pub with_cycle: bool,  // 是否允许回路（路径中重复访问顶点）
}

/// INSERT 语句
#[derive(Debug, Clone, PartialEq)]
pub struct InsertStmt {
    pub span: Span,
    pub target: InsertTarget,
    pub if_not_exists: bool,
}

/// INSERT 目标
#[derive(Debug, Clone, PartialEq)]
pub enum InsertTarget {
    Vertices {
        tags: Vec<TagInsertSpec>,
        values: Vec<VertexRow>,
    },
    Edge {
        edge_name: String,
        prop_names: Vec<String>,
        edges: Vec<(Expression, Expression, Option<Expression>, Vec<Expression>)>,
    },
}

/// Tag 插入规范
#[derive(Debug, Clone, PartialEq)]
pub struct TagInsertSpec {
    pub tag_name: String,
    pub prop_names: Vec<String>,
    pub is_default_props: bool,
}

/// 顶点行数据
#[derive(Debug, Clone, PartialEq)]
pub struct VertexRow {
    pub vid: Expression,
    pub tag_values: Vec<Vec<Expression>>,
}

/// MERGE 语句
#[derive(Debug, Clone, PartialEq)]
pub struct MergeStmt {
    pub span: Span,
    pub pattern: Pattern,
    pub on_create: Option<SetClause>,
    pub on_match: Option<SetClause>,
}

/// SHOW SESSIONS 语句
#[derive(Debug, Clone, PartialEq)]
pub struct ShowSessionsStmt {
    pub span: Span,
}

/// SHOW QUERIES 语句
#[derive(Debug, Clone, PartialEq)]
pub struct ShowQueriesStmt {
    pub span: Span,
}

/// KILL QUERY 语句
#[derive(Debug, Clone, PartialEq)]
pub struct KillQueryStmt {
    pub span: Span,
    pub session_id: i64,
    pub plan_id: i64,
}

/// SHOW CONFIGS 语句
#[derive(Debug, Clone, PartialEq)]
pub struct ShowConfigsStmt {
    pub span: Span,
    pub module: Option<String>,  // 可选的模块名过滤
}

/// UPDATE CONFIGS 语句
#[derive(Debug, Clone, PartialEq)]
pub struct UpdateConfigsStmt {
    pub span: Span,
    pub module: Option<String>,  // 可选的模块名
    pub config_name: String,
    pub config_value: crate::core::types::expression::Expression,
}

/// 变量赋值语句
#[derive(Debug, Clone, PartialEq)]
pub struct AssignmentStmt {
    pub span: Span,
    pub variable: String,  // 变量名（不包含$前缀）
    pub statement: Box<Stmt>,
}

/// 集合操作类型
#[derive(Debug, Clone, PartialEq)]
pub enum SetOperationType {
    Union,
    UnionAll,
    Intersect,
    Minus,
}

/// 集合操作语句
#[derive(Debug, Clone, PartialEq)]
pub struct SetOperationStmt {
    pub span: Span,
    pub op_type: SetOperationType,
    pub left: Box<Stmt>,
    pub right: Box<Stmt>,
}

/// UNWIND 语句
#[derive(Debug, Clone, PartialEq)]
pub struct UnwindStmt {
    pub span: Span,
    pub expression: Expression,
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
    pub where_clause: Option<Expression>,
    pub distinct: bool,
}

/// YIELD 语句
#[derive(Debug, Clone, PartialEq)]
pub struct YieldStmt {
    pub span: Span,
    pub items: Vec<YieldItem>,
    pub where_clause: Option<Expression>,
    pub distinct: bool,
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
    pub items: Vec<Expression>,
}

/// PIPE 语句
#[derive(Debug, Clone, PartialEq)]
pub struct PipeStmt {
    pub span: Span,
    pub left: Box<Stmt>,
    pub right: Box<Stmt>,
}

/// MATCH 子句（用于 MATCH 语句中的子句）
#[derive(Debug, Clone, PartialEq)]
pub struct MatchClause {
    pub span: Span,
    pub patterns: Vec<Pattern>,
    pub optional: bool,
}

/// WITH 子句（用于子查询管道）
#[derive(Debug, Clone, PartialEq)]
pub struct WithClause {
    pub span: Span,
    pub items: Vec<ReturnItem>,
    pub where_clause: Option<Expression>,
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
                    variables.extend(CoreExprUtils::find_variables(where_clause));
                }
            }
            Stmt::Create(s) => match &s.target {
                CreateTarget::Node { properties, .. } => {
                    if let Some(props) = properties {
                        variables.extend(CoreExprUtils::find_variables(props));
                    }
                }
                CreateTarget::Edge {
                    src,
                    dst,
                    properties,
                    ..
                } => {
                    variables.extend(CoreExprUtils::find_variables(src));
                    variables.extend(CoreExprUtils::find_variables(dst));
                    if let Some(props) = properties {
                        variables.extend(CoreExprUtils::find_variables(props));
                    }
                }
                _ => {}
            },
            Stmt::Delete(s) => {
                match &s.target {
                    DeleteTarget::Vertices(vertices) => {
                        for vertex in vertices {
                            variables.extend(CoreExprUtils::find_variables(vertex));
                        }
                    }
                    DeleteTarget::Edges { edges, .. } => {
                        for (src, dst, rank) in edges {
                            variables.extend(CoreExprUtils::find_variables(src));
                            variables.extend(CoreExprUtils::find_variables(dst));
                            if let Some(ref rank) = rank {
                                variables.extend(CoreExprUtils::find_variables(rank));
                            }
                        }
                    }
                    _ => {}
                }
                if let Some(ref where_clause) = s.where_clause {
                    variables.extend(CoreExprUtils::find_variables(where_clause));
                }
            }
            Stmt::Update(s) => {
                match &s.target {
                    UpdateTarget::Vertex(vertex) => {
                        variables.extend(CoreExprUtils::find_variables(vertex));
                    }
                    UpdateTarget::Edge { src, dst, rank, .. } => {
                        variables.extend(CoreExprUtils::find_variables(src));
                        variables.extend(CoreExprUtils::find_variables(dst));
                        if let Some(ref rank) = rank {
                            variables.extend(CoreExprUtils::find_variables(rank));
                        }
                    }
                    _ => {}
                }
                for assignment in &s.set_clause.assignments {
                    variables.extend(CoreExprUtils::find_variables(&assignment.value));
                }
                if let Some(ref where_clause) = s.where_clause {
                    variables.extend(CoreExprUtils::find_variables(where_clause));
                }
            }
            Stmt::Go(s) => {
                for vertex in &s.from.vertices {
                    variables.extend(CoreExprUtils::find_variables(vertex));
                }
                if let Some(ref where_clause) = s.where_clause {
                    variables.extend(CoreExprUtils::find_variables(where_clause));
                }
            }
            Stmt::Fetch(s) => match &s.target {
                FetchTarget::Vertices { ids, .. } => {
                    for id in ids {
                        variables.extend(CoreExprUtils::find_variables(id));
                    }
                }
                FetchTarget::Edges { src, dst, rank, .. } => {
                    variables.extend(CoreExprUtils::find_variables(src));
                    variables.extend(CoreExprUtils::find_variables(dst));
                    if let Some(ref rank) = rank {
                        variables.extend(CoreExprUtils::find_variables(rank));
                    }
                }
            },
            Stmt::Lookup(s) => {
                if let Some(ref where_clause) = s.where_clause {
                    variables.extend(CoreExprUtils::find_variables(where_clause));
                }
            }
            Stmt::Subgraph(s) => {
                for vertex in &s.from.vertices {
                    variables.extend(CoreExprUtils::find_variables(vertex));
                }
                if let Some(ref where_clause) = s.where_clause {
                    variables.extend(CoreExprUtils::find_variables(where_clause));
                }
            }
            Stmt::FindPath(s) => {
                for vertex in &s.from.vertices {
                    variables.extend(CoreExprUtils::find_variables(vertex));
                }
                variables.extend(CoreExprUtils::find_variables(&s.to));
                if let Some(ref where_clause) = s.where_clause {
                    variables.extend(CoreExprUtils::find_variables(where_clause));
                }
            }
            _ => {}
        }
    }
}

/// DROP 语句 - 删除空间、标签、边类型或索引
#[derive(Debug, Clone, PartialEq)]
pub struct DropStmt {
    pub span: Span,
    pub target: DropTarget,
    pub if_exists: bool,
}

/// DROP 目标
#[derive(Debug, Clone, PartialEq)]
pub enum DropTarget {
    Space(String),
    Tags(Vec<String>),
    Edges(Vec<String>),
    TagIndex {
        space_name: String,
        index_name: String,
    },
    EdgeIndex {
        space_name: String,
        index_name: String,
    },
}

/// DESCRIBE 语句 - 描述空间、标签或边类型
#[derive(Debug, Clone, PartialEq)]
pub struct DescStmt {
    pub span: Span,
    pub target: DescTarget,
}

/// DESCRIBE 目标
#[derive(Debug, Clone, PartialEq)]
pub enum DescTarget {
    Space(String),
    Tag {
        space_name: String,
        tag_name: String,
    },
    Edge {
        space_name: String,
        edge_name: String,
    },
}

/// ALTER 语句 - 修改标签或边类型
#[derive(Debug, Clone, PartialEq)]
pub struct AlterStmt {
    pub span: Span,
    pub target: AlterTarget,
}

/// 属性修改定义 (用于 CHANGE 操作)
#[derive(Debug, Clone, PartialEq)]
pub struct PropertyChange {
    pub old_name: String,
    pub new_name: String,
    pub data_type: super::types::DataType,
}

/// ALTER 目标
#[derive(Debug, Clone, PartialEq)]
pub enum AlterTarget {
    Tag {
        tag_name: String,
        additions: Vec<PropertyDef>,
        deletions: Vec<String>,
        changes: Vec<PropertyChange>,
    },
    Edge {
        edge_name: String,
        additions: Vec<PropertyDef>,
        deletions: Vec<String>,
        changes: Vec<PropertyChange>,
    },
    Space {
        space_name: String,
        partition_num: Option<usize>,
        replica_factor: Option<usize>,
        comment: Option<String>,
    },
}

/// CREATE USER 语句
#[derive(Debug, Clone, PartialEq)]
pub struct CreateUserStmt {
    pub span: Span,
    pub username: String,
    pub password: String,
    pub role: Option<String>,
    pub if_not_exists: bool,
}

/// ALTER USER 语句
#[derive(Debug, Clone, PartialEq)]
pub struct AlterUserStmt {
    pub span: Span,
    pub username: String,
    pub password: Option<String>,
    pub new_role: Option<String>,
    pub is_locked: Option<bool>,
}

/// DROP USER 语句
#[derive(Debug, Clone, PartialEq)]
pub struct DropUserStmt {
    pub span: Span,
    pub username: String,
    pub if_exists: bool,
}

/// CHANGE PASSWORD 语句
#[derive(Debug, Clone, PartialEq)]
pub struct ChangePasswordStmt {
    pub span: Span,
    pub username: Option<String>,
    pub old_password: String,
    pub new_password: String,
}

/// 角色类型 - 用于GRANT/REVOKE语句
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoleType {
    God,
    Admin,
    Dba,
    User,
    Guest,
}

impl RoleType {
    pub fn as_str(&self) -> &'static str {
        match self {
            RoleType::God => "GOD",
            RoleType::Admin => "ADMIN",
            RoleType::Dba => "DBA",
            RoleType::User => "USER",
            RoleType::Guest => "GUEST",
        }
    }
}

impl std::str::FromStr for RoleType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "GOD" => Ok(RoleType::God),
            "ADMIN" => Ok(RoleType::Admin),
            "DBA" => Ok(RoleType::Dba),
            "USER" => Ok(RoleType::User),
            "GUEST" => Ok(RoleType::Guest),
            _ => Err(format!("未知的角色类型: {}", s)),
        }
    }
}

/// GRANT 语句
#[derive(Debug, Clone, PartialEq)]
pub struct GrantStmt {
    pub span: Span,
    pub role: RoleType,
    pub space_name: String,
    pub username: String,
}

/// REVOKE 语句
#[derive(Debug, Clone, PartialEq)]
pub struct RevokeStmt {
    pub span: Span,
    pub role: RoleType,
    pub space_name: String,
    pub username: String,
}

/// DESCRIBE USER 语句
#[derive(Debug, Clone, PartialEq)]
pub struct DescribeUserStmt {
    pub span: Span,
    pub username: String,
}

/// SHOW USERS 语句
#[derive(Debug, Clone, PartialEq)]
pub struct ShowUsersStmt {
    pub span: Span,
}

/// SHOW ROLES 语句
#[derive(Debug, Clone, PartialEq)]
pub struct ShowRolesStmt {
    pub span: Span,
    pub space_name: Option<String>,
}

/// SHOW CREATE 语句
#[derive(Debug, Clone, PartialEq)]
pub struct ShowCreateStmt {
    pub span: Span,
    pub target: ShowCreateTarget,
}

/// SHOW CREATE 目标
#[derive(Debug, Clone, PartialEq)]
pub enum ShowCreateTarget {
    Space(String),
    Tag(String),
    Edge(String),
    Index(String),
}

#[cfg(test)]
mod tests {
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
            if_not_exists: false,
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
            optional: false,
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
            to: Expression::Variable("target".to_string()),
            over: None,
            where_clause: None,
            shortest: true,
            max_steps: None,
            limit: None,
            offset: None,
            yield_clause: None,
            weight_expression: None,
            heuristic_expression: None,
            with_loop: false,
            with_cycle: false,
        });

        assert!(matches!(stmt, Stmt::FindPath(_)));
    }
}
