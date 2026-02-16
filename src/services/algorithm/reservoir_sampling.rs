//! 蓄水池采样算法模块
//!
//! 包含蓄水池采样算法实现，用于大数据集的随机采样
//! 可以在不知道数据总量的情况下，均匀随机地采样固定数量的元素

use rand::Rng;
use std::collections::HashMap;

/// 蓄水池采样结构体
#[derive(Debug, Clone)]
pub struct ReservoirSampling<T> {
    /// 采样容量
    capacity: usize,
    /// 已处理的元素数量
    count: usize,
    /// 当前样本
    samples: Vec<T>,
}

impl<T: Clone> ReservoirSampling<T> {
    /// 创建新的蓄水池采样器
    ///
    /// # 参数
    /// - `capacity`: 采样容量，即要保留的样本数量
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            count: 0,
            samples: Vec::with_capacity(capacity),
        }
    }

    /// 处理一个元素
    ///
    /// # 参数
    /// - `item`: 要处理的元素
    ///
    /// # 返回
    /// 如果元素被采样返回true，否则返回false
    pub fn sample(&mut self, item: T) -> bool {
        self.count += 1;

        if self.samples.len() < self.capacity {
            // 蓄水池未满，直接添加
            self.samples.push(item);
            true
        } else {
            // 蓄水池已满，以 capacity/count 的概率替换
            let mut rng = rand::thread_rng();
            let j = rng.gen_range(0..self.count);

            if j < self.capacity {
                self.samples[j] = item;
                true
            } else {
                false
            }
        }
    }

    /// 批量处理元素
    ///
    /// # 参数
    /// - `items`: 要处理的元素迭代器
    ///
    /// # 返回
    /// 被采样的元素数量
    pub fn sample_iter<I: Iterator<Item = T>>(&mut self, items: I) -> usize {
        let mut sampled = 0;
        for item in items {
            if self.sample(item) {
                sampled += 1;
            }
        }
        sampled
    }

    /// 获取当前样本
    pub fn samples(&self) -> &[T] {
        &self.samples
    }

    /// 获取当前样本（消耗自身）
    pub fn into_samples(self) -> Vec<T> {
        self.samples
    }

    /// 获取已处理的元素数量
    pub fn count(&self) -> usize {
        self.count
    }

    /// 获取采样容量
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// 检查蓄水池是否已满
    pub fn is_full(&self) -> bool {
        self.samples.len() == self.capacity
    }

    /// 清空蓄水池
    pub fn clear(&mut self) {
        self.count = 0;
        self.samples.clear();
    }
}

/// 蓄水池采样算法函数式接口
pub struct ReservoirSamplingAlgo;

impl ReservoirSamplingAlgo {
    /// 从迭代器中采样固定数量的元素
    ///
    /// # 参数
    /// - `iter`: 元素迭代器
    /// - `k`: 采样数量
    ///
    /// # 返回
    /// 采样结果
    pub fn sample_from_iter<T: Clone, I: Iterator<Item = T>>(iter: I, k: usize) -> Vec<T> {
        let mut reservoir = ReservoirSampling::new(k);
        reservoir.sample_iter(iter);
        reservoir.into_samples()
    }

    /// 从切片中采样固定数量的元素
    pub fn sample_from_slice<T: Clone>(slice: &[T], k: usize) -> Vec<T> {
        Self::sample_from_iter(slice.iter().cloned(), k)
    }

    /// 从向量中采样固定数量的元素
    pub fn sample_from_vec<T: Clone>(vec: Vec<T>, k: usize) -> Vec<T> {
        Self::sample_from_iter(vec.into_iter(), k)
    }

    /// 加权蓄水池采样（Weighted Reservoir Sampling）
    /// 每个元素被采样的概率与其权重成正比
    pub fn weighted_sample_from_iter<T: Clone, I: Iterator<Item = (T, f64)>>(
        iter: I,
        k: usize,
    ) -> Vec<T> {
        let mut samples: Vec<(T, f64)> = Vec::with_capacity(k);
        let mut count = 0;
        let mut rng = rand::thread_rng();

        for (item, weight) in iter {
            count += 1;

            if samples.len() < k {
                samples.push((item, weight));
            } else {
                // 生成随机数决定是否替换
                let u: f64 = rng.gen();
                let threshold: f64 = samples.iter().map(|(_, w)| w).sum::<f64>() / weight;

                if u < 1.0 / threshold {
                    // 随机选择一个样本替换
                    let idx = rng.gen_range(0..k);
                    samples[idx] = (item, weight);
                }
            }
        }

        samples.into_iter().map(|(item, _)| item).collect()
    }

    /// 分层蓄水池采样
    /// 根据键将元素分组，每组独立采样
    pub fn stratified_sample_from_iter<T: Clone, K: Eq + std::hash::Hash, I: Iterator<Item = (K, T)>>(
        iter: I,
        k_per_stratum: usize,
    ) -> HashMap<K, Vec<T>> {
        use std::collections::HashMap;

        let mut stratum_reservoirs: HashMap<K, ReservoirSampling<T>> = HashMap::new();

        for (key, item) in iter {
            stratum_reservoirs
                .entry(key)
                .or_insert_with(|| ReservoirSampling::new(k_per_stratum))
                .sample(item);
        }

        stratum_reservoirs
            .into_iter()
            .map(|(k, v)| (k, v.into_samples()))
            .collect()
    }
}

