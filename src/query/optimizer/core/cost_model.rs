//! 代价模型配置和计算
//!
//! 提供查询优化的代价计算模型，参考 PostgreSQL 的设计
//! 针对单节点图数据库进行简化

use super::statistics::{TableStatistics, IndexStatistics, StatisticsProvider};

/// 代价模型配置
/// 参考 PostgreSQL 的代价参数设计
#[derive(Debug, Clone, Copy)]
pub struct CostModelConfig {
    /// 顺序页读取代价
    /// 对应 PostgreSQL 的 seq_page_cost
    pub seq_page_cost: f64,

    /// 随机页读取代价
    /// 对应 PostgreSQL 的 random_page_cost
    /// 通常高于顺序页代价，因为随机 I/O 需要更多寻道时间
    pub random_page_cost: f64,

    /// 处理每行数据的 CPU 代价
    /// 对应 PostgreSQL 的 cpu_tuple_cost
    pub cpu_tuple_cost: f64,

    /// 处理每个索引项的 CPU 代价
    /// 对应 PostgreSQL 的 cpu_index_tuple_cost
    pub cpu_index_tuple_cost: f64,

    /// 执行每个操作符的 CPU 代价
    /// 对应 PostgreSQL 的 cpu_operator_cost
    pub cpu_operator_cost: f64,

    /// 图遍历每步的额外代价
    /// 图数据库特有的参数
    pub graph_traversal_step_cost: f64,

    /// 默认页大小（字节）
    pub page_size: u64,
}

impl Default for CostModelConfig {
    fn default() -> Self {
        Self {
            seq_page_cost: 1.0,
            random_page_cost: 4.0,
            cpu_tuple_cost: 0.01,
            cpu_index_tuple_cost: 0.005,
            cpu_operator_cost: 0.0025,
            graph_traversal_step_cost: 0.1,
            page_size: 8192, // 8KB 默认页大小
        }
    }
}

impl CostModelConfig {
    /// 创建适合 SSD 环境的配置
    /// SSD 的随机 I/O 性能接近顺序 I/O
    pub fn for_ssd() -> Self {
        Self {
            random_page_cost: 1.1, // 接近顺序读取
            ..Default::default()
        }
    }

    /// 创建适合内存数据库的配置
    /// 内存访问无 I/O 代价，主要考虑 CPU
    pub fn for_memory() -> Self {
        Self {
            seq_page_cost: 0.01,
            random_page_cost: 0.01,
            ..Default::default()
        }
    }
}

/// 代价计算上下文
/// 包含代价计算所需的配置和统计信息
#[derive(Clone)]
pub struct CostContext<'a> {
    /// 代价模型配置
    pub config: &'a CostModelConfig,
    /// 统计信息提供者
    pub stats_provider: &'a dyn StatisticsProvider,
}

impl<'a> std::fmt::Debug for CostContext<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CostContext")
            .field("config", self.config)
            .field("stats_provider", &"<dyn StatisticsProvider>")
            .finish()
    }
}

impl<'a> CostContext<'a> {
    pub fn new(
        config: &'a CostModelConfig,
        stats_provider: &'a dyn StatisticsProvider,
    ) -> Self {
        Self {
            config,
            stats_provider,
        }
    }
}

/// 可代价估算 trait
/// 实现该 trait 的类型可以提供代价估算
pub trait CostEstimable {
    /// 估算执行代价
    fn estimate_cost(&self, ctx: &CostContext) -> f64;

    /// 估算输出行数
    fn estimate_rows(&self, ctx: &CostContext) -> u64;
}

/// 扫描操作代价计算器
pub struct ScanCostCalculator;

impl ScanCostCalculator {
    /// 计算全表扫描代价
    /// 公式：页数 × 顺序页代价 + 行数 × CPU代价
    pub fn sequential_scan(table_stats: &TableStatistics, config: &CostModelConfig) -> f64 {
        let pages = table_stats.estimate_pages(config.page_size);
        let io_cost = pages as f64 * config.seq_page_cost;
        let cpu_cost = table_stats.row_count as f64 * config.cpu_tuple_cost;
        io_cost + cpu_cost
    }

    /// 计算索引扫描代价
    /// 包含索引访问和回表两部分
    pub fn index_scan(
        table_stats: &TableStatistics,
        index_stats: &IndexStatistics,
        selectivity: f64,
        config: &CostModelConfig,
    ) -> f64 {
        // 索引访问代价（随机 I/O）
        let index_io_cost = index_stats.page_count as f64 * config.random_page_cost;
        let index_cpu_cost = index_stats.entry_count as f64 * config.cpu_index_tuple_cost;

        // 回表代价（选择性 × 表数据）
        let table_rows = (table_stats.row_count as f64 * selectivity) as u64;
        // 假设每行需要一次随机 I/O
        let table_io_cost = table_rows as f64 * config.random_page_cost;
        let table_cpu_cost = table_rows as f64 * config.cpu_tuple_cost;

        index_io_cost + index_cpu_cost + table_io_cost + table_cpu_cost
    }

