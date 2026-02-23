//! 统计信息模块
//!
//! 提供查询优化所需的表级、列级和图结构统计信息
//! 参考 PostgreSQL 的 pg_class 和 pg_stats 设计

use std::collections::HashMap;

/// 表级统计信息
/// 对应 PostgreSQL 的 pg_class 中的 reltuples 和 relpages
#[derive(Debug, Clone, Default)]
pub struct TableStatistics {
    /// 表名或标签名
    pub table_name: String,
    /// 估计行数
    pub row_count: u64,
    /// 数据页数
    pub page_count: u64,
    /// 平均行大小（字节）
    pub avg_row_size: u64,
    /// 最后分析时间
    pub last_analyzed: Option<std::time::SystemTime>,
    /// 列级统计信息
    pub column_stats: HashMap<String, ColumnStatistics>,
}

impl TableStatistics {
    pub fn new(table_name: impl Into<String>) -> Self {
        Self {
            table_name: table_name.into(),
            ..Default::default()
        }
    }

    /// 获取表的估计页数，如果未设置则基于行数和平均行大小估算
    pub fn estimate_pages(&self, page_size: u64) -> u64 {
        if self.page_count > 0 {
            self.page_count
        } else if self.avg_row_size > 0 {
            (self.row_count * self.avg_row_size + page_size - 1) / page_size
        } else {
            // 默认假设每页 100 行
            self.row_count / 100
        }
    }
}

/// 列级统计信息
/// 对应 PostgreSQL 的 pg_stats
#[derive(Debug, Clone, Default)]
pub struct ColumnStatistics {
    /// 列名
    pub column_name: String,
    /// 空值比例（0.0 - 1.0）
    pub null_fraction: f64,
    /// 不同值数量
    pub distinct_count: u64,
    /// 平均宽度（字节）
    pub avg_width: u64,
    /// 最常见值列表（MCV）及其频率
    pub most_common_values: Vec<(crate::core::Value, f64)>,
    /// 直方图边界值（用于范围查询）
    pub histogram_bounds: Vec<crate::core::Value>,
    /// 最小值
    pub min_value: Option<crate::core::Value>,
    /// 最大值
    pub max_value: Option<crate::core::Value>,
}

impl ColumnStatistics {
    pub fn new(column_name: impl Into<String>) -> Self {
        Self {
            column_name: column_name.into(),
            ..Default::default()
        }
    }

    /// 获取 MCV 的总频率
    pub fn mcv_total_frequency(&self) -> f64 {
        self.most_common_values.iter().map(|(_, freq)| freq).sum()
    }

    /// 获取非 MCV 的不同值数量
    pub fn non_mcv_distinct_count(&self) -> u64 {
        self.distinct_count.saturating_sub(self.most_common_values.len() as u64)
    }
}

/// 索引统计信息
#[derive(Debug, Clone, Default)]
pub struct IndexStatistics {
    /// 索引名
    pub index_name: String,
    /// 索引页数
    pub page_count: u64,
    /// 索引项数量
    pub entry_count: u64,
    /// 索引列的统计信息
    pub column_stats: Vec<ColumnStatistics>,
}

impl IndexStatistics {
    pub fn new(index_name: impl Into<String>) -> Self {
        Self {
            index_name: index_name.into(),
            ..Default::default()
        }
    }
}

/// 图结构统计信息
/// 图数据库特有的统计信息
#[derive(Debug, Clone, Default)]
pub struct GraphStatistics {
    /// 标签统计（标签名 -> 该标签的顶点数）
    pub tag_counts: HashMap<String, u64>,
    /// 边类型统计（边类型名 -> 该类型的边数）
    pub edge_type_counts: HashMap<String, u64>,
    /// 平均出度
    pub avg_out_degree: f64,
    /// 平均入度
    pub avg_in_degree: f64,
    /// 度分布直方图（度 -> 具有该度的顶点数）
    pub degree_histogram: HashMap<u32, u64>,
    /// 最大度
    pub max_degree: u32,
}

impl GraphStatistics {
    pub fn new() -> Self {
        Self::default()
    }

    /// 获取指定标签的顶点数
    pub fn get_tag_count(&self, tag: &str) -> u64 {
        self.tag_counts.get(tag).copied().unwrap_or(0)
    }

    /// 获取指定边类型的边数
    pub fn get_edge_type_count(&self, edge_type: &str) -> u64 {
        self.edge_type_counts.get(edge_type).copied().unwrap_or(0)
    }

    /// 估计从指定标签开始的遍历代价
    pub fn estimate_traversal_cost(&self, from_tag: &str) -> f64 {
        let vertex_count = self.get_tag_count(from_tag) as f64;
        let avg_degree = self.avg_out_degree.max(1.0);
        // 遍历代价 = 起始顶点数 × 平均出度
        vertex_count * avg_degree
    }
}

/// 统计信息集合
#[derive(Debug, Clone, Default)]
pub struct StatisticsSet {
    /// 表级统计（表名 -> 统计信息）
    pub table_stats: HashMap<String, TableStatistics>,
    /// 索引统计（索引名 -> 统计信息）
    pub index_stats: HashMap<String, IndexStatistics>,
    /// 图结构统计
    pub graph_stats: GraphStatistics,
}

