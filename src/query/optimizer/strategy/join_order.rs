//! 连接顺序优化器模块
//!
//! 基于代价的连接顺序优化，为多表连接选择最优的连接顺序
//!
//! ## 算法支持
//!
//! - 动态规划（DP）：精确求解最优连接顺序，适用于少量表（<=8）
//! - 贪心算法：快速求解近似最优解，适用于大量表
//! - 左深树（Left-Deep Tree）：经典的连接树形状
//! - 浓密树（Bushy Tree）：更灵活的连接树形状
//!
//! ## 使用示例
//!
//! ```rust
//! use graphdb::query::optimizer::strategy::JoinOrderOptimizer;
//! use graphdb::query::optimizer::cost::CostCalculator;
//! use std::sync::Arc;
//!
//! let optimizer = JoinOrderOptimizer::new(cost_calculator);
//! let tables = vec![table1, table2, table3];
//! let conditions = vec![join_condition];
//! let decision = optimizer.optimize_join_order(&tables, &conditions);
//! ```

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::query::optimizer::cost::CostCalculator;
use crate::query::optimizer::decision::{JoinAlgorithm, JoinOrderDecision};
use crate::core::Expression;

/// 表信息
#[derive(Debug, Clone)]
pub struct TableInfo {
    /// 表标识符（变量名）
    pub id: String,
    /// 估计行数
    pub estimated_rows: u64,
    /// 选择性（0.0 ~ 1.0）
    pub selectivity: f64,
    /// 是否有索引
    pub has_index: bool,
    /// 表的唯一标识（用于位运算）
    pub bit_id: u32,
}

impl TableInfo {
    /// 创建新的表信息
    pub fn new(id: String, estimated_rows: u64) -> Self {
        Self {
            id,
            estimated_rows,
            selectivity: 1.0,
            has_index: false,
            bit_id: 0,
        }
    }

    /// 设置选择性
    pub fn with_selectivity(mut self, selectivity: f64) -> Self {
        self.selectivity = selectivity.clamp(0.0, 1.0);
        self
    }

    /// 设置是否有索引
    pub fn with_index(mut self, has_index: bool) -> Self {
        self.has_index = has_index;
        self
    }

    /// 设置位ID
    pub fn with_bit_id(mut self, bit_id: u32) -> Self {
        self.bit_id = bit_id;
        self
    }
}

/// 连接条件
#[derive(Debug, Clone)]
pub struct JoinCondition {
    /// 左表ID
    pub left_table: String,
    /// 右表ID
    pub right_table: String,
    /// 连接选择性（估计的连接结果比例）
    pub selectivity: f64,
    /// 连接表达式
    pub expression: Option<Expression>,
}

impl JoinCondition {
    /// 创建新的连接条件
    pub fn new(left_table: String, right_table: String) -> Self {
        Self {
            left_table,
            right_table,
            selectivity: 0.3, // 默认选择性 30%
            expression: None,
        }
    }

    /// 设置选择性
    pub fn with_selectivity(mut self, selectivity: f64) -> Self {
        self.selectivity = selectivity.clamp(0.0, 1.0);
        self
    }

    /// 设置连接表达式
    pub fn with_expression(mut self, expression: Expression) -> Self {
        self.expression = Some(expression);
        self
    }
}

/// 连接顺序优化器
#[derive(Debug)]
pub struct JoinOrderOptimizer {
    cost_calculator: Arc<CostCalculator>,
    /// 动态规划表大小阈值（超过此值使用贪心算法）
    dp_threshold: usize,
}

/// 子问题解（用于动态规划）
#[derive(Debug, Clone)]
struct SubproblemSolution {
    /// 包含的表集合（位掩码）
    pub table_set: u32,
    /// 最后连接的表
    pub last_table: String,
    /// 总代价
    pub total_cost: f64,
    /// 输出行数
    pub output_rows: u64,
    /// 连接树（以字符串表示）
    pub join_tree: String,
}

