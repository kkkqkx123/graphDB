//! 代价模型配置模块
//!
//! 提供可配置的代价参数，参考 PostgreSQL 的代价模型设计
//!
//! ## 使用示例
//!
//! ```rust
//! use graphdb::query::optimizer::cost::CostModelConfig;
//!
//! // 使用默认配置
//! let config = CostModelConfig::default();
//!
//! // 针对 SSD 优化
//! let ssd_config = CostModelConfig::for_ssd();
//!
//! // 自定义配置
//! let custom_config = CostModelConfig {
//!     seq_page_cost: 0.5,
//!     random_page_cost: 1.0,
//!     ..Default::default()
//! };
//! ```

/// 代价模型配置
///
/// 定义各种操作的代价参数，用于计算查询计划的执行代价。
/// 参考 PostgreSQL 代价模型设计，并针对图数据库特性进行扩展。
/// 这些参数可以根据硬件环境进行调整。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CostModelConfig {
    // ==================== 基础 I/O 代价参数（与 PostgreSQL 一致） ====================
    /// 顺序页读取代价
    ///
    /// 顺序读取一个磁盘页面的成本。默认值 1.0
    /// 在 SSD 环境下可以适当降低
    pub seq_page_cost: f64,

    /// 随机页读取代价
    ///
    /// 随机读取一个磁盘页面的成本。默认值 4.0
    /// 传统机械硬盘上随机访问比顺序访问慢很多
    /// SSD 环境下可以接近 seq_page_cost
    pub random_page_cost: f64,

    /// 行处理 CPU 代价
    ///
    /// 处理每一行数据的 CPU 成本。默认值 0.01
    pub cpu_tuple_cost: f64,

    /// 索引行处理代价
    ///
    /// 处理每个索引项的 CPU 成本。默认值 0.005
    /// 通常比 cpu_tuple_cost 低，因为索引项较小
    pub cpu_index_tuple_cost: f64,

    /// 操作符计算代价
    ///
    /// 执行每个操作符或函数的 CPU 成本。默认值 0.0025
    pub cpu_operator_cost: f64,

    // ==================== 算法相关参数 ====================
    /// 哈希构建开销系数
    ///
    /// 构建哈希表的额外开销系数。默认值 0.1
    pub hash_build_overhead: f64,

    /// 排序比较代价系数
    ///
    /// 每次比较操作的代价系数。默认值 1.0
    pub sort_comparison_cost: f64,

    /// 内存排序阈值（行数）
    ///
    /// 超过此阈值将使用外部排序。默认值 10000
    pub memory_sort_threshold: u64,

    /// 外部排序页代价
    ///
    /// 外部排序时读写临时文件的代价。默认值 2.0
    pub external_sort_page_cost: f64,

    // ==================== 图数据库特有参数 ====================
    /// 边遍历代价
    ///
    /// 遍历一条边的代价，比顶点处理更复杂。默认值 0.02
    pub edge_traversal_cost: f64,

    /// 多跳遍历每步递增系数
    ///
    /// 每多一跳，代价递增的系数。默认值 1.2
    pub multi_hop_penalty: f64,

    /// 邻居节点查找代价
    ///
    /// 查找一个邻居节点的代价。默认值 0.015
    pub neighbor_lookup_cost: f64,

    /// 有效缓存大小（页数）
    ///
    /// 用于缓存感知的代价计算。默认值 10000
    pub effective_cache_pages: u64,

    /// 缓存命中代价系数
    ///
    /// 数据在缓存中时的代价系数。默认值 0.1
    pub cache_hit_cost_factor: f64,

    /// 最短路径算法基础代价
    ///
    /// 最短路径算法的固定开销。默认值 10.0
    pub shortest_path_base_cost: f64,

    /// 路径枚举指数系数
    ///
    /// 枚举所有路径时的复杂度系数。默认值 2.0
    pub path_enumeration_factor: f64,

    /// 超级节点阈值（度数）
    ///
    /// 超过此度数的节点被视为超级节点。默认值 10000
    pub super_node_threshold: u64,

    /// 超级节点额外代价系数
    ///
    /// 涉及超级节点时的额外代价。默认值 2.0
    pub super_node_penalty: f64,

    // ==================== 控制流默认参数 ====================
    /// Unwind 默认列表大小
    ///
    /// 当无法从表达式推断列表大小时使用的默认值。默认值 3.0
    pub default_unwind_list_size: f64,

    /// Loop 默认迭代次数
    ///
    /// 当无法从条件推断迭代次数时使用的默认值。默认值 3
    pub default_loop_iterations: u32,

    /// Select 默认分支数
    ///
    /// 当无法确定分支数时使用的默认值。默认值 2
    pub default_select_branches: usize,
}

