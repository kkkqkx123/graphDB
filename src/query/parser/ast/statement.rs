//! 语句 AST 定义
//!
//! 定义所有语句类型的 AST 节点，支持访问者模式和语义分析。

use crate::core::Value;
use super::{AstNode, Statement, Expression, Span, StatementType, types::*};
use std::fmt;

/// 基础语句节点
#[derive(Debug, Clone, PartialEq)]
pub struct BaseStatement {
    pub span: Span,
    pub stmt_type: StatementType,
}

impl BaseStatement {
    pub fn new(span: Span, stmt_type: StatementType) -> Self {
        Self { span, stmt_type }
    }
}

/// 查询语句
#[derive(Debug)]
pub struct QueryStatement {
    pub base: BaseStatement,
    pub statements: Vec<Box<dyn Statement>>,
}

impl QueryStatement {
    pub fn new(statements: Vec<Box<dyn Statement>>, span: Span) -> Self {
        Self {
            base: BaseStatement::new(span, StatementType::Query),
            statements,
        }
    }
}

impl AstNode for QueryStatement {
    fn span(&self) -> Span {
        self.base.span
    }
    
    fn accept(&self, visitor: &mut dyn super::Visitor) -> super::VisitorResult {
        visitor.visit_query_statement(self)
    }
    
    fn node_type(&self) -> &'static str {
        "QueryStatement"
    }
    
    fn to_string(&self) -> String {
        self.statements.iter()
            .map(|stmt| stmt.to_string())
            .collect::<Vec<_>>()
            .join("; ")
    }
    
    fn clone_box(&self) -> Box<dyn AstNode> {
        Box::new(QueryStatement {
            base: self.base.clone(),
            statements: self.statements.iter().map(|stmt| super::Statement::clone_box(stmt)).collect(),
        })
    }
}

impl Statement for QueryStatement {
    fn stmt_type(&self) -> StatementType {
        self.base.stmt_type
    }
    
    fn children(&self) -> Vec<Box<dyn AstNode>> {
        self.statements.iter().map(|stmt| super::Statement::clone_box(stmt) as Box<dyn AstNode>).collect()
    }

    fn clone_box(&self) -> Box<dyn Statement> {
        Box::new(QueryStatement {
            base: self.base.clone(),
            statements: self.statements.iter().map(|stmt| super::Statement::clone_box(stmt)).collect(),
        })
    }
}

/// CREATE 语句
#[derive(Debug)]
pub struct CreateStatement {
    pub base: BaseStatement,
    pub target: CreateTarget,
    pub if_not_exists: bool,
    pub properties: Vec<Property>,
    pub yield_clause: Option<YieldClause>,
}

#[derive(Debug)]
pub enum CreateTarget {
    Node {
        identifier: Option<String>,
        labels: Vec<String>,
        properties: Option<Box<dyn Expression>>,
    },
    Edge {
        identifier: Option<String>,
        edge_type: String,
        src: Box<dyn Expression>,
        dst: Box<dyn Expression>,
        direction: EdgeDirection,
        properties: Option<Box<dyn Expression>>,
    },
    Tag {
        name: String,
        properties: Vec<PropertyDefinition>,
    },
    Index {
        name: String,
        on_type: String,
        on_property: String,
    },
}

