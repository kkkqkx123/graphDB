//! A*算法模块
//!
//! 包含A*最短路径算法实现，支持启发式搜索

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::hash::Hash;

/// A*算法结构体
pub struct AStar;

/// A*节点结构体，用于优先队列
#[derive(Debug, Clone, Eq, PartialEq)]
struct AStarNode<T> {
    node: T,
    g_score: u32, // 从起点到当前节点的实际代价
    f_score: u32, // g_score + h_score（启发式估计）
}

impl<T: Eq> Ord for AStarNode<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        // 最小堆：f_score小的优先
        other.f_score.cmp(&self.f_score)
    }
}

impl<T: Eq> PartialOrd for AStarNode<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl AStar {
    /// 使用A*算法查找最短路径
    ///
    /// # 参数
    /// - `graph`: 带权图，值为(邻居, 权重)列表
    /// - `start`: 起始节点
    /// - `target`: 目标节点
    /// - `heuristic`: 启发式函数，估计从当前节点到目标节点的代价
    ///
    /// # 返回
    /// 最短路径和总代价
    pub fn shortest_path<T: Clone + Eq + Hash, F: Fn(&T, &T) -> u32>(
        graph: &HashMap<T, Vec<(T, u32)>>,
        start: &T,
        target: &T,
        heuristic: F,
    ) -> Option<(Vec<T>, u32)> {
        if start == target {
            return Some((vec![start.clone()], 0));
        }

        if !graph.contains_key(start) || !graph.contains_key(target) {
            return None;
        }

        let mut open_set: BinaryHeap<AStarNode<T>> = BinaryHeap::new();
        let mut closed_set: HashSet<T> = HashSet::new();
        let mut g_scores: HashMap<T, u32> = HashMap::new();
        let mut predecessors: HashMap<T, T> = HashMap::new();

        let h_start = heuristic(start, target);
        g_scores.insert(start.clone(), 0);
        open_set.push(AStarNode {
            node: start.clone(),
            g_score: 0,
            f_score: h_start,
        });

        while let Some(current) = open_set.pop() {
            if current.node == *target {
                // 重建路径
                let path = Self::reconstruct_path(&predecessors, target);
                return Some((path, current.g_score));
            }

            if closed_set.contains(&current.node) {
                continue;
            }

            closed_set.insert(current.node.clone());

            if let Some(neighbors) = graph.get(&current.node) {
                for (neighbor, weight) in neighbors {
                    if closed_set.contains(neighbor) {
                        continue;
                    }

                    let tentative_g_score = current.g_score + weight;
                    let current_g_score = *g_scores.get(neighbor).unwrap_or(&u32::MAX);

                    if tentative_g_score < current_g_score {
                        predecessors.insert(neighbor.clone(), current.node.clone());
                        g_scores.insert(neighbor.clone(), tentative_g_score);

                        let h_score = heuristic(neighbor, target);
                        open_set.push(AStarNode {
                            node: neighbor.clone(),
                            g_score: tentative_g_score,
                            f_score: tentative_g_score + h_score,
                        });
                    }
                }
            }
        }

        None
    }

    fn reconstruct_path<T: Clone + Eq + Hash>(
        predecessors: &HashMap<T, T>,
        target: &T,
    ) -> Vec<T> {
        let mut path = vec![target.clone()];
        let mut current = target;

        while let Some(predecessor) = predecessors.get(current) {
            path.push(predecessor.clone());
            current = predecessor;
        }

        path.reverse();
        path
    }

    /// 查找最短路径，使用默认启发式函数（返回0，退化为Dijkstra）
    pub fn shortest_path_no_heuristic<T: Clone + Eq + Hash>(
        graph: &HashMap<T, Vec<(T, u32)>>,
        start: &T,
        target: &T,
    ) -> Option<(Vec<T>, u32)> {
        Self::shortest_path(graph, start, target, |_, _| 0)
    }

