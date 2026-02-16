//! Bellman-Ford算法模块
//!
//! 包含Bellman-Ford最短路径算法实现
//! 支持负权边，可检测负权环

use std::collections::HashMap;
use std::hash::Hash;

/// Bellman-Ford算法结构体
pub struct BellmanFord;

/// Bellman-Ford算法结果
#[derive(Debug, Clone)]
pub struct BellmanFordResult<T: Clone + Eq + Hash> {
    /// 从起点到各节点的最短距离
    pub distances: HashMap<T, i32>,
    /// 前驱节点，用于重建路径
    pub predecessors: HashMap<T, T>,
    /// 是否存在负权环
    pub has_negative_cycle: bool,
}

impl<T: Clone + Eq + Hash> BellmanFordResult<T> {
    fn new() -> Self {
        Self {
            distances: HashMap::new(),
            predecessors: HashMap::new(),
            has_negative_cycle: false,
        }
    }

    /// 重建从起点到目标节点的路径
    pub fn reconstruct_path(&self, target: &T) -> Option<Vec<T>> {
        if !self.distances.contains_key(target) {
            return None;
        }

        let mut path = vec![target.clone()];
        let mut current = target;

        while let Some(predecessor) = self.predecessors.get(current) {
            // 检查是否形成环（理论上不应该发生，除非有负权环）
            if path.contains(predecessor) {
                return None;
            }
            path.push(predecessor.clone());
            current = predecessor;
        }

        path.reverse();
        Some(path)
    }

    /// 获取到目标节点的最短距离
    pub fn distance_to(&self, target: &T) -> Option<i32> {
        self.distances.get(target).copied()
    }
}

impl BellmanFord {
    /// 执行Bellman-Ford算法
    ///
    /// # 参数
    /// - `graph`: 带权图，支持负权边，值为(邻居, 权重)列表
    /// - `start`: 起始节点
    ///
    /// # 返回
    /// 算法结果，包含距离、前驱节点和负权环检测结果
    pub fn shortest_paths<T: Clone + Eq + Hash>(
        graph: &HashMap<T, Vec<(T, i32)>>,
        start: &T,
    ) -> BellmanFordResult<T> {
        let mut result = BellmanFordResult::new();
        let n = graph.len();

        // 初始化距离
        for node in graph.keys() {
            result.distances.insert(node.clone(), i32::MAX);
        }
        result.distances.insert(start.clone(), 0);

        // 松弛操作，执行n-1次
        for _ in 0..n.saturating_sub(1) {
            let mut updated = false;

            for (u, edges) in graph.iter() {
                let dist_u = match result.distances.get(u) {
                    Some(&d) if d != i32::MAX => d,
                    _ => continue,
                };

                for (v, weight) in edges {
                    let new_dist = dist_u + weight;
                    let current_dist = *result.distances.get(v).unwrap_or(&i32::MAX);

                    if new_dist < current_dist {
                        result.distances.insert(v.clone(), new_dist);
                        result.predecessors.insert(v.clone(), u.clone());
                        updated = true;
                    }
                }
            }

            // 如果没有更新，提前终止
            if !updated {
                break;
            }
        }

        // 检测负权环
        result.has_negative_cycle = Self::detect_negative_cycle(graph, &result.distances);

        result
    }

    /// 检测图中是否存在负权环
    fn detect_negative_cycle<T: Clone + Eq + Hash>(
        graph: &HashMap<T, Vec<(T, i32)>>,
        distances: &HashMap<T, i32>,
    ) -> bool {
        for (u, edges) in graph.iter() {
            let dist_u = match distances.get(u) {
                Some(&d) if d != i32::MAX => d,
                _ => continue,
            };

            for (v, weight) in edges {
                let dist_v = distances.get(v).copied().unwrap_or(i32::MAX);
                if dist_u + weight < dist_v {
                    return true;
                }
            }
        }

        false
    }

    /// 查找负权环（如果存在）
    pub fn find_negative_cycle<T: Clone + Eq + Hash>(
        graph: &HashMap<T, Vec<(T, i32)>>,
    ) -> Option<Vec<T>> {
        let mut distances: HashMap<T, i32> = HashMap::new();
        let mut predecessors: HashMap<T, T> = HashMap::new();

        // 初始化距离
        for node in graph.keys() {
            distances.insert(node.clone(), 0); // 使用0而不是MAX，以便找到任意负权环
        }

        let n = graph.len();
        let mut last_updated = None;

        // 执行n次松弛
        for _ in 0..n {
            last_updated = None;

            for (u, edges) in graph.iter() {
                let dist_u = *distances.get(u).unwrap_or(&i32::MAX);
                if dist_u == i32::MAX {
                    continue;
                }

                for (v, weight) in edges {
                    let new_dist = dist_u + weight;
                    let current_dist = *distances.get(v).unwrap_or(&i32::MAX);

                    if new_dist < current_dist {
                        distances.insert(v.clone(), new_dist);
                        predecessors.insert(v.clone(), u.clone());
                        last_updated = Some(v.clone());
                    }
                }
            }
        }

        // 如果第n次还有更新，说明存在负权环
        if let Some(start) = last_updated {
            // 回溯找到环
            let mut cycle = vec![start.clone()];
            let mut current = start.clone();

            // 回溯n步，确保进入环
            for _ in 0..n {
                if let Some(pred) = predecessors.get(&current) {
                    current = pred.clone();
                } else {
                    break;
                }
            }

            // 记录环的起点
            let cycle_start = current.clone();
            cycle.clear();
            cycle.push(cycle_start.clone());

            // 收集环上的所有节点
            current = cycle_start.clone();
            loop {
                if let Some(pred) = predecessors.get(&current) {
                    current = pred.clone();
                    if current == cycle_start {
                        break;
                    }
                    cycle.push(current.clone());
                } else {
                    break;
                }
            }

            cycle.reverse();
            Some(cycle)
        } else {
            None
        }
    }