impl CreateTarget {
    fn clone_box(&self) -> CreateTarget {
        match self {
            CreateTarget::Node { identifier, labels, properties } => CreateTarget::Node {
                identifier: identifier.clone(),
                labels: labels.clone(),
                properties: properties.as_ref().map(|expr| super::Expression::clone_box(expr) as Box<dyn Expression>),
            },
            CreateTarget::Edge { identifier, edge_type, src, dst, direction, properties } => CreateTarget::Edge {
                identifier: identifier.clone(),
                edge_type: edge_type.clone(),
                src: super::Expression::clone_box(src) as Box<dyn Expression>,
                dst: super::Expression::clone_box(dst) as Box<dyn Expression>,
                direction: direction.clone(),
                properties: properties.as_ref().map(|expr| super::Expression::clone_box(expr) as Box<dyn Expression>),
            },
            CreateTarget::Tag { name, properties } => CreateTarget::Tag {
                name: name.clone(),
                properties: properties.clone(),
            },
            CreateTarget::Index { name, on_type, on_property } => CreateTarget::Index {
                name: name.clone(),
                on_type: on_type.clone(),
                on_property: on_property.clone(),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PropertyDefinition {
    pub name: String,
    pub data_type: DataType,
    pub nullable: bool,
    pub default_value: Option<Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DataType {
    Bool,
    Int,
    Float,
    String,
    Date,
    Time,
    DateTime,
    Duration,
    List(Box<DataType>),
    Map(Box<DataType>, Box<DataType>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum EdgeDirection {
    Outbound,      // ->
    Inbound,       // <-
    Bidirectional, // -
}

impl CreateStatement {
    pub fn new(target: CreateTarget, if_not_exists: bool, span: Span) -> Self {
        Self {
            base: BaseStatement::new(span, StatementType::Create),
            target,
            if_not_exists,
            properties: Vec::new(),
            yield_clause: None,
        }
    }
    
    pub fn with_properties(mut self, properties: Vec<Property>) -> Self {
        self.properties = properties;
        self
    }
    
    pub fn with_yield_clause(mut self, yield_clause: YieldClause) -> Self {
        self.yield_clause = Some(yield_clause);
        self
    }
}

impl AstNode for CreateStatement {
    fn span(&self) -> Span {
        self.base.span
    }
    
    fn accept(&self, visitor: &mut dyn super::Visitor) -> super::VisitorResult {
        visitor.visit_create_statement(self)
    }
    
    fn node_type(&self) -> &'static str {
        "CreateStatement"
    }
    
    fn to_string(&self) -> String {
        let mut result = String::from("CREATE ");
        
        if self.if_not_exists {
            result.push_str("IF NOT EXISTS ");
        }
        
        match &self.target {
            CreateTarget::Node { identifier, labels, .. } => {
                result.push('(');
                if let Some(id) = identifier {
                    result.push_str(id);
                    result.push(':');
                }
                result.push_str(&labels.join(":"));
                result.push(')');
            }
            CreateTarget::Edge { edge_type, direction, .. } => {
                result.push('[');
                result.push_str(edge_type);
                result.push(']');
                result.push_str(&direction.to_string());
            }
            CreateTarget::Tag { name, .. } => {
                result.push_str("TAG ");
                result.push_str(name);
            }
            CreateTarget::Index { name, on_type, on_property } => {
                result.push_str("INDEX ");
                result.push_str(name);
                result.push_str(" ON ");
                result.push_str(on_type);
                result.push('(');
                result.push_str(on_property);
                result.push(')');
            }
        }
        
        result
    }
    
    fn clone_box(&self) -> Box<dyn AstNode> {
        Box::new(CreateStatement {
            base: self.base.clone(),
            target: self.target.clone_box(),
            if_not_exists: self.if_not_exists,
            properties: self.properties.clone(),
            yield_clause: self.yield_clause.clone(),
        })
    }
}

impl Statement for CreateStatement {
    fn stmt_type(&self) -> StatementType {
        self.base.stmt_type
    }
    
    fn children(&self) -> Vec<Box<dyn AstNode>> {
        let mut children = Vec::new();
        
        match &self.target {
            CreateTarget::Node { properties, .. } => {
                if let Some(props) = properties {
                    children.push(super::Expression::clone_box(props) as Box<dyn AstNode>);
                }
            }
            CreateTarget::Edge { src, dst, properties, .. } => {
                children.push(super::Expression::clone_box(src) as Box<dyn AstNode>);
                children.push(super::Expression::clone_box(dst) as Box<dyn AstNode>);
                if let Some(props) = properties {
                    children.push(super::Expression::clone_box(props) as Box<dyn AstNode>);
                }
            }
            _ => {}
        }
        
        children
    }

    fn clone_box(&self) -> Box<dyn Statement> {
        Box::new(CreateStatement {
            base: self.base.clone(),
            target: self.target.clone_box(),
            if_not_exists: self.if_not_exists,
            properties: self.properties.clone(),
            yield_clause: self.yield_clause.clone(),
        })
    }
}

impl fmt::Display for EdgeDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EdgeDirection::Outbound => write!(f, "->"),
            EdgeDirection::Inbound => write!(f, "<-"),
            EdgeDirection::Bidirectional => write!(f, "-"),
        }
    }
}

/// MATCH 语句
#[derive(Debug)]
pub struct MatchStatement {
    pub base: BaseStatement,
    pub clauses: Vec<MatchClause>,
}

impl MatchStatement {
    pub fn new(clauses: Vec<MatchClause>, span: Span) -> Self {
        Self {
            base: BaseStatement::new(span, StatementType::Match),
            clauses,
        }
    }
}

impl AstNode for MatchStatement {
    fn span(&self) -> Span {
        self.base.span
    }
    
    fn accept(&self, visitor: &mut dyn super::Visitor) -> super::VisitorResult {
        visitor.visit_match_statement(self)
    }
    
    fn node_type(&self) -> &'static str {
        "MatchStatement"
    }
    
    fn to_string(&self) -> String {
        let mut result = String::from("MATCH ");
        
        // 简化的字符串表示
        for clause in &self.clauses {
            match clause {
                MatchClause::Match(detail) => {
                    result.push_str("MATCH ");
                    // 简化的模式表示
                    if !detail.patterns.is_empty() {
                        result.push_str("pattern");
                    }
                }
                MatchClause::Return(return_clause) => {
                    result.push_str(" RETURN ");
                    result.push_str(&return_clause.to_string());
                }
                MatchClause::With(with_clause) => {
                    result.push_str(" WITH ");
                    // 简化的 WITH 子句表示
                }
                MatchClause::Where(where_clause) => {
                    result.push_str(" WHERE ");
                    result.push_str(&where_clause.expression.to_string());
                }
            }
        }
        
        result
    }
    
    fn clone_box(&self) -> Box<dyn AstNode> {
        Box::new(MatchStatement {
            base: self.base.clone(),
            clauses: self.clauses.iter().map(|clause| clause.clone_box()).collect(),
        })
    }
}

impl Statement for MatchStatement {
    fn stmt_type(&self) -> StatementType {
        self.base.stmt_type
    }
    
    fn children(&self) -> Vec<Box<dyn AstNode>> {
        let mut children = Vec::new();
        
        // 简化的子节点收集
        for clause in &self.clauses {
            match clause {
                MatchClause::Match(detail) => {
                    // 添加模式中的节点
                    for pattern in &detail.patterns {
                        // 这里需要更复杂的遍历逻辑
                    }
                    // 添加 WHERE 子句
                    if let Some(ref where_clause) = detail.where_clause {
                        children.push(super::Expression::clone_box(&where_clause.expression) as Box<dyn AstNode>);
                    }
                }
                MatchClause::Return(return_clause) => {
                    // 添加 RETURN 子句中的表达式
                    for item in &return_clause.items {
                        match item {
                            ReturnItem::Expression(expr, _) => {
                                children.push(super::Expression::clone_box(expr) as Box<dyn AstNode>);
                            }
                            _ => {}
                        }
                    }
                }
                MatchClause::With(with_clause) => {
                    // 添加 WITH 子句中的表达式
                    for item in &with_clause.items {
                        children.push(super::Expression::clone_box(&item.expression) as Box<dyn AstNode>);
                    }
                }
                MatchClause::Where(where_clause) => {
                    children.push(super::Expression::clone_box(&where_clause.expression) as Box<dyn AstNode>);
                }
            }
        }
        
        children
    }

    fn clone_box(&self) -> Box<dyn Statement> {
        Box::new(MatchStatement {
            base: self.base.clone(),
            clauses: self.clauses.iter().map(|clause| clause.clone_box()).collect(),
        })
    }
}

/// DELETE 语句
#[derive(Debug)]
pub struct DeleteStatement {
    pub base: BaseStatement,
    pub target: DeleteTarget,
    pub where_clause: Option<Box<dyn Expression>>,
    pub yield_clause: Option<YieldClause>,
}

#[derive(Debug)]
pub enum DeleteTarget {
    Vertices(Vec<Box<dyn Expression>>),
    Edges {
        edge_type: String,
        src: Box<dyn Expression>,
        dst: Box<dyn Expression>,
        rank: Option<Box<dyn Expression>>,
    },
}

impl DeleteTarget {
    fn clone_box(&self) -> DeleteTarget {
        match self {
            DeleteTarget::Vertices(vertices) => DeleteTarget::Vertices(
                vertices.iter().map(|vertex| super::Expression::clone_box(vertex) as Box<dyn Expression>).collect()
            ),
            DeleteTarget::Edges { edge_type, src, dst, rank } => DeleteTarget::Edges {
                edge_type: edge_type.clone(),
                src: super::Expression::clone_box(src) as Box<dyn Expression>,
                dst: super::Expression::clone_box(dst) as Box<dyn Expression>,
                rank: rank.as_ref().map(|expr| super::Expression::clone_box(expr) as Box<dyn Expression>),
            },
        }
    }
}

impl DeleteStatement {
    pub fn new(target: DeleteTarget, span: Span) -> Self {
        Self {
            base: BaseStatement::new(span, StatementType::Delete),
            target,
            where_clause: None,
            yield_clause: None,
        }
    }
    
    pub fn with_where_clause(mut self, where_clause: Box<dyn Expression>) -> Self {
        self.where_clause = Some(where_clause);
        self
    }
    
    pub fn with_yield_clause(mut self, yield_clause: YieldClause) -> Self {
        self.yield_clause = Some(yield_clause);
        self
    }
}

impl AstNode for DeleteStatement {
    fn span(&self) -> Span {
        self.base.span
    }
    
    fn accept(&self, visitor: &mut dyn super::Visitor) -> super::VisitorResult {
        visitor.visit_delete_statement(self)
    }
    
    fn node_type(&self) -> &'static str {
        "DeleteStatement"
    }
    
    fn to_string(&self) -> String {
        let mut result = String::from("DELETE ");
        
        match &self.target {
            DeleteTarget::Vertices(vertices) => {
                result.push_str("VERTEX ");
                let vertex_strs: Vec<String> = vertices.iter()
                    .map(|v| v.to_string())
                    .collect();
                result.push_str(&vertex_strs.join(", "));
            }
            DeleteTarget::Edges { edge_type, src, dst, .. } => {
                result.push_str("EDGE ");
                result.push_str(edge_type);
                result.push(' ');
                result.push_str(&src.to_string());
                result.push_str(" -> ");
                result.push_str(&dst.to_string());
            }
        }
        
        if let Some(ref where_clause) = self.where_clause {
            result.push_str(" WHERE ");
            result.push_str(&where_clause.to_string());
        }
        
        result
    }
    
    fn clone_box(&self) -> Box<dyn AstNode> {
        Box::new(DeleteStatement {
            base: self.base.clone(),
            target: self.target.clone_box(),
            where_clause: self.where_clause.as_ref().map(|expr| super::Expression::clone_box(expr) as Box<dyn Expression>),
            yield_clause: self.yield_clause.clone(),
        })
    }
}

impl Statement for DeleteStatement {
    fn stmt_type(&self) -> StatementType {
        self.base.stmt_type
    }
    
