//! Dijkstra算法模块
//!
//! 包含带权图最短路径算法实现

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::hash::Hash;

/// Dijkstra算法结构体
pub struct Dijkstra;

/// 节点距离结构体，用于优先队列
#[derive(Debug, Clone, Eq, PartialEq)]
struct NodeDistance<T> {
    node: T,
    distance: u32,
}

impl<T: Eq> Ord for NodeDistance<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        other.distance.cmp(&self.distance)
    }
}

impl<T: Eq> PartialOrd for NodeDistance<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Dijkstra {
    /// 计算从起点到所有节点的最短距离
    pub fn shortest_distances<T: Clone + Eq + Hash>(
        graph: &HashMap<T, Vec<(T, u32)>>,
        start: &T,
    ) -> HashMap<T, u32> {
        let mut distances: HashMap<T, u32> = HashMap::new();
        let mut visited: HashSet<T> = HashSet::new();
        let mut to_visit: BinaryHeap<NodeDistance<T>> = BinaryHeap::new();

        // 初始化距离
        for node in graph.keys() {
            distances.insert(node.clone(), u32::MAX);
        }
        distances.insert(start.clone(), 0);

        to_visit.push(NodeDistance {
            node: start.clone(),
            distance: 0,
        });

        while let Some(NodeDistance { node, distance }) = to_visit.pop() {
            if visited.contains(&node) {
                continue;
            }

            visited.insert(node.clone());

            if let Some(neighbors) = graph.get(&node) {
                for (neighbor, weight) in neighbors {
                    let new_distance = distance + weight;

                    if new_distance < *distances.get(neighbor).unwrap_or(&u32::MAX) {
                        distances.insert(neighbor.clone(), new_distance);
                        to_visit.push(NodeDistance {
                            node: neighbor.clone(),
                            distance: new_distance,
                        });
                    }
                }
            }
        }

        distances
    }

    /// 查找从起点到目标节点的最短路径
    pub fn shortest_path<T: Clone + Eq + Hash>(
        graph: &HashMap<T, Vec<(T, u32)>>,
        start: &T,
        target: &T,
    ) -> Option<(Vec<T>, u32)> {
        let mut distances: HashMap<T, u32> = HashMap::new();
        let mut predecessors: HashMap<T, T> = HashMap::new();
        let mut visited: HashSet<T> = HashSet::new();
        let mut to_visit: BinaryHeap<NodeDistance<T>> = BinaryHeap::new();

        // 初始化距离
        for node in graph.keys() {
            distances.insert(node.clone(), u32::MAX);
        }
        distances.insert(start.clone(), 0);

        to_visit.push(NodeDistance {
            node: start.clone(),
            distance: 0,
        });

        while let Some(NodeDistance { node, distance }) = to_visit.pop() {
            if node == *target {
                // 重建路径
                let mut path = vec![target.clone()];
                let mut current = target;
                while let Some(predecessor) = predecessors.get(current) {
                    path.push(predecessor.clone());
                    current = predecessor;
                }
                path.reverse();
                return Some((path, distance));
            }

            if visited.contains(&node) {
                continue;
            }

            visited.insert(node.clone());

            if let Some(neighbors) = graph.get(&node) {
                for (neighbor, weight) in neighbors {
                    let new_distance = distance + weight;

                    if new_distance < *distances.get(neighbor).unwrap_or(&u32::MAX) {
                        distances.insert(neighbor.clone(), new_distance);
                        predecessors.insert(neighbor.clone(), node.clone());
                        to_visit.push(NodeDistance {
                            node: neighbor.clone(),
                            distance: new_distance,
                        });
                    }
                }
            }
        }

        None
    }

