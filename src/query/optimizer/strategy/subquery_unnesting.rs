//! Subquery de-association optimization module
//!
//! “Analysis-based subquery deserialization optimization strategy” – This strategy converts simple PatternApply subqueries into HashInnerJoin operations.
//!
//! ## Optimization Strategies
//!
//! Convert the eligible PatternApply subquery into a HashInnerJoin.
//! Avoid executing subqueries repeatedly.
//!
//! ## Applicable Conditions
//!
//! The right input for PatternApply is a simple query (single-table scan + equality filtering).
//! 2. 过滤条件是确定性的（不含 rand(), now() 等）
//! 3. The complexity of the expressions should be less than 50 (avoid using complex expressions).
//! 4. The subquery estimates that the number of rows is less than 1000 (based on statistical information).
//!
//! ## Usage Examples
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

use crate::core::types::expr::ExpressionMeta;
use crate::core::types::operators::BinaryOperator;
use crate::core::types::ContextualExpression;
use crate::core::Expression;
use crate::query::optimizer::analysis::ExpressionAnalyzer;
use crate::query::optimizer::stats::StatisticsManager;
use crate::query::planning::plan::core::nodes::PlanNodeEnum;
use crate::query::planning::plan::core::nodes::{HashInnerJoinNode, PatternApplyNode};
use crate::query::validator::context::ExpressionAnalysisContext;

/// Decentralized decision-making using subqueries
#[derive(Debug, Clone, PartialEq)]
pub enum UnnestDecision {
    /// Convert to HashInnerJoin
    ShouldUnnest {
        /// The text to be translated is:  “You are a professional translator.”  

**Translation:**  
“You are a professional translator.”
        reason: UnnestReason,
        /// Estimated original cost
        original_cost: f64,
        /// Estimated cost after the conversion
        unnested_cost: f64,
    },
    /// Please provide the text you would like to have translated. I will then perform the translation and ensure that the format and structure of the original text are preserved in the target language.
    KeepPatternApply {
        /// Reason for retention
        reason: KeepReason,
    },
}

/// 转换原因
#[derive(Debug, Clone, PartialEq)]
pub enum UnnestReason {
    /// Simple subquery; the conversion is more efficient.
    SimpleSubquery,
    /// Based on cost analysis
    CostBased,
}

/// 保留原因
#[derive(Debug, Clone, PartialEq)]
pub enum KeepReason {
    /// The subquery is too complex.
    TooComplex,
    /// The subquery contains a non-deterministic function.
    NonDeterministic,
    /// The number of rows estimated by the subquery is too large.
    TooManyRows,
    /// The subquery contains complex conditions.
    ComplexCondition,
}

/// Subquery desaggregation optimizer
///
/// Based on expression analysis and statistical information, a decision is made as to whether to convert PatternApply to HashInnerJoin.
#[derive(Debug, Clone)]
pub struct SubqueryUnnestingOptimizer {
    /// Expression Analyzer
    expression_analyzer: ExpressionAnalyzer,
    /// Statistics Information Manager
    stats_manager: StatisticsManager,
    /// The maximum number of estimated rows allowed for a subquery
    max_subquery_rows: u64,
    /// The maximum allowable complexity of the expression
    max_complexity: u32,
}

impl SubqueryUnnestingOptimizer {
    /// Create a new optimizer.
    pub fn new(
        expression_analyzer: &ExpressionAnalyzer,
        stats_manager: &StatisticsManager,
    ) -> Self {
        Self {
            expression_analyzer: expression_analyzer.clone(),
            stats_manager: stats_manager.clone(),
            max_subquery_rows: 1000,
            max_complexity: 50,
        }
    }

    /// Set a threshold for the maximum number of rows in subqueries
    pub fn with_max_rows(mut self, max_rows: u64) -> Self {
        self.max_subquery_rows = max_rows;
        self
    }

    /// Set a threshold for the maximum complexity.
    pub fn with_max_complexity(mut self, max_complexity: u32) -> Self {
        self.max_complexity = max_complexity;
        self
    }