impl StatisticsSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_table_stats(&self, table_name: &str) -> Option<&TableStatistics> {
        self.table_stats.get(table_name)
    }

    pub fn get_table_stats_mut(&mut self, table_name: &str) -> Option<&mut TableStatistics> {
        self.table_stats.get_mut(table_name)
    }

    pub fn set_table_stats(&mut self, table_name: impl Into<String>, stats: TableStatistics) {
        self.table_stats.insert(table_name.into(), stats);
    }

    pub fn get_index_stats(&self, index_name: &str) -> Option<&IndexStatistics> {
        self.index_stats.get(index_name)
    }

    pub fn set_index_stats(&mut self, index_name: impl Into<String>, stats: IndexStatistics) {
        self.index_stats.insert(index_name.into(), stats);
    }

    /// 获取列统计信息（从表统计中查找）
    pub fn get_column_stats(&self, table_name: &str, column_name: &str) -> Option<&ColumnStatistics> {
        self.table_stats.get(table_name)
            .and_then(|table| table.column_stats.get(column_name))
    }

    /// 设置列统计信息
    pub fn set_column_stats(&mut self, table_name: impl Into<String>, column_name: impl Into<String>, stats: ColumnStatistics) {
        let table_name = table_name.into();
        let column_name = column_name.into();
        if let Some(table) = self.table_stats.get_mut(&table_name) {
            table.column_stats.insert(column_name, stats);
        }
    }
}

/// 统计信息提供者 trait
///
/// 定义优化器获取统计信息的标准接口。
/// 由存储层实现，为优化器提供统一的统计信息查询能力。
///
/// # 设计说明
///
/// - 使用表名/索引名作为查询键，更符合优化器的使用场景
/// - 支持异步查询（通过 `async_trait`）
/// - 提供批量查询接口以提高性能
/// - 包含缓存控制方法
pub trait StatisticsProvider: Send + Sync {
    /// 获取表的统计信息
    ///
    /// # 参数
    /// - `table_name`: 表名
    ///
    /// # 返回
    /// 表的统计信息，如果不存在则返回 None
    fn get_table_stats(&self, table_name: &str) -> Option<TableStatistics>;

    /// 获取索引的统计信息
    ///
    /// # 参数
    /// - `index_name`: 索引名
    ///
    /// # 返回
    /// 索引的统计信息，如果不存在则返回 None
    fn get_index_stats(&self, index_name: &str) -> Option<IndexStatistics>;

    /// 获取列的统计信息
    ///
    /// # 参数
    /// - `table_name`: 表名
    /// - `column_name`: 列名
    ///
    /// # 返回
    /// 列的统计信息，如果不存在则返回 None
    fn get_column_stats(&self, table_name: &str, column_name: &str) -> Option<ColumnStatistics>;

    /// 获取图结构统计
    ///
    /// # 返回
    /// 图结构的统计信息，如果不存在则返回 None
    fn get_graph_stats(&self) -> Option<GraphStatistics>;

    /// 获取表的最后分析时间
    ///
    /// # 参数
    /// - `table_name`: 表名
    ///
    /// # 返回
    /// 最后分析时间，如果未分析过则返回 None
    fn get_last_analyzed(&self, table_name: &str) -> Option<std::time::SystemTime>;

    /// 批量获取表的统计信息
    ///
    /// # 参数
    /// - `table_names`: 表名列表
    ///
    /// # 返回
    /// 表名到统计信息的映射
    fn get_table_stats_batch(&self, table_names: &[String]) -> HashMap<String, TableStatistics> {
        table_names
            .iter()
            .filter_map(|name| {
                self.get_table_stats(name)
                    .map(|stats| (name.clone(), stats))
            })
            .collect()
    }

    /// 检查统计信息是否存在且未过期
    ///
    /// # 参数
    /// - `table_name`: 表名
    /// - `max_age`: 最大允许的年龄
    ///
    /// # 返回
    /// 如果统计信息存在且未过期返回 true
    fn is_stats_valid(&self, table_name: &str, max_age: std::time::Duration) -> bool {
        match self.get_last_analyzed(table_name) {
            Some(last_analyzed) => {
                let age = std::time::SystemTime::now()
                    .duration_since(last_analyzed)
                    .unwrap_or(std::time::Duration::MAX);
                age <= max_age
            }
            None => false,
        }
    }

    /// 获取所有已知表的名称
    fn get_all_table_names(&self) -> Vec<String>;

    /// 获取所有已知索引的名称
    fn get_all_index_names(&self) -> Vec<String>;
}

/// 可变的统计信息提供者 trait
///
/// 扩展 `StatisticsProvider`，提供修改统计信息的能力。
/// 通常由存储层的统计信息管理器实现。
pub trait StatisticsProviderMut: StatisticsProvider {
    /// 更新表的统计信息
    ///
    /// # 参数
    /// - `table_name`: 表名
    /// - `stats`: 新的统计信息
    ///
    /// # 返回
    /// 成功返回 Ok，失败返回错误
    fn update_table_stats(
        &mut self,
        table_name: &str,
        stats: TableStatistics,
    ) -> Result<(), crate::core::error::DBError>;