    fn children(&self) -> Vec<Box<dyn AstNode>> {
        let mut children = Vec::new();
        
        match &self.target {
            DeleteTarget::Vertices(vertices) => {
                for vertex in vertices {
                    children.push(super::Expression::clone_box(vertex) as Box<dyn AstNode>);
                }
            }
            DeleteTarget::Edges { src, dst, rank, .. } => {
                children.push(super::Expression::clone_box(src) as Box<dyn AstNode>);
                children.push(super::Expression::clone_box(dst) as Box<dyn AstNode>);
                if let Some(ref rank) = rank {
                    children.push(super::Expression::clone_box(rank) as Box<dyn AstNode>);
                }
            }
        }
        
        if let Some(ref where_clause) = self.where_clause {
            children.push(super::Expression::clone_box(where_clause) as Box<dyn AstNode>);
        }
        
        children
    }

    fn clone_box(&self) -> Box<dyn Statement> {
        Box::new(DeleteStatement {
            base: self.base.clone(),
            target: self.target.clone_box(),
            where_clause: self.where_clause.as_ref().map(|expr| super::Expression::clone_box(expr) as Box<dyn Expression>),
            yield_clause: self.yield_clause.clone(),
        })
    }
}

/// UPDATE 语句
#[derive(Debug)]
pub struct UpdateStatement {
    pub base: BaseStatement,
    pub target: UpdateTarget,
    pub set_clause: SetClause,
    pub where_clause: Option<Box<dyn Expression>>,
    pub yield_clause: Option<YieldClause>,
}

#[derive(Debug)]
pub enum UpdateTarget {
    Vertex(Box<dyn Expression>),
    Edge {
        edge_type: String,
        src: Box<dyn Expression>,
        dst: Box<dyn Expression>,
        rank: Option<Box<dyn Expression>>,
    },
}

impl UpdateTarget {
    fn clone_box(&self) -> UpdateTarget {
        match self {
            UpdateTarget::Vertex(vertex) => UpdateTarget::Vertex(super::Expression::clone_box(vertex) as Box<dyn Expression>),
            UpdateTarget::Edge { edge_type, src, dst, rank } => UpdateTarget::Edge {
                edge_type: edge_type.clone(),
                src: super::Expression::clone_box(src) as Box<dyn Expression>,
                dst: super::Expression::clone_box(dst) as Box<dyn Expression>,
                rank: rank.as_ref().map(|expr| super::Expression::clone_box(expr) as Box<dyn Expression>),
            },
        }
    }
}

#[derive(Debug)]
pub struct SetClause {
    pub assignments: Vec<Assignment>,
}

impl SetClause {
    fn clone_box(&self) -> SetClause {
        SetClause {
            assignments: self.assignments.iter().map(|assignment| assignment.clone_box()).collect(),
        }
    }
}

impl UpdateStatement {
    pub fn new(target: UpdateTarget, set_clause: SetClause, span: Span) -> Self {
        Self {
            base: BaseStatement::new(span, StatementType::Update),
            target,
            set_clause,
            where_clause: None,
            yield_clause: None,
        }
    }
    
