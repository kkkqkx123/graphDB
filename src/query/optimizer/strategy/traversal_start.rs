//! 遍历起点选择器模块
//!
//! 用于选择图遍历的最优起点

use std::sync::Arc;
use std::collections::HashMap;

use crate::query::optimizer::cost::{CostCalculator, SelectivityEstimator};
use crate::query::parser::ast::pattern::{Pattern, NodePattern, EdgePattern, PathPattern, PathElement, VariablePattern};
use crate::core::types::Expression;

/// 遍历起点选择器
#[derive(Debug)]
pub struct TraversalStartSelector {
    cost_calculator: Arc<CostCalculator>,
    selectivity_estimator: Arc<SelectivityEstimator>,
    /// 变量绑定上下文，用于解析变量模式
    variable_context: HashMap<String, NodePattern>,
}

/// 候选起点信息
#[derive(Debug, Clone)]
pub struct CandidateStart {
    /// 节点模式
    pub node_pattern: NodePattern,
    /// 估计起始节点数量
    pub estimated_start_nodes: u64,
    /// 估计代价
    pub estimated_cost: f64,
    /// 选择原因
    pub reason: SelectionReason,
}

/// 选择原因
#[derive(Debug, Clone)]
pub enum SelectionReason {
    /// 显式VID指定
    ExplicitVid,
    /// 高选择性索引
    HighSelectivityIndex {
        /// 选择性值
        selectivity: f64,
    },
    /// 标签索引
    TagIndex {
        /// 顶点数量
        vertex_count: u64,
    },
    /// 全表扫描
    FullScan {
        /// 顶点数量
        vertex_count: u64,
    },
    /// 变量绑定
    VariableBinding {
        /// 变量名
        variable_name: String,
    },
}

impl TraversalStartSelector {
    /// 创建新的遍历起点选择器
    pub fn new(
        cost_calculator: Arc<CostCalculator>,
        selectivity_estimator: Arc<SelectivityEstimator>,
    ) -> Self {
        Self {
            cost_calculator,
            selectivity_estimator,
            variable_context: HashMap::new(),
        }
    }

