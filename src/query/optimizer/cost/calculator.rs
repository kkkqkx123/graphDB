//! 代价计算器模块
//!
//! 针对图数据库特性设计的轻量级代价计算
//!
//! ## 使用示例
//!
//! ```rust
//! use graphdb::query::optimizer::cost::{CostCalculator, CostModelConfig};
//! use graphdb::query::optimizer::stats::StatisticsManager;
//! use std::sync::Arc;
//!
//! let stats_manager = Arc::new(StatisticsManager::new());
//! let config = CostModelConfig::default();
//! let calculator = CostCalculator::with_config(stats_manager, config);
//!
//! // 计算扫描代价
//! let scan_cost = calculator.calculate_scan_vertices_cost("Person");
//! ```

use std::sync::Arc;

use crate::query::optimizer::stats::StatisticsManager;

use super::config::CostModelConfig;

/// 代价计算器
///
/// 针对图数据库特性设计的轻量级代价计算
#[derive(Debug, Clone)]
pub struct CostCalculator {
    stats_manager: Arc<StatisticsManager>,
    config: CostModelConfig,
}

impl CostCalculator {
    /// 创建新的代价计算器（使用默认配置）
    pub fn new(stats_manager: Arc<StatisticsManager>) -> Self {
        Self {
            stats_manager,
            config: CostModelConfig::default(),
        }
    }

    /// 创建新的代价计算器（使用指定配置）
    pub fn with_config(stats_manager: Arc<StatisticsManager>, config: CostModelConfig) -> Self {
        Self {
            stats_manager,
            config,
        }
    }

    /// 获取配置
    pub fn config(&self) -> &CostModelConfig {
        &self.config
    }

    /// 更新配置
    pub fn set_config(&mut self, config: CostModelConfig) {
        self.config = config;
    }

    // ==================== 扫描操作 ====================

    /// 计算全表扫描顶点代价
    ///
    /// 公式：行数 × CPU处理代价
    pub fn calculate_scan_vertices_cost(&self, tag_name: &str) -> f64 {
        let row_count = self.stats_manager.get_vertex_count(tag_name);
        row_count as f64 * self.config.cpu_tuple_cost
    }

    /// 计算全表扫描边代价
    ///
    /// 公式：边数 × CPU处理代价
    pub fn calculate_scan_edges_cost(&self, edge_type: &str) -> f64 {
        let edge_count = self.stats_manager.get_edge_count(edge_type);
        edge_count as f64 * self.config.cpu_tuple_cost
    }

    /// 计算索引扫描代价
    ///
    /// 公式：索引访问代价 + 回表代价
    ///
    /// # 参数
    /// - `tag_name`: 标签名称
    /// - `property_name`: 属性名称
    /// - `selectivity`: 选择性（0.0 ~ 1.0）
    pub fn calculate_index_scan_cost(
        &self,
        tag_name: &str,
        _property_name: &str,
        selectivity: f64,
    ) -> f64 {
        let table_rows = self.stats_manager.get_vertex_count(tag_name);
        let matching_rows = (selectivity * table_rows as f64).max(1.0) as u64;

        // 索引访问代价（顺序IO）
        let index_pages = (matching_rows / 10).max(1);
        let index_access_cost = index_pages as f64 * self.config.seq_page_cost
            + matching_rows as f64 * self.config.cpu_index_tuple_cost;

        // 回表代价（随机IO）
        let table_access_cost = matching_rows as f64 * self.config.random_page_cost
            + matching_rows as f64 * self.config.cpu_tuple_cost;

        index_access_cost + table_access_cost
    }

    /// 计算边索引扫描代价
    pub fn calculate_edge_index_scan_cost(
        &self,
        edge_type: &str,
        selectivity: f64,
    ) -> f64 {
        let edge_count = self.stats_manager.get_edge_count(edge_type);
        let matching_rows = (selectivity * edge_count as f64).max(1.0) as u64;

        let index_pages = (matching_rows / 10).max(1);
        let index_access_cost = index_pages as f64 * self.config.seq_page_cost
            + matching_rows as f64 * self.config.cpu_index_tuple_cost;

        let table_access_cost = matching_rows as f64 * self.config.random_page_cost
            + matching_rows as f64 * self.config.cpu_tuple_cost;

        index_access_cost + table_access_cost
    }

    // ==================== 图遍历操作 ====================

