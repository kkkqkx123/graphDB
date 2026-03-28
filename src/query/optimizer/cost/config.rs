//! Cost Model Configuration Module
//!
//! Provide configurable cost parameters, referring to the design of the cost model in PostgreSQL.
//!
//! ## Usage Examples
//!
//! ```rust
//! use graphdb::query::optimizer::cost::CostModelConfig;
//!
// Use the default configuration
//! let config = CostModelConfig::default();
//!
// Optimizations for SSDs
//! let ssd_config = CostModelConfig::for_ssd();
//!
// Custom configuration
//! let custom_config = CostModelConfig {
//!     seq_page_cost: 0.5,
//!     random_page_cost: 1.0,
//!     ..Default::default()
//! };
//! ```

/// Cost Model Configuration
///
/// Define the cost parameters for various operations, which are used to calculate the execution cost of the query plan.
/// Refer to the design of the PostgreSQL cost model and extend it to take into account the characteristics of graph databases.
/// These parameters can be adjusted according to the hardware environment.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CostModelConfig {
    // ==================== Basic I/O Cost Parameters (Consistent with PostgreSQL) ====================
    /// Alternative to reading pages in order
    ///
    /// 顺序读取一个磁盘页面的成本。默认值 1.0
    /// In an SSD environment, the requirements can be appropriately reduced.
    pub seq_page_cost: f64,

    /// Alternative to reading a random page:
    ///
    /// 随机读取一个磁盘页面的成本。默认值 4.0
    /// On traditional mechanical hard drives, random access is much slower than sequential access.
    /// In an SSD environment, the value of `seq_page_cost` can be approached.
    pub random_page_cost: f64,

    /// Cost of CPU processing per row
    ///
    /// 处理每一行数据的 CPU 成本。默认值 0.01
    pub cpu_tuple_cost: f64,

    /// Cost of processing index rows
    ///
    /// 处理每个索引项的 CPU 成本。默认值 0.005
    /// It is usually lower than `cpu_tuple_cost` because the index entries are smaller.
    pub cpu_index_tuple_cost: f64,

    /// The cost of operator calculations
    ///
    /// 执行每个操作符或函数的 CPU 成本。默认值 0.0025
    pub cpu_operator_cost: f64,

    // ==================== Algorithm-related parameters ====================
    /// Hash construction overhead coefficient
    ///
    /// 构建哈希表的额外开销系数。默认值 0.1
    pub hash_build_overhead: f64,

    /// Sorting and comparing cost coefficients
    ///
    /// 每次比较操作的代价系数。默认值 1.0
    pub sort_comparison_cost: f64,

    /// Memory sorting threshold (number of rows)
    ///
    /// If this threshold is exceeded, external sorting will be used. The default value is 10000.
    pub memory_sort_threshold: u64,

    /// Cost of external sorting pages
    ///
    /// 外部排序时读写临时文件的代价。默认值 2.0
    pub external_sort_page_cost: f64,

    // ==================== Parameters specific to graph databases ====================
    /// Edge traversal cost
    ///
    /// 遍历一条边的代价，比顶点处理更复杂。默认值 0.02
    pub edge_traversal_cost: f64,

    /// Multi-hop traversal with an incrementing coefficient at each step
    ///
    /// 每多一跳，代价递增的系数。默认值 1.2
    pub multi_hop_penalty: f64,

    /// Cost of neighbor node lookup
    ///
    /// 查找一个邻居节点的代价。默认值 0.015
    pub neighbor_lookup_cost: f64,

    /// Effective cache size (number of pages)
    ///
    /// Used for cost calculation in cache-aware scenarios. Default value: 10000
    pub effective_cache_pages: u64,

    /// Cache Hit Cost Coefficient
    ///
    /// 数据在缓存中时的代价系数。默认值 0.1
    pub cache_hit_cost_factor: f64,

    /// Fundamentals of the cost in shortest path algorithms
    ///
    /// 最短路径算法的固定开销。默认值 10.0
    pub shortest_path_base_cost: f64,

    /// Path enumeration exponent coefficient
    ///
    /// 枚举所有路径时的复杂度系数。默认值 2.0
    pub path_enumeration_factor: f64,

    /// Super Node Threshold (Degree)
    ///
    /// Nodes that exceed this degree are considered super nodes. The default value is 10000.
    pub super_node_threshold: u64,

    /// Super Node Additional Cost Coefficient
    ///
    /// 涉及超级节点时的额外代价。默认值 2.0
    pub super_node_penalty: f64,

    // ==================== Default parameters for control flow ====================
    /// The default list size for “Unwind”.
    ///
    /// 当无法从表达式推断列表大小时使用的默认值。默认值 3.0
    pub default_unwind_list_size: f64,

    /// The default number of iterations for the Loop function
    ///
    /// The default value used when the number of iterations cannot be determined from the conditions. The default value is 3.
    pub default_loop_iterations: u32,

    /// Select the default number of branches.
    ///
    /// The default value used when the number of branches cannot be determined. The default value is 2.
    pub default_select_branches: usize,

    // ==================== Memory and expression cost parameters ====================
    /// Cost per byte of memory usage
    ///
    /// 每字节内存使用成本（用于估算内存压力）。默认值 0.0001
    pub memory_byte_cost: f64,

    /// Memory pressure threshold (bytes)
    ///
    /// 内存压力阈值（超过此值增加成本惩罚）。默认值 100MB
    pub memory_pressure_threshold: usize,

    /// Memory pressure penalty factor
    ///
    /// 内存压力惩罚系数。默认值 2.0
    pub memory_pressure_penalty: f64,

    /// Simple expression (leaf node) cost
    ///
    /// 简单表达式（叶子节点）成本。默认值 0.001
    pub simple_expression_cost: f64,

    /// Function call base cost
    ///
    /// 函数调用基础成本。默认值 0.01
    pub function_call_base_cost: f64,

    /// Expression nesting cost per level
    ///
    /// 复杂表达式每层嵌套成本。默认值 0.005
    pub expression_nesting_cost: f64,

    /// Fixed-size type cost factor
    ///
    /// 固定大小类型处理成本系数。默认值 1.0
    pub fixed_type_cost_factor: f64,

    /// Variable-length type cost factor
    ///
    /// 变长类型处理成本系数。默认值 1.5
    pub variable_type_cost_factor: f64,

    /// Complex type cost factor
    ///
    /// 复杂类型处理成本系数。默认值 2.0
    pub complex_type_cost_factor: f64,

    /// Graph type cost factor
    ///
    /// 图类型处理成本系数。默认值 3.0
    pub graph_type_cost_factor: f64,

    // ==================== Strategy Threshold Parameters ====================
    /// Strategy threshold configuration
    ///
    /// 策略阈值配置，用于控制各种优化策略的行为
    pub strategy_thresholds: StrategyThresholds,
}