impl Default for CostModelConfig {
    fn default() -> Self {
        Self {
            // 基础 I/O 代价参数（与 PostgreSQL 一致）
            seq_page_cost: 1.0,
            random_page_cost: 4.0,
            cpu_tuple_cost: 0.01,
            cpu_index_tuple_cost: 0.005,
            cpu_operator_cost: 0.0025,
            // 算法相关参数
            hash_build_overhead: 0.1,
            sort_comparison_cost: 1.0,
            memory_sort_threshold: 10000,
            external_sort_page_cost: 2.0,
            // 图数据库特有参数
            edge_traversal_cost: 0.02,
            multi_hop_penalty: 1.2,
            neighbor_lookup_cost: 0.015,
            effective_cache_pages: 10000,
            cache_hit_cost_factor: 0.1,
            shortest_path_base_cost: 10.0,
            path_enumeration_factor: 2.0,
            super_node_threshold: 10000,
            super_node_penalty: 2.0,
            // 控制流默认参数
            default_unwind_list_size: 3.0,
            default_loop_iterations: 3,
            default_select_branches: 2,
        }
    }
}

impl CostModelConfig {
    /// 创建默认配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 针对 SSD 存储优化
    ///
    /// SSD 的随机访问性能接近顺序访问，因此降低 random_page_cost
    pub fn for_ssd() -> Self {
        Self {
            random_page_cost: 1.1, // SSD 随机访问接近顺序访问
            ..Default::default()
        }
    }

    /// 针对内存数据库优化
    ///
    /// 内存中没有磁盘 IO 开销，主要考虑 CPU 代价
    pub fn for_in_memory() -> Self {
        Self {
            seq_page_cost: 0.1,
            random_page_cost: 0.1,
            cache_hit_cost_factor: 0.01, // 缓存命中几乎无代价
            ..Default::default()
        }
    }

    /// 针对机械硬盘优化（保守配置）
    ///
    /// 传统机械硬盘随机访问性能较差
    pub fn for_hdd() -> Self {
        Self {
            seq_page_cost: 1.0,
            random_page_cost: 4.0,
            ..Default::default()
        }
    }

    /// 设置顺序页读取代价
    pub fn with_seq_page_cost(mut self, cost: f64) -> Self {
        self.seq_page_cost = cost;
        self
    }

    /// 设置随机页读取代价
    pub fn with_random_page_cost(mut self, cost: f64) -> Self {
        self.random_page_cost = cost;
        self
    }

    /// 设置行处理 CPU 代价
    pub fn with_cpu_tuple_cost(mut self, cost: f64) -> Self {
        self.cpu_tuple_cost = cost;
        self
    }

    /// 设置索引行处理代价
    pub fn with_cpu_index_tuple_cost(mut self, cost: f64) -> Self {
        self.cpu_index_tuple_cost = cost;
        self
    }

    /// 设置操作符计算代价
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
        // 基础 I/O 代价参数
        assert_eq!(config.seq_page_cost, 1.0);
        assert_eq!(config.random_page_cost, 4.0);
        assert_eq!(config.cpu_tuple_cost, 0.01);
        assert_eq!(config.cpu_index_tuple_cost, 0.005);
        assert_eq!(config.cpu_operator_cost, 0.0025);
        // 算法相关参数
        assert_eq!(config.hash_build_overhead, 0.1);
        assert_eq!(config.sort_comparison_cost, 1.0);
        assert_eq!(config.memory_sort_threshold, 10000);
        assert_eq!(config.external_sort_page_cost, 2.0);
        // 图数据库特有参数
        assert_eq!(config.edge_traversal_cost, 0.02);
        assert_eq!(config.multi_hop_penalty, 1.2);
        assert_eq!(config.neighbor_lookup_cost, 0.015);
        assert_eq!(config.effective_cache_pages, 10000);
        assert_eq!(config.cache_hit_cost_factor, 0.1);
        assert_eq!(config.shortest_path_base_cost, 10.0);
        assert_eq!(config.path_enumeration_factor, 2.0);
        assert_eq!(config.super_node_threshold, 10000);
        assert_eq!(config.super_node_penalty, 2.0);
        // 控制流默认参数
        assert_eq!(config.default_unwind_list_size, 3.0);
        assert_eq!(config.default_loop_iterations, 3);
        assert_eq!(config.default_select_branches, 2);
    }

    #[test]
    fn test_ssd_config() {
        let config = CostModelConfig::for_ssd();
        assert_eq!(config.random_page_cost, 1.1);
        assert_eq!(config.seq_page_cost, 1.0); // 其他保持默认
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
        assert_eq!(config.cpu_index_tuple_cost, 0.005); // 默认
    }
}