    /// 检查从起点是否可达负权环
    pub fn can_reach_negative_cycle<T: Clone + Eq + Hash>(
        graph: &HashMap<T, Vec<(T, i32)>>,
        start: &T,
    ) -> bool {
        let result = Self::shortest_paths(graph, start);
        result.has_negative_cycle
    }

    /// 获取所有可达节点
    pub fn reachable_nodes<T: Clone + Eq + Hash>(
        graph: &HashMap<T, Vec<(T, i32)>>,
        start: &T,
    ) -> Vec<T> {
        let result = Self::shortest_paths(graph, start);
        result
            .distances
            .iter()
            .filter(|(_, &d)| d != i32::MAX)
            .map(|(n, _)| n.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_positive_weight_graph() -> HashMap<char, Vec<(char, i32)>> {
        let mut graph = HashMap::new();
        graph.insert('A', vec![('B', 4), ('C', 2)]);
        graph.insert('B', vec![('C', 1), ('D', 5)]);
        graph.insert('C', vec![('D', 8)]);
        graph.insert('D', vec![]);
        graph
    }

    fn create_negative_weight_graph() -> HashMap<char, Vec<(char, i32)>> {
        let mut graph = HashMap::new();
        graph.insert('A', vec![('B', -1), ('C', 4)]);
        graph.insert('B', vec![('C', 2), ('D', 3)]);
        graph.insert('C', vec![('D', -2)]);
        graph.insert('D', vec![]);
        graph
    }

    fn create_negative_cycle_graph() -> HashMap<char, Vec<(char, i32)>> {
        let mut graph = HashMap::new();
        graph.insert('A', vec![('B', 1)]);
        graph.insert('B', vec![('C', -3)]);
        graph.insert('C', vec![('A', 1)]);
        graph.insert('D', vec![]);
        graph
    }

    #[test]
    fn test_positive_weights() {
        let graph = create_positive_weight_graph();
        let result = BellmanFord::shortest_paths(&graph, &'A');

        assert!(!result.has_negative_cycle);
        assert_eq!(result.distance_to(&'D'), Some(9)); // A -> B -> D
        assert_eq!(result.distance_to(&'C'), Some(2)); // A -> C
    }

    #[test]
    fn test_negative_weights() {
        let graph = create_negative_weight_graph();
        let result = BellmanFord::shortest_paths(&graph, &'A');

        assert!(!result.has_negative_cycle);
        assert_eq!(result.distance_to(&'D'), Some(-1)); // A -> B -> C -> D = -1 + 2 + (-2) = -1
    }

    #[test]
    fn test_negative_cycle_detection() {
        let graph = create_negative_cycle_graph();
        let result = BellmanFord::shortest_paths(&graph, &'A');

        assert!(result.has_negative_cycle);
    }

    #[test]
    fn test_find_negative_cycle() {
        let graph = create_negative_cycle_graph();
        let cycle = BellmanFord::find_negative_cycle(&graph);

        assert!(cycle.is_some());
        let cycle = cycle.expect("Cycle should exist in test");
        assert!(!cycle.is_empty());
    }

    #[test]
    fn test_reconstruct_path() {
        let graph = create_positive_weight_graph();
        let result = BellmanFord::shortest_paths(&graph, &'A');

        let path = result.reconstruct_path(&'D');
        assert!(path.is_some());
        let path = path.expect("Path should exist in test");
        assert_eq!(path, vec!['A', 'B', 'D']);
    }

    #[test]
    fn test_unreachable_node() {
        let mut graph = HashMap::new();
        graph.insert('A', vec![('B', 1)]);
        graph.insert('B', vec![]);
        graph.insert('C', vec![]);

        let result = BellmanFord::shortest_paths(&graph, &'A');

        assert_eq!(result.distance_to(&'C'), Some(i32::MAX));
    }

    #[test]
    fn test_same_node() {
        let graph = create_positive_weight_graph();
        let result = BellmanFord::shortest_paths(&graph, &'A');

        assert_eq!(result.distance_to(&'A'), Some(0));
        let path = result.reconstruct_path(&'A');
        assert_eq!(path, Some(vec!['A']));
    }

    #[test]
    fn test_reachable_nodes() {
        let graph = create_positive_weight_graph();
        let reachable = BellmanFord::reachable_nodes(&graph, &'A');

        assert_eq!(reachable.len(), 4);
        assert!(reachable.contains(&'A'));
        assert!(reachable.contains(&'B'));
        assert!(reachable.contains(&'C'));
        assert!(reachable.contains(&'D'));
    }

    #[test]
    fn test_can_reach_negative_cycle() {
        let graph = create_negative_cycle_graph();
        assert!(BellmanFord::can_reach_negative_cycle(&graph, &'A'));
    }

    #[test]
    fn test_empty_graph() {
        let graph: HashMap<char, Vec<(char, i32)>> = HashMap::new();
        let result = BellmanFord::shortest_paths(&graph, &'A');

        assert!(!result.has_negative_cycle);
        // 空图时，如果起点不在图中，distances应该为空或只包含起点
        assert!(result.distances.is_empty() || result.distances.len() == 1);
    }

    #[test]
    fn test_single_node() {
        let mut graph = HashMap::new();
        graph.insert('A', vec![]);

        let result = BellmanFord::shortest_paths(&graph, &'A');

        assert!(!result.has_negative_cycle);
        assert_eq!(result.distance_to(&'A'), Some(0));
    }
}
