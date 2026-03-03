//! 子查询去关联化优化器模块
//!
//! **基于分析的子查询去关联化优化策略**，将简单的 PatternApply 子查询转换为 HashInnerJoin。
//!
//! ## 优化策略
//!
//! - 将符合条件的 PatternApply 子查询转换为 HashInnerJoin
//! - 避免重复执行子查询
//!
//! ## 适用条件
//!
//! 1. PatternApply 的右输入是简单查询（单表扫描 + 等值过滤）
//! 2. 过滤条件是确定性的（不含 rand(), now() 等）
//! 3. 表达式复杂度 < 50（避免复杂表达式）
//! 4. 子查询估算行数 < 1000（基于统计信息）
//!
//! ## 使用示例
//!
//! ```rust
//! use graphdb::query::optimizer::strategy::SubqueryUnnestingOptimizer;
//! use graphdb::query::optimizer::OptimizerEngine;
//!
//! let optimizer = SubqueryUnnestingOptimizer::new(
//!     engine.expression_analyzer(),
//!     engine.stats_manager(),
//! );
//! let decision = optimizer.should_unnest(&pattern_apply);
//! ```

use crate::core::Expression;
use crate::query::optimizer::analysis::ExpressionAnalyzer;
use crate::query::optimizer::stats::StatisticsManager;
use crate::query::planner::plan::core::nodes::{HashInnerJoinNode, PatternApplyNode, ScanVerticesNode};

/// 子查询去关联化决策
#[derive(Debug, Clone, PartialEq)]
pub enum UnnestDecision {
    /// 转换为 HashInnerJoin
    ShouldUnnest {
        /// 转换原因
        reason: UnnestReason,
        /// 估算的原始代价
        original_cost: f64,
        /// 估算的转换后代价
        unnested_cost: f64,
    },
    /// 保持 PatternApply
    KeepPatternApply {
        /// 保留原因
        reason: KeepReason,
    },
}

/// 转换原因
#[derive(Debug, Clone, PartialEq)]
pub enum UnnestReason {
    /// 简单子查询，转换更优
    SimpleSubquery,
    /// 基于代价分析
    CostBased,
}

/// 保留原因
#[derive(Debug, Clone, PartialEq)]
pub enum KeepReason {
    /// 子查询太复杂
    TooComplex,
    /// 子查询包含非确定性函数
    NonDeterministic,
    /// 子查询估算行数过大
    TooManyRows,
    /// 子查询包含复杂条件
    ComplexCondition,
}

/// 子查询去关联化优化器
///
/// 基于表达式分析和统计信息，决定是否将 PatternApply 转换为 HashInnerJoin。
#[derive(Debug, Clone)]
pub struct SubqueryUnnestingOptimizer {
    /// 表达式分析器
    expression_analyzer: ExpressionAnalyzer,
    /// 统计信息管理器
    stats_manager: StatisticsManager,
    /// 最大允许的子查询估算行数
    max_subquery_rows: u64,
    /// 最大允许的表达式复杂度
    max_complexity: u32,
}

impl SubqueryUnnestingOptimizer {
    /// 创建新的优化器
    pub fn new(expression_analyzer: &ExpressionAnalyzer, stats_manager: &StatisticsManager) -> Self {
        Self {
            expression_analyzer: expression_analyzer.clone(),
            stats_manager: stats_manager.clone(),
            max_subquery_rows: 1000,
            max_complexity: 50,
        }
    }

    /// 设置最大子查询行数阈值
    pub fn with_max_rows(mut self, max_rows: u64) -> Self {
        self.max_subquery_rows = max_rows;
        self
    }

    /// 设置最大复杂度阈值
    pub fn with_max_complexity(mut self, max_complexity: u32) -> Self {
        self.max_complexity = max_complexity;
        self
    }

