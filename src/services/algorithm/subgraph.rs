//! 子图提取算法模块
//!
//! 包含子图提取相关算法实现，支持GET SUBGRAPH类查询

use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::Hash;

/// 边的方向
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeDirection {
    /// 出边
    Out,
    /// 入边
    In,
    /// 双向
    Both,
}

/// 子图提取算法结构体
pub struct SubgraphExtractor;

/// 子图结果
#[derive(Debug, Clone)]
pub struct SubgraphResult<T: Clone + Eq + Hash> {
    /// 子图中的节点
    pub nodes: HashSet<T>,
    /// 子图中的边 (from, to)
    pub edges: Vec<(T, T)>,
    /// 每个节点的步数（距离起点的步数）
    pub node_steps: HashMap<T, usize>,
}

impl<T: Clone + Eq + Hash> SubgraphResult<T> {
    fn new() -> Self {
        Self {
            nodes: HashSet::new(),
            edges: Vec::new(),
            node_steps: HashMap::new(),
        }
    }
}

impl SubgraphExtractor {
    /// 提取子图
    /// 
    /// # 参数
    /// - `graph`: 原始图
    /// - `start`: 起始节点
    /// - `steps`: 扩展步数
    /// - `direction`: 边的方向
    /// 
    /// # 返回
    /// 包含子图节点、边和步数信息的结果
    pub fn extract<T: Clone + Eq + Hash>(
        graph: &HashMap<T, Vec<T>>,
        start: &T,
        steps: usize,
        direction: EdgeDirection,
    ) -> SubgraphResult<T> {
        let mut result = SubgraphResult::new();
        
        if !graph.contains_key(start) {
            return result;
        }

        let mut visited: HashSet<T> = HashSet::new();
        let mut queue: VecDeque<(T, usize)> = VecDeque::new();

        queue.push_back((start.clone(), 0));
        visited.insert(start.clone());
        result.nodes.insert(start.clone());
        result.node_steps.insert(start.clone(), 0);

        while let Some((current, current_step)) = queue.pop_front() {
            if current_step >= steps {
                continue;
            }

            match direction {
                EdgeDirection::Out => {
                    Self::expand_outgoing(graph, &current, current_step, &mut visited, &mut queue, &mut result);
                }
                EdgeDirection::In => {
                    Self::expand_incoming(graph, &current, current_step, &mut visited, &mut queue, &mut result);
                }
                EdgeDirection::Both => {
                    Self::expand_outgoing(graph, &current, current_step, &mut visited, &mut queue, &mut result);
                    Self::expand_incoming(graph, &current, current_step, &mut visited, &mut queue, &mut result);
                }
            }
        }

        result
    }

    /// 扩展出边
    fn expand_outgoing<T: Clone + Eq + Hash>(
        graph: &HashMap<T, Vec<T>>,
        current: &T,
        current_step: usize,
        visited: &mut HashSet<T>,
        queue: &mut VecDeque<(T, usize)>,
        result: &mut SubgraphResult<T>,
    ) {
        if let Some(neighbors) = graph.get(current) {
            for neighbor in neighbors {
                result.edges.push((current.clone(), neighbor.clone()));
                
                if !visited.contains(neighbor) {
                    visited.insert(neighbor.clone());
                    result.nodes.insert(neighbor.clone());
                    result.node_steps.insert(neighbor.clone(), current_step + 1);
                    queue.push_back((neighbor.clone(), current_step + 1));
                }
            }
        }
    }

    /// 扩展入边
    fn expand_incoming<T: Clone + Eq + Hash>(
        graph: &HashMap<T, Vec<T>>,
        current: &T,
        current_step: usize,
        visited: &mut HashSet<T>,
        queue: &mut VecDeque<(T, usize)>,
        result: &mut SubgraphResult<T>,
    ) {
        for (node, neighbors) in graph.iter() {
            if neighbors.contains(current) {
                result.edges.push((node.clone(), current.clone()));
                
                if !visited.contains(node) {
                    visited.insert(node.clone());
                    result.nodes.insert(node.clone());
                    result.node_steps.insert(node.clone(), current_step + 1);
                    queue.push_back((node.clone(), current_step + 1));
                }
            }
        }
    }

