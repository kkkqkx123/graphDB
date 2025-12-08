//! MatchValidator用到的结构定义

use std::collections::{HashMap, HashSet};
use crate::graph::expression::expr_type::Expression;

/// Cypher查询中的别名类型
#[derive(Debug, Clone, PartialEq)]
pub enum AliasType {
    Node,
    Edge,
    EdgeList,
    Path,
    Variable,
    Runtime,
}

/// Node信息
#[derive(Debug, Clone)]
pub struct NodeInfo {
    pub alias: String,
    pub labels: Vec<String>,
    pub props: Option<Expression>,
    pub anonymous: bool,
    pub filter: Option<Expression>, // 节点过滤条件
    pub tids: Vec<i32>, // 标签ID列表
    pub label_props: Vec<Option<Expression>>, // 标签属性
}

/// Edge信息
#[derive(Debug, Clone)]
pub struct EdgeInfo {
    pub alias: String,
    pub inner_alias: String, // 内部别名
    pub types: Vec<String>,
    pub props: Option<Expression>,
    pub anonymous: bool,
    pub filter: Option<Expression>, // 边过滤条件
    pub direction: Direction, // 边方向
    pub range: Option<MatchStepRange>, // 步数范围
    pub edge_types: Vec<i32>, // 边类型ID
}

/// 边的方向
#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Forward,  // ->
    Backward, // <-
    Bidirectional, // -
}

/// 路径步数范围
#[derive(Debug, Clone)]
pub struct MatchStepRange {
    pub min: u32,
    pub max: u32,
}

impl MatchStepRange {
    pub fn new(min: u32, max: u32) -> Self {
        MatchStepRange { min, max }
    }
    
    pub fn min(&self) -> u32 {
        self.min
    }
    
    pub fn max(&self) -> u32 {
        self.max
    }
}

/// 路径信息
#[derive(Debug, Clone)]
pub struct Path {
    pub alias: String,
    pub anonymous: bool,
    pub gen_path: bool,  // 是否生成路径
    pub path_type: PathType,
    pub node_infos: Vec<NodeInfo>,
    pub edge_infos: Vec<EdgeInfo>,
    pub path_build: Option<Expression>, // 路径构建表达式
    pub is_pred: bool, // 是否为谓词
    pub is_anti_pred: bool, // 是否为反向谓词
    pub compare_variables: Vec<String>, // 比较变量
    pub collect_variable: String, // 收集变量
    pub roll_up_apply: bool, // 是否应用RollUp
}

#[derive(Debug, Clone, Copy)]
pub enum PathType {
    Default,
    Shortest,
    AllShortest,
    SingleSourceShortest,
    SingleSourceAllShortest,
}

/// 查询部分
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

#[derive(Debug, Clone)]
pub struct MatchClauseContext {
    pub paths: Vec<Path>,
    pub aliases_available: HashMap<String, AliasType>,
    pub aliases_generated: HashMap<String, AliasType>,
    pub where_clause: Option<WhereClauseContext>,
    pub is_optional: bool,
    pub skip: Option<Expression>,
    pub limit: Option<Expression>,
}

#[derive(Debug, Clone)]
pub struct WhereClauseContext {
    pub filter: Option<Expression>,
    pub aliases_available: HashMap<String, AliasType>,
    pub aliases_generated: HashMap<String, AliasType>,
    pub paths: Vec<Path>, // WHERE子句中可能包含的路径
}

#[derive(Debug, Clone)]
pub struct ReturnClauseContext {
    pub yield_clause: YieldClauseContext,
    pub aliases_available: HashMap<String, AliasType>,
    pub aliases_generated: HashMap<String, AliasType>,
    pub pagination: Option<PaginationContext>,
    pub order_by: Option<OrderByClauseContext>,
    pub distinct: bool,
}

#[derive(Debug, Clone)]
pub struct WithClauseContext {
    pub yield_clause: YieldClauseContext,
    pub aliases_available: HashMap<String, AliasType>,
    pub aliases_generated: HashMap<String, AliasType>,
    pub where_clause: Option<WhereClauseContext>,
    pub pagination: Option<PaginationContext>,
    pub order_by: Option<OrderByClauseContext>,
    pub distinct: bool,
}

#[derive(Debug, Clone)]
pub struct UnwindClauseContext {
    pub alias: String,
    pub unwind_expr: Expression,
    pub aliases_available: HashMap<String, AliasType>,
    pub aliases_generated: HashMap<String, AliasType>,
    pub paths: Vec<Path>, // Unwind子句中可能包含的路径
}

#[derive(Debug, Clone)]
pub struct PaginationContext {
    pub skip: i64,
    pub limit: i64,
}

#[derive(Debug, Clone)]
pub struct OrderByClauseContext {
    pub indexed_order_factors: Vec<(usize, OrderType)>,
}

#[derive(Debug, Clone)]
pub enum OrderType {
    Asc,
    Desc,
}

#[derive(Debug, Clone)]
pub struct YieldClauseContext {
    pub yield_columns: Vec<YieldColumn>,
    pub aliases_available: HashMap<String, AliasType>,
    pub aliases_generated: HashMap<String, AliasType>,
    pub distinct: bool,
    pub has_agg: bool,
    pub group_keys: Vec<Expression>,
    pub group_items: Vec<Expression>,
    pub need_gen_project: bool,
    pub agg_output_column_names: Vec<String>,
    pub proj_output_column_names: Vec<String>,
    pub proj_cols: Vec<YieldColumn>,
    pub paths: Vec<Path>,
}

#[derive(Debug, Clone)]
pub struct YieldColumn {
    pub expr: Expression,
    pub alias: String,
    pub is_matched: bool, // 是否已匹配
}

impl YieldColumn {
    pub fn new(expr: Expression, alias: String) -> Self {
        YieldColumn {
            expr,
            alias,
            is_matched: false,
        }
    }
    
    pub fn name(&self) -> &str {
        &self.alias
    }
}