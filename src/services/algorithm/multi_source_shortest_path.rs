//! 多源最短路径算法模块
//!
//! 包含多源最短路径算法实现，支持一次查询多对顶点间的最短路径

use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::Hash;

/// 多源最短路径算法结构体
pub struct MultiSourceShortestPath;

/// 路径结果
#[derive(Debug, Clone)]
pub struct PathResult<T: Clone + Eq + Hash> {
    /// 路径
    pub path: Vec<T>,
    /// 路径长度（边数）
    pub length: usize,
}

impl<T: Clone + Eq + Hash> PathResult<T> {
    fn new(path: Vec<T>) -> Self {
        let length = if path.is_empty() { 0 } else { path.len() - 1 };
        Self { path, length }
    }
}

impl MultiSourceShortestPath {
    /// 查找多对顶点间的最短路径
    /// 
    /// # 参数
    /// - `graph`: 图
    /// - `sources`: 源节点列表
    /// - `targets`: 目标节点列表
    /// 
    /// # 返回
    /// 从每个源节点到每个目标节点的最短路径
    pub fn find_paths<T: Clone + Eq + Hash>(
        graph: &HashMap<T, Vec<T>>,
        sources: &[T],
        targets: &[T],
    ) -> HashMap<(T, T), Option<PathResult<T>>> {
        let mut results = HashMap::new();
        let target_set: HashSet<T> = targets.iter().cloned().collect();

        // 对每个源节点执行BFS
        for source in sources {
            if !graph.contains_key(source) {
                for target in targets {
                    results.insert((source.clone(), target.clone()), None);
                }
                continue;
            }

            let paths = Self::bfs_from_source(graph, source, &target_set);
            
            for target in targets {
                let result = paths.get(target).cloned();
                results.insert((source.clone(), target.clone()), result);
            }
        }

        results
    }

    /// 从单个源节点执行BFS，找到到所有目标节点的最短路径
    fn bfs_from_source<T: Clone + Eq + Hash>(
        graph: &HashMap<T, Vec<T>>,
        source: &T,
        targets: &HashSet<T>,
    ) -> HashMap<T, PathResult<T>> {
        let mut results = HashMap::new();
        let mut visited: HashSet<T> = HashSet::new();
        let mut queue: VecDeque<(T, Vec<T>)> = VecDeque::new();
        let mut found_count = 0;

        queue.push_back((source.clone(), vec![source.clone()]));
        visited.insert(source.clone());

        if targets.contains(source) {
            results.insert(source.clone(), PathResult::new(vec![source.clone()]));
            found_count += 1;
        }

        while let Some((current, path)) = queue.pop_front() {
            if found_count >= targets.len() {
                break;
            }

            if let Some(neighbors) = graph.get(&current) {
                for neighbor in neighbors {
                    if !visited.contains(neighbor) {
                        visited.insert(neighbor.clone());
                        let mut new_path = path.clone();
                        new_path.push(neighbor.clone());

                        if targets.contains(neighbor) {
                            results.insert(neighbor.clone(), PathResult::new(new_path.clone()));
                            found_count += 1;
                        }

                        queue.push_back((neighbor.clone(), new_path));
                    }
                }
            }
        }

        results
    }

    /// 查找多对顶点间的最短路径，支持提前终止条件
    /// 
    /// # 参数
    /// - `graph`: 图
    /// - `pairs`: 源-目标节点对列表
    /// - `max_length`: 最大路径长度，超过则终止
    pub fn find_paths_with_limit<T: Clone + Eq + Hash>(
        graph: &HashMap<T, Vec<T>>,
        pairs: &[(T, T)],
        max_length: Option<usize>,
    ) -> HashMap<(T, T), Option<PathResult<T>>> {
        let mut results: HashMap<(T, T), Option<PathResult<T>>> = HashMap::new();
        
        // 按源节点分组
        let mut source_to_targets: HashMap<T, Vec<(T, T)>> = HashMap::new();
        for (source, target) in pairs {
            source_to_targets
                .entry(source.clone())
                .or_default()
                .push((source.clone(), target.clone()));
        }

        // 对每个源节点执行BFS
        for (source, pairs_group) in source_to_targets {
            if !graph.contains_key(&source) {
                for pair in pairs_group {
                    results.insert(pair, None);
                }
                continue;
            }

            let targets: HashSet<T> = pairs_group.iter().map(|(_, t)| t.clone()).collect();
            let paths = Self::bfs_from_source_with_limit(graph, &source, &targets, max_length);

            for (src, tgt) in pairs_group {
                let result = paths.get(&tgt).cloned();
                results.insert((src, tgt), result);
            }
        }

        results
    }

    fn bfs_from_source_with_limit<T: Clone + Eq + Hash>(
        graph: &HashMap<T, Vec<T>>,
        source: &T,
        targets: &HashSet<T>,
        max_length: Option<usize>,
    ) -> HashMap<T, PathResult<T>> {
        let mut results = HashMap::new();
        let mut visited: HashSet<T> = HashSet::new();
        let mut queue: VecDeque<(T, Vec<T>)> = VecDeque::new();
        let mut found_count = 0;

        queue.push_back((source.clone(), vec![source.clone()]));
        visited.insert(source.clone());

        if targets.contains(source) {
            results.insert(source.clone(), PathResult::new(vec![source.clone()]));
            found_count += 1;
        }

        while let Some((current, path)) = queue.pop_front() {
            if found_count >= targets.len() {
                break;
            }

            // 检查路径长度限制
            if let Some(max_len) = max_length {
                if path.len() > max_len {
                    continue;
                }
            }

            if let Some(neighbors) = graph.get(&current) {
                for neighbor in neighbors {
                    if !visited.contains(neighbor) {
                        visited.insert(neighbor.clone());
                        let mut new_path = path.clone();
                        new_path.push(neighbor.clone());

                        if targets.contains(neighbor) {
                            results.insert(neighbor.clone(), PathResult::new(new_path.clone()));
                            found_count += 1;
                        }

                        queue.push_back((neighbor.clone(), new_path));
                    }
                }
            }
        }

        results
    }