    /// 提取多起点子图
    pub fn extract_multi_start<T: Clone + Eq + Hash>(
        graph: &HashMap<T, Vec<T>>,
        starts: &[T],
        steps: usize,
        direction: EdgeDirection,
    ) -> SubgraphResult<T> {
        let mut result = SubgraphResult::new();
        let mut visited: HashSet<T> = HashSet::new();
        let mut queue: VecDeque<(T, usize)> = VecDeque::new();

        // 初始化所有起点
        for start in starts {
            if graph.contains_key(start) && !visited.contains(start) {
                visited.insert(start.clone());
                result.nodes.insert(start.clone());
                result.node_steps.insert(start.clone(), 0);
                queue.push_back((start.clone(), 0));
            }
        }

        while let Some((current, current_step)) = queue.pop_front() {
            if current_step >= steps {
                continue;
            }

            match direction {
                EdgeDirection::Out => {
                    Self::expand_outgoing(graph, &current, current_step, &mut visited, &mut queue, &mut result);
                }
                EdgeDirection::In => {
                    Self::expand_incoming(graph, &current, current_step, &mut visited, &mut queue, &mut result);
                }
                EdgeDirection::Both => {
                    Self::expand_outgoing(graph, &current, current_step, &mut visited, &mut queue, &mut result);
                    Self::expand_incoming(graph, &current, current_step, &mut visited, &mut queue, &mut result);
                }
            }
        }

        result
    }

    /// 提取子图并过滤环路（避免重复访问已访问的边）
    pub fn extract_without_loop<T: Clone + Eq + Hash>(
        graph: &HashMap<T, Vec<T>>,
        start: &T,
        steps: usize,
        direction: EdgeDirection,
    ) -> SubgraphResult<T> {
        let mut result = SubgraphResult::new();
        
        if !graph.contains_key(start) {
            return result;
        }

        let mut visited_nodes: HashSet<T> = HashSet::new();
        let mut visited_edges: HashSet<(T, T)> = HashSet::new();
        let mut queue: VecDeque<(T, usize)> = VecDeque::new();

        queue.push_back((start.clone(), 0));
        visited_nodes.insert(start.clone());
        result.nodes.insert(start.clone());
        result.node_steps.insert(start.clone(), 0);

        while let Some((current, current_step)) = queue.pop_front() {
            if current_step >= steps {
                continue;
            }

            match direction {
                EdgeDirection::Out => {
                    Self::expand_outgoing_no_loop(
                        graph, &current, current_step, 
                        &mut visited_nodes, &mut visited_edges, 
                        &mut queue, &mut result
                    );
                }
                EdgeDirection::In => {
                    Self::expand_incoming_no_loop(
                        graph, &current, current_step,
                        &mut visited_nodes, &mut visited_edges,
                        &mut queue, &mut result
                    );
                }
                EdgeDirection::Both => {
                    Self::expand_outgoing_no_loop(
                        graph, &current, current_step,
                        &mut visited_nodes, &mut visited_edges,
                        &mut queue, &mut result
                    );
                    Self::expand_incoming_no_loop(
                        graph, &current, current_step,
                        &mut visited_nodes, &mut visited_edges,
                        &mut queue, &mut result
                    );
                }
            }
        }

        result
    }

    fn expand_outgoing_no_loop<T: Clone + Eq + Hash>(
        graph: &HashMap<T, Vec<T>>,
        current: &T,
        current_step: usize,
        visited_nodes: &mut HashSet<T>,
        visited_edges: &mut HashSet<(T, T)>,
        queue: &mut VecDeque<(T, usize)>,
        result: &mut SubgraphResult<T>,
    ) {
        if let Some(neighbors) = graph.get(current) {
            for neighbor in neighbors {
                let edge = (current.clone(), neighbor.clone());
                
                if !visited_edges.contains(&edge) {
                    visited_edges.insert(edge.clone());
                    result.edges.push(edge);
                }
                
                if !visited_nodes.contains(neighbor) {
                    visited_nodes.insert(neighbor.clone());
                    result.nodes.insert(neighbor.clone());
                    result.node_steps.insert(neighbor.clone(), current_step + 1);
                    queue.push_back((neighbor.clone(), current_step + 1));
                }
            }
        }
    }

