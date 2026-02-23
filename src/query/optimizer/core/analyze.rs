//! 统计信息收集器
//!
//! 实现 ANALYZE 命令功能，收集表和列的统计信息
//! 参考 PostgreSQL 的 ANALYZE 实现

use super::statistics::{ColumnStatistics, GraphStatistics, IndexStatistics, TableStatistics};
use crate::core::Value;
use std::collections::HashMap;
use std::time::SystemTime;

/// 统计信息收集器
///
/// 负责从存储层采样数据并计算统计信息
#[derive(Debug)]
pub struct StatisticsCollector<S> {
    _storage: S,
    config: AnalyzeConfig,
}

/// 分析配置
#[derive(Debug, Clone)]
pub struct AnalyzeConfig {
    /// 采样率 (0.0 - 1.0)，默认 0.1
    pub sample_ratio: f64,
    /// 最大采样行数，默认 10000
    pub max_sample_rows: u64,
    /// MCV 列表大小，默认 100
    pub mcv_target: usize,
    /// 直方图桶数，默认 100
    pub histogram_buckets: usize,
    /// 最小采样行数，默认 100
    pub min_sample_rows: u64,
}

impl Default for AnalyzeConfig {
    fn default() -> Self {
        Self {
            sample_ratio: 0.1,      // 默认采样 10%
            max_sample_rows: 10000, // 最多采样 10000 行
            mcv_target: 100,        // 保存 100 个最常见值
            histogram_buckets: 100, // 100 个直方图桶
            min_sample_rows: 100,   // 最少采样 100 行
        }
    }
}

/// 分析错误类型
#[derive(Debug, thiserror::Error)]
pub enum AnalyzeError {
    #[error("表不存在: {0}")]
    TableNotFound(String),
    #[error("存储错误: {0}")]
    StorageError(String),
    #[error("采样错误: {0}")]
    SamplingError(String),
    #[error("统计计算错误: {0}")]
    CalculationError(String),
}

impl<S> StatisticsCollector<S> {
    /// 创建新的统计信息收集器
    pub fn new(storage: S) -> Self {
        Self {
            _storage: storage,
            config: AnalyzeConfig::default(),
        }
    }

    /// 使用自定义配置创建
    pub fn with_config(storage: S, config: AnalyzeConfig) -> Self {
        Self { _storage: storage, config }
    }

    /// 获取配置
    pub fn config(&self) -> &AnalyzeConfig {
        &self.config
    }

    /// 修改配置
    pub fn config_mut(&mut self) -> &mut AnalyzeConfig {
        &mut self.config
    }
}

impl<S> StatisticsCollector<S>
where
    S: crate::storage::StorageClient,
{
    /// 分析单个表
    ///
    /// # 参数
    /// - `table_name`: 表名
    ///
    /// # 返回
    /// 表的统计信息
    pub async fn analyze_table(&mut self, _table_name: &str) -> Result<TableStatistics, AnalyzeError> {
        // TODO: 实现表分析逻辑
        // 1. 从存储层获取表的元数据
        // 2. 采样数据
        // 3. 计算行数、页数
        // 4. 分析每列的统计信息
        todo!("analyze_table not yet implemented")
    }

    /// 分析多个表
    ///
    /// # 参数
    /// - `table_names`: 表名列表
    ///
    /// # 返回
    /// 每个表的统计信息结果列表
    pub async fn analyze_tables(
        &mut self,
        table_names: &[String],
    ) -> Vec<Result<TableStatistics, AnalyzeError>> {
        let mut results = Vec::with_capacity(table_names.len());
        for name in table_names {
            results.push(self.analyze_table(name).await);
        }
        results
    }

    /// 分析所有表
    ///
    /// # 返回
    /// 所有表的统计信息结果列表
    pub async fn analyze_all(&mut self) -> Vec<Result<TableStatistics, AnalyzeError>> {
        // TODO: 从存储层获取所有表名
        todo!("analyze_all not yet implemented")
    }

    /// 增量分析（仅分析变更的数据）
    ///
    /// # 参数
    /// - `table_name`: 表名
    /// - `since`: 自该时间以来的变更
    ///
    /// # 返回
    /// 表的统计信息
    pub async fn analyze_incremental(
        &mut self,
        _table_name: &str,
        _since: SystemTime,
    ) -> Result<TableStatistics, AnalyzeError> {
        // TODO: 实现增量分析逻辑
        // 1. 获取自上次分析以来的变更数据
        // 2. 仅对变更部分进行采样
        // 3. 合并新旧统计信息
        todo!("analyze_incremental not yet implemented")
    }

    /// 分析索引
    ///
    /// # 参数
    /// - `index_name`: 索引名
    ///
    /// # 返回
    /// 索引的统计信息
    pub async fn analyze_index(&mut self, _index_name: &str) -> Result<IndexStatistics, AnalyzeError> {
        // TODO: 实现索引分析逻辑
        // 1. 获取索引结构信息
        // 2. 统计索引页数、条目数
        // 3. 计算索引选择性
        todo!("analyze_index not yet implemented")
    }

    /// 分析图结构
    ///
    /// # 返回
    /// 图结构的统计信息
    pub async fn analyze_graph(&mut self) -> Result<GraphStatistics, AnalyzeError> {
        // TODO: 实现图结构分析逻辑
        // 1. 统计各标签的顶点数
        // 2. 统计各边类型的边数
        // 3. 计算度分布
        // 4. 计算平均度、最大度
        todo!("analyze_graph not yet implemented")
    }
}

