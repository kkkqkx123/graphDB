//! 索引选择器模块
//!
//! 用于为查询选择最优索引

use std::sync::Arc;

use crate::query::optimizer::cost::{CostCalculator, SelectivityEstimator};
use crate::index::Index;
use crate::core::types::Expression;

/// 索引选择器
#[derive(Debug)]
pub struct IndexSelector {
    cost_calculator: Arc<CostCalculator>,
    selectivity_estimator: Arc<SelectivityEstimator>,
}

/// 索引选择结果
#[derive(Debug, Clone)]
pub enum IndexSelection {
    /// 属性索引
    PropertyIndex {
        /// 索引名称
        index_name: String,
        /// 属性名称
        property_name: String,
        /// 估计代价
        estimated_cost: f64,
        /// 选择性
        selectivity: f64,
    },
    /// 标签索引
    TagIndex {
        /// 估计代价
        estimated_cost: f64,
        /// 顶点数量
        vertex_count: u64,
    },
    /// 全表扫描
    FullScan {
        /// 估计代价
        estimated_cost: f64,
        /// 顶点数量
        vertex_count: u64,
    },
}

/// 属性谓词
#[derive(Debug, Clone)]
pub struct PropertyPredicate {
    /// 属性名称
    pub property_name: String,
    /// 操作符
    pub operator: PredicateOperator,
    /// 值表达式
    pub value: Expression,
}

/// 谓词操作符
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PredicateOperator {
    /// 等于
    Equal,
    /// 不等于
    NotEqual,
    /// 小于
    LessThan,
    /// 小于等于
    LessThanOrEqual,
    /// 大于
    GreaterThan,
    /// 大于等于
    GreaterThanOrEqual,
    /// LIKE
    Like,
    /// IN
    In,
}

impl IndexSelector {
    /// 创建新的索引选择器
    pub fn new(
        cost_calculator: Arc<CostCalculator>,
        selectivity_estimator: Arc<SelectivityEstimator>,
    ) -> Self {
        Self {
            cost_calculator,
            selectivity_estimator,
        }
    }

    /// 为查询选择最优索引
    pub fn select_index(
        &self,
        tag_name: &str,
        predicates: &[PropertyPredicate],
        available_indexes: &[Index],
    ) -> IndexSelection {
        // 如果没有谓词，使用全表扫描
        if predicates.is_empty() {
            let vertex_count = self.cost_calculator.statistics_manager().get_vertex_count(tag_name);
            let estimated_cost = self.cost_calculator.calculate_scan_vertices_cost(tag_name);
            return IndexSelection::FullScan {
                estimated_cost,
                vertex_count,
            };
        }

        // 评估每个可用索引
        let mut best_selection: Option<IndexSelection> = None;

        for index in available_indexes {
            // 只考虑与标签匹配的索引
            if index.schema_name != tag_name {
                continue;
            }

            if let Some(selection) = self.evaluate_index(index, predicates) {
                match &best_selection {
                    None => best_selection = Some(selection),
                    Some(current_best) => {
                        if selection.estimated_cost() < current_best.estimated_cost() {
                            best_selection = Some(selection);
                        }
                    }
                }
            }
        }

        // 如果没有找到合适的索引，使用全表扫描
        best_selection.unwrap_or_else(|| {
            let vertex_count = self.cost_calculator.statistics_manager().get_vertex_count(tag_name);
            let estimated_cost = self.cost_calculator.calculate_scan_vertices_cost(tag_name);
            IndexSelection::FullScan {
                estimated_cost,
                vertex_count,
            }
        })
    }