/// Strategy Threshold Configuration
///
/// 定义各种优化策略的阈值参数，用于控制策略选择行为。
/// 这些参数可以根据工作负载特征进行调整。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StrategyThresholds {
    // ==================== Aggregate Strategy ====================
    /// Small dataset threshold for aggregate strategy
    ///
    /// 小数据集阈值（行数），低于此值使用简单聚合策略。默认值 1000
    pub small_dataset_threshold: u64,

    /// Low cardinality threshold for aggregate strategy
    ///
    /// 低基数阈值，低于此值使用排序聚合。默认值 100
    pub low_cardinality_threshold: u64,

    /// High cardinality ratio threshold
    ///
    /// 高基数比例阈值（相对于输入行数），高于此值使用流式聚合。默认值 0.1
    pub high_cardinality_ratio: f64,

    // ==================== Traversal Strategy ====================
    /// Super node threshold (degree)
    ///
    /// 超级节点阈值（度数），超过此值视为超级节点。默认值 1000.0
    pub traversal_super_node_threshold: f64,

    /// Bidirectional traversal savings threshold
    ///
    /// 双向遍历节省阈值，超过此值才使用双向遍历。默认值 0.3
    pub bidirectional_savings_threshold: f64,

    /// Default branching factor for traversal
    ///
    /// 遍历默认分支因子，用于成本估计。默认值 2.0
    pub default_branching_factor: f64,

    // ==================== TopN Strategy ====================
    /// TopN selectivity threshold
    ///
    /// TopN选择性阈值，低于此值才使用TopN优化。默认值 0.1
    pub topn_threshold: f64,

    /// TopN default limit
    ///
    /// TopN默认限制行数。默认值 100
    pub topn_default_limit: u64,

    // ==================== Materialization Strategy ====================
    /// Maximum result rows for CTE materialization
    ///
    /// CTE物化的最大结果行数。默认值 10000
    pub max_result_rows: u64,

    /// Minimum reference count for CTE materialization
    ///
    /// CTE物化的最小引用次数。默认值 2
    pub min_reference_count: usize,

    /// Minimum complexity score for CTE materialization
    ///
    /// CTE物化的最小复杂度分数。默认值 5.0
    pub min_complexity_score: f64,
}

