//! 直方图统计模块
//!
//! 使用等深直方图（equi-depth histogram）记录属性值分布
//! 每个直方图包含固定数量的桶，每个桶记录相同数量的元组

use crate::core::value::Value;
use std::time::Instant;

/// 直方图桶
#[derive(Debug, Clone)]
pub struct HistogramBucket {
    /// 桶上界（包含）
    pub upper_bound: Value,
    /// 桶内元组数量
    pub count: u64,
    /// 不同值数量（NDV）
    pub distinct_values: u64,
}

impl HistogramBucket {
    /// 创建新的直方图桶
    pub fn new(upper_bound: Value, count: u64, distinct_values: u64) -> Self {
        Self {
            upper_bound,
            count,
            distinct_values: distinct_values.max(1),
        }
    }
}

/// 范围条件类型
#[derive(Debug, Clone)]
pub enum RangeCondition {
    /// 小于
    Lt(Value),
    /// 小于等于
    Le(Value),
    /// 大于
    Gt(Value),
    /// 大于等于
    Ge(Value),
    /// 范围 [low, high)
    Range { low: Value, high: Value },
}

/// 等深直方图
#[derive(Debug, Clone)]
pub struct Histogram {
    /// 桶列表（按上界排序）
    buckets: Vec<HistogramBucket>,
    /// 空值比例
    null_fraction: f64,
    /// 总不同值数量
    total_distinct_values: u64,
    /// 总记录数
    total_count: u64,
    /// 最后更新时间
    last_updated: Instant,
}

impl Histogram {
    /// 创建空的直方图
    pub fn empty() -> Self {
        Self {
            buckets: Vec::new(),
            null_fraction: 0.0,
            total_distinct_values: 0,
            total_count: 0,
            last_updated: Instant::now(),
        }
    }

    /// 从采样数据构建等深直方图
    ///
    /// # 参数
    /// - `samples`: 采样值列表
    /// - `num_buckets`: 桶数量
    /// - `total_count`: 总记录数（用于计算空值比例）
    ///
    /// # 说明
    /// 等深直方图确保每个桶包含大致相同数量的样本
    pub fn from_samples(samples: Vec<Value>, num_buckets: usize, total_count: u64) -> Self {
        if samples.is_empty() || num_buckets == 0 {
            return Self::empty();
        }

        let mut sorted_samples = samples;
        // 分离空值和非空值
        let null_count = sorted_samples.iter().filter(|v| v.is_null()).count();
        sorted_samples.retain(|v| !v.is_null());

        if sorted_samples.is_empty() {
            return Self {
                buckets: Vec::new(),
                null_fraction: null_count as f64 / total_count.max(1) as f64,
                total_distinct_values: 0,
                total_count,
                last_updated: Instant::now(),
            };
        }

        // 按值排序
        sorted_samples.sort_by(compare_values);

        // 计算不同值数量
        let distinct_values = calculate_distinct_values(&sorted_samples);

        // 构建等深直方图
        let non_null_count = sorted_samples.len();
        let bucket_size = non_null_count / num_buckets;
        let remainder = non_null_count % num_buckets;

        let mut buckets = Vec::with_capacity(num_buckets);
        let mut start_idx = 0;

        for i in 0..num_buckets {
            let current_bucket_size = bucket_size + if i < remainder { 1 } else { 0 };
            if current_bucket_size == 0 {
                continue;
            }

            let end_idx = (start_idx + current_bucket_size).min(sorted_samples.len());
            let bucket_samples = &sorted_samples[start_idx..end_idx];

            if let Some(upper_bound) = bucket_samples.last().cloned() {
                let count = bucket_samples.len() as u64;
                let bucket_distinct = calculate_distinct_values(bucket_samples);
                buckets.push(HistogramBucket::new(upper_bound, count, bucket_distinct));
            }

            start_idx = end_idx;
        }

        Self {
            buckets,
            null_fraction: null_count as f64 / total_count.max(1) as f64,
            total_distinct_values: distinct_values,
            total_count,
            last_updated: Instant::now(),
        }
    }

    /// 估计等值查询选择性
    ///
    /// # 说明
    /// 找到包含该值的桶，使用桶内均匀分布假设：1 / 桶内NDV
    pub fn estimate_equality_selectivity(&self, value: &Value) -> f64 {
        if value.is_null() {
            return self.null_fraction;
        }

        if self.buckets.is_empty() {
            return 0.1; // 默认选择性
        }

        // 找到对应的桶
        match self.find_bucket(value) {
            Some(bucket_idx) => {
                let bucket = &self.buckets[bucket_idx];
                // 桶内均匀分布假设
                let bucket_selectivity = 1.0 / bucket.distinct_values.max(1) as f64;
                // 考虑桶在整体中的比例
                let bucket_ratio = bucket.count as f64 / self.total_count.max(1) as f64;
                bucket_selectivity * bucket_ratio
            }
            None => {
                // 值超出直方图范围，使用最小选择性估计
                1.0 / self.total_distinct_values.max(1) as f64
            }
        }
    }

    /// 估计范围查询选择性
    pub fn estimate_range_selectivity(&self, range: &RangeCondition) -> f64 {
        match range {
            RangeCondition::Lt(value) | RangeCondition::Le(value) => {
                self.estimate_less_than_selectivity(value)
            }
            RangeCondition::Gt(value) | RangeCondition::Ge(value) => {
                1.0 - self.estimate_less_than_selectivity(value)
            }
            RangeCondition::Range { low, high } => {
                let high_selectivity = self.estimate_less_than_selectivity(high);
                let low_selectivity = self.estimate_less_than_selectivity(low);
                (high_selectivity - low_selectivity).max(0.0)
            }
        }
    }

