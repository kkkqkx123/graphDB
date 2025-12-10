//! Abstract Syntax Tree Context for representing parsed queries

use std::collections::HashMap;

// 起始顶点信息
#[derive(Debug, Clone)]
pub struct Starts {
    pub from_type: String,           // "instant_expr", "variable", "pipe"
    pub src: Option<String>,         // 源表达式
    pub original_src: Option<String>, // 原始源
    pub user_defined_var_name: String,
    pub runtime_vid_name: String,
    pub vids: Vec<String>,           // 顶点ID列表
}

// 边的类型和方向信息
#[derive(Debug, Clone)]
pub struct Over {
    pub is_over_all: bool,
    pub edge_types: Vec<String>,
    pub direction: String,           // "in", "out", "both"
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
    pub tag_props: HashMap<String, Vec<String>>,      // 标签属性
    pub edge_props: HashMap<String, Vec<String>>,     // 边属性
    pub dst_tag_props: HashMap<String, Vec<String>>,  // 目标标签属性
    pub src_tag_props: HashMap<String, Vec<String>>,  // 源标签属性
}

// 基础AST上下文
#[derive(Debug, Clone)]
pub struct AstContext {
    statement_type: String,
    #[allow(dead_code)]
    query_text: String,
    contains_path: bool,
}

impl AstContext {
    pub fn new(statement_type: &str, query_text: &str) -> Self {
        Self {
            statement_type: statement_type.to_string(),
            query_text: query_text.to_string(),
            contains_path: query_text.to_lowercase().contains("path"),
        }
    }

    pub fn statement_type(&self) -> &str {
        &self.statement_type
    }

    pub fn contains_path_query(&self) -> bool {
        self.contains_path
    }
}

impl Default for AstContext {
    fn default() -> Self {
        Self {
            statement_type: "UNKNOWN".to_string(),
            query_text: "".to_string(),
            contains_path: false,
        }
    }
}

// GO查询上下文
#[derive(Debug, Clone)]
pub struct GoContext {
    pub base: AstContext,
    pub from: Starts,
    pub steps: StepClause,
    pub over: Over,
    pub filter: Option<String>,
    pub yield_expr: Option<String>,
    pub distinct: bool,
    pub random: bool,
    pub limits: Vec<i64>,
    pub col_names: Vec<String>,
    pub vids_var: String,
    pub join_input: bool,
    pub join_dst: bool,
    pub is_simple: bool,
    pub expr_props: ExpressionProps,
    pub dst_props_expr: Option<String>,
    pub src_props_expr: Option<String>,
    pub edge_props_expr: Option<String>,
    pub src_vid_col_name: String,
    pub dst_vid_col_name: String,
    pub input_var_name: String,
}

impl GoContext {
    pub fn new(base: AstContext) -> Self {
        Self {
            base,
            from: Starts {
                from_type: "instant_expr".to_string(),
                src: None,
                original_src: None,
                user_defined_var_name: String::new(),
                runtime_vid_name: String::new(),
                vids: Vec::new(),
            },
            steps: StepClause {
                m_steps: 1,
                n_steps: 1,
                is_m_to_n: false,
            },
            over: Over {
                is_over_all: false,
                edge_types: Vec::new(),
                direction: "out".to_string(),
                all_edges: Vec::new(),
            },
            filter: None,
            yield_expr: None,
            distinct: false,
            random: false,
            limits: Vec::new(),
            col_names: Vec::new(),
            vids_var: String::new(),
            join_input: false,
            join_dst: false,
            is_simple: false,
            expr_props: ExpressionProps::default(),
            dst_props_expr: None,
            src_props_expr: None,
            edge_props_expr: None,
            src_vid_col_name: String::new(),
            dst_vid_col_name: String::new(),
            input_var_name: String::new(),
        }
    }
}

// Fetch Vertices查询上下文
#[derive(Debug, Clone)]
pub struct FetchVerticesContext {
    pub base: AstContext,
    pub from: Starts,
    pub distinct: bool,
    pub yield_expr: Option<String>,
    pub expr_props: ExpressionProps,
    pub input_var_name: String,
}

impl FetchVerticesContext {
    pub fn new(base: AstContext) -> Self {
        Self {
            base,
            from: Starts {
                from_type: "instant_expr".to_string(),
                src: None,
                original_src: None,
                user_defined_var_name: String::new(),
                runtime_vid_name: String::new(),
                vids: Vec::new(),
            },
            distinct: false,
            yield_expr: None,
            expr_props: ExpressionProps::default(),
            input_var_name: String::new(),
        }
    }
}

// Fetch Edges查询上下文
#[derive(Debug, Clone)]
pub struct FetchEdgesContext {
    pub base: AstContext,
    pub src: Option<String>,
    pub dst: Option<String>,
    pub rank: Option<String>,
    pub edge_type: Option<String>,
    pub expr_props: ExpressionProps,
    pub yield_expr: Option<String>,
    pub edge_name: String,
    pub distinct: bool,
    pub input_var_name: String,
}