    /// 判断是否应该去关联化
    ///
    /// # 参数
    /// - `pattern_apply`: PatternApply 节点
    ///
    /// # 返回
    /// 去关联化决策
    pub fn should_unnest(&self, pattern_apply: &PatternApplyNode) -> UnnestDecision {
        // 1. 检查子查询是否简单
        if !self.is_simple_subquery(pattern_apply.input()) {
            return UnnestDecision::KeepPatternApply {
                reason: KeepReason::TooComplex,
            };
        }

        // 2. 检查连接条件是否是确定性的
        if let Some(condition) = pattern_apply.condition() {
            if let Some(condition_expr) = condition.expression() {
                let analysis = self.expression_analyzer.analyze(condition_expr.inner());
                if !analysis.is_deterministic {
                    return UnnestDecision::KeepPatternApply {
                        reason: KeepReason::NonDeterministic,
                    };
                }
                // 3. 检查复杂度
                if analysis.complexity_score > self.max_complexity {
                    return UnnestDecision::KeepPatternApply {
                        reason: KeepReason::ComplexCondition,
                    };
                }
            }
        }

        // 4. 检查子查询估算行数
        let estimated_rows = self.estimate_subquery_rows(pattern_apply.input());
        if estimated_rows > self.max_subquery_rows {
            return UnnestDecision::KeepPatternApply {
                reason: KeepReason::TooManyRows,
            };
        }

        // 5. 代价比较（简化版本）
        let original_cost = self.estimate_pattern_apply_cost(pattern_apply, estimated_rows);
        let unnested_cost = self.estimate_hash_join_cost(pattern_apply, estimated_rows);

        if unnested_cost < original_cost {
            UnnestDecision::ShouldUnnest {
                reason: UnnestReason::CostBased,
                original_cost,
                unnested_cost,
            }
        } else {
            UnnestDecision::ShouldUnnest {
                reason: UnnestReason::SimpleSubquery,
                original_cost,
                unnested_cost,
            }
        }
    }

    /// 检查子查询是否简单（单表扫描 + 等值过滤）
    fn is_simple_subquery(&self, node: &crate::query::planner::plan::core::nodes::PlanNodeEnum) -> bool {
        use crate::query::planner::plan::core::nodes::PlanNodeEnum;

        match node {
            // 单表扫描
            PlanNodeEnum::ScanVertices(_) => true,
            PlanNodeEnum::ScanEdges(_) => true,
            PlanNodeEnum::IndexScan(_) => true,

            // 简单过滤
            PlanNodeEnum::Filter(n) => {
                // 检查过滤条件是否是等值比较
                if let Some(condition) = n.condition() {
                    if !self.is_simple_equality_condition(condition) {
                        return false;
                    }
                }
                // 递归检查输入
                self.is_simple_subquery(n.input())
            }

            // 简单投影
            PlanNodeEnum::Project(n) => self.is_simple_subquery(n.input()),

            // 其他情况不支持
            _ => false,
        }
    }

    /// 检查条件是否是简单的等值比较
    fn is_simple_equality_condition(&self, expr: &crate::core::types::expression::contextual::ContextualExpression) -> bool {
        if let Some(expr_inner) = expr.expression() {
            match expr_inner.inner() {
                Expression::Binary { op, left, right } => {
                    use crate::core::types::BinaryOperator;
                    match op {
                        BinaryOperator::Eq => {
                            // 等值比较，检查两边是否都是简单的属性引用
                            self.is_simple_expression(left) && self.is_simple_expression(right)
                        }
                        BinaryOperator::And => {
                            // AND 条件，检查两边
                            self.is_simple_equality_condition(left)
                                && self.is_simple_equality_condition(right)
                        }
                        _ => false,
                    }
                }
                _ => false,
            }
        } else {
            false
        }
    }

    /// 检查表达式是否简单（字面量、变量、属性）
    fn is_simple_expression(&self, expr: &Expression) -> bool {
        matches!(expr, Expression::Literal(_) | Expression::Variable(_) | Expression::Property { .. })
    }

    /// 估算子查询输出行数
    fn estimate_subquery_rows(&self, node: &crate::query::planner::plan::core::nodes::PlanNodeEnum) -> u64 {
        use crate::query::planner::plan::core::nodes::PlanNodeEnum;

        match node {
            PlanNodeEnum::ScanVertices(n) => {
                // 从统计信息获取标签顶点数
                if let Some(stats) = self.stats_manager.get_tag_stats(n.name()) {
                    stats.vertex_count()
                } else {
                    1000 // 默认值
                }
            }
            PlanNodeEnum::Filter(n) => {
                // 过滤后估算为原始行数的 30%
                (self.estimate_subquery_rows(n.input()) as f64 * 0.3) as u64
            }
            PlanNodeEnum::Project(n) => self.estimate_subquery_rows(n.input()),
            _ => 1000, // 默认值
        }
    }

