//! 路径相关数据结构

use crate::core::Expression;
use crate::core::types::expression::contextual::ContextualExpression;

/// 路径信息
#[derive(Debug, Clone)]
pub struct Path {
    pub alias: String,
    pub anonymous: bool,
    pub gen_path: bool, // 是否生成路径
    pub path_type: PathYieldType,
    pub node_infos: Vec<NodeInfo>,
    pub edge_infos: Vec<EdgeInfo>,
    pub path_build: Option<Expression>, // 路径构建表达式
    pub is_pred: bool,                  // 是否为谓词
    pub is_anti_pred: bool,             // 是否为反向谓词
    pub compare_variables: Vec<String>, // 比较变量
    pub collect_variable: String,       // 收集变量
    pub roll_up_apply: bool,            // 是否应用RollUp
}

impl Path {
    /// 检查是否为默认路径类型
    pub fn is_default_path(&self) -> bool {
        matches!(self.path_type, PathYieldType::Default)
    }

    /// 获取节点信息列表
    pub fn node_infos(&self) -> &[NodeInfo] {
        &self.node_infos
    }

    /// 获取边信息列表
    pub fn edge_infos(&self) -> &[EdgeInfo] {
        &self.edge_infos
    }
}

/// 路径类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PathYieldType {
    Default,
    Shortest,
    AllShortest,
    SingleSourceShortest,
    SingleSourceAllShortest,
}

/// Node信息
#[derive(Debug, Clone)]
pub struct NodeInfo {
    pub alias: String,
    pub labels: Vec<String>,
    pub props: Option<Expression>,
    pub anonymous: bool,
    pub filter: Option<Expression>,           // 节点过滤条件
    pub tids: Vec<i32>,                       // 标签ID列表
    pub label_props: Vec<Option<Expression>>, // 标签属性
}

impl Default for NodeInfo {
    fn default() -> Self {
        Self {
            alias: String::new(),
            labels: Vec::new(),
            props: None,
            anonymous: false,
            filter: None,
            tids: Vec::new(),
            label_props: Vec::new(),
        }
    }
}

/// Edge信息
#[derive(Debug, Clone)]
pub struct EdgeInfo {
    pub alias: String,
    pub inner_alias: String, // 内部别名
    pub types: Vec<String>,
    pub props: Option<ContextualExpression>,
    pub anonymous: bool,
    pub filter: Option<ContextualExpression>,    // 边过滤条件
    pub direction: Direction,          // 边方向
    pub range: Option<MatchStepRange>, // 步数范围
    pub edge_types: Vec<i32>,          // 边类型ID
}

/// 边的方向
#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Forward,       // ->
    Backward,      // <-
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
