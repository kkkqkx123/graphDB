use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::Hash;
use std::cmp::Ordering;
use crate::core::{Vertex, Edge};

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
    pub fn connected_components<T: Clone + Eq + Hash>(
        graph: &HashMap<T, Vec<T>>,
    ) -> Vec<Vec<T>> {
        let mut components = Vec::new();
        let mut visited = HashSet::new();
        let all_nodes: HashSet<&T> = graph.keys().collect();

        for node in all_nodes {
            if !visited.contains(node) {
                let mut component = Vec::new();
                Self::dfs_collect_component(
                    graph,
                    node,
                    &mut visited,
                    &mut component,
                );
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
    pub fn has_cycle_directed<T: Clone + Eq + Hash>(
        graph: &HashMap<T, Vec<T>>,
    ) -> bool {
        let mut white: HashSet<T> = graph.keys().map(|k| k.clone()).collect();
        let mut gray: HashSet<T> = HashSet::new();
        let mut black: HashSet<T> = HashSet::new();

        while let Some(node) = white.iter().next() {
            if Self::dfs_has_cycle_directed(
                graph,
                &node.clone(),
                &mut white,
                &mut gray,
                &mut black,
            ) {
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
                Self::dfs_topological_sort(
                    graph,
                    node,
                    &mut visited,
                    &mut stack,
                );
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
                Self::dfs_collect_component(
                    &reversed_graph,
                    &node,
                    &mut visited,
                    &mut component,
                );
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

    fn reverse_graph<T: Clone + Eq + Hash>(
        graph: &HashMap<T, Vec<T>>,
    ) -> HashMap<T, Vec<T>> {
        let mut reversed = HashMap::new();

        for (node, neighbors) in graph {
            // Ensure the node exists in the reversed graph
            reversed.entry(node.clone()).or_insert_with(Vec::new);

            for neighbor in neighbors {
                reversed.entry(neighbor.clone()).or_insert_with(Vec::new).push(node.clone());
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

/// Sorting algorithms
pub struct SortingAlgorithms;

impl SortingAlgorithms {
    /// Quick sort implementation
    pub fn quick_sort<T: Ord + Clone>(arr: &mut [T]) {
        if arr.len() <= 1 {
            return;
        }
        
        let pivot_idx = Self::partition(arr);
        let (left, right) = arr.split_at_mut(pivot_idx);
        Self::quick_sort(left);
        Self::quick_sort(&mut right[1..]);
    }
    
    fn partition<T: Ord + Clone>(arr: &mut [T]) -> usize {
        let pivot_idx = arr.len() - 1;
        let mut i = 0;
        
        for j in 0..pivot_idx {
            if arr[j] <= arr[pivot_idx] {
                arr.swap(i, j);
                i += 1;
            }
        }
        
        arr.swap(i, pivot_idx);
        i
    }

    /// Merge sort implementation
    pub fn merge_sort<T: Ord + Clone + Default>(arr: &mut [T]) {
        if arr.len() <= 1 {
            return;
        }

        let mid = arr.len() / 2;
        {
            let (left, right) = arr.split_at_mut(mid);
            Self::merge_sort(left);
            Self::merge_sort(right);
        }
        
        Self::merge(arr, mid);
    }
    
    fn merge<T: Ord + Clone + Default>(arr: &mut [T], mid: usize) {
        let mut left_arr = Vec::with_capacity(mid);
        let mut right_arr = Vec::with_capacity(arr.len() - mid);
        
        left_arr.extend_from_slice(&arr[..mid]);
        right_arr.extend_from_slice(&arr[mid..]);
        
        let mut i = 0; // left index
        let mut j = 0; // right index
        let mut k = 0; // merged index
        
        while i < left_arr.len() && j < right_arr.len() {
            if left_arr[i] <= right_arr[j] {
                arr[k] = left_arr[i].clone();
                i += 1;
            } else {
                arr[k] = right_arr[j].clone();
                j += 1;
            }
            k += 1;
        }
        
        while i < left_arr.len() {
            arr[k] = left_arr[i].clone();
            i += 1;
            k += 1;
        }
        
        while j < right_arr.len() {
            arr[k] = right_arr[j].clone();
            j += 1;
            k += 1;
        }
    }

    /// Binary search in a sorted array
    pub fn binary_search<T: Ord>(arr: &[T], target: &T) -> Option<usize> {
        let mut left = 0;
        let mut right = arr.len();
        
        while left < right {
            let mid = left + (right - left) / 2;
            
            match arr[mid].cmp(target) {
                Ordering::Equal => return Some(mid),
                Ordering::Greater => right = mid,
                Ordering::Less => left = mid + 1,
            }
        }
        
        None
    }
}

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
    pub fn dfs<T: Clone + Eq + Hash, F>(
        graph: &HashMap<T, Vec<T>>,
        start: &T,
        mut visit_fn: F,
    ) where
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
                for neighbor in neighbors.iter().rev() {  // Reverse to maintain order when using stack
                    if !visited.contains(neighbor) {
                        stack.push(neighbor.clone());
                    }
                }
            }
        }
    }

    /// Breadth-first search
    pub fn bfs<T: Clone + Eq + Hash, F>(
        graph: &HashMap<T, Vec<T>>,
        start: &T,
        mut visit_fn: F,
    ) where
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

/// String algorithms
pub struct StringAlgorithms;

impl StringAlgorithms {
    /// Compute the Levenshtein distance (edit distance) between two strings
    pub fn levenshtein_distance(s1: &str, s2: &str) -> usize {
        let s1_chars: Vec<char> = s1.chars().collect();
        let s2_chars: Vec<char> = s2.chars().collect();
        let s1_len = s1_chars.len();
        let s2_len = s2_chars.len();
        
        if s1_len == 0 {
            return s2_len;
        }
        if s2_len == 0 {
            return s1_len;
        }
        
        let mut matrix = vec![vec![0; s2_len + 1]; s1_len + 1];
        
        for i in 0..=s1_len {
            matrix[i][0] = i;
        }
        for j in 0..=s2_len {
            matrix[0][j] = j;
        }
        
        for i in 1..=s1_len {
            for j in 1..=s2_len {
                let cost = if s1_chars[i-1] == s2_chars[j-1] { 0 } else { 1 };
                matrix[i][j] = std::cmp::min(
                    std::cmp::min(
                        matrix[i-1][j] + 1,      // deletion
                        matrix[i][j-1] + 1       // insertion
                    ),
                    matrix[i-1][j-1] + cost     // substitution
                );
            }
        }
        
        matrix[s1_len][s2_len]
    }

    /// Find all occurrences of a pattern in a text using naive string matching
    pub fn find_pattern_naive(text: &str, pattern: &str) -> Vec<usize> {
        let mut matches = Vec::new();
        
        if pattern.is_empty() {
            return matches;
        }
        
        let text_chars: Vec<char> = text.chars().collect();
        let pattern_chars: Vec<char> = pattern.chars().collect();
        
        for i in 0..=(text_chars.len() - pattern_chars.len()) {
            let mut found = true;
            for j in 0..pattern_chars.len() {
                if text_chars[i + j] != pattern_chars[j] {
                    found = false;
                    break;
                }
            }
            if found {
                matches.push(i);
            }
        }
        
        matches
    }

    /// KMP (Knuth-Morris-Pratt) algorithm for pattern matching
    pub fn kmp_search(text: &str, pattern: &str) -> Vec<usize> {
        if pattern.is_empty() {
            return vec![];
        }
        
        let text_chars: Vec<char> = text.chars().collect();
        let pattern_chars: Vec<char> = pattern.chars().collect();
        
        // Preprocess the pattern to create the LPS array
        let lps = Self::compute_lps_array(&pattern_chars);
        
        let mut matches = Vec::new();
        let mut text_idx = 0;
        let mut pattern_idx = 0;
        
        while text_idx < text_chars.len() {
            if pattern_chars[pattern_idx] == text_chars[text_idx] {
                text_idx += 1;
                pattern_idx += 1;
            }
            
            if pattern_idx == pattern_chars.len() {
                matches.push(text_idx - pattern_idx);
                pattern_idx = lps[pattern_idx - 1];
            } else if text_idx < text_chars.len() && 
                      pattern_chars[pattern_idx] != text_chars[text_idx] {
                if pattern_idx != 0 {
                    pattern_idx = lps[pattern_idx - 1];
                } else {
                    text_idx += 1;
                }
            }
        }
        
        matches
    }
    
    fn compute_lps_array(pattern: &[char]) -> Vec<usize> {
        let mut lps = vec![0; pattern.len()];
        let mut len = 0;
        let mut idx = 1;
        
        while idx < pattern.len() {
            if pattern[idx] == pattern[len] {
                len += 1;
                lps[idx] = len;
                idx += 1;
            } else {
                if len != 0 {
                    len = lps[len - 1];
                } else {
                    lps[idx] = 0;
                    idx += 1;
                }
            }
        }
        
        lps
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
        assert_eq!(path.unwrap(), vec![1, 2, 4]); // Or [1, 3, 4] - both are valid
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

        let sorted = GraphAlgorithms::topological_sort(&graph).unwrap();
        assert_eq!(sorted, vec!["A".to_string(), "B".to_string(), "C".to_string()]);
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
        assert_eq!(*distances.get(&'D').unwrap(), 9); // A->B->C->D = 4+1+8=13 or A->C->B->D = 2+1+5=8, so 8 is wrong, it should be A->C->B->D = 2+1+5=8, or A->B->D = 4+5=9, so min is 8
        // Actually, A->C->B->D would be 2+1+5=8, but C->B doesn't exist, so it's A->C->D = 2+8=10 or A->B->D = 4+5=9
        // Or A->C->B->D is not possible because we don't have C->B edge, we have B->C
        // The path A->B->D = 4+5=9
        // The path A->C->D = 2+8=10
        // So the shortest path to D is 9
        assert_eq!(*distances.get(&'D').unwrap(), 9);
        assert_eq!(*distances.get(&'C').unwrap(), 2);
        assert_eq!(*distances.get(&'B').unwrap(), 4);
    }

    #[test]
    fn test_quick_sort() {
        let mut arr = [64, 34, 25, 12, 22, 11, 90];
        SortingAlgorithms::quick_sort(&mut arr);
        assert_eq!(arr, [11, 12, 22, 25, 34, 64, 90]);
    }

    #[test]
    fn test_merge_sort() {
        let mut arr = [64, 34, 25, 12, 22, 11, 90];
        SortingAlgorithms::merge_sort(&mut arr);
        assert_eq!(arr, [11, 12, 22, 25, 34, 64, 90]);
    }

    #[test]
    fn test_binary_search() {
        let arr = [1, 3, 5, 7, 9, 11, 13];
        assert_eq!(SortingAlgorithms::binary_search(&arr, &7), Some(3));
        assert_eq!(SortingAlgorithms::binary_search(&arr, &4), None);
    }

    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(StringAlgorithms::levenshtein_distance("kitten", "sitting"), 3);
        assert_eq!(StringAlgorithms::levenshtein_distance("hello", "hello"), 0);
    }

    #[test]
    fn test_kmp_search() {
        let text = "ABABDABACDABABCABCABCABCABC";
        let pattern = "ABABCABCAB";
        let matches = StringAlgorithms::kmp_search(text, pattern);
        assert!(!matches.is_empty());
    }
}