    /// 计算仅索引扫描代价（不需要回表）
    pub fn index_only_scan(
        index_stats: &IndexStatistics,
        selectivity: f64,
        config: &CostModelConfig,
    ) -> f64 {
        let pages = index_stats.page_count;
        let entries = (index_stats.entry_count as f64 * selectivity) as u64;

        let io_cost = pages as f64 * config.random_page_cost;
        let cpu_cost = entries as f64 * config.cpu_index_tuple_cost;

        io_cost + cpu_cost
    }
}

/// 过滤操作代价计算器
pub struct FilterCostCalculator;

impl FilterCostCalculator {
    /// 计算过滤操作代价
    /// 输入代价 + 条件评估代价
    pub fn filter(
        input_cost: f64,
        input_rows: u64,
        selectivity: f64,
        num_conditions: usize,
        config: &CostModelConfig,
    ) -> (f64, u64) {
        // 条件评估代价
        let eval_cost = input_rows as f64 * config.cpu_operator_cost * num_conditions as f64;
        let total_cost = input_cost + eval_cost;

        // 输出行数 = 输入行数 × 选择性
        let output_rows = (input_rows as f64 * selectivity) as u64;

        (total_cost, output_rows)
    }
}

/// 连接操作代价计算器
pub struct JoinCostCalculator;

impl JoinCostCalculator {
    /// 计算嵌套循环连接代价
    /// 外表扫描 + 外表行数 × 内表扫描
    pub fn nested_loop(
        outer_cost: f64,
        outer_rows: u64,
        inner_cost: f64,
        selectivity: f64,
    ) -> (f64, u64) {
        let cost = outer_cost + outer_rows as f64 * inner_cost;
        let rows = (outer_rows as f64 * selectivity) as u64;
        (cost, rows)
    }

    /// 计算哈希连接代价
    /// 构建哈希表 + 探测
    pub fn hash_join(
        left_cost: f64,
        left_rows: u64,
        right_cost: f64,
        right_rows: u64,
        selectivity: f64,
        config: &CostModelConfig,
    ) -> (f64, u64) {
        // 构建哈希表的代价（假设需要处理左输入）
        let build_cost = left_cost + left_rows as f64 * config.cpu_tuple_cost;
        // 探测代价
        let probe_cost = right_cost + right_rows as f64 * config.cpu_operator_cost;

        let cost = build_cost + probe_cost;
        let rows = (left_rows.min(right_rows) as f64 * selectivity) as u64;

        (cost, rows)
    }
}

/// 图遍历操作代价计算器
pub struct TraversalCostCalculator;

impl TraversalCostCalculator {
    /// 计算图遍历代价
    /// 起始代价 + 遍历步数 × 每步代价
    pub fn traverse(
        start_cost: f64,
        start_rows: u64,
        steps: u32,
        avg_branching_factor: f64,
        config: &CostModelConfig,
    ) -> (f64, u64) {
        // 遍历展开的行数
        let mut total_rows = start_rows as f64;
        let mut traversal_cost = 0.0;

        for _ in 0..steps {
            total_rows *= avg_branching_factor;
            traversal_cost += total_rows * config.graph_traversal_step_cost;
        }

        let cost = start_cost + traversal_cost;
        (cost, total_rows as u64)
    }

    /// 计算邻居查询代价
    pub fn get_neighbors(
        input_cost: f64,
        input_rows: u64,
        avg_degree: f64,
        config: &CostModelConfig,
    ) -> (f64, u64) {
        // 邻居查询代价 = 输入代价 + 邻居获取代价
        let neighbor_cost = input_rows as f64 * avg_degree * config.cpu_tuple_cost;
        let cost = input_cost + neighbor_cost;
        let rows = (input_rows as f64 * avg_degree) as u64;

        (cost, rows)
    }
}

/// 排序操作代价计算器
pub struct SortCostCalculator;

impl SortCostCalculator {
    /// 计算排序代价
    /// 内存排序：O(n log n)
    /// 外部排序：考虑磁盘 I/O
    pub fn sort(
        input_cost: f64,
        input_rows: u64,
        _config: &CostModelConfig,
    ) -> f64 {
        // 简化的排序代价模型
        // 实际应该根据数据大小判断是否需要外部排序
        let n = input_rows as f64;
        let sort_cpu_cost = n * n.log2().max(1.0) * 0.001;

        input_cost + sort_cpu_cost
    }
}

/// 聚合操作代价计算器
pub struct AggregateCostCalculator;

impl AggregateCostCalculator {
    /// 计算聚合代价
    pub fn aggregate(
        input_cost: f64,
        input_rows: u64,
        num_groups: u64,
        _config: &CostModelConfig,
    ) -> (f64, u64) {
        // 哈希聚合代价
        let agg_cost = input_rows as f64 * 0.01 + num_groups as f64 * 0.001;
        let cost = input_cost + agg_cost;

        (cost, num_groups)
    }
}