    /// 估计小于条件的选择性
    fn estimate_less_than_selectivity(&self, value: &Value) -> f64 {
        if self.buckets.is_empty() {
            return 0.333; // 默认范围选择性
        }

        let mut selectivity = 0.0;
        let mut found = false;

        for bucket in &self.buckets {
            if compare_values(value, &bucket.upper_bound) != std::cmp::Ordering::Greater {
                // 值在当前桶范围内
                // 假设桶内均匀分布，估计小于value的比例
                let bucket_ratio = bucket.count as f64 / self.total_count.max(1) as f64;
                selectivity += bucket_ratio * 0.5; // 简化估计：取桶的一半
                found = true;
                break;
            } else {
                // 整个桶都小于value
                selectivity += bucket.count as f64 / self.total_count.max(1) as f64;
            }
        }

        if !found {
            // value大于所有桶的上界
            selectivity = 1.0 - self.null_fraction;
        }

        selectivity.min(1.0 - self.null_fraction)
    }

    /// 查找值所在的桶索引
    fn find_bucket(&self, value: &Value) -> Option<usize> {
        for (idx, bucket) in self.buckets.iter().enumerate() {
            if compare_values(value, &bucket.upper_bound) != std::cmp::Ordering::Greater {
                return Some(idx);
            }
        }
        None
    }

    /// 获取桶数量
    pub fn bucket_count(&self) -> usize {
        self.buckets.len()
    }

    /// 获取空值比例
    pub fn null_fraction(&self) -> f64 {
        self.null_fraction
    }

    /// 获取总记录数
    pub fn total_count(&self) -> u64 {
        self.total_count
    }

    /// 获取不同值数量
    pub fn distinct_values(&self) -> u64 {
        self.total_distinct_values
    }

    /// 获取最后更新时间
    pub fn last_updated(&self) -> Instant {
        self.last_updated
    }
}

/// 比较两个Value
fn compare_values(a: &Value, b: &Value) -> std::cmp::Ordering {
    use std::cmp::Ordering;

    match (a, b) {
        (Value::Int(a), Value::Int(b)) => a.cmp(b),
        (Value::Float(a), Value::Float(b)) => a.partial_cmp(b).unwrap_or(Ordering::Equal),
        (Value::Int(a), Value::Float(b)) => (*a as f64).partial_cmp(b).unwrap_or(Ordering::Equal),
        (Value::Float(a), Value::Int(b)) => a.partial_cmp(&(*b as f64)).unwrap_or(Ordering::Equal),
        (Value::String(a), Value::String(b)) => a.cmp(b),
        (Value::Bool(a), Value::Bool(b)) => a.cmp(b),
        // 不同类型按类型排序（使用类型ID作为排序依据）
        _ => {
            let type_a = std::mem::discriminant(a);
            let type_b = std::mem::discriminant(b);
            // 使用地址比较作为稳定的排序依据
            let ptr_a = &type_a as *const _ as usize;
            let ptr_b = &type_b as *const _ as usize;
            ptr_a.cmp(&ptr_b)
        }
    }
}

/// 计算不同值数量
fn calculate_distinct_values(values: &[Value]) -> u64 {
    use std::collections::HashSet;

    let mut seen = HashSet::new();
    for value in values {
        // 使用值的字符串表示作为哈希键
        let key = format!("{:?}", value);
        seen.insert(key);
    }
    seen.len() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_histogram_empty() {
        let hist = Histogram::empty();
        assert_eq!(hist.bucket_count(), 0);
        assert_eq!(hist.null_fraction(), 0.0);
    }

    #[test]
    fn test_histogram_from_samples() {
        let samples: Vec<Value> = (1..=100).map(|i| Value::Int(i)).collect();
        let hist = Histogram::from_samples(samples, 10, 100);

        assert_eq!(hist.bucket_count(), 10);
        assert_eq!(hist.total_count(), 100);
    }

    #[test]
    fn test_estimate_equality_selectivity() {
        let samples: Vec<Value> = (1..=100).map(|i| Value::Int(i)).collect();
        let hist = Histogram::from_samples(samples, 10, 100);

        // 对于均匀分布的数据，选择性应该接近 1/100 = 0.01
        let selectivity = hist.estimate_equality_selectivity(&Value::Int(50));
        assert!(selectivity > 0.0 && selectivity < 0.1);
    }

    #[test]
    fn test_estimate_range_selectivity() {
        let samples: Vec<Value> = (1..=100).map(|i| Value::Int(i)).collect();
        let hist = Histogram::from_samples(samples, 10, 100);

        // 估计小于50的选择性，应该接近 0.5
        let range = RangeCondition::Lt(Value::Int(50));
        let selectivity = hist.estimate_range_selectivity(&range);
        assert!(selectivity > 0.3 && selectivity < 0.7);
    }

    #[test]
    fn test_null_handling() {
        let mut samples: Vec<Value> = (1..=90).map(|i| Value::Int(i)).collect();
        // 添加10个空值
        for _ in 0..10 {
            samples.push(Value::Null(crate::core::value::NullType::Null));
        }

        let hist = Histogram::from_samples(samples, 10, 100);
        assert!((hist.null_fraction() - 0.1).abs() < 0.01);

        let null_selectivity =
            hist.estimate_equality_selectivity(&Value::Null(crate::core::value::NullType::Null));
        assert!((null_selectivity - 0.1).abs() < 0.01);
    }
}
