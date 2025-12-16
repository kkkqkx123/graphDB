//! AST上下文中的共享结构定义

use std::collections::HashMap;

// 起始顶点信息
#[derive(Debug, Clone)]
pub struct Starts {
    pub from_type: String,            // "instant_expr", "variable", "pipe"
    pub src: Option<String>,          // 源表达式
    pub original_src: Option<String>, // 原始源
    pub user_defined_var_name: String,
    pub runtime_vid_name: String,
    pub vids: Vec<String>, // 顶点ID列表
}

// 边的类型和方向信息
#[derive(Debug, Clone)]
pub struct Over {
    pub is_over_all: bool,
    pub edge_types: Vec<String>,
    pub direction: String, // "in", "out", "both"
    pub all_edges: Vec<String>,
}

// 步数限制信息
#[derive(Debug, Clone)]
pub struct StepClause {
    pub m_steps: usize,
    pub n_steps: usize,
    pub is_m_to_n: bool,
}

// 表达式属性信息
#[derive(Debug, Clone, Default)]
pub struct ExpressionProps {
    pub tag_props: HashMap<String, Vec<String>>,  // 标签属性
    pub edge_props: HashMap<String, Vec<String>>, // 边属性
    pub dst_tag_props: HashMap<String, Vec<String>>, // 目标标签属性
    pub src_tag_props: HashMap<String, Vec<String>>, // 源标签属性
}