    pub fn with_where_clause(mut self, where_clause: Box<dyn Expression>) -> Self {
        self.where_clause = Some(where_clause);
        self
    }
    
    pub fn with_yield_clause(mut self, yield_clause: YieldClause) -> Self {
        self.yield_clause = Some(yield_clause);
        self
    }
}

impl AstNode for UpdateStatement {
    fn span(&self) -> Span {
        self.base.span
    }
    
    fn accept(&self, visitor: &mut dyn super::Visitor) -> super::VisitorResult {
        visitor.visit_update_statement(self)
    }
    
    fn node_type(&self) -> &'static str {
        "UpdateStatement"
    }
    
    fn to_string(&self) -> String {
        let mut result = String::from("UPDATE ");
        
        match &self.target {
            UpdateTarget::Vertex(vertex) => {
                result.push_str(&vertex.to_string());
            }
            UpdateTarget::Edge { edge_type, src, dst, .. } => {
                result.push_str(edge_type);
                result.push(' ');
                result.push_str(&src.to_string());
                result.push_str(" -> ");
                result.push_str(&dst.to_string());
            }
        }
        
        result.push_str(" SET ");
        let assignment_strs: Vec<String> = self.set_clause.assignments.iter()
            .map(|a| a.to_string())
            .collect();
        result.push_str(&assignment_strs.join(", "));
        
        if let Some(ref where_clause) = self.where_clause {
            result.push_str(" WHERE ");
            result.push_str(&where_clause.to_string());
        }
        
        result
    }
    
    fn clone_box(&self) -> Box<dyn AstNode> {
        Box::new(UpdateStatement {
            base: self.base.clone(),
            target: self.target.clone_box(),
            set_clause: self.set_clause.clone_box(),
            where_clause: self.where_clause.as_ref().map(|expr| super::Expression::clone_box(expr) as Box<dyn Expression>),
            yield_clause: self.yield_clause.clone(),
        })
    }
}

impl Statement for UpdateStatement {
    fn stmt_type(&self) -> StatementType {
        self.base.stmt_type
    }
    
    fn children(&self) -> Vec<Box<dyn AstNode>> {
        let mut children = Vec::new();
        
        match &self.target {
            UpdateTarget::Vertex(vertex) => {
                children.push(super::Expression::clone_box(vertex) as Box<dyn AstNode>);
            }
            UpdateTarget::Edge { src, dst, rank, .. } => {
                children.push(super::Expression::clone_box(src) as Box<dyn AstNode>);
                children.push(super::Expression::clone_box(dst) as Box<dyn AstNode>);
                if let Some(ref rank) = rank {
                    children.push(super::Expression::clone_box(rank) as Box<dyn AstNode>);
                }
            }
        }
        
        // 添加 SET 子句中的表达式
        for assignment in &self.set_clause.assignments {
            children.push(super::Expression::clone_box(&assignment.value) as Box<dyn AstNode>);
        }
        
        if let Some(ref where_clause) = self.where_clause {
            children.push(super::Expression::clone_box(where_clause) as Box<dyn AstNode>);
        }
        
        children
    }