/// 列分析器
///
/// 负责分析单列的统计信息
#[derive(Debug)]
pub struct ColumnAnalyzer;

impl ColumnAnalyzer {
    /// 分析列统计信息
    ///
    /// # 参数
    /// - `column_name`: 列名
    /// - `values`: 列值样本
    /// - `config`: 分析配置
    ///
    /// # 返回
    /// 列的统计信息
    pub fn analyze(
        column_name: &str,
        values: Vec<Value>,
        config: &AnalyzeConfig,
    ) -> Result<ColumnStatistics, AnalyzeError> {
        if values.is_empty() {
            return Err(AnalyzeError::SamplingError("空样本".to_string()));
        }

        let total_count = values.len();

        // 1. 计算空值比例
        let null_count = values.iter().filter(|v| v.is_null()).count();
        let null_fraction = null_count as f64 / total_count as f64;

        // 2. 计算不同值
        let distinct_values = Self::compute_distinct_values(&values);
        let distinct_count = distinct_values.len() as u64;

        // 3. 识别 MCV（最常见值）
        let most_common_values = Self::compute_mcv(&distinct_values, config.mcv_target);

        // 4. 构建直方图（对非 MCV 值）
        let histogram_bounds =
            Self::compute_histogram(&values, &most_common_values, config.histogram_buckets);

        // 5. 计算最小/最大值
        let (min_value, max_value) = Self::compute_min_max(&values);

        Ok(ColumnStatistics {
            column_name: column_name.to_string(),
            null_fraction,
            distinct_count,
            avg_width: Self::compute_avg_width(&values),
            most_common_values,
            histogram_bounds,
            min_value,
            max_value,
        })
    }

    /// 计算不同值及其频率
    fn compute_distinct_values(values: &[Value]) -> HashMap<Value, u64> {
        let mut freq_map: HashMap<Value, u64> = HashMap::new();
        for value in values.iter().filter(|v| !v.is_null()) {
            *freq_map.entry(value.clone()).or_insert(0) += 1;
        }
        freq_map
    }

    /// 计算最常见值（MCV）
    ///
    /// # 参数
    /// - `distinct`: 不同值及其频率的映射
    /// - `target`: 要保留的 MCV 数量
    ///
    /// # 返回
    /// 最常见值列表及其频率（已归一化）
    fn compute_mcv(distinct: &HashMap<Value, u64>, target: usize) -> Vec<(Value, f64)> {
        let total_count: u64 = distinct.values().sum();
        if total_count == 0 {
            return Vec::new();
        }

        let mut items: Vec<(Value, u64)> = distinct.iter().map(|(k, v)| (k.clone(), *v)).collect();

        // 按频率降序排序
        items.sort_by(|a, b| b.1.cmp(&a.1));

        // 取前 target 个，并归一化频率
        items
            .into_iter()
            .take(target)
            .map(|(v, count)| (v, count as f64 / total_count as f64))
            .collect()
    }