    /// 评估单个索引
    fn evaluate_index(
        &self,
        index: &Index,
        predicates: &[PropertyPredicate],
    ) -> Option<IndexSelection> {
        // 检查索引是否覆盖谓词
        let covered_predicates: Vec<&PropertyPredicate> = predicates
            .iter()
            .filter(|p| index.properties.contains(&p.property_name))
            .collect();

        if covered_predicates.is_empty() {
            return None;
        }

        // 计算选择性
        let mut total_selectivity = 1.0;
        for predicate in &covered_predicates {
            let selectivity = match predicate.operator {
                PredicateOperator::Equal => {
                    self.selectivity_estimator.estimate_equality_selectivity(
                        Some(&index.schema_name),
                        &predicate.property_name,
                    )
                }
                PredicateOperator::LessThan | PredicateOperator::LessThanOrEqual => {
                    self.selectivity_estimator.estimate_less_than_selectivity(None)
                }
                PredicateOperator::GreaterThan | PredicateOperator::GreaterThanOrEqual => {
                    self.selectivity_estimator.estimate_greater_than_selectivity(None)
                }
                PredicateOperator::Like => {
                    // 尝试从表达式中提取模式
                    if let Expression::Literal(value) = &predicate.value {
                        if let crate::core::value::Value::String(pattern) = value {
                            self.selectivity_estimator.estimate_like_selectivity(pattern)
                        } else {
                            0.3
                        }
                    } else {
                        0.3
                    }
                }
                _ => 0.3,
            };
            total_selectivity *= selectivity;
        }

        // 计算代价
        let estimated_cost = self.cost_calculator.calculate_index_scan_cost(
            &index.schema_name,
            &covered_predicates[0].property_name,
            total_selectivity,
        );

        // 获取第一个覆盖的属性名
        let property_name = covered_predicates[0].property_name.clone();

        Some(IndexSelection::PropertyIndex {
            index_name: index.name.clone(),
            property_name,
            estimated_cost,
            selectivity: total_selectivity,
        })
    }

    /// 选择最优的复合索引策略
    pub fn select_composite_index_strategy(
        &self,
        tag_name: &str,
        predicates: &[PropertyPredicate],
        available_indexes: &[Index],
    ) -> Vec<IndexSelection> {
        let mut strategies = Vec::new();

        // 添加全表扫描作为基准
        let vertex_count = self.cost_calculator.statistics_manager().get_vertex_count(tag_name);
        let full_scan_cost = self.cost_calculator.calculate_scan_vertices_cost(tag_name);
        strategies.push(IndexSelection::FullScan {
            estimated_cost: full_scan_cost,
            vertex_count,
        });

        // 评估每个索引
        for index in available_indexes {
            if index.schema_name != tag_name {
                continue;
            }

            if let Some(selection) = self.evaluate_index(index, predicates) {
                strategies.push(selection);
            }
        }

        // 按代价排序
        strategies.sort_by(|a, b| {
            a.estimated_cost()
                .partial_cmp(&b.estimated_cost())
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        strategies
    }
}

impl Clone for IndexSelector {
    fn clone(&self) -> Self {
        Self {
            cost_calculator: self.cost_calculator.clone(),
            selectivity_estimator: self.selectivity_estimator.clone(),
        }
    }
}

impl IndexSelection {
    /// 获取估计代价
    pub fn estimated_cost(&self) -> f64 {
        match self {
            IndexSelection::PropertyIndex { estimated_cost, .. } => *estimated_cost,
            IndexSelection::TagIndex { estimated_cost, .. } => *estimated_cost,
            IndexSelection::FullScan { estimated_cost, .. } => *estimated_cost,
        }
    }

    /// 获取选择性（如果有）
    pub fn selectivity(&self) -> Option<f64> {
        match self {
            IndexSelection::PropertyIndex { selectivity, .. } => Some(*selectivity),
            _ => None,
        }
    }

    /// 判断是否为索引扫描
    pub fn is_index_scan(&self) -> bool {
        matches!(self, IndexSelection::PropertyIndex { .. })
    }

    /// 判断是否为全表扫描
    pub fn is_full_scan(&self) -> bool {
        matches!(self, IndexSelection::FullScan { .. })
    }
}
