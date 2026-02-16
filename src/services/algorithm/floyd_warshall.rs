//! Floyd-Warshall算法模块
//!
//! 包含Floyd-Warshall全源最短路径算法实现
//! 适用于稠密图，时间复杂度O(V^3)

use std::collections::HashMap;
use std::hash::Hash;

/// Floyd-Warshall算法结构体
pub struct FloydWarshall;

/// Floyd-Warshall算法结果
#[derive(Debug, Clone)]
pub struct FloydWarshallResult<T: Clone + Eq + Hash> {
    /// 距离矩阵
    pub distances: HashMap<(T, T), u32>,
    /// 下一步矩阵，用于重建路径
    pub next: HashMap<(T, T), T>,
    /// 图中所有节点
    pub nodes: Vec<T>,
}

impl<T: Clone + Eq + Hash> FloydWarshallResult<T> {
    fn new(nodes: Vec<T>) -> Self {
        Self {
            distances: HashMap::new(),
            next: HashMap::new(),
            nodes,
        }
    }

    /// 获取从u到v的最短距离
    pub fn distance(&self, u: &T, v: &T) -> Option<u32> {
        self.distances.get(&(u.clone(), v.clone())).copied()
    }

    /// 重建从u到v的最短路径
    pub fn reconstruct_path(&self, u: &T, v: &T) -> Option<Vec<T>> {
        if !self.distances.contains_key(&(u.clone(), v.clone())) {
            return None;
        }

        if u == v {
            return Some(vec![u.clone()]);
        }

        let mut path = vec![u.clone()];
        let mut current = u.clone();

        while current != *v {
            if let Some(next_node) = self.next.get(&(current.clone(), v.clone())) {
                current = next_node.clone();
                path.push(current.clone());
            } else {
                return None;
            }
        }

        Some(path)
    }

    /// 获取所有最短路径
    pub fn all_paths(&self) -> HashMap<(T, T), (Vec<T>, u32)> {
        let mut paths = HashMap::new();

        for u in &self.nodes {
            for v in &self.nodes {
                if let Some(dist) = self.distance(u, v) {
                    if let Some(path) = self.reconstruct_path(u, v) {
                        paths.insert((u.clone(), v.clone()), (path, dist));
                    }
                }
            }
        }

        paths
    }

    /// 获取图的直径（最长最短路径）
    pub fn graph_diameter(&self) -> Option<u32> {
        self.distances
            .values()
            .filter(|&&d| d != u32::MAX)
            .max()
            .copied()
    }

    /// 获取图的中心（偏心距最小的节点）
    pub fn graph_center(&self) -> Option<T> {
        let mut min_eccentricity = u32::MAX;
        let mut center = None;

        for u in &self.nodes {
            let eccentricity = self
                .nodes
                .iter()
                .filter(|v| *v != u)
                .filter_map(|v| self.distance(u, v))
                .filter(|&d| d != u32::MAX)
                .max()
                .unwrap_or(u32::MAX);

            if eccentricity < min_eccentricity {
                min_eccentricity = eccentricity;
                center = Some(u.clone());
            }
        }

        center
    }

    /// 计算节点的偏心距（到最远节点的距离）
    pub fn eccentricity(&self, node: &T) -> Option<u32> {
        self.nodes
            .iter()
            .filter(|v| *v != node)
            .filter_map(|v| self.distance(node, v))
            .filter(|&d| d != u32::MAX)
            .max()
    }

    /// 获取图的半径（最小偏心距）
    pub fn graph_radius(&self) -> Option<u32> {
        self.nodes
            .iter()
            .filter_map(|node| self.eccentricity(node))
            .min()
    }
}