    fn clone_box(&self) -> Box<dyn Statement> {
        Box::new(UpdateStatement {
            base: self.base.clone(),
            target: self.target.clone_box(),
            set_clause: self.set_clause.clone_box(),
            where_clause: self.where_clause.as_ref().map(|expr| super::Expression::clone_box(expr) as Box<dyn Expression>),
            yield_clause: self.yield_clause.clone(),
        })
    }
}

/// GO 语句
#[derive(Debug)]
pub struct GoStatement {
    pub base: BaseStatement,
    pub steps: Steps,
    pub from: FromClause,
    pub over: OverClause,
    pub where_clause: Option<Box<dyn Expression>>,
    pub yield_clause: Option<YieldClause>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Steps {
    Fixed(u32),
    Range(Option<u32>, Option<u32>),
}

#[derive(Debug)]
pub struct FromClause {
    pub vertices: Vec<Box<dyn Expression>>,
}

impl FromClause {
    fn clone_box(&self) -> FromClause {
        FromClause {
            vertices: self.vertices.iter().map(|vertex| super::Expression::clone_box(vertex) as Box<dyn Expression>).collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverClause {
    pub edge_types: Vec<String>,
    pub direction: EdgeDirection,
    pub reversely: bool,
}

impl GoStatement {
    pub fn new(steps: Steps, from: FromClause, over: OverClause, span: Span) -> Self {
        Self {
            base: BaseStatement::new(span, StatementType::Go),
            steps,
            from,
            over,
            where_clause: None,
            yield_clause: None,
        }
    }
    
    pub fn with_where_clause(mut self, where_clause: Box<dyn Expression>) -> Self {
        self.where_clause = Some(where_clause);
        self
    }
    
    pub fn with_yield_clause(mut self, yield_clause: YieldClause) -> Self {
        self.yield_clause = Some(yield_clause);
        self
    }
}

impl AstNode for GoStatement {
    fn span(&self) -> Span {
        self.base.span
    }
    
    fn accept(&self, visitor: &mut dyn super::Visitor) -> super::VisitorResult {
        visitor.visit_go_statement(self)
    }
    
    fn node_type(&self) -> &'static str {
        "GoStatement"
    }
    
    fn to_string(&self) -> String {
        let mut result = String::from("GO ");
        
        // 添加步数
        match self.steps {
            Steps::Fixed(n) => result.push_str(&format!("{} STEP", n)),
            Steps::Range(Some(min), Some(max)) => result.push_str(&format!("{} TO {} STEPS", min, max)),
            Steps::Range(None, Some(max)) => result.push_str(&format!("UPTO {} STEPS", max)),
            Steps::Range(Some(min), None) => result.push_str(&format!("{} TO STEPS", min)),
            Steps::Range(None, None) => result.push_str("STEPS"),
        }
        
        result.push(' ');
        
        // 添加 FROM 子句
        result.push_str("FROM ");
        let vertex_strs: Vec<String> = self.from.vertices.iter()
            .map(|v| v.to_string())
            .collect();
        result.push_str(&vertex_strs.join(", "));
        
        // 添加 OVER 子句
        result.push_str(" OVER ");
        let edge_strs: Vec<String> = self.over.edge_types.iter()
            .map(|e| e.to_string())
            .collect();
        result.push_str(&edge_strs.join(", "));
        
        if self.over.reversely {
            result.push_str(" REVERSELY");
        }
        
        if let Some(ref where_clause) = self.where_clause {
            result.push_str(" WHERE ");
            result.push_str(&where_clause.to_string());
        }
        
        result
    }
    
    fn clone_box(&self) -> Box<dyn AstNode> {
        Box::new(GoStatement {
            base: self.base.clone(),
            steps: self.steps.clone(),
            from: self.from.clone_box(),
            over: self.over.clone(),
            where_clause: self.where_clause.as_ref().map(|expr| super::Expression::clone_box(expr) as Box<dyn Expression>),
            yield_clause: self.yield_clause.clone(),
        })
    }
}

impl Statement for GoStatement {
    fn stmt_type(&self) -> StatementType {
        self.base.stmt_type
    }
    
    fn children(&self) -> Vec<Box<dyn AstNode>> {
        let mut children = Vec::new();
        
        // 添加 FROM 子句中的顶点
        for vertex in &self.from.vertices {
            children.push(super::Expression::clone_box(vertex) as Box<dyn AstNode>);
        }
        
        // 添加 WHERE 子句
        if let Some(ref where_clause) = self.where_clause {
            children.push(super::Expression::clone_box(where_clause) as Box<dyn AstNode>);
        }
        
        children
    }

    fn clone_box(&self) -> Box<dyn Statement> {
        Box::new(GoStatement {
            base: self.base.clone(),
            steps: self.steps.clone(),
            from: self.from.clone_box(),
            over: self.over.clone(),
            where_clause: self.where_clause.as_ref().map(|expr| super::Expression::clone_box(expr) as Box<dyn Expression>),
            yield_clause: self.yield_clause.clone(),
        })
    }
}

/// FETCH 语句
#[derive(Debug)]
pub struct FetchStatement {
    pub base: BaseStatement,
    pub target: FetchTarget,
    pub yield_clause: Option<YieldClause>,
}

#[derive(Debug)]
pub enum FetchTarget {
    Vertices {
        ids: Vec<Box<dyn Expression>>,
        properties: Vec<String>,
    },
    Edges {
        edge_type: String,
        src: Box<dyn Expression>,
        dst: Box<dyn Expression>,
        rank: Option<Box<dyn Expression>>,
        properties: Vec<String>,
    },
}

impl FetchTarget {
    fn clone_box(&self) -> FetchTarget {
        match self {
            FetchTarget::Vertices { ids, properties } => FetchTarget::Vertices {
                ids: ids.iter().map(|id| super::Expression::clone_box(id) as Box<dyn Expression>).collect(),
                properties: properties.clone(),
            },
            FetchTarget::Edges { edge_type, src, dst, rank, properties } => FetchTarget::Edges {
                edge_type: edge_type.clone(),
                src: super::Expression::clone_box(src) as Box<dyn Expression>,
                dst: super::Expression::clone_box(dst) as Box<dyn Expression>,
                rank: rank.as_ref().map(|expr| super::Expression::clone_box(expr) as Box<dyn Expression>),
                properties: properties.clone(),
            },
        }
    }
}

impl FetchStatement {
    pub fn new(target: FetchTarget, span: Span) -> Self {
        Self {
            base: BaseStatement::new(span, StatementType::Fetch),
            target,
            yield_clause: None,
        }
    }
    
    pub fn with_yield_clause(mut self, yield_clause: YieldClause) -> Self {
        self.yield_clause = Some(yield_clause);
        self
    }
}

impl AstNode for FetchStatement {
    fn span(&self) -> Span {
        self.base.span
    }
    
    fn accept(&self, visitor: &mut dyn super::Visitor) -> super::VisitorResult {
        visitor.visit_fetch_statement(self)
    }
    
    fn node_type(&self) -> &'static str {
        "FetchStatement"
    }
    
    fn to_string(&self) -> String {
        let mut result = String::from("FETCH ");
        
        match &self.target {
            FetchTarget::Vertices { ids, properties } => {
                result.push_str("PROP ON ");
                let id_strs: Vec<String> = ids.iter()
                    .map(|id| id.to_string())
                    .collect();
                result.push_str(&id_strs.join(", "));
                
                if !properties.is_empty() {
                    result.push_str(" YIELD ");
                    result.push_str(&properties.join(", "));
                }
            }
            FetchTarget::Edges { edge_type, src, dst, rank, properties } => {
                result.push_str("PROP ON ");
                result.push_str(edge_type);
                result.push(' ');
                result.push_str(&src.to_string());
                result.push_str(" -> ");
                result.push_str(&dst.to_string());
                
                if let Some(ref rank) = rank {
                    result.push('@');
                    result.push_str(&rank.to_string());
                }
                
                if !properties.is_empty() {
                    result.push_str(" YIELD ");
                    result.push_str(&properties.join(", "));
                }
            }
        }
        
        result
    }
    
    fn clone_box(&self) -> Box<dyn AstNode> {
        Box::new(FetchStatement {
            base: self.base.clone(),
            target: self.target.clone_box(),
            yield_clause: self.yield_clause.clone(),
        })
    }
}

impl Statement for FetchStatement {
    fn stmt_type(&self) -> StatementType {
        self.base.stmt_type
    }
    