    /// 从模式中选择最优遍历起点
    pub fn select_start_node(&self, pattern: &Pattern) -> Option<CandidateStart> {
        let candidates = self.evaluate_pattern(pattern);

        if candidates.is_empty() {
            return None;
        }

        // 选择代价最小的起点
        candidates.into_iter()
            .min_by(|a, b| {
                a.estimated_cost
                    .partial_cmp(&b.estimated_cost)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    /// 评估模式中的所有候选节点
    fn evaluate_pattern(&self, pattern: &Pattern) -> Vec<CandidateStart> {
        let mut candidates = Vec::new();

        match pattern {
            Pattern::Node(node) => {
                if let Some(candidate) = self.evaluate_node(node) {
                    candidates.push(candidate);
                }
            }
            Pattern::Path(path) => {
                candidates.extend(self.evaluate_path(path));
            }
            Pattern::Edge(edge) => {
                // 边模式可以转换为节点模式作为起点
                // 获取边的源节点或目标节点作为候选
                candidates.extend(self.evaluate_edge_as_start(edge));
            }
            Pattern::Variable(var) => {
                // 变量模式尝试从上下文中解析
                if let Some(candidate) = self.evaluate_variable(var) {
                    candidates.push(candidate);
                }
            }
        }

        candidates
    }

    /// 评估路径模式中的所有节点
    fn evaluate_path(&self, path: &PathPattern) -> Vec<CandidateStart> {
        let mut candidates = Vec::new();

        for element in &path.elements {
            match element {
                PathElement::Node(node) => {
                    if let Some(candidate) = self.evaluate_node(node) {
                        candidates.push(candidate);
                    }
                }
                PathElement::Edge(edge) => {
                    // 边模式可以转换为节点模式作为起点
                    candidates.extend(self.evaluate_edge_as_start(edge));
                }
                PathElement::Alternative(patterns) => {
                    // 评估替代模式中的每个模式
                    for pattern in patterns {
                        candidates.extend(self.evaluate_pattern(pattern));
                    }
                }
                PathElement::Optional(inner) => {
                    match inner.as_ref() {
                        PathElement::Node(node) => {
                            if let Some(candidate) = self.evaluate_node(node) {
                                candidates.push(candidate);
                            }
                        }
                        PathElement::Edge(edge) => {
                            candidates.extend(self.evaluate_edge_as_start(edge));
                        }
                        _ => {}
                    }
                }
                PathElement::Repeated(inner, _) => {
                    match inner.as_ref() {
                        PathElement::Node(node) => {
                            if let Some(candidate) = self.evaluate_node(node) {
                                candidates.push(candidate);
                            }
                        }
                        PathElement::Edge(edge) => {
                            candidates.extend(self.evaluate_edge_as_start(edge));
                        }
                        _ => {}
                    }
                }
            }
        }

        candidates
    }

    /// 将边模式评估为起点候选
    /// 
    /// 边模式本身不能直接作为遍历起点，但可以通过以下方式转换：
    /// 1. 如果有边类型，可以估计边的数量作为参考
    /// 2. 返回一个虚拟的节点模式表示可以从边的任一端开始
    fn evaluate_edge_as_start(&self, edge: &EdgePattern) -> Vec<CandidateStart> {
        let mut candidates = Vec::new();

        // 边模式不能直接作为起点，但我们可以创建一个表示边端点的虚拟节点
        // 这在实际查询规划中可能用于决定遍历方向
        
        // 如果边有类型信息，我们可以基于边统计创建候选
        if let Some(edge_type) = edge.edge_types.first() {
            let edge_stats = self.cost_calculator.statistics_manager().get_edge_stats(edge_type);
            
            if let Some(stats) = edge_stats {
                // 创建一个虚拟节点模式表示边的源端点
                let virtual_node = NodePattern {
                    span: edge.span,
                    variable: edge.variable.clone(),
                    labels: Vec::new(), // 边没有标签，但可能有类型
                    properties: edge.properties.clone(),
                    predicates: edge.predicates.clone(),
                };

                // 基于边统计估计代价
                let estimated_cost = stats.estimate_expand_cost(1) as f64;
                
                candidates.push(CandidateStart {
                    node_pattern: virtual_node,
                    estimated_start_nodes: stats.unique_src_vertices,
                    estimated_cost,
                    reason: SelectionReason::TagIndex { vertex_count: stats.unique_src_vertices },
                });
            }
        }

        candidates
    }

    /// 评估变量模式
    /// 
    /// 从变量上下文中查找变量对应的节点模式
    fn evaluate_variable(&self, var: &VariablePattern) -> Option<CandidateStart> {
        // 在变量上下文中查找变量
        if let Some(node) = self.variable_context.get(&var.name) {
            return self.evaluate_node(node);
        }

        // 如果变量未绑定，创建一个占位候选
        // 这在查询规划的早期阶段可能发生
        let placeholder_node = NodePattern {
            span: var.span,
            variable: Some(var.name.clone()),
            labels: Vec::new(),
            properties: None,
            predicates: Vec::new(),
        };

        Some(CandidateStart {
            node_pattern: placeholder_node,
            estimated_start_nodes: 1000, // 默认估计值
            estimated_cost: 1000.0, // 高代价表示不确定性
            reason: SelectionReason::VariableBinding { variable_name: var.name.clone() },
        })
    }

    /// 评估单个节点
    fn evaluate_node(&self, node: &NodePattern) -> Option<CandidateStart> {
        // 检查是否有显式VID（通过属性或谓词）
        if let Some(vid_selectivity) = self.check_explicit_vid(node) {
            return Some(CandidateStart {
                node_pattern: node.clone(),
                estimated_start_nodes: 1,
                estimated_cost: 1.0 * vid_selectivity,
                reason: SelectionReason::ExplicitVid,
            });
        }

        // 获取标签信息
        let tag_name = node.labels.first()?;

        // 计算选择性
        let selectivity = self.calculate_node_selectivity(node, tag_name);

        // 获取顶点数量
        let vertex_count = self.cost_calculator.statistics_manager().get_vertex_count(tag_name);

        // 计算估计的起始节点数
        let estimated_start_nodes = ((vertex_count as f64 * selectivity) as u64).max(1);

        // 计算扫描代价
        let estimated_cost = if selectivity < 0.1 {
            // 使用索引扫描
            self.cost_calculator.calculate_index_scan_cost(tag_name, "", selectivity)
        } else {
            // 全表扫描
            self.cost_calculator.calculate_scan_vertices_cost(tag_name)
        };

        let reason = if selectivity < 0.1 {
            SelectionReason::HighSelectivityIndex { selectivity }
        } else if vertex_count > 0 {
            SelectionReason::TagIndex { vertex_count }
        } else {
            SelectionReason::FullScan { vertex_count }
        };

        Some(CandidateStart {
            node_pattern: node.clone(),
            estimated_start_nodes,
            estimated_cost,
            reason,
        })
    }

    /// 检查是否有显式VID
    /// 
    /// 检查节点模式中的属性和谓词是否包含显式的VID条件，例如：
    /// - id(v) == "xxx"
    /// - v.id == "xxx"
    /// - {id: "xxx"}
    fn check_explicit_vid(&self, node: &NodePattern) -> Option<f64> {
        // 检查属性中是否有VID条件
        if let Some(props) = &node.properties {
            // 检查属性表达式中是否包含 id 字段
            if self.has_vid_condition(props) {
                return Some(0.01); // VID条件具有极高选择性
            }
        }

        // 检查谓词中是否有VID条件
        for predicate in &node.predicates {
            if self.has_vid_condition(predicate) {
                return Some(0.01);
            }
        }

        None
    }

    /// 检查表达式中是否包含VID条件
    /// 
    /// 识别以下模式：
    /// - id(v) == value
    /// - v.id == value
    /// - {id: value}
    fn has_vid_condition(&self, expr: &Expression) -> bool {
        use crate::core::types::BinaryOperator;

        match expr {
            // 检查是否包含 id() 函数调用
            Expression::Function { name, args } => {
                let name_upper = name.to_uppercase();
                if name_upper == "ID" && !args.is_empty() {
                    return true;
                }
                // 递归检查参数
                args.iter().any(|arg| self.has_vid_condition(arg))
            }
            // 检查属性访问是否为 .id
            Expression::Property { property, .. } => {
                if property.eq_ignore_ascii_case("id") {
                    return true;
                }
                false
            }
            // 检查二元运算中是否包含VID条件
            Expression::Binary { left, right, op } => {
                // 检查是否为等值比较且包含VID
                if matches!(op, BinaryOperator::Equal | BinaryOperator::NotEqual) {
                    if self.is_vid_expression(left) || self.is_vid_expression(right) {
                        return true;
                    }
                }
                // 递归检查
                self.has_vid_condition(left) || self.has_vid_condition(right)
            }
            // 检查Map中是否包含id字段
            Expression::Map(pairs) => {
                pairs.iter().any(|(key, value)| {
                    key.eq_ignore_ascii_case("id") || self.has_vid_condition(value)
                })
            }
            // 对其他表达式类型递归检查子表达式
            _ => {
                expr.children().iter().any(|child| self.has_vid_condition(child))
            }
        }
    }

    /// 判断表达式是否为VID表达式
    /// 
    /// 识别 id() 函数调用或 .id 属性访问
    fn is_vid_expression(&self, expr: &Expression) -> bool {
        match expr {
            Expression::Function { name, args } => {
                let name_upper = name.to_uppercase();
                name_upper == "ID" && !args.is_empty()
            }
            Expression::Property { property, .. } => {
                property.eq_ignore_ascii_case("id")
            }
            _ => false,
        }
    }

    /// 计算节点的选择性
    fn calculate_node_selectivity(&self, node: &NodePattern, tag_name: &str) -> f64 {
        let mut selectivity = 1.0;

        // 从属性条件估计选择性
        if let Some(props) = &node.properties {
            let prop_selectivity = self.selectivity_estimator.estimate_from_expression(
                props,
                Some(tag_name),
            );
            selectivity *= prop_selectivity;
        }

        // 从谓词条件估计选择性
        for predicate in &node.predicates {
            let pred_selectivity = self.selectivity_estimator.estimate_from_expression(
                predicate,
                Some(tag_name),
            );
            selectivity *= pred_selectivity;
        }

        selectivity
    }

    /// 添加变量绑定到上下文
    /// 
    /// 用于在查询规划过程中建立变量与节点模式的映射
    pub fn bind_variable(&mut self, var_name: String, node: NodePattern) {
        self.variable_context.insert(var_name, node);
    }

    /// 清除变量上下文
    pub fn clear_context(&mut self) {
        self.variable_context.clear();
    }
}

impl Clone for TraversalStartSelector {
    fn clone(&self) -> Self {
        Self {
            cost_calculator: self.cost_calculator.clone(),
            selectivity_estimator: self.selectivity_estimator.clone(),
            variable_context: self.variable_context.clone(),
        }
    }
}