/// 连接顺序优化结果
#[derive(Debug, Clone)]
pub struct JoinOrderResult {
    /// 最优连接顺序（表ID列表）
    pub order: Vec<String>,
    /// 每个连接的算法选择
    pub algorithms: Vec<JoinAlgorithm>,
    /// 总估计代价
    pub total_cost: f64,
    /// 最终估计输出行数
    pub final_output_rows: u64,
    /// 使用的优化算法
    pub optimization_method: OptimizationMethod,
}

/// 优化方法
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptimizationMethod {
    /// 动态规划
    DynamicProgramming,
    /// 贪心算法
    Greedy,
    /// 启发式（表数量过少）
    Heuristic,
}

impl JoinOrderOptimizer {
    /// 创建新的连接顺序优化器
    pub fn new(cost_calculator: Arc<CostCalculator>) -> Self {
        Self {
            cost_calculator,
            dp_threshold: 8, // 默认8个表以下使用动态规划
        }
    }

    /// 设置DP阈值
    pub fn with_dp_threshold(mut self, threshold: usize) -> Self {
        self.dp_threshold = threshold;
        self
    }

    /// 优化连接顺序
    ///
    /// # 参数
    /// - `tables`: 参与连接的表列表
    /// - `conditions`: 连接条件列表
    ///
    /// # 返回
    /// 连接顺序优化结果
    pub fn optimize_join_order(
        &self,
        tables: &[TableInfo],
        conditions: &[JoinCondition],
    ) -> JoinOrderResult {
        if tables.len() <= 1 {
            return JoinOrderResult {
                order: tables.iter().map(|t| t.id.clone()).collect(),
                algorithms: Vec::new(),
                total_cost: 0.0,
                final_output_rows: tables.first().map(|t| t.estimated_rows).unwrap_or(0),
                optimization_method: OptimizationMethod::Heuristic,
            };
        }

        if tables.len() <= self.dp_threshold {
            self.optimize_with_dp(tables, conditions)
        } else {
            self.optimize_with_greedy(tables, conditions)
        }
    }

    /// 使用动态规划优化连接顺序
    fn optimize_with_dp(
        &self,
        tables: &[TableInfo],
        conditions: &[JoinCondition],
    ) -> JoinOrderResult {
        let n = tables.len();

        // 构建连接条件查找表
        let condition_map = self.build_condition_map(conditions);

        // DP表：key = 表集合位掩码，value = 最优解
        let mut dp: HashMap<u32, SubproblemSolution> = HashMap::new();

        // 初始化：单表情况
        for table in tables {
            let solution = SubproblemSolution {
                table_set: 1 << table.bit_id,
                last_table: table.id.clone(),
                total_cost: 0.0,
                output_rows: table.estimated_rows,
                join_tree: table.id.clone(),
            };
            dp.insert(solution.table_set, solution);
        }

        // 动态规划：从小到大构建子集
        for subset_size in 2..=n {
            for subset in self.generate_subsets(n, subset_size) {
                let mut best_solution: Option<SubproblemSolution> = None;

                // 尝试将subset分解为两个非空子集
                for table in tables {
                    let table_bit = 1 << table.bit_id;
                    if subset & table_bit == 0 {
                        continue;
                    }

                    let remaining = subset ^ table_bit;
                    if remaining == 0 {
                        continue;
                    }

                    // 查找剩余部分的最优解
                    if let Some(left_solution) = dp.get(&remaining) {
                        // 计算连接代价
                        let (join_cost, output_rows) = self.calculate_join_cost(
                            left_solution.output_rows,
                            table.estimated_rows,
                            &table.id,
                            &condition_map,
                        );

                        let total_cost = left_solution.total_cost + join_cost;

                        let solution = SubproblemSolution {
                            table_set: subset,
                            last_table: table.id.clone(),
                            total_cost,
                            output_rows,
                            join_tree: format!(
                                "Join({}, {})",
                                left_solution.join_tree, table.id
                            ),
                        };

                        if best_solution
                            .as_ref()
                            .map_or(true, |best| solution.total_cost < best.total_cost)
                        {
                            best_solution = Some(solution);
                        }
                    }
                }

                if let Some(solution) = best_solution {
                    dp.insert(subset, solution);
                }
            }
        }

        // 获取最优解
        let full_set = (1 << n) - 1;
        let best_solution = dp.get(&full_set).cloned().unwrap_or_else(|| {
            // 回退：按原始顺序
            SubproblemSolution {
                table_set: full_set,
                last_table: tables.last().unwrap().id.clone(),
                total_cost: f64::MAX,
                output_rows: 0,
                join_tree: "fallback".to_string(),
            }
        });

        // 重构连接顺序
        let order = self.reconstruct_order(&best_solution, &dp, tables);
        let algorithms = self.select_algorithms(&order, conditions, tables);

        JoinOrderResult {
            order,
            algorithms,
            total_cost: best_solution.total_cost,
            final_output_rows: best_solution.output_rows,
            optimization_method: OptimizationMethod::DynamicProgramming,
        }
    }

