//! AST上下文中的共享结构定义

use std::collections::HashMap;
use crate::query::parser::ast::expr::Expr;

/// 起始顶点类型 - 强类型枚举替代String
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FromType {
    /// 瞬时表达式
    InstantExpr,
    /// 变量引用
    Variable,
    /// 管道输入
    Pipe,
}

impl Default for FromType {
    fn default() -> Self {
        FromType::InstantExpr
    }
}

impl From<FromType> for String {
    fn from(t: FromType) -> Self {
        match t {
            FromType::InstantExpr => "instant_expr".to_string(),
            FromType::Variable => "variable".to_string(),
            FromType::Pipe => "pipe".to_string(),
        }
    }
}

impl From<&str> for FromType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "instant_expr" => FromType::InstantExpr,
            "variable" => FromType::Variable,
            "pipe" => FromType::Pipe,
            _ => FromType::InstantExpr,
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

/// 边方向类型 - 强类型枚举替代String
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EdgeDirection {
    /// 出边
    Out,
    /// 入边
    In,
    /// 双向
    Both,
}

impl Default for EdgeDirection {
    fn default() -> Self {
        EdgeDirection::Out
    }
}

impl From<EdgeDirection> for String {
    fn from(d: EdgeDirection) -> Self {
        match d {
            EdgeDirection::Out => "out".to_string(),
            EdgeDirection::In => "in".to_string(),
            EdgeDirection::Both => "both".to_string(),
        }
    }
}

impl From<&str> for EdgeDirection {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "out" => EdgeDirection::Out,
            "in" => EdgeDirection::In,
            "both" => EdgeDirection::Both,
            _ => EdgeDirection::Out,
        }
    }
}

impl EdgeDirection {
    pub fn as_str(&self) -> &str {
        match self {
            EdgeDirection::Out => "out",
            EdgeDirection::In => "in",
            EdgeDirection::Both => "both",
        }
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

/// 输出列定义
#[derive(Debug, Clone)]
pub struct YieldColumn {
    pub expr: Expr,
    pub alias: Option<String>,
}

impl YieldColumn {
    pub fn new(expr: Expr, alias: Option<String>) -> Self {
        YieldColumn { expr, alias }
    }

    pub fn name(&self) -> String {
        self.alias.clone().unwrap_or_else(|| {
            match &self.expr {
                Expr::Variable(v) => v.name.clone(),
                _ => format!("{:?}", self.expr),
            }
        })
    }
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
        self.columns.iter().map(|c| c.name()).collect()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yield_column_new() {
        let expr = Expr::Variable(crate::query::parser::ast::expr::VariableExpr {
            name: "test_var".to_string(),
            span: Default::default(),
        });
        let column = YieldColumn::new(expr.clone(), Some("alias".to_string()));
        assert_eq!(column.name(), "alias");
    }

    #[test]
    fn test_yield_column_name_without_alias() {
        let expr = Expr::Variable(crate::query::parser::ast::expr::VariableExpr {
            name: "test_var".to_string(),
            span: Default::default(),
        });
        let column = YieldColumn::new(expr, None);
        assert_eq!(column.name(), "test_var");
    }

    #[test]
    fn test_yield_columns_new() {
        let columns = YieldColumns::new();
        assert!(columns.is_empty());
        assert_eq!(columns.len(), 0);
    }

    #[test]
    fn test_yield_columns_with_capacity() {
        let columns = YieldColumns::with_capacity(10);
        assert!(columns.is_empty());
        assert_eq!(columns.len(), 0);
    }

    #[test]
    fn test_yield_columns_add_column() {
        let mut columns = YieldColumns::new();
        let expr = Expr::Variable(crate::query::parser::ast::expr::VariableExpr {
            name: "var1".to_string(),
            span: Default::default(),
        });
        columns.add_column(YieldColumn::new(expr, Some("alias1".to_string())));
        assert_eq!(columns.len(), 1);
        assert_eq!(columns.get_column_names(), vec!["alias1"]);
    }

    #[test]
    fn test_yield_columns_get_column_names() {
        let mut columns = YieldColumns::new();
        let expr1 = Expr::Variable(crate::query::parser::ast::expr::VariableExpr {
            name: "var1".to_string(),
            span: Default::default(),
        });
        let expr2 = Expr::Variable(crate::query::parser::ast::expr::VariableExpr {
            name: "var2".to_string(),
            span: Default::default(),
        });
        columns.add_column(YieldColumn::new(expr1, Some("alias1".to_string())));
        columns.add_column(YieldColumn::new(expr2, Some("alias2".to_string())));
        assert_eq!(columns.get_column_names(), vec!["alias1", "alias2"]);
    }

    #[test]
    fn test_yield_columns_iter() {
        let mut columns = YieldColumns::new();
        let expr = Expr::Variable(crate::query::parser::ast::expr::VariableExpr {
            name: "var".to_string(),
            span: Default::default(),
        });
        columns.add_column(YieldColumn::new(expr, Some("alias".to_string())));
        let mut iter = columns.iter();
        assert!(iter.next().is_some());
        assert!(iter.next().is_none());
    }
}