    /// 计算单步扩展代价
    ///
    /// # 参数
    /// - `start_nodes`: 起始节点数量
    /// - `edge_type`: 边类型（可选）
    pub fn calculate_expand_cost(&self, start_nodes: u64, edge_type: Option<&str>) -> f64 {
        let (avg_degree, is_super_node) = match edge_type {
            Some(et) => self
                .stats_manager
                .get_edge_stats(et)
                .map(|s| {
                    let is_super = s.avg_out_degree > self.config.super_node_threshold as f64;
                    (s.avg_out_degree, is_super)
                })
                .unwrap_or((2.0, false)),
            None => (2.0, false), // 默认平均度数
        };

        let output_rows = (start_nodes as f64 * avg_degree) as u64;

        // IO代价：读取边数据（考虑缓存）
        let io_cost = self.calculate_io_cost(output_rows);
        // CPU代价：边遍历代价（比顶点处理更复杂）
        let cpu_cost = output_rows as f64 * self.config.edge_traversal_cost;

        let base_cost = io_cost + cpu_cost;

        // 超级节点额外代价惩罚
        if is_super_node {
            base_cost * self.config.super_node_penalty
        } else {
            base_cost
        }
    }

    /// 计算全扩展代价（ExpandAll）
    pub fn calculate_expand_all_cost(&self, start_nodes: u64, edge_type: Option<&str>) -> f64 {
        // ExpandAll 比 Expand 返回更多数据（包括顶点信息）
        let base_cost = self.calculate_expand_cost(start_nodes, edge_type);
        // 额外 50% 开销用于获取顶点信息
        base_cost * 1.5
    }

    /// 计算多步遍历代价
    ///
    /// # 参数
    /// - `start_nodes`: 起始节点数量
    /// - `edge_type`: 边类型（可选）
    /// - `steps`: 遍历步数
    pub fn calculate_traverse_cost(
        &self,
        start_nodes: u64,
        edge_type: Option<&str>,
        steps: u32,
    ) -> f64 {
        let avg_degree = match edge_type {
            Some(et) => self
                .stats_manager
                .get_edge_stats(et)
                .map(|s| (s.avg_out_degree + s.avg_in_degree) / 2.0)
                .unwrap_or(2.0),
            None => 2.0,
        };

        // 计算每步的输出行数累加（考虑多跳惩罚）
        let mut total_cost = 0.0;
        let mut current_rows = start_nodes as f64;

        for step in 0..steps {
            current_rows *= avg_degree;
            // 每多一跳，代价递增
            let step_penalty = self.config.multi_hop_penalty.powi(step as i32);
            let step_cost = current_rows * self.config.edge_traversal_cost * step_penalty;
            let io_cost = self.calculate_io_cost(current_rows as u64);
            total_cost += step_cost + io_cost;
        }

        total_cost
    }

    /// 计算追加顶点代价
    pub fn calculate_append_vertices_cost(&self, input_rows: u64) -> f64 {
        // 为每行输入追加顶点信息
        input_rows as f64 * self.config.cpu_tuple_cost * 2.0
    }

    /// 计算获取邻居节点代价
    pub fn calculate_get_neighbors_cost(&self, start_nodes: u64, edge_type: Option<&str>) -> f64 {
        let avg_degree = match edge_type {
            Some(et) => self
                .stats_manager
                .get_edge_stats(et)
                .map(|s| s.avg_out_degree)
                .unwrap_or(2.0),
            None => 2.0,
        };

        let neighbor_count = (start_nodes as f64 * avg_degree) as u64;
        let lookup_cost = neighbor_count as f64 * self.config.neighbor_lookup_cost;
        let io_cost = self.calculate_io_cost(neighbor_count);

        lookup_cost + io_cost
    }

    /// 计算获取顶点代价
    pub fn calculate_get_vertices_cost(&self, vid_count: u64) -> f64 {
        vid_count as f64 * self.config.random_page_cost
    }

    /// 计算获取边代价
    pub fn calculate_get_edges_cost(&self, edge_count: u64) -> f64 {
        edge_count as f64 * self.config.random_page_cost
    }

    // ==================== 过滤和投影 ====================

    /// 计算过滤代价
    ///
    /// 公式：输入行数 × 条件数量 × 操作符代价
    ///
    /// # 参数
    /// - `input_rows`: 输入行数
    /// - `condition_count`: 条件数量
    pub fn calculate_filter_cost(&self, input_rows: u64, condition_count: usize) -> f64 {
        input_rows as f64 * condition_count as f64 * self.config.cpu_operator_cost
    }