    /// 使用曼哈顿距离启发式（适用于网格图）
    pub fn shortest_path_manhattan<T: Clone + Eq + Hash>(
        graph: &HashMap<T, Vec<(T, u32)>>,
        start: &T,
        target: &T,
        get_coordinates: impl Fn(&T) -> (i32, i32),
    ) -> Option<(Vec<T>, u32)> {
        let heuristic = |node: &T, target: &T| {
            let (x1, y1) = get_coordinates(node);
            let (x2, y2) = get_coordinates(target);
            ((x1 - x2).abs() + (y1 - y2).abs()) as u32
        };

        Self::shortest_path(graph, start, target, heuristic)
    }

    /// 使用欧几里得距离启发式
    pub fn shortest_path_euclidean<T: Clone + Eq + Hash>(
        graph: &HashMap<T, Vec<(T, u32)>>,
        start: &T,
        target: &T,
        get_coordinates: impl Fn(&T) -> (f64, f64),
    ) -> Option<(Vec<T>, u32)> {
        let heuristic = |node: &T, target: &T| {
            let (x1, y1) = get_coordinates(node);
            let (x2, y2) = get_coordinates(target);
            let dx = x1 - x2;
            let dy = y1 - y2;
            ((dx * dx + dy * dy).sqrt()) as u32
        };

        Self::shortest_path(graph, start, target, heuristic)
    }

