//! 拓扑排序算法模块
//!
//! 包含有向无环图（DAG）的拓扑排序算法实现

use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::Hash;

use crate::services::algorithm::cycle_detection::CycleDetection;

/// 拓扑排序算法结构体
pub struct TopologicalSort;

impl TopologicalSort {
    /// 使用DFS对有向无环图进行拓扑排序
    pub fn sort_dfs<T: Clone + Eq + Hash>(
        graph: &HashMap<T, Vec<T>>,
    ) -> Result<Vec<T>, String> {
        if CycleDetection::has_cycle_directed(graph) {
            return Err("Graph contains a cycle, cannot perform topological sort".to_string());
        }

        let mut visited = HashSet::new();
        let mut stack = Vec::new();
        let all_nodes: HashSet<&T> = graph.keys().collect();

        for node in all_nodes {
            if !visited.contains(node) {
                Self::dfs_topological_sort(graph, node, &mut visited, &mut stack);
            }
        }

        stack.reverse();
        Ok(stack)
    }

    fn dfs_topological_sort<T: Clone + Eq + Hash>(
        graph: &HashMap<T, Vec<T>>,
        node: &T,
        visited: &mut HashSet<T>,
        stack: &mut Vec<T>,
    ) {
        visited.insert(node.clone());

        if let Some(neighbors) = graph.get(node) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    Self::dfs_topological_sort(graph, neighbor, visited, stack);
                }
            }
        }

        stack.push(node.clone());
    }

    /// 使用Kahn算法（基于入度）进行拓扑排序
    pub fn sort_kahn<T: Clone + Eq + Hash>(
        graph: &HashMap<T, Vec<T>>,
    ) -> Result<Vec<T>, String> {
        // 计算入度
        let mut in_degree: HashMap<T, usize> = HashMap::new();
        let mut adjacency_list: HashMap<T, Vec<T>> = HashMap::new();

        // 初始化所有节点的入度为0
        for node in graph.keys() {
            in_degree.entry(node.clone()).or_insert(0);
            adjacency_list.entry(node.clone()).or_insert_with(Vec::new);
        }

        // 计算入度
        for (node, neighbors) in graph {
            for neighbor in neighbors {
                *in_degree.entry(neighbor.clone()).or_insert(0) += 1;
                adjacency_list
                    .entry(node.clone())
                    .or_insert_with(Vec::new)
                    .push(neighbor.clone());
            }
        }

        // 将所有入度为0的节点加入队列
        let mut queue: VecDeque<T> = VecDeque::new();
        for (node, degree) in &in_degree {
            if *degree == 0 {
                queue.push_back(node.clone());
            }
        }

        let mut result = Vec::new();

        while let Some(node) = queue.pop_front() {
            result.push(node.clone());

            if let Some(neighbors) = adjacency_list.get(&node) {
                for neighbor in neighbors {
                    if let Some(degree) = in_degree.get_mut(neighbor) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(neighbor.clone());
                        }
                    }
                }
            }
        }

        // 如果结果中的节点数不等于图中的节点数，说明存在环
        if result.len() != graph.len() {
            return Err("Graph contains a cycle, cannot perform topological sort".to_string());
        }

        Ok(result)
    }

    /// 获取所有入度为0的节点（没有依赖的节点）
    pub fn get_source_nodes<T: Clone + Eq + Hash>(graph: &HashMap<T, Vec<T>>) -> Vec<T> {
        let mut in_degree: HashMap<T, usize> = HashMap::new();

        // 初始化所有节点的入度为0
        for node in graph.keys() {
            in_degree.entry(node.clone()).or_insert(0);
        }

        // 计算入度
        for neighbors in graph.values() {
            for neighbor in neighbors {
                *in_degree.entry(neighbor.clone()).or_insert(0) += 1;
            }
        }

        in_degree
            .into_iter()
            .filter(|(_, degree)| *degree == 0)
            .map(|(node, _)| node)
            .collect()
    }

    /// 获取所有出度为0的节点（没有后继的节点）
    pub fn get_sink_nodes<T: Clone + Eq + Hash>(graph: &HashMap<T, Vec<T>>) -> Vec<T> {
        graph
            .iter()
            .filter(|(_, neighbors)| neighbors.is_empty())
            .map(|(node, _)| node.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topological_sort_dfs() {
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();
        graph.insert("A".to_string(), vec!["B".to_string()]);
        graph.insert("B".to_string(), vec!["C".to_string()]);
        graph.insert("C".to_string(), vec![]);

        let sorted = TopologicalSort::sort_dfs(&graph)
            .expect("Topological sort should succeed in test");
        assert_eq!(
            sorted,
            vec!["A".to_string(), "B".to_string(), "C".to_string()]
        );
    }

    #[test]
    fn test_topological_sort_kahn() {
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();
        graph.insert("A".to_string(), vec!["B".to_string()]);
        graph.insert("B".to_string(), vec!["C".to_string()]);
        graph.insert("C".to_string(), vec![]);

        let sorted = TopologicalSort::sort_kahn(&graph)
            .expect("Topological sort should succeed in test");
        assert_eq!(
            sorted,
            vec!["A".to_string(), "B".to_string(), "C".to_string()]
        );
    }

    #[test]
    fn test_topological_sort_with_cycle() {
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();
        graph.insert("A".to_string(), vec!["B".to_string()]);
        graph.insert("B".to_string(), vec!["C".to_string()]);
        graph.insert("C".to_string(), vec!["A".to_string()]);

        assert!(TopologicalSort::sort_dfs(&graph).is_err());
        assert!(TopologicalSort::sort_kahn(&graph).is_err());
    }

    #[test]
    fn test_get_source_nodes() {
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();
        graph.insert("A".to_string(), vec!["B".to_string(), "C".to_string()]);
        graph.insert("B".to_string(), vec!["C".to_string()]);
        graph.insert("C".to_string(), vec![]);

        let sources = TopologicalSort::get_source_nodes(&graph);
        assert_eq!(sources.len(), 1);
        assert!(sources.contains(&"A".to_string()));
    }

    #[test]
    fn test_get_sink_nodes() {
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();
        graph.insert("A".to_string(), vec!["B".to_string()]);
        graph.insert("B".to_string(), vec!["C".to_string()]);
        graph.insert("C".to_string(), vec![]);

        let sinks = TopologicalSort::get_sink_nodes(&graph);
        assert_eq!(sinks.len(), 1);
        assert!(sinks.contains(&"C".to_string()));
    }

    #[test]
    fn test_empty_graph() {
        let graph: HashMap<String, Vec<String>> = HashMap::new();
        let sorted = TopologicalSort::sort_dfs(&graph)
            .expect("Topological sort should succeed in test");
        assert!(sorted.is_empty());
    }
}