    fn children(&self) -> Vec<Box<dyn AstNode>> {
        let mut children = Vec::new();
        
        match &self.target {
            FetchTarget::Vertices { ids, .. } => {
                for id in ids {
                    children.push(super::Expression::clone_box(id) as Box<dyn AstNode>);
                }
            }
            FetchTarget::Edges { src, dst, rank, .. } => {
                children.push(super::Expression::clone_box(src) as Box<dyn AstNode>);
                children.push(super::Expression::clone_box(dst) as Box<dyn AstNode>);
                if let Some(ref rank) = rank {
                    children.push(super::Expression::clone_box(rank) as Box<dyn AstNode>);
                }
            }
        }
        
        children
    }

    fn clone_box(&self) -> Box<dyn Statement> {
        Box::new(FetchStatement {
            base: self.base.clone(),
            target: self.target.clone_box(),
            yield_clause: self.yield_clause.clone(),
        })
    }
}

/// USE 语句
#[derive(Debug, Clone, PartialEq)]
pub struct UseStatement {
    pub base: BaseStatement,
    pub space: String,
}

impl UseStatement {
    pub fn new(space: String, span: Span) -> Self {
        Self {
            base: BaseStatement::new(span, StatementType::Use),
            space,
        }
    }
}

impl AstNode for UseStatement {
    fn span(&self) -> Span {
        self.base.span
    }
    
    fn accept(&self, visitor: &mut dyn super::Visitor) -> super::VisitorResult {
        visitor.visit_use_statement(self)
    }
    
    fn node_type(&self) -> &'static str {
        "UseStatement"
    }
    
    fn to_string(&self) -> String {
        format!("USE {}", self.space)
    }
    