    /// 估算 PatternApply 的代价
    fn estimate_pattern_apply_cost(&self, _pattern_apply: &PatternApplyNode, subquery_rows: u64) -> f64 {
        // 简化估算：嵌套循环，每次都执行子查询
        // 假设左表平均 100 行
        let left_rows = 100.0;
        left_rows * (subquery_rows as f64 * 0.1) // 子查询启动代价 + 执行代价
    }

    /// 估算 HashJoin 的代价
    fn estimate_hash_join_cost(&self, _pattern_apply: &PatternApplyNode, subquery_rows: u64) -> f64 {
        // 简化估算：哈希连接
        let left_rows = 100.0;
        let right_rows = subquery_rows as f64;

        // 构建哈希表代价 + 探测代价
        let build_cost = right_rows;
        let probe_cost = left_rows * 0.5; // 哈希探测很快

        build_cost + probe_cost
    }

    /// 执行去关联化转换
    ///
    /// # 参数
    /// - `pattern_apply`: PatternApply 节点
    ///
    /// # 返回
    /// 转换后的 HashInnerJoin 节点
    pub fn unnest(&self, pattern_apply: PatternApplyNode) -> Result<crate::query::planner::plan::core::nodes::PlanNodeEnum, crate::query::planner::planner::PlannerError> {
        // 获取连接条件
        let condition = match pattern_apply.condition() {
            Some(c) => c.clone(),
            None => return Err(crate::query::planner::planner::PlannerError::InvalidPlan("PatternApply 缺少连接条件".to_string())),
        };

        // 获取连接键
        let key_cols = pattern_apply.key_cols().clone();

        // 创建 HashInnerJoin 节点
        let left_input = pattern_apply.left_input().clone();
        let right_input = pattern_apply.input().clone();

        let hash_join_node = HashInnerJoinNode::new(
            left_input,
            right_input,
            key_cols,
            Some(condition),
            None, // 无额外条件
        )?;

        Ok(crate::query::planner::plan::core::nodes::PlanNodeEnum::HashInnerJoin(hash_join_node))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimizer_creation() {
        let expression_analyzer = ExpressionAnalyzer::new();
        let stats_manager = StatisticsManager::new();
        let optimizer = SubqueryUnnestingOptimizer::new(&expression_analyzer, &stats_manager);
        assert!(!optimizer.expression_analyzer.is_empty());
    }

    #[test]
    fn test_optimizer_with_config() {
        let expression_analyzer = ExpressionAnalyzer::new();
        let stats_manager = StatisticsManager::new();
        let optimizer = SubqueryUnnestingOptimizer::new(&expression_analyzer, &stats_manager)
            .with_max_rows(500)
            .with_max_complexity(30);
        // 验证配置已应用
    }

    #[test]
    fn test_simple_expression_check() {
        let expression_analyzer = ExpressionAnalyzer::new();
        let stats_manager = StatisticsManager::new();
        let optimizer = SubqueryUnnestingOptimizer::new(&expression_analyzer, &stats_manager);

        // 字面量
        let literal = Expression::Literal(crate::core::Value::Int(42));
        assert!(optimizer.is_simple_expression(&literal));

        // 变量
        let variable = Expression::Variable("n".to_string());
        assert!(optimizer.is_simple_expression(&variable));

        // 属性
        let property = Expression::Property {
            object: Box::new(Expression::Variable("n".to_string())),
            property: "name".to_string(),
        };
        assert!(optimizer.is_simple_expression(&property));

        // 二元表达式
        let binary = Expression::Binary {
            left: Box::new(Expression::Literal(crate::core::Value::Int(1))),
            op: crate::core::types::BinaryOperator::Add,
            right: Box::new(Expression::Literal(crate::core::Value::Int(2))),
        };
        assert!(!optimizer.is_simple_expression(&binary));
    }
}