    fn expand_incoming_no_loop<T: Clone + Eq + Hash>(
        graph: &HashMap<T, Vec<T>>,
        current: &T,
        current_step: usize,
        visited_nodes: &mut HashSet<T>,
        visited_edges: &mut HashSet<(T, T)>,
        queue: &mut VecDeque<(T, usize)>,
        result: &mut SubgraphResult<T>,
    ) {
        for (node, neighbors) in graph.iter() {
            if neighbors.contains(current) {
                let edge = (node.clone(), current.clone());
                
                if !visited_edges.contains(&edge) {
                    visited_edges.insert(edge.clone());
                    result.edges.push(edge);
                }
                
                if !visited_nodes.contains(node) {
                    visited_nodes.insert(node.clone());
                    result.nodes.insert(node.clone());
                    result.node_steps.insert(node.clone(), current_step + 1);
                    queue.push_back((node.clone(), current_step + 1));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_graph() -> HashMap<i32, Vec<i32>> {
        let mut graph = HashMap::new();
        graph.insert(1, vec![2, 3]);
        graph.insert(2, vec![4, 5]);
        graph.insert(3, vec![5, 6]);
        graph.insert(4, vec![7]);
        graph.insert(5, vec![7]);
        graph.insert(6, vec![7]);
        graph.insert(7, vec![]);
        graph
    }

    #[test]
    fn test_extract_outgoing() {
        let graph = create_test_graph();
        let result = SubgraphExtractor::extract(&graph, &1, 2, EdgeDirection::Out);

        assert!(result.nodes.contains(&1));
        assert!(result.nodes.contains(&2));
        assert!(result.nodes.contains(&3));
        assert!(result.nodes.contains(&4));
        assert!(result.nodes.contains(&5));
        assert!(result.nodes.contains(&6));
        assert!(!result.nodes.contains(&7)); // 需要3步

        assert_eq!(*result.node_steps.get(&1).expect("Step should exist in test"), 0);
        assert_eq!(*result.node_steps.get(&2).expect("Step should exist in test"), 1);
        assert_eq!(*result.node_steps.get(&4).expect("Step should exist in test"), 2);
    }

    #[test]
    fn test_extract_incoming() {
        let graph = create_test_graph();
        let result = SubgraphExtractor::extract(&graph, &7, 2, EdgeDirection::In);

        assert!(result.nodes.contains(&7));
        assert!(result.nodes.contains(&4));
        assert!(result.nodes.contains(&5));
        assert!(result.nodes.contains(&6));
        assert!(!result.nodes.contains(&1));
    }

    #[test]
    fn test_extract_both() {
        let mut graph = HashMap::new();
        graph.insert(1, vec![2]);
        graph.insert(2, vec![3]);
        graph.insert(3, vec![]);

        let result = SubgraphExtractor::extract(&graph, &2, 1, EdgeDirection::Both);

        assert!(result.nodes.contains(&1));
        assert!(result.nodes.contains(&2));
        assert!(result.nodes.contains(&3));
    }

    #[test]
    fn test_extract_multi_start() {
        let graph = create_test_graph();
        let result = SubgraphExtractor::extract_multi_start(&graph, &[1, 7], 1, EdgeDirection::Out);

        assert!(result.nodes.contains(&1));
        assert!(result.nodes.contains(&7));
        assert!(result.nodes.contains(&2));
        assert!(result.nodes.contains(&3));
    }

    #[test]
    fn test_extract_without_loop() {
        let mut graph = HashMap::new();
        graph.insert(1, vec![2]);
        graph.insert(2, vec![1, 3]);
        graph.insert(3, vec![]);

        let result = SubgraphExtractor::extract_without_loop(&graph, &1, 3, EdgeDirection::Both);

        // 边不应该重复
        let edge_count = result.edges.len();
        let unique_edges: HashSet<_> = result.edges.iter().cloned().collect();
        assert_eq!(edge_count, unique_edges.len());
    }

    #[test]
    fn test_start_not_in_graph() {
        let graph = create_test_graph();
        let result = SubgraphExtractor::extract(&graph, &100, 2, EdgeDirection::Out);

        assert!(result.nodes.is_empty());
        assert!(result.edges.is_empty());
    }

    #[test]
    fn test_zero_steps() {
        let graph = create_test_graph();
        let result = SubgraphExtractor::extract(&graph, &1, 0, EdgeDirection::Out);

        assert_eq!(result.nodes.len(), 1);
        assert!(result.nodes.contains(&1));
        assert!(result.edges.is_empty());
    }
}
