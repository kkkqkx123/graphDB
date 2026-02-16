//! 环检测算法模块
//!
//! 包含有向图和无向图的环检测算法实现

use std::collections::{HashMap, HashSet};
use std::hash::Hash;

/// 环检测算法结构体
pub struct CycleDetection;

impl CycleDetection {
    /// 检测有向图是否包含环（使用三色标记法）
    pub fn has_cycle_directed<T: Clone + Eq + Hash>(graph: &HashMap<T, Vec<T>>) -> bool {
        let mut white: HashSet<T> = graph.keys().map(|k| k.clone()).collect();
        let mut gray: HashSet<T> = HashSet::new();
        let mut black: HashSet<T> = HashSet::new();

        while let Some(node) = white.iter().next() {
            if Self::dfs_has_cycle_directed(
                graph,
                &node.clone(),
                &mut white,
                &mut gray,
                &mut black,
            ) {
                return true;
            }
        }

        false
    }

    fn dfs_has_cycle_directed<T: Clone + Eq + Hash>(
        graph: &HashMap<T, Vec<T>>,
        node: &T,
        white: &mut HashSet<T>,
        gray: &mut HashSet<T>,
        black: &mut HashSet<T>,
    ) -> bool {
        white.remove(node);
        gray.insert(node.clone());

        if let Some(neighbors) = graph.get(node) {
            for neighbor in neighbors {
                if black.contains(neighbor) {
                    continue;
                }
                if gray.contains(neighbor) {
                    return true;
                }
                if Self::dfs_has_cycle_directed(graph, neighbor, white, gray, black) {
                    return true;
                }
            }
        }

        gray.remove(node);
        black.insert(node.clone());
        false
    }

    /// 检测无向图是否包含环
    pub fn has_cycle_undirected<T: Clone + Eq + Hash>(graph: &HashMap<T, Vec<T>>) -> bool {
        let mut visited: HashSet<T> = HashSet::new();

        for node in graph.keys() {
            if !visited.contains(node) {
                if Self::dfs_has_cycle_undirected(graph, node, None, &mut visited) {
                    return true;
                }
            }
        }

        false
    }

    fn dfs_has_cycle_undirected<T: Clone + Eq + Hash>(
        graph: &HashMap<T, Vec<T>>,
        node: &T,
        parent: Option<&T>,
        visited: &mut HashSet<T>,
    ) -> bool {
        visited.insert(node.clone());

        if let Some(neighbors) = graph.get(node) {
            for neighbor in neighbors {
                if Some(neighbor) == parent {
                    continue;
                }
                if visited.contains(neighbor) {
                    return true;
                }
                if Self::dfs_has_cycle_undirected(graph, neighbor, Some(node), visited) {
                    return true;
                }
            }
        }

        false
    }

    /// 查找有向图中的所有环（简单实现，可能不适用于大图）
    pub fn find_all_cycles<T: Clone + Eq + Hash>(
        graph: &HashMap<T, Vec<T>>,
        max_length: usize,
    ) -> Vec<Vec<T>> {
        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut path = Vec::new();

        for start in graph.keys() {
            visited.clear();
            path.clear();
            Self::dfs_find_cycles(graph, start, start, max_length, &mut visited, &mut path, &mut cycles);
        }

        cycles
    }

    fn dfs_find_cycles<T: Clone + Eq + Hash>(
        graph: &HashMap<T, Vec<T>>,
        start: &T,
        current: &T,
        max_length: usize,
        visited: &mut HashSet<T>,
        path: &mut Vec<T>,
        cycles: &mut Vec<Vec<T>>,
    ) {
        if path.len() >= max_length {
            return;
        }

        if !path.is_empty() && current == start {
            cycles.push(path.clone());
            return;
        }

        if visited.contains(current) {
            return;
        }

        visited.insert(current.clone());
        path.push(current.clone());

        if let Some(neighbors) = graph.get(current) {
            for neighbor in neighbors {
                Self::dfs_find_cycles(graph, start, neighbor, max_length, visited, path, cycles);
            }
        }

        path.pop();
        visited.remove(current);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_cycle_directed_with_cycle() {
        let mut graph: HashMap<i32, Vec<i32>> = HashMap::new();
        graph.insert(1, vec![2]);
        graph.insert(2, vec![3]);
        graph.insert(3, vec![1]);

        assert!(CycleDetection::has_cycle_directed(&graph));
    }

    #[test]
    fn test_has_cycle_directed_no_cycle() {
        let mut graph: HashMap<i32, Vec<i32>> = HashMap::new();
        graph.insert(1, vec![2]);
        graph.insert(2, vec![3]);
        graph.insert(3, vec![]);

        assert!(!CycleDetection::has_cycle_directed(&graph));
    }

    #[test]
    fn test_has_cycle_undirected_with_cycle() {
        let mut graph: HashMap<i32, Vec<i32>> = HashMap::new();
        graph.insert(1, vec![2, 3]);
        graph.insert(2, vec![1, 3]);
        graph.insert(3, vec![1, 2]);

        assert!(CycleDetection::has_cycle_undirected(&graph));
    }

    #[test]
    fn test_has_cycle_undirected_no_cycle() {
        let mut graph: HashMap<i32, Vec<i32>> = HashMap::new();
        graph.insert(1, vec![2]);
        graph.insert(2, vec![1, 3]);
        graph.insert(3, vec![2]);

        assert!(!CycleDetection::has_cycle_undirected(&graph));
    }

    #[test]
    fn test_find_all_cycles() {
        let mut graph: HashMap<i32, Vec<i32>> = HashMap::new();
        graph.insert(1, vec![2]);
        graph.insert(2, vec![3]);
        graph.insert(3, vec![1, 4]);
        graph.insert(4, vec![]);

        let cycles = CycleDetection::find_all_cycles(&graph, 5);
        assert!(!cycles.is_empty());
    }

    #[test]
    fn test_empty_graph() {
        let graph: HashMap<i32, Vec<i32>> = HashMap::new();
        assert!(!CycleDetection::has_cycle_directed(&graph));
        assert!(!CycleDetection::has_cycle_undirected(&graph));
    }
}