    /// Determine whether decoupling should be performed.
    ///
    /// # Parameters
    /// `pattern_apply`: The PatternApply node
    ///
    /// // 1. Check whether the subquery is simple (including checks for certainty and complexity).
    /// De-associative decision-making
    pub fn should_unnest(&self, pattern_apply: &PatternApplyNode) -> UnnestDecision {
        // 1. 检查子查询是否简单（包含确定性、复杂度检查）
        if !self.is_simple_subquery(pattern_apply.right_input()) {
            return UnnestDecision::KeepPatternApply {
                reason: KeepReason::TooComplex,
        // 2. Check whether the connection key is deterministic.
        // The current `key_cols` is a `Vec<ContextualExpression>`, and it can be analyzed using the `ExpressionAnalyzer`.

        // 2// Pass the `ContextualExpression` directly to the `ExpressionAnalyzer`.
        // 现在的 key_cols 是 Vec<ContextualExpression>，可以使用 ExpressionAnalyzer 分析
        for key_col in pattern_apply.key_cols() {
            // Check the certainty.textualExpression 给 ExpressionAnalyzer
            let analysis = self.expression_analyzer.analyze(key_col);

            // 检查确定性
            if !analysis.is_deterministic {
                return UnnestDecision::KeepPatternApply {
                    reason: KeepReason::NonDeterministic,
            // Check the complexity
            }

            // 检查复杂度
            if analysis.complexity_score > self.max_complexity {
                return UnnestDecision::KeepPatternApply {
                    reason: KeepReason::ComplexCondition,
                };
        // 3. Check the estimated number of rows returned by the subquery.
        }

        // 3. 检查子查询估算行数
        let estimated_rows = self.estimate_subquery_rows(pattern_apply.right_input());
        if estimated_rows > self.max_subquery_rows {
            return UnnestDecision::KeepPatternApply {
                reason: KeepReason::TooManyRows,
        // 4. Comparison of costs (simplified version)
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

    /// Chec// Single-table scanbquery is simple (involving a scan of a single table and equality filtering).
    fn is_simple_subquery(&self, node: &PlanNodeEnum) -> bool {
        match node {
            // 单表扫描
            PlanNodeEnum::ScanVertices(_) => true,
            // Simple filteringanEdges(_) => true,
            PlanNodeEnum::IndexScan(_) => true,

            // 简单过滤
            Plan// Pass the `ContextualExpression` directly to the `ExpressionAnalyzer`.
                // 检查过滤条件是否是等值比较
                let condition = n.condition();
                // Check the certainty.textualExpression 给 ExpressionAnalyzer
                let analysis = self.expression_analyzer.analyze(condition);

                // 检查确定性
                if !analysis.is_deterministic {
                // Check the complexity.
                }

                // 检查复杂度
                if analysis.complexity_score > self.max_complexity {
                // Check whether it is a simple equality comparison.
                }

                // 检查是否是简单的等值比较
                if let Some(expr_meta) = condition.expression() {
                    if !self.is_simple_equality_condition(expr_meta.inner()) {
                // Perform a recursive check on the input.
                    }
                }
                // 递归检查输入
            // Simple projectionle_subquery(crate::query::planning::plan::core::nodes::base::plan_node_traits::SingleInputNode::input(n))
            }

            // 简单投影
            PlanNodeEnum::Project(n) => self.is_simple_subquery(
                crate::query::planning::plan::core::nodes::base::plan_node_traits::SingleInputNode::input(
                    n,
            // Other situations are not supported.
            ),

            // 其他情况不支持
            _ => false,
        }
    }

    /// Check whether the condition is a simple equality comparison.
    fn is_simple_equality_condition(&self, expr: &Expression) -> bool {
        match expr {
            Expression::// Equivalence comparison: Check whether both sides are simple property references.
                match op {
                    BinaryOperator::Equal => {
                        // 等值比较，检查两边是否都是简单的属性引用
                        self.is_simple_expression(left.as_ref())
                        // AND condition: Check both sidesion(right.as_ref())
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

    /// Check whether the expression is simple (consisting of literals, variables, or properties).
    fn is_simple_expression(&self, expr: &Expression) -> bool {
        matches!(
            expr,
            Expression::Literal(_) | Expression::Variable(_) | Expression::Property { .. }
        )
    }

    /// Estimating the number of rows returned by a subquery
    fn estimate_// Obtain the number of label vertices from the statistical information.) -> u64 {
        match node {
            PlanNodeEnum::ScanVertices(n) => {
                // 从统计信息获取标签顶点数
                if let Some(tag_name) = n.tag() {
                    if let So// Default valueelf.stats_manager.get_tag_stats(tag_name) {
                        stats.vertex_count
                    } else {
                        1// Default value值
                    }
                } else {
                    1000 // 默认值
                // The estimated number of rows after filtering is 30% of the original number of rows.
            }
            PlanNodeEnum::Filter(n) => {
                // 过滤后估算为原始行数的 30%
                (self.estimate_subquery_rows(crate::query::planning::plan::core::nodes::base::plan_node_traits::SingleInputNode::input(n)) as f64 * 0.3) as u64
            }
            PlanNodeEnum::Project(n) => self.estimate_subquery_rows(
                crate::query::planning::plan::core::nodes::base::plan_node_traits::SingleInputNode::input(
                    n,
                ),
            ),
            _ => 1000, // 默认值
        }
    }

    /// Estimate the cost of applying the PatternApply method
    fn estimate_pattern_apply_cost(
        &self,
        // Simplified estimation: Nested loops, with the subquery being executed in each iteration.
        // Assume that the left table contains an average of 100 rows.
    ) -> f64 {
        // 简化估算：嵌套循环，每次都// Cost of initiating the subquery + Cost of executing the subquery
        // 假设左表平均 100 行
        let left_rows = 100.0;
        left_rows * (subquery_rows as f64 * 0.1) // 子查询启动代价 + 执行代价
    }

    /// Estimating the cost of a HashJoin operation
    fn estimate_hash_join_cost(
        &self,
        // Simplified estimation: Hash joinde,
        subquery_rows: u64,
    ) -> f64 {
        // 简化估算：哈希连接
        // Cost of building a hash table + Cost of detection
        let right_rows = subquery_rows as f64;

        // 构建哈希表代价 + 探测代价
        let build_cost = right_rows;
        let probe_cost = left_rows * 0.5; // 哈希探测很快

        build_cost + probe_cost
    }

    /// Perform the de-association transformation.
    ///
    /// # 参数
    /// - `pattern_apply`: PatternApply 节点
    ///
    /// # 返回
    /// The transformed HashInnerJoin node
    pub fn unnest(
        &self,
        pattern_apply: PatternApplyNode,
    ) -> Result<
        // Obtain the connection key (which is already of the ContextualExpression type).
        crate::query::planning::planner::PlannerError,
    > {
        // Obtain the variable names for the left and right inputs.textualExpression 类型）
        let key_cols = pattern_apply.key_cols().to_vec();

        // 获取左右输入的变量名
        let left_var = pattern_apply
            .left_input_var()
            .cloned()
            .unwrap_or_else(|| "left".to_string());
        let right_var = pattern_apply
            .right_input_var()
        // Create the context for the expression.
            .unwrap_or_else(|| "right".to_string());

        // Create `hash_keys` and `probe_keys`.
        let expr_ctx = std::sync::Arc::new(ExpressionAnalysisContext::new());

        // 创建 hash_keys 和 probe_keys
        let mut hash_keys = Vec::new();
        let // Obtain the original expression();

        for key_col in &key_cols {
            // 获取原始表达式
            if l// Create the left-click expression (from the left input).
                // Replace all variable references in the original expression with `left_var`.

                // 创建左侧键表达式（来自左输入）
                // 将原始表达式中的所有变量引用替换为 left_var
                let left_key_expr = self.replace_all_variables(original_expr, &left_var);
                let left_key_meta = ExpressionMeta::new(left_key_expr);
                let left_key_id = expr_ctx.register_expression(left_key_meta);
                // Create a right-click expression (from the right-side input).ew(left_key_id, expr_ctx.clone());
                // Replace all variable references in the original expression with `right_var`.

                // 创建右侧键表达式（来自右输入）
                // 将原始表达式中的所有变量引用替换为 right_var
                let right_key_expr = self.replace_all_variables(original_expr, &right_var);
                let right_key_meta = ExpressionMeta::new(right_key_expr);
                let right_key_id = expr_ctx.register_expression(right_key_meta);
                let right_key_contextual =
                    ContextualExpression::new(right_key_id, expr_ctx.clone());
                probe_keys.push(right_key_contextual);
        // Create a HashInnerJoin node.
        }

        // 创建 HashInnerJoin 节点
        let left_input = pattern_apply.left_input().clone();
        let right_input = pattern_apply.right_input().clone();

        let hash_join_node =
            HashInnerJoinNode::new(left_input, right_input, hash_keys, probe_keys)?;

        Ok(crate::query::planning::plan::core::nodes::PlanNodeEnum::HashInnerJoin(hash_join_node))
    }

    /// Replace all variable references in the expression with the specified variables.
    ///
    /// This method recursively traverses the expression tree and replaces all Variable nodes with the specified variable name.
    /// This is used to convert the variables in the original expression when transforming PatternApply to HashInnerJoin.
    /// The placeholders (usually “_”) should be replaced with the variable names provided on the left and on the right.
    ///
    /// # 参数
    /// `expr`: The expression that needs to be converted.
    /// `new_var`: The name of the new variable
    ///
    /// # 返回
    /// Of course! Please provide the text you would like to have translated.
    fn replace_all_variables(&self, expr: &Expression, new_var: &str) -> Expression {
        match expr {
            Expression::Variable(_) => Expression::Variable(new_var.to_string()),
            Expression::Property { object, property } => Expression::Property {
                object: Box::new(self.replace_all_variables(object, new_var)),
                property: property.clone(),
            },
            Expression::Binary { op, left, right } => Expression::Binary {
                op: *op,
                left: Box::new(self.replace_all_variables(left, new_var)),
                right: Box::new(self.replace_all_variables(right, new_var)),
            },
            Expression::Unary { op, operand } => Expression::Unary {
                op: *op,
                operand: Box::new(self.replace_all_variables(operand, new_var)),
            },
            Expression::Function { name, args } => Expression::Function {
                name: name.clone(),
                args: args
                    .iter()
                    .map(|arg| self.replace_all_variables(arg, new_var))
                    .collect(),
            },
            Expression::Aggregate {
                func,
                arg,
                distinct,
            } => Expression::Aggregate {
                func: func.clone(),
                arg: Box::new(self.replace_all_variables(arg, new_var)),
                distinct: *distinct,
            },
            Expression::List(items) => Expression::List(
                items
                    .iter()
                    .map(|item| self.replace_all_variables(item, new_var))
                    .collect(),
            ),
            Expression::Map(entries) => Expression::Map(
                entries
                    .iter()
                    .map(|(k, v)| (k.clone(), self.replace_all_variables(v, new_var)))
                    .collect(),
            ),
            Expression::Case {
                test_expr,
                conditions,
                default,
            } => Expression::Case {
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
            },
            Expression::TypeCast {
                expression,
                target_type,
            } => Expression::TypeCast {
                expression: Box::new(self.replace_all_variables(expression, new_var)),
                target_type: target_type.clone(),
            },
            Expression::Subscript { collection, index } => Expression::Subscript {
                collection: Box::new(self.replace_all_variables(collection, new_var)),
                index: Box::new(self.replace_all_variables(index, new_var)),
            },
            Expression::Range {
                collection,
                start,
                end,
            } => Expression::Range {
                collection: Box::new(self.replace_all_variables(collection, new_var)),
                start: start
                    .as_ref()
                    .map(|e| Box::new(self.replace_all_variables(e, new_var))),
                end: end
                    .as_ref()
                    .map(|e| Box::new(self.replace_all_variables(e, new_var))),
            },
            Expression::Path(exprs) => Expression::Path(
                exprs
                    .iter()
                    .map(|e| self.replace_all_variables(e, new_var))
                    .collect(),
            ),
            Expression::Label(_) => expr.clone(),
            Expression::ListComprehension {
                variable,
                source,
                filter,
                map,
            } => Expression::ListComprehension {
                variable: variable.clone(),
                source: Box::new(self.replace_all_variables(source, new_var)),
                filter: filter
                    .as_ref()
                    .map(|e| Box::new(self.replace_all_variables(e, new_var))),
                map: map
                    .as_ref()
                    .map(|e| Box::new(self.replace_all_variables(e, new_var))),
            },
            Expression::LabelTagProperty { tag, property } => Expression::LabelTagProperty {
                tag: Box::new(self.replace_all_variables(tag, new_var)),
                property: property.clone(),
            },
            Expression::TagProperty { tag_name, property } => Expression::TagProperty {
                tag_name: tag_name.clone(),
                property: property.clone(),
            },
            Expression::EdgeProperty {
                edge_name,
                property,
            } => Expression::EdgeProperty {
                edge_name: edge_name.clone(),
                property: property.clone(),
            },
            Expression::Predicate { func, args } => Expression::Predicate {
                func: func.clone(),
                args: args
                    .iter()
                    .map(|arg| self.replace_all_variables(arg, new_var))
                    .collect(),
            },
            Expression::Reduce {
                accumulator,
                initial,
                variable,
                source,
                mapping,
            } => Expression::Reduce {
                accumulator: accumulator.clone(),
                initial: Box::new(self.replace_all_variables(initial, new_var)),
                variable: variable.clone(),
                source: Box::new(self.replace_all_variables(source, new_var)),
                mapping: Box::new(self.replace_all_variables(mapping, new_var)),
            },
            Expression::PathBuild(exprs) => Expression::PathBuild(
                exprs
                    .iter()
                    .map(|e| self.replace_all_variables(e, new_var))
                    .collect(),
            ),
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
        // The optimization optimizer was created successfully.ssionAnalyzer::new();
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
        // Verify that the configuration has been applied.UnnestingOptimizer::new(&expression_analyzer, &stats_manager)
            .with_max_rows(500)
            .with_max_complexity(30);
        // 验证配置已应用
    }

    #[test]
    fn test_simple_expression_check() {
        let expression_analyzer = ExpressionAnalyzer::new();
        // Literal valueager = StatisticsManager::new();
        let optimizer = SubqueryUnnestingOptimizer::new(&expression_analyzer, &stats_manager);

        // 字面量
        // variablel = Expression::Literal(crate::core::Value::Int(42));
        assert!(optimizer.is_simple_expression(&literal));

        // 变量
        // Attributele = Expression::Variable("n".to_string());
        assert!(optimizer.is_simple_expression(&variable));

        // 属性
        let property = Expression::Property {
            object: Box::new(Expression::Variable("n".to_string())),
            property: "name".to_string(),
        // Binary expression
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