    fn clone_box(&self) -> Box<dyn AstNode> {
        Box::new(UseStatement {
            base: self.base.clone(),
            space: self.space.clone(),
        })
    }
}

impl Statement for UseStatement {
    fn stmt_type(&self) -> StatementType {
        self.base.stmt_type
    }
    
    fn children(&self) -> Vec<Box<dyn AstNode>> {
        vec![]
    }

    fn clone_box(&self) -> Box<dyn Statement> {
        Box::new(UseStatement {
            base: self.base.clone(),
            space: self.space.clone(),
        })
    }
}

/// SHOW 语句
#[derive(Debug, Clone, PartialEq)]
pub struct ShowStatement {
    pub base: BaseStatement,
    pub target: ShowTarget,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ShowTarget {
    Spaces,
    Tags,
    Edges,
    TagIndex(String),
    EdgeIndex(String),
    Users,
    Roles(Option<String>),
    Hosts,
    Parts(Option<String>),
    Charset,
    Collation,
    Stats,
}

impl ShowStatement {
    pub fn new(target: ShowTarget, span: Span) -> Self {
        Self {
            base: BaseStatement::new(span, StatementType::Show),
            target,
        }
    }
}

impl AstNode for ShowStatement {
    fn span(&self) -> Span {
        self.base.span
    }
    
    fn accept(&self, visitor: &mut dyn super::Visitor) -> super::VisitorResult {
        visitor.visit_show_statement(self)
    }
    
    fn node_type(&self) -> &'static str {
        "ShowStatement"
    }
    
    fn to_string(&self) -> String {
        let mut result = String::from("SHOW ");
        
        match &self.target {
            ShowTarget::Spaces => result.push_str("SPACES"),
            ShowTarget::Tags => result.push_str("TAGS"),
            ShowTarget::Edges => result.push_str("EDGES"),
            ShowTarget::TagIndex(name) => {
                result.push_str("TAG INDEX ");
                result.push_str(name);
            }
            ShowTarget::EdgeIndex(name) => {
                result.push_str("EDGE INDEX ");
                result.push_str(name);
            }
            ShowTarget::Users => result.push_str("USERS"),
            ShowTarget::Roles(None) => result.push_str("ROLES"),
            ShowTarget::Roles(Some(role)) => {
                result.push_str("ROLES ");
                result.push_str(role);
            }
            ShowTarget::Hosts => result.push_str("HOSTS"),
            ShowTarget::Parts(None) => result.push_str("PARTS"),
            ShowTarget::Parts(Some(part)) => {
                result.push_str("PARTS ");
                result.push_str(part);
            }
            ShowTarget::Charset => result.push_str("CHARSET"),
            ShowTarget::Collation => result.push_str("COLLATION"),
            ShowTarget::Stats => result.push_str("STATS"),
        }
        
        result
    }
    
    fn clone_box(&self) -> Box<dyn AstNode> {
        Box::new(ShowStatement {
            base: self.base.clone(),
            target: self.target.clone(),
        })
    }
}

impl Statement for ShowStatement {
    fn stmt_type(&self) -> StatementType {
        self.base.stmt_type
    }
    
    fn children(&self) -> Vec<Box<dyn AstNode>> {
        vec![]
    }

    fn clone_box(&self) -> Box<dyn Statement> {
        Box::new(ShowStatement {
            base: self.base.clone(),
            target: self.target.clone(),
        })
    }
}

/// EXPLAIN 语句
#[derive(Debug)]
pub struct ExplainStatement {
    pub base: BaseStatement,
    pub statement: Box<dyn Statement>,
}

impl ExplainStatement {
    pub fn new(statement: Box<dyn Statement>, span: Span) -> Self {
        Self {
            base: BaseStatement::new(span, StatementType::Explain),
            statement,
        }
    }
}

impl AstNode for ExplainStatement {
    fn span(&self) -> Span {
        self.base.span
    }
    
    fn accept(&self, visitor: &mut dyn super::Visitor) -> super::VisitorResult {
        visitor.visit_explain_statement(self)
    }
    
    fn node_type(&self) -> &'static str {
        "ExplainStatement"
    }
    
    fn to_string(&self) -> String {
        format!("EXPLAIN {}", self.statement.to_string())
    }
    
    fn clone_box(&self) -> Box<dyn AstNode> {
        Box::new(ExplainStatement {
            base: self.base.clone(),
            statement: super::Statement::clone_box(&self.statement) as Box<dyn Statement>,
        })
    }
}

impl Statement for ExplainStatement {
    fn stmt_type(&self) -> StatementType {
        self.base.stmt_type
    }
    
    fn children(&self) -> Vec<Box<dyn AstNode>> {
        vec![super::Statement::clone_box(&self.statement) as Box<dyn AstNode>]
    }

    fn clone_box(&self) -> Box<dyn Statement> {
        Box::new(ExplainStatement {
            base: self.base.clone(),
            statement: super::Statement::clone_box(&self.statement) as Box<dyn Statement>,
        })
    }
}

/// 辅助结构定义

#[derive(Debug)]
pub struct YieldClause {
    pub distinct: bool,
    pub items: Vec<YieldItem>,
}

impl Clone for YieldClause {
    fn clone(&self) -> Self {
        Self {
            distinct: self.distinct,
            items: self.items.iter().map(|item| item.clone_box()).collect(),
        }
    }
}