impl FloydWarshall {
    /// 执行Floyd-Warshall算法
    ///
    /// # 参数
    /// - `graph`: 带权图，值为(邻居, 权重)列表
    ///
    /// # 返回
    /// 算法结果，包含距离矩阵和路径重建信息
    pub fn shortest_paths<T: Clone + Eq + Hash>(
        graph: &HashMap<T, Vec<(T, u32)>>,
    ) -> FloydWarshallResult<T> {
        let nodes: Vec<T> = graph.keys().cloned().collect();
        let n = nodes.len();
        let mut result = FloydWarshallResult::new(nodes.clone());

        // 初始化距离矩阵
        for i in 0..n {
            for j in 0..n {
                let u = nodes[i].clone();
                let v = nodes[j].clone();

                if i == j {
                    result.distances.insert((u.clone(), v.clone()), 0);
                } else {
                    result.distances.insert((u.clone(), v.clone()), u32::MAX);
                }
            }
        }

        // 填充直接连接的边
        for (u, edges) in graph.iter() {
            for (v, weight) in edges {
                result.distances.insert((u.clone(), v.clone()), *weight);
                result.next.insert((u.clone(), v.clone()), v.clone());
            }
        }

        // Floyd-Warshall核心算法
        for k in 0..n {
            for i in 0..n {
                for j in 0..n {
                    let u = nodes[i].clone();
                    let intermediate = nodes[k].clone();
                    let v = nodes[j].clone();

                    let dist_ik = result.distances.get(&(u.clone(), intermediate.clone()));
                    let dist_kj = result.distances.get(&(intermediate.clone(), v.clone()));

                    if let (Some(&d_ik), Some(&d_kj)) = (dist_ik, dist_kj) {
                        if d_ik != u32::MAX && d_kj != u32::MAX {
                            let new_dist = d_ik + d_kj;
                            let current_dist = *result.distances.get(&(u.clone(), v.clone())).unwrap_or(&u32::MAX);

                            if new_dist < current_dist {
                                    result.distances.insert((u.clone(), v.clone()), new_dist);
                                    if let Some(next) = result.next.get(&(u.clone(), intermediate.clone())) {
                                        result.next.insert((u.clone(), v.clone()), next.clone());
                                    }
                                }
                        }
                    }
                }
            }
        }

        result
    }

    /// 检测图中是否存在负权环（使用Floyd-Warshall）
    /// 注意：Floyd-Warshall本身不直接支持负权，这里用于检测
    pub fn has_negative_cycle<T: Clone + Eq + Hash>(
        graph: &HashMap<T, Vec<(T, i32)>>,
    ) -> bool {
        let nodes: Vec<T> = graph.keys().cloned().collect();
        let n = nodes.len();
        let mut dist: HashMap<(T, T), i32> = HashMap::new();

        // 初始化
        for i in 0..n {
            for j in 0..n {
                let u = nodes[i].clone();
                let v = nodes[j].clone();

                if i == j {
                    dist.insert((u, v), 0);
                } else {
                    dist.insert((u, v), i32::MAX);
                }
            }
        }

        // 填充直接边
        for (u, edges) in graph.iter() {
            for (v, weight) in edges {
                dist.insert((u.clone(), v.clone()), *weight);
            }
        }

        // Floyd-Warshall
        for k in 0..n {
            for i in 0..n {
                for j in 0..n {
                    let u = nodes[i].clone();
                    let intermediate = nodes[k].clone();
                    let v = nodes[j].clone();

                    let dist_ik = *dist.get(&(u.clone(), intermediate.clone())).unwrap_or(&i32::MAX);
                    let dist_kj = *dist.get(&(intermediate.clone(), v.clone())).unwrap_or(&i32::MAX);

                    if dist_ik != i32::MAX && dist_kj != i32::MAX {
                        let new_dist = dist_ik + dist_kj;
                        let current_dist = *dist.get(&(u.clone(), v.clone())).unwrap_or(&i32::MAX);

                        if new_dist < current_dist {
                            dist.insert((u.clone(), v.clone()), new_dist);
                        }
                    }
                }
            }
        }

        // 检查对角线是否有负值
        for i in 0..n {
            let u = nodes[i].clone();
            if let Some(&d) = dist.get(&(u.clone(), u.clone())) {
                if d < 0 {
                    return true;
                }
            }
        }

        false
    }