    /// 查找从起点到多个目标节点的最短路径
    pub fn shortest_paths_to_multiple_targets<T: Clone + Eq + Hash, F: Fn(&T, &T) -> u32>(
        graph: &HashMap<T, Vec<(T, u32)>>,
        start: &T,
        targets: &[T],
        heuristic: F,
    ) -> HashMap<T, Option<(Vec<T>, u32)>> {
        let mut results = HashMap::new();
        let target_set: HashSet<T> = targets.iter().cloned().collect();
        let mut found_targets = HashSet::new();

        if !graph.contains_key(start) {
            for target in targets {
                results.insert(target.clone(), None);
            }
            return results;
        }

        let mut open_set: BinaryHeap<AStarNode<T>> = BinaryHeap::new();
        let mut closed_set: HashSet<T> = HashSet::new();
        let mut g_scores: HashMap<T, u32> = HashMap::new();
        let mut predecessors: HashMap<T, T> = HashMap::new();

        // 使用最近的目标作为启发式估计
        let h_start = targets
            .iter()
            .map(|t| heuristic(start, t))
            .min()
            .unwrap_or(0);

        g_scores.insert(start.clone(), 0);
        open_set.push(AStarNode {
            node: start.clone(),
            g_score: 0,
            f_score: h_start,
        });

        while let Some(current) = open_set.pop() {
            // 检查是否到达任一目标
            if target_set.contains(&current.node) && !found_targets.contains(&current.node) {
                let path = Self::reconstruct_path(&predecessors, &current.node);
                results.insert(current.node.clone(), Some((path, current.g_score)));
                found_targets.insert(current.node.clone());

                if found_targets.len() == targets.len() {
                    break;
                }
            }

            if closed_set.contains(&current.node) {
                continue;
            }

            closed_set.insert(current.node.clone());

            if let Some(neighbors) = graph.get(&current.node) {
                for (neighbor, weight) in neighbors {
                    if closed_set.contains(neighbor) {
                        continue;
                    }

                    let tentative_g_score = current.g_score + weight;
                    let current_g_score = *g_scores.get(neighbor).unwrap_or(&u32::MAX);

                    if tentative_g_score < current_g_score {
                        predecessors.insert(neighbor.clone(), current.node.clone());
                        g_scores.insert(neighbor.clone(), tentative_g_score);

                        // 使用最近未找到目标的启发式
                        let h_score = targets
                            .iter()
                            .filter(|t| !found_targets.contains(*t))
                            .map(|t| heuristic(neighbor, t))
                            .min()
                            .unwrap_or(0);

                        open_set.push(AStarNode {
                            node: neighbor.clone(),
                            g_score: tentative_g_score,
                            f_score: tentative_g_score + h_score,
                        });
                    }
                }
            }
        }

        // 为未找到的目标添加None
        for target in targets {
            if !results.contains_key(target) {
                results.insert(target.clone(), None);
            }
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_grid_graph() -> HashMap<(i32, i32), Vec<((i32, i32), u32)>> {
        let mut graph = HashMap::new();

        // 创建一个3x3的网格图
        for x in 0..3 {
            for y in 0..3 {
                let mut neighbors = Vec::new();

                // 右
                if x < 2 {
                    neighbors.push(((x + 1, y), 1));
                }
                // 左
                if x > 0 {
                    neighbors.push(((x - 1, y), 1));
                }
                // 上
                if y < 2 {
                    neighbors.push(((x, y + 1), 1));
                }
                // 下
                if y > 0 {
                    neighbors.push(((x, y - 1), 1));
                }

                graph.insert((x, y), neighbors);
            }
        }

        graph
    }

    fn create_weighted_graph() -> HashMap<char, Vec<(char, u32)>> {
        let mut graph = HashMap::new();
        graph.insert('A', vec![('B', 4), ('C', 2)]);
        graph.insert('B', vec![('C', 1), ('D', 5)]);
        graph.insert('C', vec![('D', 8)]);
        graph.insert('D', vec![]);
        graph
    }

    #[test]
    fn test_shortest_path() {
        let graph = create_weighted_graph();

        let result = AStar::shortest_path_no_heuristic(&graph, &'A', &'D');
        assert!(result.is_some());

        let (path, cost) = result.expect("Path should exist in test");
        assert_eq!(path, vec!['A', 'B', 'D']);
        assert_eq!(cost, 9);
    }

    #[test]
    fn test_same_node() {
        let graph = create_weighted_graph();

        let result = AStar::shortest_path_no_heuristic(&graph, &'A', &'A');
        assert!(result.is_some());

        let (path, cost) = result.expect("Path should exist in test");
        assert_eq!(path, vec!['A']);
        assert_eq!(cost, 0);
    }

    #[test]
    fn test_no_path() {
        let mut graph = HashMap::new();
        graph.insert('A', vec![('B', 1)]);
        graph.insert('C', vec![]);

        let result = AStar::shortest_path_no_heuristic(&graph, &'A', &'C');
        assert!(result.is_none());
    }

    #[test]
    fn test_manhattan_heuristic() {
        let graph = create_grid_graph();

        let result = AStar::shortest_path_manhattan(
            &graph,
            &(0, 0),
            &(2, 2),
            |&(x, y)| (x, y), // 坐标就是节点本身
        );

        assert!(result.is_some());
        let (path, cost) = result.expect("Path should exist in test");
        assert_eq!(path.len(), 5); // (0,0) -> (1,0) -> (2,0) -> (2,1) -> (2,2) 或其他最短路径
        assert_eq!(cost, 4);
    }

    #[test]
    fn test_euclidean_heuristic() {
        let graph = create_grid_graph();

        let result = AStar::shortest_path_euclidean(
            &graph,
            &(0, 0),
            &(2, 2),
            |&(x, y)| (x as f64, y as f64),
        );

        assert!(result.is_some());
        let (path, cost) = result.expect("Path should exist in test");
        assert_eq!(cost, 4);
    }

    #[test]
    fn test_shortest_paths_to_multiple_targets() {
        let graph = create_weighted_graph();
        let targets = vec!['C', 'D'];

        let results =
            AStar::shortest_paths_to_multiple_targets(&graph, &'A', &targets, |_, _| 0);

        let path_to_c = results.get(&'C').expect("Result should exist in test").as_ref();
        assert!(path_to_c.is_some());
        let (_path, cost) = path_to_c.expect("Path should exist in test");
        assert_eq!(*cost, 2); // A -> C

        let path_to_d = results.get(&'D').expect("Result should exist in test").as_ref();
        assert!(path_to_d.is_some());
    }

    #[test]
    fn test_with_heuristic() {
        let graph = create_weighted_graph();

        // 自定义启发式：估计到D的距离
        let heuristic = |node: &char, target: &char| -> u32 {
            let estimates = [('A', 10), ('B', 5), ('C', 8), ('D', 0)];
            let estimate_map: HashMap<char, u32> = estimates.iter().cloned().collect();
            if node == target {
                0
            } else {
                *estimate_map.get(node).unwrap_or(&0)
            }
        };

        let result = AStar::shortest_path(&graph, &'A', &'D', heuristic);
        assert!(result.is_some());

        let (path, cost) = result.expect("Path should exist in test");
        assert_eq!(path, vec!['A', 'B', 'D']);
        assert_eq!(cost, 9);
    }
}