    /// 使用贪心算法优化连接顺序
    fn optimize_with_greedy(
        &self,
        tables: &[TableInfo],
        conditions: &[JoinCondition],
    ) -> JoinOrderResult {
        let condition_map = self.build_condition_map(conditions);
        let mut remaining: HashSet<String> = tables.iter().map(|t| t.id.clone()).collect();
        let table_map: HashMap<String, &TableInfo> =
            tables.iter().map(|t| (t.id.clone(), t)).collect();

        let mut order = Vec::new();
        let mut algorithms = Vec::new();
        let mut total_cost = 0.0;
        let mut current_rows = 0u64;

        // 选择起始表（行数最少的表）
        if let Some(start_table) = tables.iter().min_by_key(|t| t.estimated_rows) {
            order.push(start_table.id.clone());
            remaining.remove(&start_table.id);
            current_rows = start_table.estimated_rows;
        }

        // 贪心选择下一个表
        while !remaining.is_empty() {
            let mut best_next: Option<(String, f64, u64)> = None;

            for table_id in &remaining {
                if let Some(table) = table_map.get(table_id) {
                    let (cost, output_rows) = self.calculate_join_cost(
                        current_rows,
                        table.estimated_rows,
                        &table.id,
                        &condition_map,
                    );

                    if best_next.as_ref().map_or(true, |(_, best_cost, _)| cost < *best_cost) {
                        best_next = Some((table_id.clone(), cost, output_rows));
                    }
                }
            }

            if let Some((next_id, cost, output_rows)) = best_next {
                // 获取当前表和下一个表的索引信息
                let current_has_index = order
                    .last()
                    .and_then(|id| table_map.get(id))
                    .map(|t| t.has_index)
                    .unwrap_or(false);
                let next_has_index = table_map
                    .get(&next_id)
                    .map(|t| t.has_index)
                    .unwrap_or(false);
                let current_id = order.last().cloned().unwrap_or_default();

                order.push(next_id.clone());
                remaining.remove(&next_id);
                total_cost += cost;

                // 选择连接算法
                let algorithm = self.select_algorithm(
                    current_rows,
                    output_rows,
                    current_has_index,
                    next_has_index,
                    &current_id,
                    &next_id,
                );
                algorithms.push(algorithm);

                current_rows = output_rows;
            } else {
                break;
            }
        }

        JoinOrderResult {
            order,
            algorithms,
            total_cost,
            final_output_rows: current_rows,
            optimization_method: OptimizationMethod::Greedy,
        }
    }

    /// 构建连接条件查找表
    fn build_condition_map(
        &self,
        conditions: &[JoinCondition],
    ) -> HashMap<(String, String), f64> {
        let mut map = HashMap::new();
        for cond in conditions {
            let key = (cond.left_table.clone(), cond.right_table.clone());
            let reverse_key = (cond.right_table.clone(), cond.left_table.clone());
            map.insert(key, cond.selectivity);
            map.insert(reverse_key, cond.selectivity);
        }
        map
    }

    /// 生成指定大小的子集
    fn generate_subsets(&self, n: usize, k: usize) -> Vec<u32> {
        let mut result = Vec::new();
        self.generate_subsets_recursive(0, n, k, 0, &mut result);
        result
    }

    fn generate_subsets_recursive(
        &self,
        start: usize,
        n: usize,
        k: usize,
        current: u32,
        result: &mut Vec<u32>,
    ) {
        if k == 0 {
            result.push(current);
            return;
        }
        if start >= n {
            return;
        }
        for i in start..n {
            self.generate_subsets_recursive(i + 1, n, k - 1, current | (1 << i), result);
        }
    }

