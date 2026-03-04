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

use crate::core::types::operators::BinaryOperator;
use crate::core::types::ContextualExpression;
use crate::core::types::expression::{ExpressionAnalysisContext, ExpressionMeta};
use crate::core::Expression;
use crate::query::optimizer::analysis::ExpressionAnalyzer;
use crate::query::optimizer::stats::StatisticsManager;
use crate::query::planner::plan::core::nodes::{HashInnerJoinNode, PatternApplyNode};
use crate::query::planner::plan::core::nodes::PlanNodeEnum;

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
        // 1. 检查子查询是否简单（包含确定性、复杂度检查）
        if !self.is_simple_subquery(pattern_apply.right_input()) {
            return UnnestDecision::KeepPatternApply {
                reason: KeepReason::TooComplex,
            };
        }

        // 2. 检查连接键是否是确定性的
        // 现在的 key_cols 是 Vec<ContextualExpression>，可以使用 ExpressionAnalyzer 分析
        for key_col in pattern_apply.key_cols() {
            // 直接传递 ContextualExpression 给 ExpressionAnalyzer
            let analysis = self.expression_analyzer.analyze(key_col);
            
            // 检查确定性
            if !analysis.is_deterministic {
                return UnnestDecision::KeepPatternApply {
                    reason: KeepReason::NonDeterministic,
                };
            }
            
            // 检查复杂度
            if analysis.complexity_score > self.max_complexity {
                return UnnestDecision::KeepPatternApply {
                    reason: KeepReason::ComplexCondition,
                };
            }
        }

        // 3. 检查子查询估算行数
        let estimated_rows = self.estimate_subquery_rows(pattern_apply.right_input());
        if estimated_rows > self.max_subquery_rows {
            return UnnestDecision::KeepPatternApply {
                reason: KeepReason::TooManyRows,
            };
        }

        // 4. 代价比较（简化版本）
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
    fn is_simple_subquery(&self, node: &PlanNodeEnum) -> bool {
        match node {
            // 单表扫描
            PlanNodeEnum::ScanVertices(_) => true,
            PlanNodeEnum::ScanEdges(_) => true,
            PlanNodeEnum::IndexScan(_) => true,

            // 简单过滤
            PlanNodeEnum::Filter(n) => {
                // 检查过滤条件是否是等值比较
                let condition = n.condition();
                // 直接传递 ContextualExpression 给 ExpressionAnalyzer
                let analysis = self.expression_analyzer.analyze(condition);
                
                // 检查确定性
                if !analysis.is_deterministic {
                    return false;
                }
                
                // 检查复杂度
                if analysis.complexity_score > self.max_complexity {
                    return false;
                }
                
                // 检查是否是简单的等值比较
                if let Some(expr_meta) = condition.expression() {
                    if !self.is_simple_equality_condition(expr_meta.inner()) {
                        return false;
                    }
                }
                // 递归检查输入
                self.is_simple_subquery(crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode::input(n))
            }

            // 简单投影
            PlanNodeEnum::Project(n) => self.is_simple_subquery(crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode::input(n)),

            // 其他情况不支持
            _ => false,
        }
    }

    /// 检查条件是否是简单的等值比较
    fn is_simple_equality_condition(&self, expr: &Expression) -> bool {
        match expr {
            Expression::Binary { op, left, right } => {
                match op {
                    BinaryOperator::Equal => {
                        // 等值比较，检查两边是否都是简单的属性引用
                        self.is_simple_expression(left.as_ref()) && self.is_simple_expression(right.as_ref())
                    }
                    BinaryOperator::And => {
                        // AND 条件，检查两边
                        self.is_simple_equality_condition(left.as_ref())
                            && self.is_simple_equality_condition(right.as_ref())
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    /// 检查表达式是否简单（字面量、变量、属性）
    fn is_simple_expression(&self, expr: &Expression) -> bool {
        matches!(expr, Expression::Literal(_) | Expression::Variable(_) | Expression::Property { .. })
    }

    /// 估算子查询输出行数
    fn estimate_subquery_rows(&self, node: &PlanNodeEnum) -> u64 {
        match node {
            PlanNodeEnum::ScanVertices(n) => {
                // 从统计信息获取标签顶点数
                if let Some(tag_name) = n.tag() {
                    if let Some(stats) = self.stats_manager.get_tag_stats(tag_name) {
                        stats.vertex_count
                    } else {
                        1000 // 默认值
                    }
                } else {
                    1000 // 默认值
                }
            }
            PlanNodeEnum::Filter(n) => {
                // 过滤后估算为原始行数的 30%
                (self.estimate_subquery_rows(crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode::input(n)) as f64 * 0.3) as u64
            }
            PlanNodeEnum::Project(n) => self.estimate_subquery_rows(crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode::input(n)),
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
        // 获取连接键（已经是 ContextualExpression 类型）
        let key_cols = pattern_apply.key_cols().to_vec();

        // 获取左右输入的变量名
        let left_var = pattern_apply.left_input_var()
            .cloned()
            .unwrap_or_else(|| "left".to_string());
        let right_var = pattern_apply.right_input_var()
            .cloned()
            .unwrap_or_else(|| "right".to_string());

        // 创建表达式上下文
        let expr_ctx = std::sync::Arc::new(ExpressionAnalysisContext::new());

        // 创建 hash_keys 和 probe_keys
        let mut hash_keys = Vec::new();
        let mut probe_keys = Vec::new();

        for key_col in &key_cols {
            // 获取原始表达式
            if let Some(expr_meta) = key_col.expression() {
                let original_expr = expr_meta.inner();

                // 创建左侧键表达式（来自左输入）
                // 将原始表达式中的所有变量引用替换为 left_var
                let left_key_expr = self.replace_all_variables(original_expr, &left_var);
                let left_key_meta = ExpressionMeta::new(left_key_expr);
                let left_key_id = expr_ctx.register_expression(left_key_meta);
                let left_key_contextual = ContextualExpression::new(left_key_id, expr_ctx.clone());
                hash_keys.push(left_key_contextual);

                // 创建右侧键表达式（来自右输入）
                // 将原始表达式中的所有变量引用替换为 right_var
                let right_key_expr = self.replace_all_variables(original_expr, &right_var);
                let right_key_meta = ExpressionMeta::new(right_key_expr);
                let right_key_id = expr_ctx.register_expression(right_key_meta);
                let right_key_contextual = ContextualExpression::new(right_key_id, expr_ctx.clone());
                probe_keys.push(right_key_contextual);
            }
        }

        // 创建 HashInnerJoin 节点
        let left_input = pattern_apply.left_input().clone();
        let right_input = pattern_apply.right_input().clone();

        let hash_join_node = HashInnerJoinNode::new(
            left_input,
            right_input,
            hash_keys,
            probe_keys,
        )?;

        Ok(crate::query::planner::plan::core::nodes::PlanNodeEnum::HashInnerJoin(hash_join_node))
    }

    /// 替换表达式中的所有变量引用为指定变量
    ///
    /// 这个方法递归遍历表达式树，将所有 Variable 节点替换为指定的变量名。
    /// 这用于在将 PatternApply 转换为 HashInnerJoin 时，将原始表达式中的变量
    /// 引用（通常是 "_"）替换为左右输入的变量名。
    ///
    /// # 参数
    /// - `expr`: 要转换的表达式
    /// - `new_var`: 新的变量名
    ///
    /// # 返回
    /// 转换后的表达式
    fn replace_all_variables(&self, expr: &Expression, new_var: &str) -> Expression {
        match expr {
            Expression::Variable(_) => Expression::Variable(new_var.to_string()),
            Expression::Property { object, property } => {
                Expression::Property {
                    object: Box::new(self.replace_all_variables(object, new_var)),
                    property: property.clone(),
                }
            }
            Expression::Binary { op, left, right } => {
                Expression::Binary {
                    op: *op,
                    left: Box::new(self.replace_all_variables(left, new_var)),
                    right: Box::new(self.replace_all_variables(right, new_var)),
                }
            }
            Expression::Unary { op, operand } => {
                Expression::Unary {
                    op: *op,
                    operand: Box::new(self.replace_all_variables(operand, new_var)),
                }
            }
            Expression::Function { name, args } => {
                Expression::Function {
                    name: name.clone(),
                    args: args
                        .iter()
                        .map(|arg| self.replace_all_variables(arg, new_var))
                        .collect(),
                }
            }
            Expression::Aggregate { func, arg, distinct } => {
                Expression::Aggregate {
                    func: func.clone(),
                    arg: Box::new(self.replace_all_variables(arg, new_var)),
                    distinct: *distinct,
                }
            }
            Expression::List(items) => {
                Expression::List(
                    items
                        .iter()
                        .map(|item| self.replace_all_variables(item, new_var))
                        .collect(),
                )
            }
            Expression::Map(entries) => {
                Expression::Map(
                    entries
                        .iter()
                        .map(|(k, v)| {
                            (
                                k.clone(),
                                self.replace_all_variables(v, new_var),
                            )
                        })
                        .collect(),
                )
            }
            Expression::Case {
                test_expr,
                conditions,
                default,
            } => {
                Expression::Case {
                    test_expr: test_expr
                        .as_ref()
                        .map(|e| Box::new(self.replace_all_variables(e, new_var))),
                    conditions: conditions
                        .iter()
                        .map(|(w, t)| {
                            (
                                self.replace_all_variables(w, new_var),
                                self.replace_all_variables(t, new_var),
                            )
                        })
                        .collect(),
                    default: default
                        .as_ref()
                        .map(|e| Box::new(self.replace_all_variables(e, new_var))),
                }
            }
            Expression::TypeCast {
                expression,
                target_type,
            } => {
                Expression::TypeCast {
                    expression: Box::new(self.replace_all_variables(expression, new_var)),
                    target_type: target_type.clone(),
                }
            }
            Expression::Subscript { collection, index } => {
                Expression::Subscript {
                    collection: Box::new(self.replace_all_variables(collection, new_var)),
                    index: Box::new(self.replace_all_variables(index, new_var)),
                }
            }
            Expression::Range {
                collection,
                start,
                end,
            } => {
                Expression::Range {
                    collection: Box::new(self.replace_all_variables(collection, new_var)),
                    start: start
                        .as_ref()
                        .map(|e| Box::new(self.replace_all_variables(e, new_var))),
                    end: end
                        .as_ref()
                        .map(|e| Box::new(self.replace_all_variables(e, new_var))),
                }
            }
            Expression::Path(exprs) => {
                Expression::Path(
                    exprs
                        .iter()
                        .map(|e| self.replace_all_variables(e, new_var))
                        .collect(),
                )
            }
            Expression::Label(_) => expr.clone(),
            Expression::ListComprehension {
                variable,
                source,
                filter,
                map,
            } => {
                Expression::ListComprehension {
                    variable: variable.clone(),
                    source: Box::new(self.replace_all_variables(source, new_var)),
                    filter: filter
                        .as_ref()
                        .map(|e| Box::new(self.replace_all_variables(e, new_var))),
                    map: map
                        .as_ref()
                        .map(|e| Box::new(self.replace_all_variables(e, new_var))),
                }
            }
            Expression::LabelTagProperty { tag, property } => {
                Expression::LabelTagProperty {
                    tag: Box::new(self.replace_all_variables(tag, new_var)),
                    property: property.clone(),
                }
            }
            Expression::TagProperty { tag_name, property } => {
                Expression::TagProperty {
                    tag_name: tag_name.clone(),
                    property: property.clone(),
                }
            }
            Expression::EdgeProperty { edge_name, property } => {
                Expression::EdgeProperty {
                    edge_name: edge_name.clone(),
                    property: property.clone(),
                }
            }
            Expression::Predicate { func, args } => {
                Expression::Predicate {
                    func: func.clone(),
                    args: args
                        .iter()
                        .map(|arg| self.replace_all_variables(arg, new_var))
                        .collect(),
                }
            }
            Expression::Reduce {
                accumulator,
                initial,
                variable,
                source,
                mapping,
            } => {
                Expression::Reduce {
                    accumulator: accumulator.clone(),
                    initial: Box::new(self.replace_all_variables(initial, new_var)),
                    variable: variable.clone(),
                    source: Box::new(self.replace_all_variables(source, new_var)),
                    mapping: Box::new(self.replace_all_variables(mapping, new_var)),
                }
            }
            Expression::PathBuild(exprs) => {
                Expression::PathBuild(
                    exprs
                        .iter()
                        .map(|e| self.replace_all_variables(e, new_var))
                        .collect(),
                )
            }
            Expression::Parameter(_) => expr.clone(),
            Expression::Literal(_) => expr.clone(),
        }
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
        // 验证优化器创建成功
        assert_eq!(optimizer.max_subquery_rows, 1000);
        assert_eq!(optimizer.max_complexity, 50);
    }

    #[test]
    fn test_optimizer_with_config() {
        let expression_analyzer = ExpressionAnalyzer::new();
        let stats_manager = StatisticsManager::new();
        let _optimizer = SubqueryUnnestingOptimizer::new(&expression_analyzer, &stats_manager)
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
            op: crate::core::types::operators::BinaryOperator::Add,
            right: Box::new(Expression::Literal(crate::core::Value::Int(2))),
        };
        assert!(!optimizer.is_simple_expression(&binary));
    }
}