impl Default for StrategyThresholds {
    fn default() -> Self {
        Self {
            // Aggregate strategy
            small_dataset_threshold: 1000,
            low_cardinality_threshold: 100,
            high_cardinality_ratio: 0.1,
            // Traversal strategy
            traversal_super_node_threshold: 1000.0,
            bidirectional_savings_threshold: 0.3,
            default_branching_factor: 2.0,
            // TopN strategy
            topn_threshold: 0.1,
            topn_default_limit: 100,
            // Materialization strategy
            max_result_rows: 10000,
            min_reference_count: 2,
            min_complexity_score: 5.0,
        }
    }
}

impl Default for CostModelConfig {
    fn default() -> Self {
        Self {
            // Basic I/O cost parameters (consistent with PostgreSQL)
            seq_page_cost: 1.0,
            random_page_cost: 4.0,
            cpu_tuple_cost: 0.01,
            cpu_index_tuple_cost: 0.005,
            cpu_operator_cost: 0.0025,
            // Algorithm-related parameters
            hash_build_overhead: 0.1,
            sort_comparison_cost: 1.0,
            memory_sort_threshold: 10000,
            external_sort_page_cost: 2.0,
            // Parameter specific to graph databases
            edge_traversal_cost: 0.02,
            multi_hop_penalty: 1.2,
            neighbor_lookup_cost: 0.015,
            effective_cache_pages: 10000,
            cache_hit_cost_factor: 0.1,
            shortest_path_base_cost: 10.0,
            path_enumeration_factor: 2.0,
            super_node_threshold: 10000,
            super_node_penalty: 2.0,
            // Default parameters for control flow
            default_unwind_list_size: 3.0,
            default_loop_iterations: 3,
            default_select_branches: 2,
            // Memory and expression cost parameters
            memory_byte_cost: 0.0001,
            memory_pressure_threshold: 100 * 1024 * 1024, // 100MB
            memory_pressure_penalty: 2.0,
            simple_expression_cost: 0.001,
            function_call_base_cost: 0.01,
            expression_nesting_cost: 0.005,
            fixed_type_cost_factor: 1.0,
            variable_type_cost_factor: 1.5,
            complex_type_cost_factor: 2.0,
            graph_type_cost_factor: 3.0,
            // Strategy thresholds
            strategy_thresholds: StrategyThresholds::default(),
        }
    }
}

impl CostModelConfig {
    /// Create the default configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Optimizations for SSD storage
    ///
    /// The random access performance of SSDs is close to that of sequential access; therefore, reducing the value of `random_page_cost` has a positive impact on overall system performance.
    pub fn for_ssd() -> Self {
        Self {
            random_page_cost: 1.1, // Random access to SSDs is similar to sequential access.
            ..Default::default()
        }
    }

