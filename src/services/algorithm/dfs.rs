//! DFS算法模块
//!
//! 包含深度优先搜索相关算法实现

use std::collections::{HashMap, HashSet};
use std::hash::Hash;

/// DFS算法结构体
pub struct Dfs;

impl Dfs {
    /// 查找两个节点之间的所有路径（带深度限制以防止无限循环）
    pub fn find_all_paths<T: Clone + Eq + Hash + std::fmt::Debug>(
        graph: &HashMap<T, Vec<T>>,
        start: &T,
        target: &T,
        max_depth: usize,
    ) -> Vec<Vec<T>> {
        let mut all_paths = Vec::new();
        let mut current_path = Vec::new();
        let mut visited = HashSet::new();

        Self::dfs_find_all_paths(
            graph,
            start,
            target,
            max_depth,
            &mut current_path,
            &mut visited,
            &mut all_paths,
        );

        all_paths
    }

    fn dfs_find_all_paths<T: Clone + Eq + Hash>(
        graph: &HashMap<T, Vec<T>>,
        current: &T,
        target: &T,
        max_depth: usize,
        current_path: &mut Vec<T>,
        visited: &mut HashSet<T>,
        all_paths: &mut Vec<Vec<T>>,
    ) {
        if current_path.len() >= max_depth {
            return;
        }

        current_path.push(current.clone());
        visited.insert(current.clone());

        if current == target {
            all_paths.push(current_path.clone());
        } else if let Some(neighbors) = graph.get(current) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    Self::dfs_find_all_paths(
                        graph,
                        neighbor,
                        target,
                        max_depth,
                        current_path,
                        visited,
                        all_paths,
                    );
                }
            }
        }

        current_path.pop();
        visited.remove(current);
    }

    /// DFS遍历图，返回从起点可达的所有节点
    pub fn traverse<T: Clone + Eq + Hash>(
        graph: &HashMap<T, Vec<T>>,
        start: &T,
    ) -> Vec<T> {
        let mut visited: HashSet<T> = HashSet::new();
        let mut result = Vec::new();

        Self::dfs_traverse(graph, start, &mut visited, &mut result);

        result
    }

    fn dfs_traverse<T: Clone + Eq + Hash>(
        graph: &HashMap<T, Vec<T>>,
        current: &T,
        visited: &mut HashSet<T>,
        result: &mut Vec<T>,
    ) {
        if visited.contains(current) {
            return;
        }

        visited.insert(current.clone());
        result.push(current.clone());

        if let Some(neighbors) = graph.get(current) {
            for neighbor in neighbors {
                Self::dfs_traverse(graph, neighbor, visited, result);
            }
        }
    }

    /// 检查图中是否存在路径
    pub fn has_path<T: Clone + Eq + Hash>(
        graph: &HashMap<T, Vec<T>>,
        start: &T,
        target: &T,
    ) -> bool {
        let mut visited: HashSet<T> = HashSet::new();
        Self::dfs_has_path(graph, start, target, &mut visited)
    }

    fn dfs_has_path<T: Clone + Eq + Hash>(
        graph: &HashMap<T, Vec<T>>,
        current: &T,
        target: &T,
        visited: &mut HashSet<T>,
    ) -> bool {
        if current == target {
            return true;
        }

        if visited.contains(current) {
            return false;
        }

        visited.insert(current.clone());

        if let Some(neighbors) = graph.get(current) {
            for neighbor in neighbors {
                if Self::dfs_has_path(graph, neighbor, target, visited) {
                    return true;
                }
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_all_paths() {
        let mut graph: HashMap<i32, Vec<i32>> = HashMap::new();
        graph.insert(1, vec![2, 3]);
        graph.insert(2, vec![4]);
        graph.insert(3, vec![4]);
        graph.insert(4, vec![]);

        let paths = Dfs::find_all_paths(&graph, &1, &4, 10);
        assert_eq!(paths.len(), 2);
    }

    #[test]
    fn test_find_all_paths_with_depth_limit() {
        let mut graph: HashMap<i32, Vec<i32>> = HashMap::new();
        graph.insert(1, vec![2]);
        graph.insert(2, vec![3]);
        graph.insert(3, vec![4]);
        graph.insert(4, vec![]);

        let paths = Dfs::find_all_paths(&graph, &1, &4, 3);
        assert_eq!(paths.len(), 0);

        let paths = Dfs::find_all_paths(&graph, &1, &4, 4);
        assert_eq!(paths.len(), 1);
    }

    #[test]
    fn test_traverse() {
        let mut graph: HashMap<i32, Vec<i32>> = HashMap::new();
        graph.insert(1, vec![2, 3]);
        graph.insert(2, vec![4]);
        graph.insert(3, vec![]);
        graph.insert(4, vec![]);

        let nodes = Dfs::traverse(&graph, &1);
        assert_eq!(nodes.len(), 4);
    }

    #[test]
    fn test_has_path() {
        let mut graph: HashMap<i32, Vec<i32>> = HashMap::new();
        graph.insert(1, vec![2, 3]);
        graph.insert(2, vec![4]);
        graph.insert(3, vec![]);
        graph.insert(4, vec![]);

        assert!(Dfs::has_path(&graph, &1, &4));
        assert!(!Dfs::has_path(&graph, &4, &1));
        assert!(Dfs::has_path(&graph, &1, &3));
    }
}
