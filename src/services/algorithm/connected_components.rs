//! 连通分量算法模块
//!
//! 包含无向图连通分量检测相关算法实现

use std::collections::{HashMap, HashSet};
use std::hash::Hash;

/// 连通分量算法结构体
pub struct ConnectedComponents;

impl ConnectedComponents {
    /// 查找无向图中的所有连通分量
    pub fn find<T: Clone + Eq + Hash>(graph: &HashMap<T, Vec<T>>) -> Vec<Vec<T>> {
        let mut components = Vec::new();
        let mut visited = HashSet::new();
        let all_nodes: HashSet<&T> = graph.keys().collect();

        for node in all_nodes {
            if !visited.contains(node) {
                let mut component = Vec::new();
                Self::dfs_collect_component(graph, node, &mut visited, &mut component);
                components.push(component);
            }
        }

        components
    }

    fn dfs_collect_component<T: Clone + Eq + Hash>(
        graph: &HashMap<T, Vec<T>>,
        start: &T,
        visited: &mut HashSet<T>,
        component: &mut Vec<T>,
    ) {
        visited.insert(start.clone());
        component.push(start.clone());

        if let Some(neighbors) = graph.get(start) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    Self::dfs_collect_component(graph, neighbor, visited, component);
                }
            }
        }
    }

    /// 计算连通分量的数量
    pub fn count<T: Clone + Eq + Hash>(graph: &HashMap<T, Vec<T>>) -> usize {
        Self::find(graph).len()
    }

    /// 查找包含指定节点的连通分量
    pub fn find_component_of<T: Clone + Eq + Hash>(
        graph: &HashMap<T, Vec<T>>,
        node: &T,
    ) -> Option<Vec<T>> {
        if !graph.contains_key(node) {
            return None;
        }

        let mut visited = HashSet::new();
        let mut component = Vec::new();
        Self::dfs_collect_component(graph, node, &mut visited, &mut component);
        Some(component)
    }

    /// 检查两个节点是否在同一个连通分量中
    pub fn in_same_component<T: Clone + Eq + Hash>(
        graph: &HashMap<T, Vec<T>>,
        node1: &T,
        node2: &T,
    ) -> bool {
        if let Some(component) = Self::find_component_of(graph, node1) {
            component.contains(node2)
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_connected_components() {
        let mut graph: HashMap<i32, Vec<i32>> = HashMap::new();
        graph.insert(1, vec![2]);
        graph.insert(2, vec![1]);
        graph.insert(3, vec![4]);
        graph.insert(4, vec![3]);
        graph.insert(5, vec![]);

        let components = ConnectedComponents::find(&graph);
        assert_eq!(components.len(), 3);
    }

    #[test]
    fn test_count() {
        let mut graph: HashMap<i32, Vec<i32>> = HashMap::new();
        graph.insert(1, vec![2]);
        graph.insert(2, vec![1]);
        graph.insert(3, vec![4]);
        graph.insert(4, vec![3]);
        graph.insert(5, vec![]);

        assert_eq!(ConnectedComponents::count(&graph), 3);
    }

    #[test]
    fn test_find_component_of() {
        let mut graph: HashMap<i32, Vec<i32>> = HashMap::new();
        graph.insert(1, vec![2]);
        graph.insert(2, vec![1, 3]);
        graph.insert(3, vec![2]);
        graph.insert(4, vec![]);

        let component = ConnectedComponents::find_component_of(&graph, &1);
        assert!(component.is_some());
        let component = component.expect("Component should exist in test");
        assert_eq!(component.len(), 3);
        assert!(component.contains(&1));
        assert!(component.contains(&2));
        assert!(component.contains(&3));
    }

    #[test]
    fn test_in_same_component() {
        let mut graph: HashMap<i32, Vec<i32>> = HashMap::new();
        graph.insert(1, vec![2]);
        graph.insert(2, vec![1]);
        graph.insert(3, vec![4]);
        graph.insert(4, vec![3]);

        assert!(ConnectedComponents::in_same_component(&graph, &1, &2));
        assert!(!ConnectedComponents::in_same_component(&graph, &1, &3));
    }

    #[test]
    fn test_empty_graph() {
        let graph: HashMap<i32, Vec<i32>> = HashMap::new();
        let components = ConnectedComponents::find(&graph);
        assert!(components.is_empty());
    }
}