    /// 计算连接代价
    fn calculate_join_cost(
        &self,
        left_rows: u64,
        right_rows: u64,
        right_table: &str,
        condition_map: &HashMap<(String, String), f64>,
    ) -> (f64, u64) {
        // 查找连接选择性
        let selectivity = condition_map
            .iter()
            .find(|((l, _), _)| l == right_table)
            .map(|(_, s)| *s)
            .unwrap_or(0.3);

        // 计算输出行数
        let output_rows = ((left_rows as f64 * right_rows as f64 * selectivity) as u64).max(1);

        // 计算连接代价（使用哈希连接）
        let cost = self
            .cost_calculator
            .calculate_hash_join_cost(left_rows, right_rows);

        (cost, output_rows)
    }

    /// 选择连接算法
    ///
    /// 基于代价模型选择最优的连接算法：
    /// 1. 如果一侧有索引且数据量适中，优先选择索引连接
    /// 2. 比较哈希连接和嵌套循环连接的代价，选择代价较低的
    /// 3. 哈希连接时，选择较小的表作为构建侧
    fn select_algorithm(
        &self,
        left_rows: u64,
        right_rows: u64,
        left_has_index: bool,
        right_has_index: bool,
        left_id: &str,
        right_id: &str,
    ) -> JoinAlgorithm {
        // 阈值定义
        const NESTED_LOOP_MAX_ROWS: u64 = 100; // 嵌套循环连接适用的最大行数
        const INDEX_JOIN_MAX_ROWS: u64 = 10000; // 索引连接适用的最大行数

        // 策略1：如果一侧有索引且另一侧数据量适中，使用索引连接
        if left_has_index && right_rows <= INDEX_JOIN_MAX_ROWS {
            return JoinAlgorithm::IndexJoin {
                indexed_side: left_id.to_string(),
            };
        }
        if right_has_index && left_rows <= INDEX_JOIN_MAX_ROWS {
            return JoinAlgorithm::IndexJoin {
                indexed_side: right_id.to_string(),
            };
        }

        // 策略2：如果数据量都很小，使用嵌套循环连接（避免哈希表构建开销）
        if left_rows <= NESTED_LOOP_MAX_ROWS && right_rows <= NESTED_LOOP_MAX_ROWS {
            return JoinAlgorithm::NestedLoopJoin {
                outer: left_id.to_string(),
                inner: right_id.to_string(),
            };
        }

        // 策略3：默认使用哈希连接，选择较小的表作为构建侧
        if left_rows <= right_rows {
            JoinAlgorithm::HashJoin {
                build_side: left_id.to_string(),
                probe_side: right_id.to_string(),
            }
        } else {
            JoinAlgorithm::HashJoin {
                build_side: right_id.to_string(),
                probe_side: left_id.to_string(),
            }
        }
    }