    /// 计算投影代价
    ///
    /// # 参数
    /// - `input_rows`: 输入行数
    /// - `columns`: 投影列数
    pub fn calculate_project_cost(&self, input_rows: u64, columns: usize) -> f64 {
        input_rows as f64 * columns as f64 * self.config.cpu_operator_cost
    }

    // ==================== 连接操作 ====================

    /// 计算哈希内连接代价
    ///
    /// # 参数
    /// - `left_rows`: 左表行数
    /// - `right_rows`: 右表行数
    pub fn calculate_hash_join_cost(&self, left_rows: u64, right_rows: u64) -> f64 {
        // 构建哈希表代价
        let build_cost = left_rows as f64 * self.config.cpu_tuple_cost;
        // 探测代价
        let probe_cost = right_rows as f64 * self.config.cpu_tuple_cost;
        // 哈希构建开销
        let hash_overhead = left_rows as f64 * self.config.hash_build_overhead * self.config.cpu_operator_cost;

        build_cost + probe_cost + hash_overhead
    }

    /// 计算哈希左连接代价
    pub fn calculate_hash_left_join_cost(&self, left_rows: u64, right_rows: u64) -> f64 {
        // 左连接与内连接代价类似，但可能有更多输出
        self.calculate_hash_join_cost(left_rows, right_rows) * 1.1
    }

    /// 计算内连接代价（非哈希）
    pub fn calculate_inner_join_cost(&self, left_rows: u64, right_rows: u64) -> f64 {
        // 使用嵌套循环连接的估算
        self.calculate_nested_loop_join_cost(left_rows, right_rows)
    }

    /// 计算左连接代价（非哈希）
    pub fn calculate_left_join_cost(&self, left_rows: u64, right_rows: u64) -> f64 {
        self.calculate_nested_loop_join_cost(left_rows, right_rows) * 1.1
    }

    /// 计算交叉连接代价
    pub fn calculate_cross_join_cost(&self, left_rows: u64, right_rows: u64) -> f64 {
        let output_rows = left_rows as f64 * right_rows as f64;
        output_rows * self.config.cpu_tuple_cost
    }

    /// 计算嵌套循环连接代价
    pub fn calculate_nested_loop_join_cost(&self, left_rows: u64, right_rows: u64) -> f64 {
        let outer_cost = left_rows as f64 * self.config.cpu_tuple_cost;
        let inner_cost = left_rows as f64 * right_rows as f64 * self.config.cpu_tuple_cost;

        outer_cost + inner_cost
    }

    /// 计算全外连接代价
    pub fn calculate_full_outer_join_cost(&self, left_rows: u64, right_rows: u64) -> f64 {
        let base_cost = self.calculate_hash_join_cost(left_rows, right_rows);
        base_cost * 1.5 // 全外连接更复杂
    }

    // ==================== 排序和聚合 ====================

    /// 计算排序代价
    ///
    /// 公式：输入行数 × log(输入行数) × 比较代价
    /// 超过内存阈值时使用外部排序
    ///
    /// # 参数
    /// - `input_rows`: 输入行数
    /// - `sort_columns`: 排序列数
    pub fn calculate_sort_cost(&self, input_rows: u64, sort_columns: usize) -> f64 {
        if input_rows == 0 {
            return 0.0;
        }
        let rows = input_rows as f64;
        let comparisons = rows * rows.log2().max(1.0);
        let cpu_cost = comparisons * sort_columns as f64 * self.config.cpu_operator_cost * self.config.sort_comparison_cost;

        // 判断是否使用外部排序
        if input_rows > self.config.memory_sort_threshold {
            // 外部排序：需要读写临时文件
            let pages = (input_rows / 100).max(1); // 假设每页100行
            let io_cost = pages as f64 * self.config.external_sort_page_cost * 2.0; // 读写两次
            cpu_cost + io_cost
        } else {
            cpu_cost
        }
    }

    /// 计算Limit代价
    pub fn calculate_limit_cost(&self, input_rows: u64, _limit: i64) -> f64 {
        // Limit 主要是内存操作，代价较低
        input_rows as f64 * self.config.cpu_operator_cost * 0.5
    }

    /// 计算TopN代价（优先队列）
    ///
    /// 比全排序更高效，使用堆实现
    pub fn calculate_topn_cost(&self, input_rows: u64, limit: i64) -> f64 {
        let n = input_rows as f64;
        let k = limit as f64;
        // 使用堆的复杂度：n × log(k)
        n * k.log2().max(1.0) * self.config.cpu_operator_cost
    }