    /// 计算图的传递闭包（可达性矩阵）
    pub fn transitive_closure<T: Clone + Eq + Hash>(
        graph: &HashMap<T, Vec<T>>,
    ) -> HashMap<(T, T), bool> {
        let nodes: Vec<T> = graph.keys().cloned().collect();
        let n = nodes.len();
        let mut reachable: HashMap<(T, T), bool> = HashMap::new();

        // 初始化
        for i in 0..n {
            for j in 0..n {
                let u = nodes[i].clone();
                let v = nodes[j].clone();
                reachable.insert((u.clone(), v.clone()), u == v);
            }
        }

        // 填充直接边
        for (u, neighbors) in graph.iter() {
            for v in neighbors {
                reachable.insert((u.clone(), v.clone()), true);
            }
        }

        // Warshall算法计算传递闭包
        for k in 0..n {
            for i in 0..n {
                for j in 0..n {
                    let u = nodes[i].clone();
                    let intermediate = nodes[k].clone();
                    let v = nodes[j].clone();

                    let via_k = *reachable.get(&(u.clone(), intermediate.clone())).unwrap_or(&false)
                        && *reachable.get(&(intermediate.clone(), v.clone())).unwrap_or(&false);

                    let current = *reachable.get(&(u.clone(), v.clone())).unwrap_or(&false);
                    reachable.insert((u.clone(), v.clone()), current || via_k);
                }
            }
        }

        reachable
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_graph() -> HashMap<char, Vec<(char, u32)>> {
        let mut graph = HashMap::new();
        graph.insert('A', vec![('B', 4), ('C', 2)]);
        graph.insert('B', vec![('C', 1), ('D', 5)]);
        graph.insert('C', vec![('D', 8)]);
        graph.insert('D', vec![]);
        graph
    }

    fn create_dense_graph() -> HashMap<i32, Vec<(i32, u32)>> {
        let mut graph = HashMap::new();
        graph.insert(1, vec![(2, 3), (3, 8), (4, 1)]);
        graph.insert(2, vec![(1, 3), (3, 5), (4, 2)]);
        graph.insert(3, vec![(1, 8), (2, 5), (4, 4)]);
        graph.insert(4, vec![(1, 1), (2, 2), (3, 4)]);
        graph
    }

    #[test]
    fn test_shortest_paths() {
        let graph = create_test_graph();
        let result = FloydWarshall::shortest_paths(&graph);

        assert_eq!(result.distance(&'A', &'D'), Some(9)); // A -> B -> D
        assert_eq!(result.distance(&'A', &'C'), Some(2)); // A -> C
        assert_eq!(result.distance(&'B', &'D'), Some(5)); // B -> D
    }

    #[test]
    fn test_reconstruct_path() {
        let graph = create_test_graph();
        let result = FloydWarshall::shortest_paths(&graph);

        let path = result.reconstruct_path(&'A', &'D');
        assert!(path.is_some());
        let path = path.expect("Path should exist in test");
        assert_eq!(path, vec!['A', 'B', 'D']);
    }

    #[test]
    fn test_same_node() {
        let graph = create_test_graph();
        let result = FloydWarshall::shortest_paths(&graph);

        assert_eq!(result.distance(&'A', &'A'), Some(0));
        let path = result.reconstruct_path(&'A', &'A');
        assert_eq!(path, Some(vec!['A']));
    }

    #[test]
    fn test_unreachable_node() {
        let mut graph = HashMap::new();
        graph.insert('A', vec![('B', 1)]);
        graph.insert('B', vec![]);
        graph.insert('C', vec![]);

        let result = FloydWarshall::shortest_paths(&graph);

        // 不可达节点返回MAX或None
        let dist = result.distance(&'A', &'C');
        assert!(dist.is_none() || dist == Some(u32::MAX));
    }

    #[test]
    fn test_graph_diameter() {
        let graph = create_test_graph();
        let result = FloydWarshall::shortest_paths(&graph);

        // 直径应该是 A -> D = 9
        assert_eq!(result.graph_diameter(), Some(9));
    }

    #[test]
    fn test_graph_center() {
        let graph = create_dense_graph();
        let result = FloydWarshall::shortest_paths(&graph);

        let center = result.graph_center();
        assert!(center.is_some());
    }

    #[test]
    fn test_eccentricity() {
        let graph = create_test_graph();
        let result = FloydWarshall::shortest_paths(&graph);

        // A的偏心距应该是到D的距离9
        assert_eq!(result.eccentricity(&'A'), Some(9));
    }

    #[test]
    fn test_graph_radius() {
        let graph = create_test_graph();
        let result = FloydWarshall::shortest_paths(&graph);

        let radius = result.graph_radius();
        assert!(radius.is_some());
    }

    #[test]
    fn test_transitive_closure() {
        let mut graph = HashMap::new();
        graph.insert('A', vec!['B']);
        graph.insert('B', vec!['C']);
        graph.insert('C', vec![]);

        let closure = FloydWarshall::transitive_closure(&graph);

        assert!(closure.get(&('A', 'C')).copied().unwrap_or(false)); // A可以到达C
        assert!(!closure.get(&('C', 'A')).copied().unwrap_or(false)); // C不能到达A
    }

    #[test]
    fn test_all_paths() {
        let graph = create_test_graph();
        let result = FloydWarshall::shortest_paths(&graph);

        let all_paths = result.all_paths();
        assert!(!all_paths.is_empty());

        // 检查A到D的路径
        let path_ad = all_paths.get(&('A', 'D'));
        assert!(path_ad.is_some());
    }

    #[test]
    fn test_empty_graph() {
        let graph: HashMap<char, Vec<(char, u32)>> = HashMap::new();
        let result = FloydWarshall::shortest_paths(&graph);

        assert!(result.distances.is_empty());
        assert!(result.nodes.is_empty());
    }

    #[test]
    fn test_single_node() {
        let mut graph = HashMap::new();
        graph.insert('A', vec![]);

        let result = FloydWarshall::shortest_paths(&graph);

        assert_eq!(result.distance(&'A', &'A'), Some(0));
    }
}