    /// 更新索引的统计信息
    ///
    /// # 参数
    /// - `index_name`: 索引名
    /// - `stats`: 新的统计信息
    ///
    /// # 返回
    /// 成功返回 Ok，失败返回错误
    fn update_index_stats(
        &mut self,
        index_name: &str,
        stats: IndexStatistics,
    ) -> Result<(), crate::core::error::DBError>;

    /// 更新图结构统计
    ///
    /// # 参数
    /// - `stats`: 新的图结构统计信息
    ///
    /// # 返回
    /// 成功返回 Ok，失败返回错误
    fn update_graph_stats(
        &mut self,
        stats: GraphStatistics,
    ) -> Result<(), crate::core::error::DBError>;

    /// 删除表的统计信息
    ///
    /// # 参数
    /// - `table_name`: 表名
    ///
    /// # 返回
    /// 成功返回 Ok，失败返回错误
    fn remove_table_stats(&mut self, table_name: &str) -> Result<(), crate::core::error::DBError>;

    /// 删除索引的统计信息
    ///
    /// # 参数
    /// - `index_name`: 索引名
    ///
    /// # 返回
    /// 成功返回 Ok，失败返回错误
    fn remove_index_stats(&mut self, index_name: &str) -> Result<(), crate::core::error::DBError>;

    /// 清除所有统计信息
    ///
    /// # 返回
    /// 成功返回 Ok，失败返回错误
    fn clear_all_stats(&mut self) -> Result<(), crate::core::error::DBError>;
}

/// 内存统计信息提供者（用于测试和内存数据库）
///
/// 将统计信息存储在内存中的 HashMap 中，适用于：
/// - 单元测试
/// - 内存数据库场景
/// - 开发和调试
#[derive(Debug, Default)]
pub struct MemoryStatisticsProvider {
    pub table_stats: HashMap<String, TableStatistics>,
    pub index_stats: HashMap<String, IndexStatistics>,
    pub graph_stats: Option<GraphStatistics>,
}

impl MemoryStatisticsProvider {
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加表统计信息
    pub fn add_table_stats(&mut self, table_name: impl Into<String>, stats: TableStatistics) {
        self.table_stats.insert(table_name.into(), stats);
    }

    /// 添加索引统计信息
    pub fn add_index_stats(&mut self, index_name: impl Into<String>, stats: IndexStatistics) {
        self.index_stats.insert(index_name.into(), stats);
    }

    /// 设置图结构统计
    pub fn set_graph_stats(&mut self, stats: GraphStatistics) {
        self.graph_stats = Some(stats);
    }
}

impl StatisticsProvider for MemoryStatisticsProvider {
    fn get_table_stats(&self, table_name: &str) -> Option<TableStatistics> {
        self.table_stats.get(table_name).cloned()
    }

    fn get_index_stats(&self, index_name: &str) -> Option<IndexStatistics> {
        self.index_stats.get(index_name).cloned()
    }

    fn get_column_stats(&self, table_name: &str, column_name: &str) -> Option<ColumnStatistics> {
        self.table_stats
            .get(table_name)
            .and_then(|table| table.column_stats.get(column_name).cloned())
    }

    fn get_graph_stats(&self) -> Option<GraphStatistics> {
        self.graph_stats.clone()
    }

    fn get_last_analyzed(&self, table_name: &str) -> Option<std::time::SystemTime> {
        self.table_stats
            .get(table_name)
            .and_then(|stats| stats.last_analyzed)
    }

    fn get_all_table_names(&self) -> Vec<String> {
        self.table_stats.keys().cloned().collect()
    }

    fn get_all_index_names(&self) -> Vec<String> {
        self.index_stats.keys().cloned().collect()
    }
}

impl StatisticsProviderMut for MemoryStatisticsProvider {
    fn update_table_stats(
        &mut self,
        table_name: &str,
        stats: TableStatistics,
    ) -> Result<(), crate::core::error::DBError> {
        self.table_stats.insert(table_name.to_string(), stats);
        Ok(())
    }

    fn update_index_stats(
        &mut self,
        index_name: &str,
        stats: IndexStatistics,
    ) -> Result<(), crate::core::error::DBError> {
        self.index_stats.insert(index_name.to_string(), stats);
        Ok(())
    }

    fn update_graph_stats(
        &mut self,
        stats: GraphStatistics,
    ) -> Result<(), crate::core::error::DBError> {
        self.graph_stats = Some(stats);
        Ok(())
    }

    fn remove_table_stats(&mut self, table_name: &str) -> Result<(), crate::core::error::DBError> {
        self.table_stats.remove(table_name);
        Ok(())
    }

    fn remove_index_stats(&mut self, index_name: &str) -> Result<(), crate::core::error::DBError> {
        self.index_stats.remove(index_name);
        Ok(())
    }

    fn clear_all_stats(&mut self) -> Result<(), crate::core::error::DBError> {
        self.table_stats.clear();
        self.index_stats.clear();
        self.graph_stats = None;
        Ok(())
    }
}