    /// 重构连接顺序
    fn reconstruct_order(
        &self,
        solution: &SubproblemSolution,
        dp: &HashMap<u32, SubproblemSolution>,
        tables: &[TableInfo],
    ) -> Vec<String> {
        let mut order = Vec::new();
        let mut current_set = solution.table_set;

        // 从后向前重构
        while current_set != 0 {
            if let Some(sol) = dp.get(&current_set) {
                order.push(sol.last_table.clone());

                // 找到对应的表并清除位
                if let Some(table) = tables.iter().find(|t| t.id == sol.last_table) {
                    current_set &= !(1 << table.bit_id);
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        order.reverse();
        order
    }

    /// 为连接顺序选择算法
    fn select_algorithms(
        &self,
        order: &[String],
        _conditions: &[JoinCondition],
        tables: &[TableInfo],
    ) -> Vec<JoinAlgorithm> {
        let mut algorithms = Vec::new();
        let table_map: HashMap<String, &TableInfo> =
            tables.iter().map(|t| (t.id.clone(), t)).collect();

        for i in 1..order.len() {
            let left = &order[i - 1];
            let right = &order[i];

            // 获取表的索引信息
            let left_info = table_map.get(left);
            let right_info = table_map.get(right);

            let left_rows = left_info.map(|t| t.estimated_rows).unwrap_or(0);
            let right_rows = right_info.map(|t| t.estimated_rows).unwrap_or(0);
            let left_has_index = left_info.map(|t| t.has_index).unwrap_or(false);
            let right_has_index = right_info.map(|t| t.has_index).unwrap_or(false);

            // 使用基于代价的算法选择
            let algorithm = self.select_algorithm(
                left_rows,
                right_rows,
                left_has_index,
                right_has_index,
                left,
                right,
            );

            algorithms.push(algorithm);
        }

        algorithms
    }

    /// 生成 JoinOrderDecision
    pub fn to_decision(&self, result: &JoinOrderResult) -> JoinOrderDecision {
        let mut decision = JoinOrderDecision::empty();

        for (i, table) in result.order.iter().enumerate() {
            if i < result.algorithms.len() {
                decision.add_join_step(table.clone(), result.algorithms[i].clone());
            } else {
                decision.add_join_step(table.clone(), JoinAlgorithm::HashJoin {
                    build_side: "default".to_string(),
                    probe_side: "default".to_string(),
                });
            }
        }

        decision
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::optimizer::stats::StatisticsManager;

    fn create_test_optimizer() -> JoinOrderOptimizer {
        let stats_manager = Arc::new(StatisticsManager::new());
        let cost_calculator = Arc::new(CostCalculator::new(stats_manager));
        JoinOrderOptimizer::new(cost_calculator)
    }

    fn create_test_tables() -> Vec<TableInfo> {
        vec![
            TableInfo::new("A".to_string(), 1000).with_bit_id(0),
            TableInfo::new("B".to_string(), 500).with_bit_id(1),
            TableInfo::new("C".to_string(), 2000).with_bit_id(2),
        ]
    }

    #[test]
    fn test_single_table() {
        let optimizer = create_test_optimizer();
        let tables = vec![TableInfo::new("A".to_string(), 1000)];
        let result = optimizer.optimize_join_order(&tables, &[]);

        assert_eq!(result.order.len(), 1);
        assert_eq!(result.order[0], "A");
        assert_eq!(result.total_cost, 0.0);
    }

    #[test]
    fn test_two_tables() {
        let optimizer = create_test_optimizer();
        let tables = vec![
            TableInfo::new("A".to_string(), 1000).with_bit_id(0),
            TableInfo::new("B".to_string(), 500).with_bit_id(1),
        ];
        let conditions = vec![JoinCondition::new("A".to_string(), "B".to_string())];

        let result = optimizer.optimize_join_order(&tables, &conditions);

        assert_eq!(result.order.len(), 2);
        assert!(!result.algorithms.is_empty());
    }

    #[test]
    fn test_dp_vs_greedy() {
        let optimizer = create_test_optimizer();
        let tables = create_test_tables();
        let conditions = vec![
            JoinCondition::new("A".to_string(), "B".to_string()).with_selectivity(0.1),
            JoinCondition::new("B".to_string(), "C".to_string()).with_selectivity(0.2),
        ];

        // 使用DP（表数量 <= 8）
        let result = optimizer.optimize_join_order(&tables, &conditions);
        assert_eq!(result.optimization_method, OptimizationMethod::DynamicProgramming);
        assert_eq!(result.order.len(), 3);
    }

    #[test]
    fn test_table_with_selectivity() {
        let table = TableInfo::new("A".to_string(), 1000)
            .with_selectivity(0.5)
            .with_index(true);

        assert_eq!(table.selectivity, 0.5);
        assert!(table.has_index);
    }

    #[test]
    fn test_condition_with_selectivity() {
        let condition = JoinCondition::new("A".to_string(), "B".to_string())
            .with_selectivity(0.25);

        assert_eq!(condition.selectivity, 0.25);
    }

    #[test]
    fn test_to_decision() {
        let optimizer = create_test_optimizer();
        let tables = create_test_tables();
        let conditions = vec![JoinCondition::new("A".to_string(), "B".to_string())];

        let result = optimizer.optimize_join_order(&tables, &conditions);
        let decision = optimizer.to_decision(&result);

        assert_eq!(decision.join_order.len(), result.order.len());
    }
}