    /// Optimizations for in-memory databases
    ///
    /// There are no disk I/O overheads in memory; the main consideration is the cost associated with the CPU.
    pub fn for_in_memory() -> Self {
        Self {
            seq_page_cost: 0.1,
            random_page_cost: 0.1,
            cache_hit_cost_factor: 0.01, // A cache hit incurs almost no cost.
            ..Default::default()
        }
    }

    /// Optimizations for mechanical hard drives (conservative configuration)
    ///
    /// Traditional mechanical hard drives have poor random access performance.
    pub fn for_hdd() -> Self {
        Self {
            seq_page_cost: 1.0,
            random_page_cost: 4.0,
            ..Default::default()
        }
    }

    /// Set the order for reading pages as an alternative option.
    pub fn with_seq_page_cost(mut self, cost: f64) -> Self {
        self.seq_page_cost = cost;
        self
    }

    /// Set a substitute price for random page reading.
    pub fn with_random_page_cost(mut self, cost: f64) -> Self {
        self.random_page_cost = cost;
        self
    }

    /// Setting the CPU cost for line processing
    pub fn with_cpu_tuple_cost(mut self, cost: f64) -> Self {
        self.cpu_tuple_cost = cost;
        self
    }

    /// Setting the cost of processing index rows
    pub fn with_cpu_index_tuple_cost(mut self, cost: f64) -> Self {
        self.cpu_index_tuple_cost = cost;
        self
    }

    /// Setting the cost of operator calculations
    pub fn with_cpu_operator_cost(mut self, cost: f64) -> Self {
        self.cpu_operator_cost = cost;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = CostModelConfig::default();
        // Basic I/O Cost Parameters
        assert_eq!(config.seq_page_cost, 1.0);
        assert_eq!(config.random_page_cost, 4.0);
        assert_eq!(config.cpu_tuple_cost, 0.01);
        assert_eq!(config.cpu_index_tuple_cost, 0.005);
        assert_eq!(config.cpu_operator_cost, 0.0025);
        // Algorithm-related parameters
        assert_eq!(config.hash_build_overhead, 0.1);
        assert_eq!(config.sort_comparison_cost, 1.0);
        assert_eq!(config.memory_sort_threshold, 10000);
        assert_eq!(config.external_sort_page_cost, 2.0);
        // Unique parameters specific to graph databases
        assert_eq!(config.edge_traversal_cost, 0.02);
        assert_eq!(config.multi_hop_penalty, 1.2);
        assert_eq!(config.neighbor_lookup_cost, 0.015);
        assert_eq!(config.effective_cache_pages, 10000);
        assert_eq!(config.cache_hit_cost_factor, 0.1);
        assert_eq!(config.shortest_path_base_cost, 10.0);
        assert_eq!(config.path_enumeration_factor, 2.0);
        assert_eq!(config.super_node_threshold, 10000);
        assert_eq!(config.super_node_penalty, 2.0);
        // Default parameters for control flow
        assert_eq!(config.default_unwind_list_size, 3.0);
        assert_eq!(config.default_loop_iterations, 3);
        assert_eq!(config.default_select_branches, 2);
    }

    #[test]
    fn test_ssd_config() {
        let config = CostModelConfig::for_ssd();
        assert_eq!(config.random_page_cost, 1.1);
        assert_eq!(config.seq_page_cost, 1.0); // The rest should remain as default.
    }

    #[test]
    fn test_in_memory_config() {
        let config = CostModelConfig::for_in_memory();
        assert_eq!(config.seq_page_cost, 0.1);
        assert_eq!(config.random_page_cost, 0.1);
    }

    #[test]
    fn test_builder_pattern() {
        let config = CostModelConfig::new()
            .with_seq_page_cost(0.5)
            .with_random_page_cost(2.0)
            .with_cpu_tuple_cost(0.02);

        assert_eq!(config.seq_page_cost, 0.5);
        assert_eq!(config.random_page_cost, 2.0);
        assert_eq!(config.cpu_tuple_cost, 0.02);
        assert_eq!(config.cpu_index_tuple_cost, 0.005); // “Default”
    }
}
