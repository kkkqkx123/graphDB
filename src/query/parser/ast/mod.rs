//! 抽象语法树（AST）模块
//!
//! 本模块提供了基于 trait 的 AST 设计，支持访问者模式，具有更好的扩展性和类型安全性。

// 基础 AST 节点定义
pub mod node;

// 表达式 AST
pub mod expression;

// 语句 AST
pub mod statement;

// 模式 AST（图模式匹配）
pub mod pattern;

// 类型定义
pub mod types;
pub use types::{
    Label, MatchClause, MatchClauseDetail, MatchEdge, MatchNode, MatchPath, MatchPathSegment,
    Property, TagIdentifier, WhereClause, WithClause, WithItem,
};

// 从 compat 导入以保持兼容性
pub use compat::{BinaryOp, PredicateType, UnaryOp};

// 访问者模式
pub mod visitor;
pub use visitor::*;

// 显式导出节点类型
pub use node::{
    BinaryExpr, CaseExpr, ConstantExpr, FunctionCallExpr, ListExpr, MapExpr, PredicateExpr,
    PropertyAccessExpr, SubscriptExpr, UnaryExpr, VariableExpr,
};

// 显式导出语句类型
pub use statement::{
    Assignment, BaseStatement, CreateStatement, CreateTarget, DataType, DeleteStatement, DeleteTarget,
    EdgeDirection, ExplainStatement, FetchStatement, FetchTarget, FromClause, GoStatement,
    MatchStatement, OrderByClause, OverClause, PropertyRef, QueryStatement, ReturnClause, SetClause,
    ShowStatement, ShowTarget, Steps, UpdateStatement, UpdateTarget, UseStatement, YieldClause,
};

// 显式导出模式类型
pub use pattern::{
    EdgeDirection as PatternEdgeDirection, EdgePattern, EdgeRange, NodePattern, PathElement,
    PathPattern, RepetitionType, VariablePattern,
};

// AST 构建器
pub mod builder;
pub use builder::*;

// 兼容性适配层
pub mod compat;

// 位置信息
pub mod span;
pub use span::*;

// 公共类型别名
pub type Identifier = String;
pub type Result<T, E = AstError> = std::result::Result<T, E>;
pub type VisitorResult = Result<(), AstError>;

/// AST 错误类型
#[derive(Debug, Clone, PartialEq)]
pub enum AstError {
    InvalidNode(String),
    TypeMismatch(String),
    SemanticError(String),
    UnsupportedFeature(String),
}

impl std::fmt::Display for AstError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AstError::InvalidNode(msg) => write!(f, "Invalid AST node: {}", msg),
            AstError::TypeMismatch(msg) => write!(f, "Type mismatch: {}", msg),
            AstError::SemanticError(msg) => write!(f, "Semantic error: {}", msg),
            AstError::UnsupportedFeature(msg) => write!(f, "Unsupported feature: {}", msg),
        }
    }
}

impl std::error::Error for AstError {}

/// AST 节点特征 - 所有 AST 节点都必须实现此 trait
pub trait AstNode: std::fmt::Debug {
    /// 获取节点的位置信息
    fn span(&self) -> Span;

    /// 接受访问者
    fn accept(&self, visitor: &mut dyn Visitor) -> VisitorResult;

    /// 获取节点类型名称
    fn node_type(&self) -> &'static str;

    /// 转换为字符串表示
    fn to_string(&self) -> String;

    /// 克隆为 Box<dyn AstNode>
    fn clone_box(&self) -> Box<dyn AstNode>;
}

/// 表达式特征 - 所有表达式节点都必须实现此 trait
pub trait Expression: AstNode {
    /// 获取表达式类型
    fn expr_type(&self) -> ExpressionType;

    /// 检查是否为常量表达式
    fn is_constant(&self) -> bool;

    /// 获取子表达式
    fn children(&self) -> Vec<Box<dyn Expression>>;

    /// 克隆为 Box<dyn Expression>
    fn clone_box(&self) -> Box<dyn Expression>;

    /// 转换为 Any 类型，用于向下转型
    fn as_any(&self) -> &dyn std::any::Any;
}

/// 语句特征 - 所有语句节点都必须实现此 trait
pub trait Statement: AstNode {
    /// 获取语句类型
    fn stmt_type(&self) -> StatementType;

    /// 获取语句的子节点
    fn children(&self) -> Vec<Box<dyn AstNode>>;

    /// 克隆为 Box<dyn Statement>
    fn clone_box(&self) -> Box<dyn Statement>;
}

