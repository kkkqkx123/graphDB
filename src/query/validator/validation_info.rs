//! 验证结果信息模块
//!
//! 本模块定义验证阶段产生的结构化信息，用于传递给规划阶段
//! 避免规划器重复解析 AST

use std::collections::HashMap;

use crate::core::Expression;
use crate::core::types::Span;

use super::structs::AliasType;
use super::validator_trait::ValueType;

/// 验证后的语句包装器
/// 包含原始语句和验证信息
#[derive(Debug, Clone)]
pub struct ValidatedStatement {
    /// 原始 AST 语句
    pub stmt: crate::query::parser::ast::Stmt,
    /// 验证阶段收集的信息
    pub validation_info: ValidationInfo,
}

impl ValidatedStatement {
    /// 创建新的验证后语句
    pub fn new(stmt: crate::query::parser::ast::Stmt, validation_info: ValidationInfo) -> Self {
        Self {
            stmt,
            validation_info,
        }
    }

    /// 获取语句类型
    pub fn statement_type(&self) -> &'static str {
        self.stmt.kind()
    }

    /// 获取别名映射
    pub fn alias_map(&self) -> &HashMap<String, AliasType> {
        &self.validation_info.alias_map
    }
}

/// 表达式类型信息
/// 使用表达式字符串表示作为键
#[derive(Debug, Clone)]
pub struct ExprTypeInfo {
    /// 表达式字符串表示
    pub expr_str: String,
    /// 表达式类型
    pub value_type: ValueType,
}

/// 验证信息结构体
/// 包含验证阶段收集的所有有用信息
#[derive(Debug, Clone, Default)]
pub struct ValidationInfo {
    /// 别名映射（变量名 -> 类型）
    pub alias_map: HashMap<String, AliasType>,

    /// 表达式类型推断结果（使用字符串键）
    pub expr_types: HashMap<String, ValueType>,

    /// 路径分析结果
    pub path_analysis: Vec<PathAnalysis>,

    /// 优化提示
    pub optimization_hints: Vec<OptimizationHint>,

    /// 变量定义位置
    pub variable_definitions: HashMap<String, Span>,

    /// 使用的索引信息
    pub index_hints: Vec<IndexHint>,

    /// 验证通过的子句
    pub validated_clauses: Vec<ClauseKind>,

    /// 语义分析结果
    pub semantic_info: SemanticInfo,
}

impl ValidationInfo {
    /// 创建空的验证信息
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加别名
    pub fn add_alias(&mut self, name: String, alias_type: AliasType) {
        self.alias_map.insert(name, alias_type);
    }

    /// 添加表达式类型
    /// 使用表达式字符串表示作为键
    pub fn add_expr_type(&mut self, expr: &Expression, value_type: ValueType) {
        let expr_str = format!("{:?}", expr);
        self.expr_types.insert(expr_str, value_type);
    }

    /// 根据表达式字符串获取类型
    pub fn get_expr_type(&self, expr: &Expression) -> Option<&ValueType> {
        let expr_str = format!("{:?}", expr);
        self.expr_types.get(&expr_str)
    }

    /// 添加路径分析
    pub fn add_path_analysis(&mut self, analysis: PathAnalysis) {
        self.path_analysis.push(analysis);
    }

    /// 添加优化提示
    pub fn add_optimization_hint(&mut self, hint: OptimizationHint) {
        self.optimization_hints.push(hint);
    }

    /// 添加索引提示
    pub fn add_index_hint(&mut self, hint: IndexHint) {
        self.index_hints.push(hint);
    }

    /// 获取变量的类型
    pub fn get_alias_type(&self, name: &str) -> Option<&AliasType> {
        self.alias_map.get(name)
    }

    /// 检查变量是否为节点类型
    pub fn is_node_variable(&self, name: &str) -> bool {
        matches!(
            self.alias_map.get(name),
            Some(AliasType::Node) | Some(AliasType::NodeList)
        )
    }

    /// 检查变量是否为边类型
    pub fn is_edge_variable(&self, name: &str) -> bool {
        matches!(
            self.alias_map.get(name),
            Some(AliasType::Edge) | Some(AliasType::EdgeList)
        )
    }
}

/// 路径分析信息
#[derive(Debug, Clone)]
pub struct PathAnalysis {
    /// 路径别名
    pub alias: Option<String>,
    /// 节点数量
    pub node_count: usize,
    /// 边数量
    pub edge_count: usize,
    /// 是否有方向
    pub has_direction: bool,
    /// 最小跳数
    pub min_hops: Option<usize>,
    /// 最大跳数
    pub max_hops: Option<usize>,
    /// 路径中的变量
    pub variables: Vec<String>,
    /// 路径中的标签
    pub labels: Vec<String>,
    /// 路径中的边类型
    pub edge_types: Vec<String>,
}

impl PathAnalysis {
    /// 创建新的路径分析
    pub fn new() -> Self {
        Self {
            alias: None,
            node_count: 0,
            edge_count: 0,
            has_direction: true,
            min_hops: None,
            max_hops: None,
            variables: Vec::new(),
            labels: Vec::new(),
            edge_types: Vec::new(),
        }
    }
}

impl Default for PathAnalysis {
    fn default() -> Self {
        Self::new()
    }
}

/// 优化提示类型
#[derive(Debug, Clone)]
pub enum OptimizationHint {
    /// 建议使用索引扫描
    UseIndexScan {
        table: String,
        column: String,
        condition: Expression,
    },
    /// 建议限制结果数量
    LimitResults {
        reason: String,
        suggested_limit: usize,
    },
    /// 建议预过滤
    PreFilter {
        condition: Expression,
        selectivity: f64,
    },
    /// 建议连接顺序
    JoinOrder {
        optimal_order: Vec<String>,
        estimated_cost: f64,
    },
    /// 提示可能的性能问题
    PerformanceWarning {
        message: String,
        severity: HintSeverity,
    },
}

/// 提示严重程度
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HintSeverity {
    Info,
    Warning,
    Critical,
}

/// 索引提示
#[derive(Debug, Clone)]
pub struct IndexHint {
    /// 索引名称
    pub index_name: String,
    /// 表/标签名
    pub table_name: String,
    /// 索引列
    pub columns: Vec<String>,
    /// 适用条件
    pub applicable_conditions: Vec<Expression>,
    /// 预估选择性
    pub estimated_selectivity: f64,
}

/// 子句类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClauseKind {
    Match,
    Where,
    Return,
    OrderBy,
    Limit,
    Skip,
    With,
    Unwind,
    Create,
    Delete,
    Set,
    Remove,
    Yield,
    Go,
    Over,
    From,
}

/// 语义信息
#[derive(Debug, Clone, Default)]
pub struct SemanticInfo {
    /// 引用的标签
    pub referenced_tags: Vec<String>,
    /// 引用的边类型
    pub referenced_edges: Vec<String>,
    /// 引用的属性
    pub referenced_properties: Vec<String>,
    /// 使用的变量
    pub used_variables: Vec<String>,
    /// 定义的变量
    pub defined_variables: Vec<String>,
    /// 聚合函数调用
    pub aggregate_calls: Vec<AggregateCallInfo>,
}

/// 聚合函数调用信息
#[derive(Debug, Clone)]
pub struct AggregateCallInfo {
    /// 函数名
    pub function_name: String,
    /// 参数表达式
    pub arguments: Vec<Expression>,
    /// 是否去重
    pub distinct: bool,
    /// 别名
    pub alias: Option<String>,
}


