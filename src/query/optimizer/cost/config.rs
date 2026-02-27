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
/// 这些参数可以根据硬件环境进行调整。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CostModelConfig {
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

    /// 哈希构建开销系数
    ///
    /// 构建哈希表的额外开销系数。默认值 0.1
    pub hash_build_overhead: f64,

    /// 排序比较代价系数
    ///
    /// 每次比较操作的代价系数。默认值 1.0
    pub sort_comparison_cost: f64,
}

impl Default for CostModelConfig {
    fn default() -> Self {
        Self {
            seq_page_cost: 1.0,
            random_page_cost: 4.0,
            cpu_tuple_cost: 0.01,
            cpu_index_tuple_cost: 0.005,
            cpu_operator_cost: 0.0025,
            hash_build_overhead: 0.1,
            sort_comparison_cost: 1.0,
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
        assert_eq!(config.seq_page_cost, 1.0);
        assert_eq!(config.random_page_cost, 4.0);
        assert_eq!(config.cpu_tuple_cost, 0.01);
        assert_eq!(config.cpu_index_tuple_cost, 0.005);
        assert_eq!(config.cpu_operator_cost, 0.0025);
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
