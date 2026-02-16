use std::cmp::Ordering;
use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::Hash;

/// Graph algorithm utilities
pub struct GraphAlgorithms;

impl GraphAlgorithms {
    /// Find shortest path using BFS (unweighted graph)
    pub fn bfs_shortest_path<T: Clone + Eq + Hash>(
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

        None // No path found
    }

    /// Find all paths between two nodes (limited depth to prevent infinite loops)
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
            return; // Limit depth to prevent infinite loops
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

    /// Find connected components in an undirected graph
    pub fn connected_components<T: Clone + Eq + Hash>(graph: &HashMap<T, Vec<T>>) -> Vec<Vec<T>> {
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

    /// Check if the graph contains a cycle (for directed graphs)
    pub fn has_cycle_directed<T: Clone + Eq + Hash>(graph: &HashMap<T, Vec<T>>) -> bool {
        let mut white: HashSet<T> = graph.keys().map(|k| k.clone()).collect();
        let mut gray: HashSet<T> = HashSet::new();
        let mut black: HashSet<T> = HashSet::new();

        while let Some(node) = white.iter().next() {
            if Self::dfs_has_cycle_directed(graph, &node.clone(), &mut white, &mut gray, &mut black)
            {
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
                    continue; // Already processed
                }
                if gray.contains(neighbor) {
                    return true; // Cycle detected
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

    /// Topological sort for directed acyclic graph (DAG)
    pub fn topological_sort<T: Clone + Eq + Hash>(
        graph: &HashMap<T, Vec<T>>,
    ) -> Result<Vec<T>, String> {
        if Self::has_cycle_directed(graph) {
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

    /// Find strongly connected components using Kosaraju's algorithm
    pub fn strongly_connected_components<T: Clone + Eq + Hash + std::fmt::Debug>(
        graph: &HashMap<T, Vec<T>>,
    ) -> Vec<Vec<T>> {
        // Step 1: Fill the stack with nodes in order of finishing times
        let mut visited = HashSet::new();
        let mut stack = Vec::new();
        let all_nodes: HashSet<&T> = graph.keys().collect();

        for node in all_nodes {
            if !visited.contains(node) {
                Self::dfs_finish_time(graph, node, &mut visited, &mut stack);
            }
        }

        // Step 2: Create a reversed graph
        let reversed_graph = Self::reverse_graph(graph);

        // Step 3: Process nodes in order of finishing times on reversed graph
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

    fn reverse_graph<T: Clone + Eq + Hash>(graph: &HashMap<T, Vec<T>>) -> HashMap<T, Vec<T>> {
        let mut reversed = HashMap::new();

        for (node, neighbors) in graph {
            // Ensure the node exists in the reversed graph
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

    /// Dijkstra's algorithm for shortest path in weighted graph
    pub fn dijkstra<T: Clone + Eq + Hash + std::fmt::Debug>(
        graph: &HashMap<T, Vec<(T, u32)>>, // (neighbor, weight)
        start: &T,
    ) -> HashMap<T, u32> {
        use std::collections::BinaryHeap;

        #[derive(Debug, Clone, Eq, PartialEq)]
        struct NodeDistance<T> {
            node: T,
            distance: u32,
        }

        impl<T: Eq> Ord for NodeDistance<T> {
            fn cmp(&self, other: &Self) -> Ordering {
                // Reverse the ordering to create a min-heap
                other.distance.cmp(&self.distance)
            }
        }

        impl<T: Eq> PartialOrd for NodeDistance<T> {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }

        let mut distances: HashMap<T, u32> = HashMap::new();
        let mut visited: HashSet<T> = HashSet::new();
        let mut to_visit: BinaryHeap<NodeDistance<T>> = BinaryHeap::new();

        // Initialize distances
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

            // Update distances for neighbors
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bfs_shortest_path() {
        let mut graph: HashMap<i32, Vec<i32>> = HashMap::new();
        graph.insert(1, vec![2, 3]);
        graph.insert(2, vec![4]);
        graph.insert(3, vec![4]);
        graph.insert(4, vec![]);

        let path = GraphAlgorithms::bfs_shortest_path(&graph, &1, &4);
        assert!(path.is_some());
        assert_eq!(path.expect("Path should exist in test"), vec![1, 2, 4]); // Or [1, 3, 4] - both are valid
    }

    #[test]
    fn test_connected_components() {
        let mut graph: HashMap<i32, Vec<i32>> = HashMap::new();
        graph.insert(1, vec![2]);
        graph.insert(2, vec![1]);
        graph.insert(3, vec![4]);
        graph.insert(4, vec![3]);
        graph.insert(5, vec![]);

        let components = GraphAlgorithms::connected_components(&graph);
        assert_eq!(components.len(), 3);
    }

    #[test]
    fn test_topological_sort() {
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();
        graph.insert("A".to_string(), vec!["B".to_string()]);
        graph.insert("B".to_string(), vec!["C".to_string()]);
        graph.insert("C".to_string(), vec![]);

        let sorted = GraphAlgorithms::topological_sort(&graph)
            .expect("Topological sort should succeed in test");
        assert_eq!(
            sorted,
            vec!["A".to_string(), "B".to_string(), "C".to_string()]
        );
    }

    #[test]
    fn test_dijkstra() {
        // Create a weighted graph: A->B (weight 4), A->C (weight 2), B->C (weight 1), B->D (weight 5), C->D (weight 8)
        let mut graph: HashMap<char, Vec<(char, u32)>> = HashMap::new();
        graph.insert('A', vec![('B', 4), ('C', 2)]);
        graph.insert('B', vec![('C', 1), ('D', 5)]);
        graph.insert('C', vec![('D', 8)]);
        graph.insert('D', vec![]);

        let distances = GraphAlgorithms::dijkstra(&graph, &'A');
        assert_eq!(
            *distances.get(&'D').expect("Distance should exist in test"),
            9
        ); // A->B->D = 4+5=9
        assert_eq!(
            *distances.get(&'C').expect("Distance should exist in test"),
            2
        );
        assert_eq!(
            *distances.get(&'B').expect("Distance should exist in test"),
            4
        );
    }
}
