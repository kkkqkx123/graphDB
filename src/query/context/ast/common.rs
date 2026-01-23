//! AST上下文中的共享结构定义

use crate::core::types::EdgeDirection;
use crate::query::validator::structs::clause_structs::YieldColumn;
use std::collections::HashMap;

/// 起始顶点类型 - 强类型枚举替代String
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FromType {
    /// 瞬时表达式
    InstantExpression,
    /// 变量引用
    Variable,
    /// 管道输入
    Pipe,
}

impl Default for FromType {
    fn default() -> Self {
        FromType::InstantExpression
    }
}

impl From<FromType> for String {
    fn from(t: FromType) -> Self {
        match t {
            FromType::InstantExpression => "instant_expression".to_string(),
            FromType::Variable => "variable".to_string(),
            FromType::Pipe => "pipe".to_string(),
        }
    }
}

impl From<&str> for FromType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "instant_expression" => FromType::InstantExpression,
            "variable" => FromType::Variable,
            "pipe" => FromType::Pipe,
            _ => FromType::InstantExpression,
        }
    }
}

/// Cypher子句类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CypherClauseKind {
    /// MATCH子句
    Match,
    /// UNWIND子句
    Unwind,
    /// WITH子句
    With,
    /// WHERE子句
    Where,
    /// RETURN子句
    Return,
    /// ORDER BY子句
    OrderBy,
    /// 分页子句
    Pagination,
    /// YIELD子句
    Yield,
    /// SHORTEST PATH子句
    ShortestPath,
    /// ALL SHORTEST PATHS子句
    AllShortestPaths,
}

impl Default for CypherClauseKind {
    fn default() -> Self {
        CypherClauseKind::Match
    }
}

/// 别名类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AliasType {
    /// 节点别名
    Node,
    /// 边别名
    Edge,
    /// 路径别名
    Path,
    /// 节点列表
    NodeList,
    /// 边列表
    EdgeList,
    /// 运行时变量
    Runtime,
}

impl Default for AliasType {
    fn default() -> Self {
        AliasType::Node
    }
}

/// 模式类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PatternKind {
    /// 简单模式
    Simple,
    /// 可变长度模式
    VariableLength,
    /// 路径模式
    Path,
    /// 子图模式
    Subgraph,
}

impl Default for PatternKind {
    fn default() -> Self {
        PatternKind::Simple
    }
}

// 起始顶点信息
#[derive(Debug, Clone)]
pub struct Starts {
    pub from_type: FromType,
    pub src: Option<String>,
    pub original_src: Option<String>,
    pub user_defined_var_name: String,
    pub runtime_vid_name: String,
    pub vids: Vec<String>,
}

impl Starts {
    pub fn new(from_type: FromType) -> Self {
        Self {
            from_type,
            src: None,
            original_src: None,
            user_defined_var_name: String::new(),
            runtime_vid_name: String::new(),
            vids: Vec::new(),
        }
    }
}

// 边的类型和方向信息
#[derive(Debug, Clone)]
pub struct Over {
    pub is_over_all: bool,
    pub edge_types: Vec<String>,
    pub direction: EdgeDirection,
    pub all_edges: Vec<String>,
}

impl Over {
    pub fn new() -> Self {
        Self {
            is_over_all: false,
            edge_types: Vec::new(),
            direction: EdgeDirection::Out,
            all_edges: Vec::new(),
        }
    }
}

// 步数限制信息
#[derive(Debug, Clone)]
pub struct StepClause {
    pub m_steps: usize,
    pub n_steps: usize,
    pub is_m_to_n: bool,
}

impl StepClause {
    pub fn new() -> Self {
        Self {
            m_steps: 1,
            n_steps: 1,
            is_m_to_n: false,
        }
    }
}

// 表达式属性信息
#[derive(Debug, Clone, Default)]
pub struct ExpressionProps {
    pub tag_props: HashMap<String, Vec<String>>,
    pub edge_props: HashMap<String, Vec<String>>,
    pub dst_tag_props: HashMap<String, Vec<String>>,
    pub src_tag_props: HashMap<String, Vec<String>>,
}

/// 输出列集合
#[derive(Debug, Clone, Default)]
pub struct YieldColumns {
    pub columns: Vec<YieldColumn>,
}

impl YieldColumns {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            columns: Vec::with_capacity(capacity),
        }
    }

    pub fn add_column(&mut self, column: YieldColumn) {
        self.columns.push(column);
    }

    pub fn get_column_names(&self) -> Vec<String> {
        self.columns.iter().map(|c| c.name().to_string()).collect()
    }

    pub fn is_empty(&self) -> bool {
        self.columns.is_empty()
    }

    pub fn len(&self) -> usize {
        self.columns.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &YieldColumn> {
        self.columns.iter()
    }
}