    /// 计算聚合代价
    ///
    /// # 参数
    /// - `input_rows`: 输入行数
    /// - `agg_functions`: 聚合函数数量
    pub fn calculate_aggregate_cost(&self, input_rows: u64, agg_functions: usize) -> f64 {
        input_rows as f64 * agg_functions as f64 * self.config.cpu_operator_cost
    }

    /// 计算去重代价（使用哈希表）
    pub fn calculate_dedup_cost(&self, input_rows: u64) -> f64 {
        // 哈希插入和检查的开销
        input_rows as f64 * self.config.cpu_operator_cost * 2.0
    }

    // ==================== 数据处理和集合操作 ====================

    /// 计算Union代价
    pub fn calculate_union_cost(&self, left_rows: u64, right_rows: u64, distinct: bool) -> f64 {
        let base_cost = (left_rows + right_rows) as f64 * self.config.cpu_tuple_cost;
        if distinct {
            // 需要去重
            base_cost + self.calculate_dedup_cost(left_rows + right_rows)
        } else {
            base_cost
        }
    }

    /// 计算Minus代价
    pub fn calculate_minus_cost(&self, left_rows: u64, right_rows: u64) -> f64 {
        let base_cost = (left_rows + right_rows) as f64 * self.config.cpu_tuple_cost;
        // 需要哈希集合操作
        let set_op_cost = right_rows as f64 * self.config.cpu_operator_cost;
        base_cost + set_op_cost
    }

    /// 计算Intersect代价
    pub fn calculate_intersect_cost(&self, left_rows: u64, right_rows: u64) -> f64 {
        let base_cost = (left_rows + right_rows) as f64 * self.config.cpu_tuple_cost;
        let set_op_cost = left_rows.min(right_rows) as f64 * self.config.cpu_operator_cost;
        base_cost + set_op_cost
    }

    /// 计算Unwind代价
    pub fn calculate_unwind_cost(&self, input_rows: u64, avg_list_size: f64) -> f64 {
        let output_rows = input_rows as f64 * avg_list_size;
        output_rows * self.config.cpu_tuple_cost
    }

    /// 计算数据收集代价
    pub fn calculate_data_collect_cost(&self, input_rows: u64) -> f64 {
        input_rows as f64 * self.config.cpu_tuple_cost
    }

    /// 计算采样代价
    pub fn calculate_sample_cost(&self, input_rows: u64) -> f64 {
        // 采样需要遍历数据
        input_rows as f64 * self.config.cpu_operator_cost
    }

    // ==================== 控制流节点 ====================

    /// 计算循环代价
    ///
    /// # 参数
    /// - `body_cost`: 循环体代价
    /// - `iterations`: 估计迭代次数
    pub fn calculate_loop_cost(&self, body_cost: f64, iterations: u32) -> f64 {
        body_cost * iterations as f64
    }

    /// 计算选择节点代价
    pub fn calculate_select_cost(&self, input_rows: u64, branch_count: usize) -> f64 {
        input_rows as f64 * branch_count as f64 * self.config.cpu_operator_cost
    }

    /// 计算透传节点代价
    pub fn calculate_pass_through_cost(&self, input_rows: u64) -> f64 {
        input_rows as f64 * self.config.cpu_operator_cost * 0.1
    }

    // ==================== 图算法 ====================

    /// 计算最短路径代价
    pub fn calculate_shortest_path_cost(&self, start_nodes: u64, max_depth: u32) -> f64 {
        // 基于BFS的复杂度估算
        let avg_branching = 2.0_f64; // 假设平均分支因子
        let explored_nodes = start_nodes as f64 * avg_branching.powf(max_depth as f64);
        let traversal_cost = explored_nodes * self.config.edge_traversal_cost;
        let io_cost = self.calculate_io_cost(explored_nodes as u64);

        // 加上基础开销
        traversal_cost + io_cost + self.config.shortest_path_base_cost
    }

    /// 计算所有路径代价
    pub fn calculate_all_paths_cost(&self, start_nodes: u64, max_depth: u32) -> f64 {
        // 所有路径的复杂度比最短路径高很多
        let base_cost = self.calculate_shortest_path_cost(start_nodes, max_depth);
        base_cost * self.config.path_enumeration_factor
    }

    /// 计算多源最短路径代价
    pub fn calculate_multi_shortest_path_cost(&self, source_count: u64, max_depth: u32) -> f64 {
        self.calculate_shortest_path_cost(source_count, max_depth) * 1.5
    }