    /// 查找从起点到所有节点的最短路径
    pub fn all_shortest_paths<T: Clone + Eq + Hash>(
        graph: &HashMap<T, Vec<(T, u32)>>,
        start: &T,
    ) -> HashMap<T, (Vec<T>, u32)> {
        let mut distances: HashMap<T, u32> = HashMap::new();
        let mut predecessors: HashMap<T, T> = HashMap::new();
        let mut visited: HashSet<T> = HashSet::new();
        let mut to_visit: BinaryHeap<NodeDistance<T>> = BinaryHeap::new();

        // 初始化距离
        for node in graph.keys() {
            distances.insert(node.clone(), u32::MAX);
        }
        distances.insert(start.clone(), 0);

        to_visit.push(NodeDistance {
            node: start.clone(),
            distance: 0,
        });

        while let Some(NodeDistance { node, distance }) = to_visit.pop() {
            if visited.contains(&node) {
                continue;
            }

            visited.insert(node.clone());

            if let Some(neighbors) = graph.get(&node) {
                for (neighbor, weight) in neighbors {
                    let new_distance = distance + weight;

                    if new_distance < *distances.get(neighbor).unwrap_or(&u32::MAX) {
                        distances.insert(neighbor.clone(), new_distance);
                        predecessors.insert(neighbor.clone(), node.clone());
                        to_visit.push(NodeDistance {
                            node: neighbor.clone(),
                            distance: new_distance,
                        });
                    }
                }
            }
        }

        // 为每个节点重建路径
        let mut result = HashMap::new();
        for node in graph.keys() {
            if *distances.get(node).unwrap_or(&u32::MAX) != u32::MAX {
                let mut path = vec![node.clone()];
                let mut current = node;
                while let Some(predecessor) = predecessors.get(current) {
                    path.push(predecessor.clone());
                    current = predecessor;
                }
                path.reverse();
                let dist = *distances.get(node).expect("Distance should exist in test");
                result.insert(node.clone(), (path, dist));
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shortest_distances() {
        let mut graph: HashMap<char, Vec<(char, u32)>> = HashMap::new();
        graph.insert('A', vec![('B', 4), ('C', 2)]);
        graph.insert('B', vec![('C', 1), ('D', 5)]);
        graph.insert('C', vec![('D', 8)]);
        graph.insert('D', vec![]);

        let distances = Dijkstra::shortest_distances(&graph, &'A');
        assert_eq!(*distances.get(&'D').expect("Distance should exist in test"), 9);
        assert_eq!(*distances.get(&'C').expect("Distance should exist in test"), 2);
        assert_eq!(*distances.get(&'B').expect("Distance should exist in test"), 4);
    }

    #[test]
    fn test_shortest_path() {
        let mut graph: HashMap<char, Vec<(char, u32)>> = HashMap::new();
        graph.insert('A', vec![('B', 4), ('C', 2)]);
        graph.insert('B', vec![('C', 1), ('D', 5)]);
        graph.insert('C', vec![('D', 8)]);
        graph.insert('D', vec![]);

        let result = Dijkstra::shortest_path(&graph, &'A', &'D');
        assert!(result.is_some());
        let (path, distance) = result.expect("Path should exist in test");
        assert_eq!(path, vec!['A', 'B', 'D']);
        assert_eq!(distance, 9);
    }

    #[test]
    fn test_all_shortest_paths() {
        let mut graph: HashMap<char, Vec<(char, u32)>> = HashMap::new();
        graph.insert('A', vec![('B', 4), ('C', 2)]);
        graph.insert('B', vec![('C', 1), ('D', 5)]);
        graph.insert('C', vec![('D', 8)]);
        graph.insert('D', vec![]);

        let paths = Dijkstra::all_shortest_paths(&graph, &'A');
        assert_eq!(paths.len(), 4);

        let (path_to_d, dist_to_d) = paths.get(&'D').expect("Path should exist in test");
        assert_eq!(*path_to_d, vec!['A', 'B', 'D']);
        assert_eq!(*dist_to_d, 9);
    }

    #[test]
    fn test_no_path() {
        let mut graph: HashMap<char, Vec<(char, u32)>> = HashMap::new();
        graph.insert('A', vec![('B', 4)]);
        graph.insert('B', vec![]);
        graph.insert('C', vec![]);

        let result = Dijkstra::shortest_path(&graph, &'A', &'C');
        assert!(result.is_none());
    }

    #[test]
    fn test_same_node() {
        let mut graph: HashMap<char, Vec<(char, u32)>> = HashMap::new();
        graph.insert('A', vec![('B', 4)]);
        graph.insert('B', vec![]);

        let result = Dijkstra::shortest_path(&graph, &'A', &'A');
        assert!(result.is_some());
        let (path, distance) = result.expect("Path should exist in test");
        assert_eq!(path, vec!['A']);
        assert_eq!(distance, 0);
    }

    #[test]
    fn test_empty_graph() {
        let graph: HashMap<char, Vec<(char, u32)>> = HashMap::new();
        let distances = Dijkstra::shortest_distances(&graph, &'A');
        // 空图时，如果起点不在图中，distances可能为空或只包含起点
        assert!(distances.is_empty() || distances.len() <= 1);
    }
}
