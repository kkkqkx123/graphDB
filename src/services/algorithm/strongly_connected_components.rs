//! 强连通分量算法模块
//!
//! 包含有向图强连通分量检测相关算法实现（Kosaraju算法）

use std::collections::{HashMap, HashSet};
use std::hash::Hash;

/// 强连通分量算法结构体
pub struct StronglyConnectedComponents;

impl StronglyConnectedComponents {
    /// 使用Kosaraju算法查找有向图的所有强连通分量
    pub fn find<T: Clone + Eq + Hash + std::fmt::Debug>(
        graph: &HashMap<T, Vec<T>>,
    ) -> Vec<Vec<T>> {
        // 步骤1：按照完成时间填充栈
        let mut visited = HashSet::new();
        let mut stack = Vec::new();
        let all_nodes: HashSet<&T> = graph.keys().collect();

        for node in all_nodes {
            if !visited.contains(node) {
                Self::dfs_finish_time(graph, node, &mut visited, &mut stack);
            }
        }

        // 步骤2：创建反向图
        let reversed_graph = Self::reverse_graph(graph);

        // 步骤3：按照完成时间顺序在反向图上处理节点
        let mut visited = HashSet::new();
        let mut sccs = Vec::new();

        while let Some(node) = stack.pop() {
            if !visited.contains(&node) {
                let mut component = Vec::new();
                Self::dfs_collect_component(&reversed_graph, &node, &mut visited, &mut component);
                sccs.push(component);
            }
        }

        sccs
    }

    fn dfs_finish_time<T: Clone + Eq + Hash>(
        graph: &HashMap<T, Vec<T>>,
        node: &T,
        visited: &mut HashSet<T>,
        stack: &mut Vec<T>,
    ) {
        visited.insert(node.clone());

        if let Some(neighbors) = graph.get(node) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    Self::dfs_finish_time(graph, neighbor, visited, stack);
                }
            }
        }

        stack.push(node.clone());
    }

    fn dfs_collect_component<T: Clone + Eq + Hash>(
        graph: &HashMap<T, Vec<T>>,
        node: &T,
        visited: &mut HashSet<T>,
        component: &mut Vec<T>,
    ) {
        visited.insert(node.clone());
        component.push(node.clone());

        if let Some(neighbors) = graph.get(node) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    Self::dfs_collect_component(graph, neighbor, visited, component);
                }
            }
        }
    }

    fn reverse_graph<T: Clone + Eq + Hash>(graph: &HashMap<T, Vec<T>>) -> HashMap<T, Vec<T>> {
        let mut reversed = HashMap::new();

        for (node, neighbors) in graph {
            reversed.entry(node.clone()).or_insert_with(Vec::new);

            for neighbor in neighbors {
                reversed
                    .entry(neighbor.clone())
                    .or_insert_with(Vec::new)
                    .push(node.clone());
            }
        }

        reversed
    }

    /// 计算强连通分量的数量
    pub fn count<T: Clone + Eq + Hash + std::fmt::Debug>(graph: &HashMap<T, Vec<T>>) -> usize {
        Self::find(graph).len()
    }

    /// 查找包含指定节点的强连通分量
    pub fn find_component_of<T: Clone + Eq + Hash + std::fmt::Debug>(
        graph: &HashMap<T, Vec<T>>,
        node: &T,
    ) -> Option<Vec<T>> {
        let sccs = Self::find(graph);
        sccs.into_iter().find(|component| component.contains(node))
    }

    /// 检查图是否是强连通的（只有一个强连通分量）
    pub fn is_strongly_connected<T: Clone + Eq + Hash + std::fmt::Debug>(
        graph: &HashMap<T, Vec<T>>,
    ) -> bool {
        Self::count(graph) == 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_scc() {
        let mut graph: HashMap<i32, Vec<i32>> = HashMap::new();
        // SCC 1: 1 -> 2 -> 3 -> 1
        graph.insert(1, vec![2]);
        graph.insert(2, vec![3]);
        graph.insert(3, vec![1]);
        // SCC 2: 4 -> 5 -> 4
        graph.insert(4, vec![5]);
        graph.insert(5, vec![4]);
        // Edge between SCCs
        graph.insert(3, vec![1, 4]);

        let sccs = StronglyConnectedComponents::find(&graph);
        assert_eq!(sccs.len(), 2);
    }

    #[test]
    fn test_count() {
        let mut graph: HashMap<i32, Vec<i32>> = HashMap::new();
        graph.insert(1, vec![2]);
        graph.insert(2, vec![3]);
        graph.insert(3, vec![1]);
        graph.insert(4, vec![5]);
        graph.insert(5, vec![4]);

        assert_eq!(StronglyConnectedComponents::count(&graph), 2);
    }

    #[test]
    fn test_is_strongly_connected() {
        let mut graph1: HashMap<i32, Vec<i32>> = HashMap::new();
        graph1.insert(1, vec![2]);
        graph1.insert(2, vec![3]);
        graph1.insert(3, vec![1]);

        assert!(StronglyConnectedComponents::is_strongly_connected(&graph1));

        let mut graph2: HashMap<i32, Vec<i32>> = HashMap::new();
        graph2.insert(1, vec![2]);
        graph2.insert(2, vec![]);

        assert!(!StronglyConnectedComponents::is_strongly_connected(&graph2));
    }

    #[test]
    fn test_find_component_of() {
        let mut graph: HashMap<i32, Vec<i32>> = HashMap::new();
        graph.insert(1, vec![2]);
        graph.insert(2, vec![3]);
        graph.insert(3, vec![1]);
        graph.insert(4, vec![5]);
        graph.insert(5, vec![4]);

        let component = StronglyConnectedComponents::find_component_of(&graph, &1);
        assert!(component.is_some());
        let component = component.expect("Component should exist in test");
        assert_eq!(component.len(), 3);
        assert!(component.contains(&1));
        assert!(component.contains(&2));
        assert!(component.contains(&3));
    }

    #[test]
    fn test_empty_graph() {
        let graph: HashMap<i32, Vec<i32>> = HashMap::new();
        let sccs = StronglyConnectedComponents::find(&graph);
        assert!(sccs.is_empty());
    }
}