    /// 计算直方图边界
    ///
    /// # 参数
    /// - `values`: 所有值
    /// - `mcv`: 最常见值列表
    /// - `buckets`: 直方图桶数
    ///
    /// # 返回
    /// 直方图边界值列表
    fn compute_histogram(
        values: &[Value],
        mcv: &[(Value, f64)],
        buckets: usize,
    ) -> Vec<Value> {
        // 过滤掉 MCV 中的值
        let mcv_set: std::collections::HashSet<_> = mcv.iter().map(|(v, _)| v.clone()).collect();
        let non_mcv_values: Vec<_> = values
            .iter()
            .filter(|v| !v.is_null() && !mcv_set.contains(v))
            .cloned()
            .collect();

        if non_mcv_values.is_empty() || buckets == 0 {
            return Vec::new();
        }

        // 排序非 MCV 值
        let mut sorted = non_mcv_values;
        sorted.sort();

        // 计算等频直方图边界
        let bucket_size = sorted.len() / buckets;
        if bucket_size == 0 {
            return sorted;
        }

        let mut bounds = Vec::with_capacity(buckets + 1);
        for i in 0..=buckets {
            let idx = (i * sorted.len() / buckets).min(sorted.len() - 1);
            bounds.push(sorted[idx].clone());
        }

        // 去重
        bounds.dedup();
        bounds
    }

    /// 计算最小最大值
    fn compute_min_max(values: &[Value]) -> (Option<Value>, Option<Value>) {
        let non_null: Vec<_> = values.iter().filter(|v| !v.is_null()).cloned().collect();
        if non_null.is_empty() {
            return (None, None);
        }

        let min = non_null.iter().min().cloned();
        let max = non_null.iter().max().cloned();
        (min, max)
    }

    /// 计算平均宽度（字节数）
    fn compute_avg_width(values: &[Value]) -> u64 {
        if values.is_empty() {
            return 0;
        }

        let total_width: usize = values.iter().map(|v| v.estimated_size()).sum();
        (total_width / values.len()) as u64
    }
}

/// 采样器 trait
///
/// 定义数据采样策略
pub trait Sampler {
    /// 随机采样
    ///
    /// # 参数
    /// - `data`: 原始数据
    /// - `ratio`: 采样比例 (0.0 - 1.0)
    ///
    /// # 返回
    /// 采样后的数据
    fn sample_random<T: Clone>(&self, data: &[T], ratio: f64) -> Vec<T>;

    /// 系统采样（每隔 n 个取一个）
    ///
    /// # 参数
    /// - `data`: 原始数据
    /// - `interval`: 采样间隔
    ///
    /// # 返回
    /// 采样后的数据
    fn sample_systematic<T: Clone>(&self, data: &[T], interval: usize) -> Vec<T>;
}

/// 默认采样器实现
#[derive(Debug, Default)]
pub struct DefaultSampler;

impl DefaultSampler {
    pub fn new() -> Self {
        Self
    }
}

impl Sampler for DefaultSampler {
    fn sample_random<T: Clone>(&self, data: &[T], ratio: f64) -> Vec<T> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        data.iter()
            .filter(|_| rng.gen::<f64>() < ratio.clamp(0.0, 1.0))
            .cloned()
            .collect()
    }

    fn sample_systematic<T: Clone>(&self, data: &[T], interval: usize) -> Vec<T> {
        if interval == 0 {
            return data.to_vec();
        }
        data.iter()
            .enumerate()
            .filter(|(i, _)| i % interval == 0)
            .map(|(_, v)| v.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_config_default() {
        let config = AnalyzeConfig::default();
        assert_eq!(config.sample_ratio, 0.1);
        assert_eq!(config.max_sample_rows, 10000);
        assert_eq!(config.mcv_target, 100);
        assert_eq!(config.histogram_buckets, 100);
        assert_eq!(config.min_sample_rows, 100);
    }

    #[test]
    fn test_column_analyzer_empty() {
        let result = ColumnAnalyzer::analyze("test", vec![], &AnalyzeConfig::default());
        assert!(result.is_err());
    }

    #[test]
    fn test_compute_mcv() {
        let mut distinct = HashMap::new();
        distinct.insert(Value::from("a"), 100);
        distinct.insert(Value::from("b"), 50);
        distinct.insert(Value::from("c"), 25);

        let mcv = ColumnAnalyzer::compute_mcv(&distinct, 2);
        assert_eq!(mcv.len(), 2);
        assert_eq!(mcv[0].0, Value::from("a"));
        assert_eq!(mcv[1].0, Value::from("b"));
    }

    #[test]
    fn test_default_sampler_random() {
        let sampler = DefaultSampler::new();
        let data: Vec<i32> = (0..100).collect();
        let sampled = sampler.sample_random(&data, 0.5);
        // 随机采样，结果数量应在合理范围内
        assert!(!sampled.is_empty());
        assert!(sampled.len() <= data.len());
    }

    #[test]
    fn test_default_sampler_systematic() {
        let sampler = DefaultSampler::new();
        let data: Vec<i32> = (0..10).collect();
        let sampled = sampler.sample_systematic(&data, 2);
        assert_eq!(sampled, vec![0, 2, 4, 6, 8]);
    }
}