impl FetchEdgesContext {
    pub fn new(base: AstContext) -> Self {
        Self {
            base,
            src: None,
            dst: None,
            rank: None,
            edge_type: None,
            expr_props: ExpressionProps::default(),
            yield_expr: None,
            edge_name: String::new(),
            distinct: false,
            input_var_name: String::new(),
        }
    }
}

// Lookup查询上下文
#[derive(Debug, Clone)]
pub struct LookupContext {
    pub base: AstContext,
    pub is_edge: bool,
    pub dedup: bool,
    pub schema_id: i32,
    pub filter: Option<String>,
    pub yield_expr: Option<String>,
    pub idx_return_cols: Vec<String>,
    pub idx_col_names: Vec<String>,
    pub is_fulltext_index: bool,
    pub has_score: bool,
    pub fulltext_expr: Option<String>,
}

impl LookupContext {
    pub fn new(base: AstContext) -> Self {
        Self {
            base,
            is_edge: false,
            dedup: false,
            schema_id: -1,
            filter: None,
            yield_expr: None,
            idx_return_cols: Vec::new(),
            idx_col_names: Vec::new(),
            is_fulltext_index: false,
            has_score: false,
            fulltext_expr: None,
        }
    }
}

// Path查询上下文
#[derive(Debug, Clone)]
pub struct PathContext {
    pub base: AstContext,
    pub from: Starts,
    pub to: Starts,
    pub steps: StepClause,
    pub over: Over,
    pub limit: i64,
    pub filter: Option<String>,
    pub col_names: Vec<String>,
    pub from_vids_var: String,
    pub to_vids_var: String,
    pub is_shortest: bool,
    pub single_shortest: bool,
    pub is_weight: bool,
    pub no_loop: bool,
    pub with_prop: bool,
    pub runtime_from_project: Option<String>,
    pub runtime_from_dedup: Option<String>,
    pub runtime_to_project: Option<String>,
    pub runtime_to_dedup: Option<String>,
    pub input_var_name: String,
    pub expr_props: ExpressionProps,
}

impl PathContext {
    pub fn new(base: AstContext) -> Self {
        Self {
            base,
            from: Starts {
                from_type: "instant_expr".to_string(),
                src: None,
                original_src: None,
                user_defined_var_name: String::new(),
                runtime_vid_name: String::new(),
                vids: Vec::new(),
            },
            to: Starts {
                from_type: "instant_expr".to_string(),
                src: None,
                original_src: None,
                user_defined_var_name: String::new(),
                runtime_vid_name: String::new(),
                vids: Vec::new(),
            },
            steps: StepClause {
                m_steps: 1,
                n_steps: 1,
                is_m_to_n: false,
            },
            over: Over {
                is_over_all: false,
                edge_types: Vec::new(),
                direction: "out".to_string(),
                all_edges: Vec::new(),
            },
            limit: -1,
            filter: None,
            col_names: Vec::new(),
            from_vids_var: String::new(),
            to_vids_var: String::new(),
            is_shortest: false,
            single_shortest: false,
            is_weight: false,
            no_loop: false,
            with_prop: false,
            runtime_from_project: None,
            runtime_from_dedup: None,
            runtime_to_project: None,
            runtime_to_dedup: None,
            input_var_name: String::new(),
            expr_props: ExpressionProps::default(),
        }
    }
}

// Subgraph查询上下文
#[derive(Debug, Clone)]
pub struct SubgraphContext {
    pub base: AstContext,
    pub from: Starts,
    pub steps: StepClause,
    pub loop_steps: String,
    pub filter: Option<String>,
    pub tag_filter: Option<String>,
    pub edge_filter: Option<String>,
    pub col_names: Vec<String>,
    pub edge_names: Vec<String>,
    pub edge_types: Vec<String>,
    pub bi_direct_edge_types: Vec<String>,
    pub col_type: Vec<String>,
    pub expr_props: ExpressionProps,
    pub with_prop: bool,
    pub get_vertex_prop: bool,
    pub get_edge_prop: bool,
}

impl SubgraphContext {
    pub fn new(base: AstContext) -> Self {
        Self {
            base,
            from: Starts {
                from_type: "instant_expr".to_string(),
                src: None,
                original_src: None,
                user_defined_var_name: String::new(),
                runtime_vid_name: String::new(),
                vids: Vec::new(),
            },
            steps: StepClause {
                m_steps: 1,
                n_steps: 1,
                is_m_to_n: false,
            },
            loop_steps: String::new(),
            filter: None,
            tag_filter: None,
            edge_filter: None,
            col_names: Vec::new(),
            edge_names: Vec::new(),
            edge_types: Vec::new(),
            bi_direct_edge_types: Vec::new(),
            col_type: Vec::new(),
            expr_props: ExpressionProps::default(),
            with_prop: false,
            get_vertex_prop: false,
            get_edge_prop: false,
        }
    }
}

// 维护操作查询上下文
#[derive(Debug, Clone)]
pub struct MaintainContext {
    pub base: AstContext,
    // 可以根据具体维护操作添加更多字段
}