//! BFS算法模块
//!
//! 包含广度优先搜索相关算法实现

use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::Hash;

/// BFS算法结构体
pub struct Bfs;

impl Bfs {
    /// 使用BFS查找最短路径（无权图）
    pub fn shortest_path<T: Clone + Eq + Hash>(
        graph: &HashMap<T, Vec<T>>,
        start: &T,
        target: &T,
    ) -> Option<Vec<T>> {
        if start == target {
            return Some(vec![start.clone()]);
        }

        let mut queue: VecDeque<(T, Vec<T>)> = VecDeque::new();
        let mut visited: HashSet<T> = HashSet::new();

        queue.push_back((start.clone(), vec![start.clone()]));
        visited.insert(start.clone());

        while let Some((current, path)) = queue.pop_front() {
            if let Some(neighbors) = graph.get(&current) {
                for neighbor in neighbors {
                    if neighbor == target {
                        let mut result_path = path.clone();
                        result_path.push(neighbor.clone());
                        return Some(result_path);
                    }

                    if !visited.contains(neighbor) {
                        visited.insert(neighbor.clone());
                        let mut new_path = path.clone();
                        new_path.push(neighbor.clone());
                        queue.push_back((neighbor.clone(), new_path));
                    }
                }
            }
        }

        None
    }

    /// 使用BFS遍历图，返回从起点可达的所有节点
    pub fn traverse<T: Clone + Eq + Hash>(
        graph: &HashMap<T, Vec<T>>,
        start: &T,
    ) -> Vec<T> {
        let mut visited: HashSet<T> = HashSet::new();
        let mut queue: VecDeque<T> = VecDeque::new();
        let mut result = Vec::new();

        queue.push_back(start.clone());
        visited.insert(start.clone());

        while let Some(current) = queue.pop_front() {
            result.push(current.clone());

            if let Some(neighbors) = graph.get(&current) {
                for neighbor in neighbors {
                    if !visited.contains(neighbor) {
                        visited.insert(neighbor.clone());
                        queue.push_back(neighbor.clone());
                    }
                }
            }
        }

        result
    }

    /// 计算从起点到所有节点的最短距离
    pub fn distances<T: Clone + Eq + Hash>(
        graph: &HashMap<T, Vec<T>>,
        start: &T,
    ) -> HashMap<T, usize> {
        let mut distances: HashMap<T, usize> = HashMap::new();
        let mut queue: VecDeque<(T, usize)> = VecDeque::new();
        let mut visited: HashSet<T> = HashSet::new();

        queue.push_back((start.clone(), 0));
        visited.insert(start.clone());
        distances.insert(start.clone(), 0);

        while let Some((current, dist)) = queue.pop_front() {
            if let Some(neighbors) = graph.get(&current) {
                for neighbor in neighbors {
                    if !visited.contains(neighbor) {
                        visited.insert(neighbor.clone());
                        let new_dist = dist + 1;
                        distances.insert(neighbor.clone(), new_dist);
                        queue.push_back((neighbor.clone(), new_dist));
                    }
                }
            }
        }

        distances
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shortest_path() {
        let mut graph: HashMap<i32, Vec<i32>> = HashMap::new();
        graph.insert(1, vec![2, 3]);
        graph.insert(2, vec![4]);
        graph.insert(3, vec![4]);
        graph.insert(4, vec![]);

        let path = Bfs::shortest_path(&graph, &1, &4);
        assert!(path.is_some());
        let path = path.expect("Path should exist in test");
        assert_eq!(path.len(), 3);
        assert_eq!(path[0], 1);
        assert_eq!(path[2], 4);
    }

    #[test]
    fn test_shortest_path_same_node() {
        let mut graph: HashMap<i32, Vec<i32>> = HashMap::new();
        graph.insert(1, vec![2]);

        let path = Bfs::shortest_path(&graph, &1, &1);
        assert_eq!(path.expect("Path should exist in test"), vec![1]);
    }

    #[test]
    fn test_shortest_path_no_path() {
        let mut graph: HashMap<i32, Vec<i32>> = HashMap::new();
        graph.insert(1, vec![2]);
        graph.insert(3, vec![4]);

        let path = Bfs::shortest_path(&graph, &1, &4);
        assert!(path.is_none());
    }

    #[test]
    fn test_traverse() {
        let mut graph: HashMap<i32, Vec<i32>> = HashMap::new();
        graph.insert(1, vec![2, 3]);
        graph.insert(2, vec![4]);
        graph.insert(3, vec![]);
        graph.insert(4, vec![]);

        let nodes = Bfs::traverse(&graph, &1);
        assert_eq!(nodes.len(), 4);
        assert!(nodes.contains(&1));
        assert!(nodes.contains(&2));
        assert!(nodes.contains(&3));
        assert!(nodes.contains(&4));
    }

    #[test]
    fn test_distances() {
        let mut graph: HashMap<i32, Vec<i32>> = HashMap::new();
        graph.insert(1, vec![2, 3]);
        graph.insert(2, vec![4]);
        graph.insert(3, vec![]);
        graph.insert(4, vec![]);

        let distances = Bfs::distances(&graph, &1);
        assert_eq!(*distances.get(&1).expect("Distance should exist in test"), 0);
        assert_eq!(*distances.get(&2).expect("Distance should exist in test"), 1);
        assert_eq!(*distances.get(&3).expect("Distance should exist in test"), 1);
        assert_eq!(*distances.get(&4).expect("Distance should exist in test"), 2);
    }
}