    // ==================== 辅助方法 ====================

    /// 获取统计信息管理器
    pub fn statistics_manager(&self) -> Arc<StatisticsManager> {
        self.stats_manager.clone()
    }

    /// 估算标签的选择性
    pub fn estimate_tag_selectivity(&self, tag_name: &str) -> f64 {
        let vertex_count = self.stats_manager.get_vertex_count(tag_name);
        if vertex_count == 0 {
            1.0
        } else {
            // 简化估算：假设标签分布均匀
            0.1
        }
    }

    /// 估算边类型的选择性
    pub fn estimate_edge_selectivity(&self, edge_type: &str) -> f64 {
        let edge_stats = self.stats_manager.get_edge_stats(edge_type);
        match edge_stats {
            Some(stats) if stats.edge_count > 0 => {
                // 基于边数量的估算
                (1.0 / (stats.edge_count as f64).sqrt()).min(1.0).max(0.001)
            }
            _ => 0.1,
        }
    }

    // ==================== 缓存感知 IO 代价计算 ====================

    /// 计算 IO 代价（考虑缓存）
    ///
    /// 根据有效缓存大小调整 I/O 代价：
    /// - 如果访问的数据页数 < effective_cache_pages: 大部分在缓存中
    /// - 否则: 部分需要磁盘 IO
    fn calculate_io_cost(&self, rows: u64) -> f64 {
        // 假设每页 100 行
        let pages = (rows / 100).max(1);

        if pages <= self.config.effective_cache_pages {
            // 数据可能在缓存中
            pages as f64 * self.config.seq_page_cost * self.config.cache_hit_cost_factor
        } else {
            // 部分数据需要从磁盘读取
            let cached_pages = self.config.effective_cache_pages;
            let disk_pages = pages - cached_pages;

            let cached_cost = cached_pages as f64 * self.config.seq_page_cost * self.config.cache_hit_cost_factor;
            let disk_cost = disk_pages as f64 * self.config.seq_page_cost;

            cached_cost + disk_cost
        }
    }
}

impl Default for CostCalculator {
    fn default() -> Self {
        Self {
            stats_manager: Arc::new(StatisticsManager::new()),
            config: CostModelConfig::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_scan_cost() {
        let stats_manager = Arc::new(StatisticsManager::new());
        let calculator = CostCalculator::new(stats_manager);

        // 无统计信息时应该返回 0
        let cost = calculator.calculate_scan_vertices_cost("NonExistent");
        assert_eq!(cost, 0.0);
    }

    #[test]
    fn test_calculate_filter_cost() {
        let stats_manager = Arc::new(StatisticsManager::new());
        let calculator = CostCalculator::new(stats_manager);

        let cost = calculator.calculate_filter_cost(1000, 3);
        assert!(cost > 0.0);
        // 1000 * 3 * 0.0025 = 7.5
        assert_eq!(cost, 7.5);
    }

    #[test]
    fn test_calculate_hash_join_cost() {
        let stats_manager = Arc::new(StatisticsManager::new());
        let calculator = CostCalculator::new(stats_manager);

        let cost = calculator.calculate_hash_join_cost(100, 200);
        assert!(cost > 0.0);
        // (100 + 200) * 0.01 + 100 * 0.1 * 0.0025 = 3.0 + 0.025 = 3.025
        assert_eq!(cost, 3.025);
    }

    #[test]
    fn test_calculate_sort_cost() {
        let stats_manager = Arc::new(StatisticsManager::new());
        let calculator = CostCalculator::new(stats_manager);

        let cost = calculator.calculate_sort_cost(1000, 2);
        assert!(cost > 0.0);

        // 空输入应该返回 0
        let zero_cost = calculator.calculate_sort_cost(0, 2);
        assert_eq!(zero_cost, 0.0);
    }

    #[test]
    fn test_calculate_topn_cost() {
        let stats_manager = Arc::new(StatisticsManager::new());
        let calculator = CostCalculator::new(stats_manager);

        let cost = calculator.calculate_topn_cost(10000, 10);
        assert!(cost > 0.0);
        // 10000 * log2(10) * 0.0025 ≈ 83.05
        assert!(cost > 80.0 && cost < 85.0);
    }

    #[test]
    fn test_with_config() {
        let stats_manager = Arc::new(StatisticsManager::new());
        let config = CostModelConfig::for_ssd();
        let calculator = CostCalculator::with_config(stats_manager, config);

        assert_eq!(calculator.config().random_page_cost, 1.1);
    }
}
