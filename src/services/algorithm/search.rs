use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::Hash;

/// Search algorithms
pub struct SearchAlgorithms;

impl SearchAlgorithms {
    /// Linear search
    pub fn linear_search<T: PartialEq>(arr: &[T], target: &T) -> Option<usize> {
        for (i, item) in arr.iter().enumerate() {
            if item == target {
                return Some(i);
            }
        }
        None
    }

    /// Depth-first search
    pub fn dfs<T: Clone + Eq + Hash, F>(graph: &HashMap<T, Vec<T>>, start: &T, mut visit_fn: F)
    where
        F: FnMut(&T) -> bool, // Return true to continue, false to stop
    {
        let mut visited = HashSet::new();
        let mut stack = Vec::new();

        stack.push(start.clone());

        while let Some(current) = stack.pop() {
            if visited.contains(&current) {
                continue;
            }

            visited.insert(current.clone());

            if !visit_fn(&current) {
                break;
            }

            if let Some(neighbors) = graph.get(&current) {
                for neighbor in neighbors.iter().rev() {
                    // Reverse to maintain order when using stack
                    if !visited.contains(neighbor) {
                        stack.push(neighbor.clone());
                    }
                }
            }
        }
    }

    /// Breadth-first search
    pub fn bfs<T: Clone + Eq + Hash, F>(graph: &HashMap<T, Vec<T>>, start: &T, mut visit_fn: F)
    where
        F: FnMut(&T) -> bool, // Return true to continue, false to stop
    {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        queue.push_back(start.clone());
        visited.insert(start.clone());

        while let Some(current) = queue.pop_front() {
            if !visit_fn(&current) {
                break;
            }

            if let Some(neighbors) = graph.get(&current) {
                for neighbor in neighbors {
                    if !visited.contains(neighbor) {
                        visited.insert(neighbor.clone());
                        queue.push_back(neighbor.clone());
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_search() {
        let arr = [1, 3, 5, 7, 9];
        assert_eq!(SearchAlgorithms::linear_search(&arr, &5), Some(2));
        assert_eq!(SearchAlgorithms::linear_search(&arr, &4), None);
    }

    #[test]
    fn test_dfs() {
        let mut graph: HashMap<i32, Vec<i32>> = HashMap::new();
        graph.insert(1, vec![2, 3]);
        graph.insert(2, vec![4]);
        graph.insert(3, vec![4]);
        graph.insert(4, vec![]);

        let mut visited_order = Vec::new();
        SearchAlgorithms::dfs(&graph, &1, |node| {
            visited_order.push(*node);
            true
        });

        // Should visit all nodes reachable from 1
        assert_eq!(visited_order.len(), 4);
        assert!(visited_order.contains(&1));
        assert!(visited_order.contains(&2));
        assert!(visited_order.contains(&3));
        assert!(visited_order.contains(&4));
    }

    #[test]
    fn test_bfs() {
        let mut graph: HashMap<i32, Vec<i32>> = HashMap::new();
        graph.insert(1, vec![2, 3]);
        graph.insert(2, vec![4]);
        graph.insert(3, vec![4]);
        graph.insert(4, vec![]);

        let mut visited_order = Vec::new();
        SearchAlgorithms::bfs(&graph, &1, |node| {
            visited_order.push(*node);
            true
        });

        // Should visit all nodes reachable from 1
        assert_eq!(visited_order.len(), 4);
        assert!(visited_order.contains(&1));
        assert!(visited_order.contains(&2));
        assert!(visited_order.contains(&3));
        assert!(visited_order.contains(&4));
    }
}