    /// 查找所有源节点到所有其他节点的最短路径（全源最短路径的简化版）
    pub fn find_all_paths_from_sources<T: Clone + Eq + Hash>(
        graph: &HashMap<T, Vec<T>>,
        sources: &[T],
    ) -> HashMap<T, HashMap<T, PathResult<T>>> {
        let mut all_results = HashMap::new();

        for source in sources {
            if !graph.contains_key(source) {
                continue;
            }

            let results = Self::bfs_all_reachable(graph, source);
            all_results.insert(source.clone(), results);
        }

        all_results
    }

    fn bfs_all_reachable<T: Clone + Eq + Hash>(
        graph: &HashMap<T, Vec<T>>,
        source: &T,
    ) -> HashMap<T, PathResult<T>> {
        let mut results = HashMap::new();
        let mut visited: HashSet<T> = HashSet::new();
        let mut queue: VecDeque<(T, Vec<T>)> = VecDeque::new();

        queue.push_back((source.clone(), vec![source.clone()]));
        visited.insert(source.clone());
        results.insert(source.clone(), PathResult::new(vec![source.clone()]));

        while let Some((current, path)) = queue.pop_front() {
            if let Some(neighbors) = graph.get(&current) {
                for neighbor in neighbors {
                    if !visited.contains(neighbor) {
                        visited.insert(neighbor.clone());
                        let mut new_path = path.clone();
                        new_path.push(neighbor.clone());
                        results.insert(neighbor.clone(), PathResult::new(new_path.clone()));
                        queue.push_back((neighbor.clone(), new_path));
                    }
                }
            }
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_graph() -> HashMap<i32, Vec<i32>> {
        let mut graph = HashMap::new();
        graph.insert(1, vec![2, 3]);
        graph.insert(2, vec![4, 5]);
        graph.insert(3, vec![5, 6]);
        graph.insert(4, vec![7]);
        graph.insert(5, vec![7]);
        graph.insert(6, vec![7]);
        graph.insert(7, vec![]);
        graph
    }

    #[test]
    fn test_find_paths() {
        let graph = create_test_graph();
        let sources = vec![1, 2];
        let targets = vec![5, 7];

        let results = MultiSourceShortestPath::find_paths(&graph, &sources, &targets);

        // 1 -> 5
        let path_1_5 = results.get(&(1, 5)).expect("Result should exist in test").as_ref();
        assert!(path_1_5.is_some());
        assert_eq!(path_1_5.expect("Path should exist in test").length, 2);

        // 1 -> 7
        let path_1_7 = results.get(&(1, 7)).expect("Result should exist in test").as_ref();
        assert!(path_1_7.is_some());
        assert_eq!(path_1_7.expect("Path should exist in test").length, 3);

        // 2 -> 5 (直接相连)
        let path_2_5 = results.get(&(2, 5)).expect("Result should exist in test").as_ref();
        assert!(path_2_5.is_some());
        assert_eq!(path_2_5.expect("Path should exist in test").length, 1);
    }

    #[test]
    fn test_find_paths_with_limit() {
        let graph = create_test_graph();
        let pairs = vec![(1, 7), (2, 7)];

        // 限制路径长度为2（即最多3个节点）
        let results = MultiSourceShortestPath::find_paths_with_limit(&graph, &pairs, Some(2));

        // 1 -> 7 需要3步，应该找不到
        let path_1_7 = results.get(&(1, 7)).expect("Result should exist in test");
        assert!(path_1_7.is_none());

        // 2 -> 7 需要2步，应该能找到
        let path_2_7 = results.get(&(2, 7)).expect("Result should exist in test").as_ref();
        assert!(path_2_7.is_some());
    }

    #[test]
    fn test_find_all_paths_from_sources() {
        let graph = create_test_graph();
        let sources = vec![1];

        let results = MultiSourceShortestPath::find_all_paths_from_sources(&graph, &sources);

        let paths_from_1 = results.get(&1).expect("Result should exist in test");
        assert_eq!(paths_from_1.len(), 7); // 可以到达所有节点

        let path_to_7 = paths_from_1.get(&7).expect("Path should exist in test");
        assert_eq!(path_to_7.length, 3);
    }

    #[test]
    fn test_no_path() {
        let mut graph = HashMap::new();
        graph.insert(1, vec![2]);
        graph.insert(3, vec![4]);

        let sources = vec![1];
        let targets = vec![4];

        let results = MultiSourceShortestPath::find_paths(&graph, &sources, &targets);

        let path = results.get(&(1, 4)).expect("Result should exist in test");
        assert!(path.is_none());
    }

    #[test]
    fn test_same_source_target() {
        let graph = create_test_graph();
        let sources = vec![1];
        let targets = vec![1];

        let results = MultiSourceShortestPath::find_paths(&graph, &sources, &targets);

        let path = results.get(&(1, 1)).expect("Result should exist in test").as_ref();
        assert!(path.is_some());
        assert_eq!(path.expect("Path should exist in test").length, 0);
    }
}