/// 模式特征 - 所有模式节点都必须实现此 trait
pub trait Pattern: AstNode {
    /// 获取模式类型
    fn pattern_type(&self) -> PatternType;

    /// 获取模式中的变量
    fn variables(&self) -> Vec<String>;

    /// 转换为 Any 类型，用于向下转型
    fn as_any(&self) -> &dyn std::any::Any;
}

/// 表达式类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpressionType {
    Constant,
    Variable,
    FunctionCall,
    PropertyAccess,
    AttributeAccess,
    Binary,
    Unary,
    List,
    Map,
    Subscript,
    Case,
    Predicate,
    Pattern,
}

impl ExpressionType {
    /// 检查是否是数值类型
    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            ExpressionType::Constant
                | ExpressionType::Variable
                | ExpressionType::Binary
                | ExpressionType::Unary
        )
    }
}

/// 语句类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatementType {
    Query,
    Create,
    Match,
    Delete,
    Update,
    Use,
    Show,
    Explain,
    Go,
    Fetch,
    Lookup,
    FindPath,
    Insert,
    Merge,
    Call,
    Return,
}

/// 模式类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatternType {
    Node,
    Edge,
    Path,
    Variable,
}

// 为 Box<dyn Statement> 实现 Statement trait
impl Statement for Box<dyn Statement> {
    fn stmt_type(&self) -> StatementType {
        self.as_ref().stmt_type()
    }

    fn children(&self) -> Vec<Box<dyn AstNode>> {
        self.as_ref().children()
    }

    fn clone_box(&self) -> Box<dyn Statement> {
        Statement::clone_box(self.as_ref())
    }
}

// 为 Box<dyn Statement> 实现 AstNode trait
impl AstNode for Box<dyn Statement> {
    fn span(&self) -> Span {
        self.as_ref().span()
    }

    fn accept(&self, visitor: &mut dyn Visitor) -> VisitorResult {
        self.as_ref().accept(visitor)
    }

    fn node_type(&self) -> &'static str {
        self.as_ref().node_type()
    }

    fn to_string(&self) -> String {
        self.as_ref().to_string()
    }

    fn clone_box(&self) -> Box<dyn AstNode> {
        Box::new(Statement::clone_box(self))
    }
}

// 为 Box<dyn Expression> 实现 Expression trait
impl Expression for Box<dyn Expression> {
    fn expr_type(&self) -> ExpressionType {
        self.as_ref().expr_type()
    }

    fn is_constant(&self) -> bool {
        self.as_ref().is_constant()
    }

    fn children(&self) -> Vec<Box<dyn Expression>> {
        self.as_ref().children()
    }

    fn clone_box(&self) -> Box<dyn Expression> {
        Expression::clone_box(self.as_ref())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self.as_ref().as_any()
    }
}

// 为 Box<dyn Expression> 实现 AstNode trait
impl AstNode for Box<dyn Expression> {
    fn span(&self) -> Span {
        self.as_ref().span()
    }

    fn accept(&self, visitor: &mut dyn Visitor) -> VisitorResult {
        self.as_ref().accept(visitor)
    }

    fn node_type(&self) -> &'static str {
        self.as_ref().node_type()
    }

    fn to_string(&self) -> String {
        self.as_ref().to_string()
    }

    fn clone_box(&self) -> Box<dyn AstNode> {
        Box::new(Expression::clone_box(self))
    }
}

/// 查询根节点
#[derive(Debug)]
pub struct Query {
    pub statements: Vec<Box<dyn Statement>>,
    pub span: Span,
}

impl Query {
    pub fn new(statements: Vec<Box<dyn Statement>>, span: Span) -> Self {
        Self { statements, span }
    }

    pub fn single_statement(stmt: Box<dyn Statement>, span: Span) -> Self {
        Self {
            statements: vec![stmt],
            span,
        }
    }
}

impl AstNode for Query {
    fn span(&self) -> Span {
        self.span
    }

    fn accept(&self, visitor: &mut dyn Visitor) -> VisitorResult {
        visitor.visit_query(self)
    }

    fn node_type(&self) -> &'static str {
        "Query"
    }

    fn to_string(&self) -> String {
        self.statements
            .iter()
            .map(|stmt| stmt.to_string())
            .collect::<Vec<_>>()
            .join("; ")
    }

    fn clone_box(&self) -> Box<dyn AstNode> {
        Box::new(Query {
            statements: self
                .statements
                .iter()
                .map(|stmt| Statement::clone_box(stmt))
                .collect(),
            span: self.span.clone(),
        })
    }
}