#[derive(Debug)]
pub enum YieldItem {
    Expression(Box<dyn Expression>, Option<String>), // 表达式和别名
    All, // *
}

impl Clone for YieldItem {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

impl YieldItem {
    fn clone_box(&self) -> YieldItem {
        match self {
            YieldItem::Expression(expr, alias) => YieldItem::Expression(super::Expression::clone_box(expr) as Box<dyn Expression>, alias.clone()),
            YieldItem::All => YieldItem::All,
        }
    }
}

#[derive(Debug)]
pub struct ReturnClause {
    pub distinct: bool,
    pub items: Vec<ReturnItem>,
}

impl Clone for ReturnClause {
    fn clone(&self) -> Self {
        Self {
            distinct: self.distinct,
            items: self.items.iter().map(|item| item.clone_box()).collect(),
        }
    }
}

impl PartialEq for ReturnClause {
    fn eq(&self, other: &Self) -> bool {
        self.distinct == other.distinct && self.items.len() == other.items.len()
    }
}

#[derive(Debug)]
pub enum ReturnItem {
    Expression(Box<dyn Expression>, Option<String>), // 表达式和别名
    All, // *
}

impl Clone for ReturnItem {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

impl ReturnItem {
    fn clone_box(&self) -> ReturnItem {
        match self {
            ReturnItem::Expression(expr, alias) => ReturnItem::Expression(super::Expression::clone_box(expr) as Box<dyn Expression>, alias.clone()),
            ReturnItem::All => ReturnItem::All,
        }
    }
}

#[derive(Debug)]
pub struct OrderByClause {
    pub items: Vec<OrderByItem>,
}

#[derive(Debug)]
pub struct OrderByItem {
    pub expression: Box<dyn Expression>,
    pub ascending: bool,
}

impl OrderByItem {
    fn clone_box(&self) -> OrderByItem {
        OrderByItem {
            expression: super::Expression::clone_box(&self.expression) as Box<dyn Expression>,
            ascending: self.ascending,
        }
    }
}

#[derive(Debug)]
pub struct Assignment {
    pub property: PropertyRef,
    pub value: Box<dyn Expression>,
}

impl Assignment {
    fn clone_box(&self) -> Assignment {
        Assignment {
            property: self.property.clone(),
            value: super::Expression::clone_box(&self.value) as Box<dyn Expression>,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PropertyRef {
    Simple(String),                    // property_name
    Qualified(String, String),       // tag_name.property_name
    Variable(String, String),        // variable.property_name
}

impl fmt::Display for Assignment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.property {
            PropertyRef::Simple(prop) => write!(f, "{} = {}", prop, self.value.to_string()),
            PropertyRef::Qualified(tag, prop) => write!(f, "{}.{}", tag, prop),
            PropertyRef::Variable(var, prop) => write!(f, "{}.{}", var, prop),
        }
    }
}

impl fmt::Display for YieldItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            YieldItem::Expression(expr, None) => write!(f, "{}", expr.to_string()),
            YieldItem::Expression(expr, Some(alias)) => write!(f, "{} AS {}", expr.to_string(), alias),
            YieldItem::All => write!(f, "*"),
        }
    }
}

impl fmt::Display for ReturnItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReturnItem::Expression(expr, None) => write!(f, "{}", expr.to_string()),
            ReturnItem::Expression(expr, Some(alias)) => write!(f, "{} AS {}", expr.to_string(), alias),
            ReturnItem::All => write!(f, "*"),
        }
    }
}

impl fmt::Display for OrderByItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.ascending {
            write!(f, "{} ASC", self.expression.to_string())
        } else {
            write!(f, "{} DESC", self.expression.to_string())
        }
    }
}

impl fmt::Display for ReturnClause {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RETURN ")?;
        
        if self.distinct {
            write!(f, "DISTINCT ")?;
        }
        
        let item_strs: Vec<String> = self.items.iter()
            .map(|item| item.to_string())
            .collect();
        write!(f, "{}", item_strs.join(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_create_statement() {
        let span = Span::default();
        let target = CreateTarget::Node {
            identifier: Some("n".to_string()),
            labels: vec!["Person".to_string()],
            properties: None,
        };
        
        let stmt = CreateStatement::new(target, false, span);
        assert_eq!(stmt.stmt_type(), StatementType::Create);
        assert_eq!(stmt.to_string(), "CREATE (n:Person)");
    }
    
    #[test]
    fn test_match_statement() {
        let span = Span::default();
        let patterns = vec![]; // 空模式用于测试
        
        let stmt = MatchStatement::new(patterns, span);
        assert_eq!(stmt.stmt_type(), StatementType::Match);
        assert_eq!(stmt.to_string(), "MATCH ");
    }
    
    #[test]
    fn test_go_statement() {
        let span = Span::default();
        let steps = Steps::Fixed(1);
        let from = FromClause {
            vertices: vec![],
        };
        let over = OverClause {
            edge_types: vec!["friend".to_string()],
            direction: EdgeDirection::Outbound,
            reversely: false,
        };
        
        let stmt = GoStatement::new(steps, from, over, span);
        assert_eq!(stmt.stmt_type(), StatementType::Go);
        assert_eq!(stmt.to_string(), "GO 1 STEP FROM  OVER friend");
    }
}