/// 用于图遍历的采样扩展
pub struct GraphSampling;

impl GraphSampling {
    /// 在BFS遍历中进行蓄水池采样
    /// 用于从大规模图中随机采样节点
    pub fn sample_bfs_nodes<T: Clone + Eq + std::hash::Hash, F: Fn(&T) -> Vec<T>>(
        start: T,
        get_neighbors: F,
        sample_size: usize,
        max_depth: usize,
    ) -> Vec<T> {
        use std::collections::{HashSet, VecDeque};

        let mut reservoir = ReservoirSampling::new(sample_size);
        let mut visited: HashSet<T> = HashSet::new();
        let mut queue: VecDeque<(T, usize)> = VecDeque::new();

        queue.push_back((start.clone(), 0));
        visited.insert(start.clone());
        reservoir.sample(start);

        while let Some((current, depth)) = queue.pop_front() {
            if depth >= max_depth {
                continue;
            }

            for neighbor in get_neighbors(&current) {
                if !visited.contains(&neighbor) {
                    visited.insert(neighbor.clone());
                    reservoir.sample(neighbor.clone());
                    queue.push_back((neighbor, depth + 1));
                }
            }
        }

        reservoir.into_samples()
    }

    /// 在边遍历中进行蓄水池采样
    pub fn sample_edges<T: Clone + Eq + std::hash::Hash>(
        edges: &[(T, T)],
        sample_size: usize,
    ) -> Vec<(T, T)> {
        ReservoirSamplingAlgo::sample_from_slice(edges, sample_size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_sampling() {
        let mut reservoir = ReservoirSampling::new(5);
        let data: Vec<i32> = (0..100).collect();

        for item in &data {
            reservoir.sample(*item);
        }

        let samples = reservoir.samples();
        assert_eq!(samples.len(), 5);
        assert_eq!(reservoir.count(), 100);

        // 检查所有样本都在原始数据范围内
        for sample in samples {
            assert!((0..100).contains(sample));
        }
    }

    #[test]
    fn test_sample_less_than_capacity() {
        let mut reservoir = ReservoirSampling::new(10);
        let data: Vec<i32> = (0..5).collect();

        for item in &data {
            reservoir.sample(*item);
        }

        let samples = reservoir.samples();
        assert_eq!(samples.len(), 5);
        assert!(!reservoir.is_full());
    }

    #[test]
    fn test_sample_from_iter() {
        let data: Vec<i32> = (0..100).collect();
        let samples = ReservoirSamplingAlgo::sample_from_iter(data.into_iter(), 10);

        assert_eq!(samples.len(), 10);
    }

    #[test]
    fn test_sample_from_slice() {
        let data: Vec<i32> = (0..100).collect();
        let samples = ReservoirSamplingAlgo::sample_from_slice(&data, 10);

        assert_eq!(samples.len(), 10);
    }

    #[test]
    fn test_clear() {
        let mut reservoir = ReservoirSampling::new(5);
        let data: Vec<i32> = (0..10).collect();

        for item in &data {
            reservoir.sample(*item);
        }

        assert_eq!(reservoir.count(), 10);
        reservoir.clear();
        assert_eq!(reservoir.count(), 0);
        assert!(reservoir.samples().is_empty());
    }

    #[test]
    fn test_weighted_sampling() {
        // 创建带权重的数据
        let data: Vec<(i32, f64)> = (0..100).map(|i| (i, (i + 1) as f64)).collect();
        let samples = ReservoirSamplingAlgo::weighted_sample_from_iter(data.into_iter(), 10);

        assert_eq!(samples.len(), 10);
    }

    #[test]
    fn test_stratified_sampling() {

        // 创建分层数据
        let data: Vec<(String, i32)> = (0..100)
            .map(|i| {
                let key = if i < 50 { "A".to_string() } else { "B".to_string() };
                (key, i)
            })
            .collect();

        let samples =
            ReservoirSamplingAlgo::stratified_sample_from_iter(data.into_iter(), 5);

        assert_eq!(samples.len(), 2);
        assert!(samples.contains_key("A"));
        assert!(samples.contains_key("B"));
        assert_eq!(samples.get("A").expect("Sample should exist in test").len(), 5);
        assert_eq!(samples.get("B").expect("Sample should exist in test").len(), 5);
    }

    #[test]
    fn test_graph_sampling() {
        // 简单的链式图
        let neighbors = |n: &i32| -> Vec<i32> {
            if *n < 10 {
                vec![n + 1]
            } else {
                vec![]
            }
        };

        let samples = GraphSampling::sample_bfs_nodes(0, neighbors, 5, 100);

        assert_eq!(samples.len(), 5);
    }

    #[test]
    fn test_edge_sampling() {
        let edges: Vec<(i32, i32)> = (0..100).map(|i| (i, i + 1)).collect();
        let samples = GraphSampling::sample_edges(&edges, 10);

        assert_eq!(samples.len(), 10);
    }

    #[test]
    fn test_empty_input() {
        let data: Vec<i32> = vec![];
        let samples = ReservoirSamplingAlgo::sample_from_iter(data.into_iter(), 10);

        assert!(samples.is_empty());
    }

    #[test]
    fn test_zero_capacity() {
        let mut reservoir = ReservoirSampling::<i32>::new(0);
        let data: Vec<i32> = (0..10).collect();

        for item in &data {
            reservoir.sample(*item);
        }

        assert!(reservoir.samples().is_empty());
    }
}
