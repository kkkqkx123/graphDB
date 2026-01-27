use async_trait::async_trait;
use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};
use std::cmp::Reverse;
use std::sync::{Arc, Mutex};

use crate::core::error::{DBError, DBResult};
use crate::core::{Edge, Path, Value, Vertex};
use crate::core::vertex_edge_path::Step;
use crate::query::executor::base::{BaseExecutor, EdgeDirection, InputExecutor};
use crate::query::executor::executor_enum::ExecutorEnum;
use crate::query::executor::traits::{ExecutionResult, Executor, HasStorage};
use crate::query::QueryError;
use crate::storage::StorageEngine;
use crate::utils::safe_lock;

#[derive(Debug, Clone)]
pub struct DistanceNode {
    pub distance: f64,
    pub vertex_id: Value,
}

impl Eq for DistanceNode {}

impl PartialEq for DistanceNode {
    fn eq(&self, other: &Self) -> bool {
        self.distance == other.distance && self.vertex_id == other.vertex_id
    }
}

impl Ord for DistanceNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.distance.partial_cmp(&self.distance).unwrap_or(std::cmp::Ordering::Equal)
    }
}

impl PartialOrd for DistanceNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone)]
pub struct BidirectionalBFSState {
    pub left_queue: VecDeque<(Value, Path)>,
    pub right_queue: VecDeque<(Value, Path)>,
    pub left_visited: HashMap<Value, (Path, f64)>,
    pub right_visited: HashMap<Value, (Path, f64)>,
    pub left_edges: Vec<HashMap<Value, Vec<(Edge, Value)>>>,
    pub right_edges: Vec<HashMap<Value, Vec<(Edge, Value)>>>,
}

impl BidirectionalBFSState {
    pub fn new() -> Self {
        Self {
            left_queue: VecDeque::new(),
            right_queue: VecDeque::new(),
            left_visited: HashMap::new(),
            right_visited: HashMap::new(),
            left_edges: Vec::new(),
            right_edges: Vec::new(),
        }
    }
}

impl Default for BidirectionalBFSState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub enum ShortestPathAlgorithmType {
    BFS,
    Dijkstra,
    AStar,
}

pub type ShortestPathAlgorithm = ShortestPathAlgorithmType;

pub struct ShortestPathExecutor<S: StorageEngine + Send + 'static> {
    base: BaseExecutor<S>,
    start_vertex_ids: Vec<Value>,
    end_vertex_ids: Vec<Value>,
    pub edge_direction: EdgeDirection,
    pub edge_types: Option<Vec<String>>,
    pub max_depth: Option<usize>,
    algorithm: ShortestPathAlgorithmType,
    input_executor: Option<Box<ExecutorEnum<S>>>,
    pub shortest_paths: Vec<Path>,
    pub nodes_visited: usize,
    pub edges_traversed: usize,
    pub execution_time_ms: u64,
    pub max_depth_reached: usize,
    pub single_shortest: bool,
    pub limit: usize,
    termination_map: HashMap<(Value, Value), bool>,
}

impl<S: StorageEngine> std::fmt::Debug for ShortestPathExecutor<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ShortestPathExecutor")
            .field("base", &"BaseExecutor")
            .field("start_vertex_ids", &self.start_vertex_ids)
            .field("end_vertex_ids", &self.end_vertex_ids)
            .field("edge_direction", &self.edge_direction)
            .field("edge_types", &self.edge_types)
            .field("max_depth", &self.max_depth)
            .field("algorithm", &self.algorithm)
            .field("single_shortest", &self.single_shortest)
            .field("limit", &self.limit)
            .field("shortest_paths", &self.shortest_paths)
            .field("nodes_visited", &self.nodes_visited)
            .field("edges_traversed", &self.edges_traversed)
            .finish()
    }
}

impl<S: StorageEngine> ShortestPathExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        start_vertex_ids: Vec<Value>,
        end_vertex_ids: Vec<Value>,
        edge_direction: EdgeDirection,
        edge_types: Option<Vec<String>>,
        max_depth: Option<usize>,
        algorithm: ShortestPathAlgorithmType,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "ShortestPathExecutor".to_string(), storage),
            start_vertex_ids,
            end_vertex_ids,
            edge_direction,
            edge_types,
            max_depth,
            algorithm,
            input_executor: None,
            shortest_paths: Vec::new(),
            nodes_visited: 0,
            edges_traversed: 0,
            execution_time_ms: 0,
            max_depth_reached: 0,
            single_shortest: false,
            limit: std::usize::MAX,
            termination_map: HashMap::new(),
        }
    }

    pub fn with_limits(mut self, single_shortest: bool, limit: usize) -> Self {
        self.single_shortest = single_shortest;
        self.limit = limit;
        self
    }

    pub fn get_algorithm(&self) -> ShortestPathAlgorithmType {
        self.algorithm.clone()
    }

    pub fn set_algorithm(&mut self, algorithm: ShortestPathAlgorithmType) {
        self.algorithm = algorithm;
    }

    pub fn get_start_vertex_ids(&self) -> &Vec<Value> {
        &self.start_vertex_ids
    }

    pub fn get_end_vertex_ids(&self) -> &Vec<Value> {
        &self.end_vertex_ids
    }

    pub fn set_start_vertex_ids(&mut self, ids: Vec<Value>) {
        self.start_vertex_ids = ids;
    }

    pub fn set_end_vertex_ids(&mut self, ids: Vec<Value>) {
        self.end_vertex_ids = ids;
    }

    fn init_termination_map(&mut self) {
        self.termination_map.clear();
        for start_id in &self.start_vertex_ids {
            for end_id in &self.end_vertex_ids {
                self.termination_map.insert((start_id.clone(), end_id.clone()), true);
            }
        }
    }

    fn check_termination(&self) -> bool {
        self.termination_map.values().all(|&v| !v)
    }

    fn mark_termination(&mut self, start_id: &Value, end_id: &Value) {
        if let Some(found) = self.termination_map.get_mut(&(start_id.clone(), end_id.clone())) {
            *found = false;
        }
    }

    async fn get_neighbors_with_edges(
        &self,
        node_id: &Value,
    ) -> Result<Vec<(Value, Edge, f64)>, QueryError> {
        let storage = safe_lock(&*self.get_storage())
            .expect("ShortestPathExecutor storage lock should not be poisoned");

        let edges = storage
            .get_node_edges(node_id, EdgeDirection::Both)
            .map_err(|e| QueryError::StorageError(e.to_string()))?;

        let filtered_edges = if let Some(ref edge_types) = self.edge_types {
            edges
                .into_iter()
                .filter(|edge| edge_types.contains(&edge.edge_type))
                .collect()
        } else {
            edges
        };

        let neighbors_with_edges = filtered_edges
            .into_iter()
            .filter_map(|edge| {
                let (neighbor_id, weight) = match self.edge_direction {
                    EdgeDirection::In => {
                        if *edge.dst == *node_id {
                            ((*edge.src).clone(), edge.ranking as f64)
                        } else {
                            return None;
                        }
                    }
                    EdgeDirection::Out => {
                        if *edge.src == *node_id {
                            ((*edge.dst).clone(), edge.ranking as f64)
                        } else {
                            return None;
                        }
                    }
                    EdgeDirection::Both => {
                        if *edge.src == *node_id {
                            ((*edge.dst).clone(), edge.ranking as f64)
                        } else if *edge.dst == *node_id {
                            ((*edge.src).clone(), edge.ranking as f64)
                        } else {
                            return None;
                        }
                    }
                };
                Some((neighbor_id, edge, weight))
            })
            .collect();

        Ok(neighbors_with_edges)
    }

    pub fn has_duplicate_edges(&self, path: &Path) -> bool {
        let mut edge_set = HashSet::new();
        
        for step in &path.steps {
            let edge = &step.edge;
            let edge_key = format!("{}_{}_{}", edge.src, edge.dst, edge.ranking);
            if !edge_set.insert(edge_key) {
                return true;
            }
        }
        
        false
    }

    pub async fn bidirectional_bfs(
        &mut self,
        start_ids: &[Value],
        end_ids: &[Value],
    ) -> Result<Vec<Path>, QueryError> {
        let mut state = BidirectionalBFSState::new();
        let mut result_paths = Vec::new();
        let mut visited_left: HashMap<Value, Path> = HashMap::new();
        let mut visited_right: HashMap<Value, Path> = HashMap::new();
        let mut left_edges: Vec<HashMap<Value, Vec<(Edge, Value)>>> = Vec::new();
        let mut right_edges: Vec<HashMap<Value, Vec<(Edge, Value)>>> = Vec::new();

        for start_id in start_ids {
            let storage = safe_lock(&*self.get_storage())
                .expect("ShortestPathExecutor storage lock should not be poisoned");
            if let Ok(Some(start_vertex)) = storage.get_node(start_id) {
                let initial_path = Path {
                    src: Box::new(start_vertex),
                    steps: Vec::new(),
                };
                state.left_queue.push_back((start_id.clone(), initial_path.clone()));
                visited_left.insert(start_id.clone(), initial_path);
            }
        }

        for end_id in end_ids {
            let storage = safe_lock(&*self.get_storage())
                .expect("ShortestPathExecutor storage lock should not be poisoned");
            if let Ok(Some(end_vertex)) = storage.get_node(end_id) {
                let initial_path = Path {
                    src: Box::new(end_vertex),
                    steps: Vec::new(),
                };
                state.right_queue.push_back((end_id.clone(), initial_path.clone()));
                visited_right.insert(end_id.clone(), initial_path);
            }
        }

        while !state.left_queue.is_empty() && !state.right_queue.is_empty() {
            if self.single_shortest && !result_paths.is_empty() {
                break;
            }

            if self.shortest_paths.len() >= self.limit {
                break;
            }

            left_edges.push(HashMap::new());
            let left_step_edges = left_edges.last_mut().unwrap();
            
            while let Some((current_id, current_path)) = state.left_queue.pop_front() {
                self.nodes_visited += 1;

                if let Some(right_path) = visited_right.get(&current_id) {
                    let mut combined_path = current_path.clone();
                    let mut right_path = right_path.clone();
                    right_path.steps.reverse();
                    combined_path.steps.extend(right_path.steps);
                    
                    if !self.has_duplicate_edges(&combined_path) {
                        result_paths.push(combined_path);
                    }
                    continue;
                }

                if let Some(max_depth) = self.max_depth {
                    if current_path.steps.len() >= max_depth {
                        continue;
                    }
                }

                let neighbors = self.get_neighbors_with_edges(&current_id).await?;
                self.edges_traversed += neighbors.len();

                for (neighbor_id, edge, _weight) in neighbors {
                    let neighbor_id = neighbor_id.clone();
                    let edge = edge.clone();
                    let current_id = current_id.clone();

                    if visited_left.contains_key(&neighbor_id) {
                        continue;
                    }

                    let storage = safe_lock(&*self.get_storage())
                        .expect("ShortestPathExecutor storage lock should not be poisoned");
                    if let Ok(Some(neighbor_vertex)) = storage.get_node(&neighbor_id) {
                        let mut new_path = current_path.clone();
                        new_path.steps.push(Step {
                            dst: Box::new(neighbor_vertex),
                            edge: Box::new(edge.clone()),
                        });

                        state.left_queue.push_back((neighbor_id.clone(), new_path.clone()));
                        visited_left.insert(neighbor_id.clone(), new_path);
                        left_step_edges.insert(neighbor_id.clone(), vec![(edge, current_id)]);
                    }
                }
            }

            if self.check_termination() {
                break;
            }

            right_edges.push(HashMap::new());
            let right_step_edges = right_edges.last_mut().unwrap();
            
            while let Some((current_id, current_path)) = state.right_queue.pop_front() {
                self.nodes_visited += 1;

                if visited_left.contains_key(&current_id) {
                    continue;
                }

                if let Some(max_depth) = self.max_depth {
                    if current_path.steps.len() >= max_depth {
                        continue;
                    }
                }

                let neighbors = self.get_neighbors_with_edges(&current_id).await?;
                self.edges_traversed += neighbors.len();

                for (neighbor_id, edge, _weight) in neighbors {
                    let neighbor_id = neighbor_id.clone();
                    let edge = edge.clone();
                    let current_id = current_id.clone();

                    if visited_right.contains_key(&neighbor_id) {
                        continue;
                    }

                    let storage = safe_lock(&*self.get_storage())
                        .expect("ShortestPathExecutor storage lock should not be poisoned");
                    if let Ok(Some(neighbor_vertex)) = storage.get_node(&neighbor_id) {
                        let mut new_path = current_path.clone();
                        new_path.steps.push(Step {
                            dst: Box::new(neighbor_vertex),
                            edge: Box::new(edge.clone()),
                        });

                        state.right_queue.push_back((neighbor_id.clone(), new_path.clone()));
                        visited_right.insert(neighbor_id.clone(), new_path);
                        right_step_edges.insert(neighbor_id.clone(), vec![(edge, current_id)]);
                    }
                }
            }

            if state.left_queue.is_empty() && state.right_queue.is_empty() {
                break;
            }
        }

        if self.single_shortest && !result_paths.is_empty() {
            result_paths.sort_by(|a, b| a.steps.len().cmp(&b.steps.len()));
            result_paths.truncate(1);
        }

        if result_paths.len() > self.limit {
            result_paths.truncate(self.limit);
        }

        Ok(result_paths)
    }

    pub async fn dijkstra_with_binary_heap(
        &mut self,
        start_ids: &[Value],
        end_ids: &[Value],
    ) -> Result<Vec<Path>, QueryError> {
        let mut distance_map: HashMap<Value, f64> = HashMap::new();
        let mut previous_map: HashMap<Value, (Value, Edge)> = HashMap::new();
        let mut visited_nodes: HashSet<Value> = HashSet::new();
        let mut priority_queue: BinaryHeap<Reverse<DistanceNode>> = BinaryHeap::new();

        for start_id in start_ids {
            distance_map.insert(start_id.clone(), 0.0);
            priority_queue.push(Reverse(DistanceNode {
                distance: 0.0,
                vertex_id: start_id.clone(),
            }));
        }

        let mut result_paths = Vec::new();

        while let Some(Reverse(current)) = priority_queue.pop() {
            if self.single_shortest && !result_paths.is_empty() {
                break;
            }

            if self.shortest_paths.len() >= self.limit {
                break;
            }

            if visited_nodes.contains(&current.vertex_id) {
                continue;
            }
            visited_nodes.insert(current.vertex_id.clone());
            self.nodes_visited += 1;

            if end_ids.contains(&current.vertex_id) {
                if let Some(path) = self.reconstruct_path_with_previous(&current.vertex_id, &previous_map, start_ids)? {
                    if !self.has_duplicate_edges(&path) {
                        result_paths.push(path);
                    }
                }
                continue;
            }

            if let Some(max_depth) = self.max_depth {
                if current.distance as usize >= max_depth {
                    continue;
                }
            }

            let neighbors = self.get_neighbors_with_edges(&current.vertex_id).await?;
            self.edges_traversed += neighbors.len();

            for (neighbor_id, edge, weight) in neighbors {
                if visited_nodes.contains(&neighbor_id) {
                    continue;
                }

                let new_distance = current.distance + weight;
                let existing_distance = distance_map.get(&neighbor_id).unwrap_or(&f64::INFINITY);

                if new_distance < *existing_distance {
                    distance_map.insert(neighbor_id.clone(), new_distance);
                    previous_map.insert(neighbor_id.clone(), (current.vertex_id.clone(), edge));
                    priority_queue.push(Reverse(DistanceNode {
                        distance: new_distance,
                        vertex_id: neighbor_id,
                    }));
                }
            }
        }

        if self.single_shortest && !result_paths.is_empty() {
            result_paths.sort_by(|a, b| {
                let weight_a: f64 = a.steps.iter().map(|s| s.edge.ranking as f64).sum();
                let weight_b: f64 = b.steps.iter().map(|s| s.edge.ranking as f64).sum();
                weight_a.partial_cmp(&weight_b).unwrap()
            });
            result_paths.truncate(1);
        }

        if result_paths.len() > self.limit {
            result_paths.truncate(self.limit);
        }

        Ok(result_paths)
    }

    pub async fn a_star(
        &mut self,
        start_ids: &[Value],
        end_ids: &[Value],
    ) -> Result<Vec<Path>, QueryError> {
        let heuristic = |_current: &Value, _end: &Value| -> f64 {
            0.0f64
        };

        self.dijkstra_with_heuristic(start_ids, end_ids, &heuristic).await
    }

    pub async fn dijkstra_with_heuristic<F>(
        &mut self,
        start_ids: &[Value],
        end_ids: &[Value],
        heuristic: &F,
    ) -> Result<Vec<Path>, QueryError>
    where
        F: Fn(&Value, &Value) -> f64,
    {
        let mut distance_map: HashMap<Value, f64> = HashMap::new();
        let mut previous_map: HashMap<Value, (Value, Edge)> = HashMap::new();
        let mut visited_nodes: HashSet<Value> = HashSet::new();
        let mut priority_queue: BinaryHeap<Reverse<DistanceNode>> = BinaryHeap::new();

        for start_id in start_ids {
            distance_map.insert(start_id.clone(), 0.0);
            priority_queue.push(Reverse(DistanceNode {
                distance: 0.0,
                vertex_id: start_id.clone(),
            }));
        }

        let mut result_paths = Vec::new();

        while let Some(Reverse(current)) = priority_queue.pop() {
            if self.single_shortest && !result_paths.is_empty() {
                break;
            }

            if self.shortest_paths.len() >= self.limit {
                break;
            }

            if visited_nodes.contains(&current.vertex_id) {
                continue;
            }
            visited_nodes.insert(current.vertex_id.clone());
            self.nodes_visited += 1;

            if end_ids.contains(&current.vertex_id) {
                if let Some(path) = self.reconstruct_path_with_previous(&current.vertex_id, &previous_map, start_ids)? {
                    if !self.has_duplicate_edges(&path) {
                        result_paths.push(path);
                    }
                }
                continue;
            }

            if let Some(max_depth) = self.max_depth {
                if current.distance as usize >= max_depth {
                    continue;
                }
            }

            let neighbors = self.get_neighbors_with_edges(&current.vertex_id).await?;
            self.edges_traversed += neighbors.len();

            for (neighbor_id, edge, weight) in neighbors {
                if visited_nodes.contains(&neighbor_id) {
                    continue;
                }

                let g_score = current.distance + weight;
                let h_score = if let Some(end_id) = end_ids.first() {
                    heuristic(&neighbor_id, end_id)
                } else {
                    0.0
                };
                let f_score = g_score + h_score;

                let existing_distance = distance_map.get(&neighbor_id).unwrap_or(&f64::INFINITY);

                if g_score < *existing_distance {
                    distance_map.insert(neighbor_id.clone(), g_score);
                    previous_map.insert(neighbor_id.clone(), (current.vertex_id.clone(), edge));
                    priority_queue.push(Reverse(DistanceNode {
                        distance: f_score,
                        vertex_id: neighbor_id,
                    }));
                }
            }
        }

        if self.single_shortest && !result_paths.is_empty() {
            result_paths.sort_by(|a, b| {
                let weight_a: f64 = a.steps.iter().map(|s| s.edge.ranking as f64).sum();
                let weight_b: f64 = b.steps.iter().map(|s| s.edge.ranking as f64).sum();
                weight_a.partial_cmp(&weight_b).unwrap()
            });
            result_paths.truncate(1);
        }

        if result_paths.len() > self.limit {
            result_paths.truncate(self.limit);
        }

        Ok(result_paths)
    }

    fn reconstruct_path_with_previous(
        &self,
        end_id: &Value,
        previous_map: &HashMap<Value, (Value, Edge)>,
        start_ids: &[Value],
    ) -> Result<Option<Path>, QueryError> {
        let mut path_steps = Vec::new();
        let mut current_id = end_id.clone();

        while let Some((prev_id, edge)) = previous_map.get(&current_id) {
            let storage = safe_lock(&*self.get_storage())
                .expect("ShortestPathExecutor storage lock should not be poisoned");
            if let Ok(Some(current_vertex)) = storage.get_node(&current_id) {
                path_steps.push(Step {
                    dst: Box::new(current_vertex),
                    edge: Box::new(edge.clone()),
                });
            }
            current_id = prev_id.clone();
        }

        if !start_ids.contains(&current_id) {
            return Ok(None);
        }

        let storage = safe_lock(&*self.get_storage())
            .expect("ShortestPathExecutor storage lock should not be poisoned");
        if let Ok(Some(start_vertex)) = storage.get_node(&current_id) {
            path_steps.reverse();

            Ok(Some(Path {
                src: Box::new(start_vertex),
                steps: path_steps,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn bfs_shortest_path(
        &mut self,
        start_ids: &[Value],
        end_ids: &[Value],
    ) -> Result<Vec<Path>, QueryError> {
        let mut queue = VecDeque::new();
        let mut path_map: HashMap<Value, Path> = HashMap::new();
        let mut result_paths = Vec::new();

        for start_id in start_ids {
            let storage = safe_lock(&*self.get_storage())
                .expect("ShortestPathExecutor storage lock should not be poisoned");
            if let Ok(Some(start_vertex)) = storage.get_node(start_id) {
                let initial_path = Path {
                    src: Box::new(start_vertex),
                    steps: Vec::new(),
                };
                queue.push_back((start_id.clone(), initial_path.clone()));
                path_map.insert(start_id.clone(), initial_path);
            }
        }

        while let Some((current_id, current_path)) = queue.pop_front() {
            if self.single_shortest && !result_paths.is_empty() {
                break;
            }

            if end_ids.contains(&current_id) {
                if !self.has_duplicate_edges(&current_path) {
                    result_paths.push(current_path);
                }
                continue;
            }

            if let Some(max_depth) = self.max_depth {
                if current_path.steps.len() >= max_depth {
                    continue;
                }
            }

            let neighbors = self.get_neighbors_with_edges(&current_id).await?;
            self.edges_traversed += neighbors.len();

            for (neighbor_id, edge, _weight) in neighbors {
                if path_map.contains_key(&neighbor_id) {
                    continue;
                }

                let storage = safe_lock(&*self.get_storage())
                    .expect("ShortestPathExecutor storage lock should not be poisoned");
                if let Ok(Some(neighbor_vertex)) = storage.get_node(&neighbor_id) {
                    let mut new_path = current_path.clone();
                    new_path.steps.push(Step {
                        dst: Box::new(neighbor_vertex),
                        edge: Box::new(edge),
                    });

                    queue.push_back((neighbor_id.clone(), new_path.clone()));
                    path_map.insert(neighbor_id, new_path);
                }
            }
        }

        if self.single_shortest && !result_paths.is_empty() {
            result_paths.sort_by(|a, b| a.steps.len().cmp(&b.steps.len()));
            result_paths.truncate(1);
        }

        if result_paths.len() > self.limit {
            result_paths.truncate(self.limit);
        }

        Ok(result_paths)
    }

    pub async fn compute_shortest_paths(&mut self) -> Result<(), QueryError> {
        self.init_termination_map();

        let start_time = std::time::Instant::now();

        let start_ids = self.start_vertex_ids.clone();
        let end_ids = self.end_vertex_ids.clone();

        match self.algorithm {
            ShortestPathAlgorithmType::BFS => {
                self.shortest_paths = self.bfs_shortest_path(&start_ids, &end_ids).await?;
            }
            ShortestPathAlgorithmType::Dijkstra => {
                self.shortest_paths = self.dijkstra_with_binary_heap(&start_ids, &end_ids).await?;
            }
            ShortestPathAlgorithmType::AStar => {
                self.shortest_paths = self.a_star(&start_ids, &end_ids).await?;
            }
        }

        self.execution_time_ms = start_time.elapsed().as_millis() as u64;

        if self.single_shortest && !self.shortest_paths.is_empty() {
            self.shortest_paths.truncate(1);
        }

        if self.shortest_paths.len() > self.limit {
            self.shortest_paths.truncate(self.limit);
        }

        Ok(())
    }

    pub fn build_result(&self) -> ExecutionResult {
        let mut path_values = Vec::new();

        for path in &self.shortest_paths {
            let mut path_value = Vec::new();
            path_value.push(Value::Vertex(path.src.clone()));

            for step in &path.steps {
                path_value.push(Value::Edge((*step.edge).clone()));
                path_value.push(Value::Vertex(step.dst.clone()));
            }

            path_values.push(Value::Path(path.clone()));
        }

        ExecutionResult::Values(path_values)
    }
}

impl<S: StorageEngine + Send + 'static> InputExecutor<S> for ShortestPathExecutor<S> {
    fn set_input(&mut self, input: ExecutorEnum<S>) {
        self.input_executor = Some(Box::new(input));
    }

    fn get_input(&self) -> Option<&ExecutorEnum<S>> {
        self.input_executor.as_deref()
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> Executor<S> for ShortestPathExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        let input_result = if let Some(ref mut input_exec) = self.input_executor {
            input_exec.execute().await?
        } else {
            ExecutionResult::Vertices(Vec::new())
        };

        let (start_nodes, end_nodes) = match input_result {
            ExecutionResult::Vertices(vertices) => {
                if vertices.len() >= 2 {
                    (vec![(*vertices[0].vid).clone()], vec![(*vertices[1].vid).clone()])
                } else {
                    (Vec::new(), Vec::new())
                }
            }
            ExecutionResult::Edges(edges) => {
                if !edges.is_empty() {
                    let first_edge = &edges[0];
                    (vec![(*first_edge.src).clone()], vec![(*first_edge.dst).clone()])
                } else {
                    (Vec::new(), Vec::new())
                }
            }
            ExecutionResult::Values(values) => {
                if values.len() >= 2 {
                    (vec![values[0].clone()], vec![values[1].clone()])
                } else {
                    (Vec::new(), Vec::new())
                }
            }
            _ => (Vec::new(), Vec::new()),
        };

        let start_nodes = if start_nodes.is_empty() {
            self.start_vertex_ids.clone()
        } else {
            start_nodes
        };

        let end_nodes = if end_nodes.is_empty() {
            self.end_vertex_ids.clone()
        } else {
            end_nodes
        };

        if start_nodes.is_empty() || end_nodes.is_empty() {
            return Ok(ExecutionResult::Values(Vec::new()));
        }

        self.start_vertex_ids = start_nodes;
        self.end_vertex_ids = end_nodes;

        self.compute_shortest_paths().await.map_err(DBError::from)?;

        Ok(self.build_result())
    }

    fn open(&mut self) -> DBResult<()> {
        self.shortest_paths.clear();
        self.termination_map.clear();
        self.nodes_visited = 0;
        self.edges_traversed = 0;
        self.max_depth_reached = 0;

        if let Some(ref mut input_exec) = self.input_executor {
            input_exec.open()?;
        }
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        self.shortest_paths.clear();
        self.termination_map.clear();

        if let Some(ref mut input_exec) = self.input_executor {
            input_exec.close()?;
        }
        Ok(())
    }

    fn is_open(&self) -> bool {
        self.base.is_open()
    }

    fn id(&self) -> i64 {
        self.base.id
    }

    fn name(&self) -> &str {
        &self.base.name
    }

    fn description(&self) -> &str {
        &self.base.description
    }

    fn stats(&self) -> &crate::query::executor::traits::ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::traits::ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageEngine + Send> HasStorage<S> for ShortestPathExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.storage.as_ref().expect("ShortestPathExecutor storage should be set")
    }
}

pub struct MultiShortestPathExecutor<S: StorageEngine + Send + 'static> {
    base: BaseExecutor<S>,
    left_start_vertices: Vec<Value>,
    right_target_vertices: Vec<Value>,
    max_steps: usize,
    edge_types: Option<Vec<String>>,
    single_shortest: bool,
    limit: usize,
    input_executor: Option<Box<ExecutorEnum<S>>>,
    left_visited: HashSet<Value>,
    right_visited: HashSet<Value>,
    left_paths: HashMap<Value, Vec<Path>>,
    right_paths: HashMap<Value, Vec<Path>>,
    history_left_paths: HashMap<Value, HashMap<Value, Vec<Path>>>,
    history_right_paths: HashMap<Value, HashMap<Value, Vec<Path>>>,
    current_step: usize,
    result_paths: Vec<Path>,
    nodes_visited: usize,
    edges_traversed: usize,
}

impl<S: StorageEngine> std::fmt::Debug for MultiShortestPathExecutor<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MultiShortestPathExecutor")
            .field("base", &"BaseExecutor")
            .field("left_start_vertices", &self.left_start_vertices)
            .field("right_target_vertices", &self.right_target_vertices)
            .field("max_steps", &self.max_steps)
            .field("single_shortest", &self.single_shortest)
            .field("limit", &self.limit)
            .field("input_executor", &"Option<Box<dyn Executor<S>>>")
            .field("current_step", &self.current_step)
            .finish()
    }
}

impl<S: StorageEngine> MultiShortestPathExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        left_start_vertices: Vec<Value>,
        right_target_vertices: Vec<Value>,
        max_steps: usize,
        edge_types: Option<Vec<String>>,
        single_shortest: bool,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "MultiShortestPathExecutor".to_string(), storage),
            left_start_vertices,
            right_target_vertices,
            max_steps,
            edge_types,
            single_shortest,
            limit: std::usize::MAX,
            input_executor: None,
            left_visited: HashSet::new(),
            right_visited: HashSet::new(),
            left_paths: HashMap::new(),
            right_paths: HashMap::new(),
            history_left_paths: HashMap::new(),
            history_right_paths: HashMap::new(),
            current_step: 1,
            result_paths: Vec::new(),
            nodes_visited: 0,
            edges_traversed: 0,
        }
    }

    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    fn has_duplicate_edges(&self, path: &Path) -> bool {
        let mut edge_set = HashSet::new();
        
        for step in &path.steps {
            let edge = &step.edge;
            let edge_key = format!("{}_{}_{}", edge.src, edge.dst, edge.ranking);
            if !edge_set.insert(edge_key) {
                return true;
            }
        }
        
        false
    }

    fn get_neighbors_with_edges(&self, node_id: &Value) -> Result<Vec<(Value, Edge)>, QueryError> {
        let storage = safe_lock(&*self.get_storage())
            .expect("MultiShortestPathExecutor storage lock should not be poisoned");

        let edges = storage
            .get_node_edges(node_id, EdgeDirection::Both)
            .map_err(|e| QueryError::StorageError(e.to_string()))?;

        let filtered_edges = if let Some(ref edge_types) = self.edge_types {
            edges
                .into_iter()
                .filter(|edge| edge_types.contains(&edge.edge_type))
                .collect()
        } else {
            edges
        };

        Ok(filtered_edges
            .into_iter()
            .map(|edge| (*edge.dst.clone(), edge))
            .collect())
    }

    fn build_left_paths(&mut self) -> Result<(), QueryError> {
        self.left_paths.clear();

        let mut initial_data: Vec<(Value, Path, Value)> = Vec::new();
        for start_vertex in &self.left_start_vertices {
            let storage = safe_lock(&*self.get_storage())
                .expect("MultiShortestPathExecutor storage lock should not be poisoned");

            if let Ok(Some(vertex)) = storage.get_node(start_vertex) {
                let path = Path {
                    src: Box::new(vertex),
                    steps: Vec::new(),
                };
                initial_data.push((start_vertex.clone(), path, start_vertex.clone()));
            }
        }

        for (vid, path, visited_vid) in initial_data {
            self.left_paths.insert(vid, vec![path]);
            self.left_visited.insert(visited_vid);
        }

        let mut new_discoveries: Vec<(Value, Path)> = Vec::new();
        for (current_id, paths) in &self.left_paths {
            let neighbors = self.get_neighbors_with_edges(current_id)?;
            self.edges_traversed += neighbors.len();
            
            for (neighbor_id, edge) in neighbors {
                if self.left_visited.contains(&neighbor_id) {
                    continue;
                }

                self.left_visited.insert(neighbor_id.clone());
                self.nodes_visited += 1;

                let edge_clone = edge.clone();
                for path in paths {
                    let storage = safe_lock(&*self.get_storage())
                        .expect("MultiShortestPathExecutor storage lock should not be poisoned");

                    if let Ok(Some(dst_vertex)) = storage.get_node(&neighbor_id) {
                        let mut new_path = path.clone();
                        new_path.steps.push(Step {
                            dst: Box::new(dst_vertex),
                            edge: Box::new(edge_clone.clone()),
                        });
                        new_discoveries.push((neighbor_id.clone(), new_path));
                    }
                }
            }
        }

        for (vid, path) in new_discoveries {
            self.left_paths
                .entry(vid)
                .or_insert_with(Vec::new)
                .push(path);
        }

        Ok(())
    }

    fn build_right_paths(&mut self) -> Result<(), QueryError> {
        self.right_paths.clear();

        let mut initial_data: Vec<(Value, Path, Value)> = Vec::new();
        for target_vertex in &self.right_target_vertices {
            let storage = safe_lock(&*self.get_storage())
                .expect("MultiShortestPathExecutor storage lock should not be poisoned");

            if let Ok(Some(vertex)) = storage.get_node(target_vertex) {
                let path = Path {
                    src: Box::new(vertex),
                    steps: Vec::new(),
                };
                initial_data.push((target_vertex.clone(), path, target_vertex.clone()));
            }
        }

        for (vid, path, visited_vid) in initial_data {
            self.right_paths.insert(vid, vec![path]);
            self.right_visited.insert(visited_vid);
        }

        let mut new_discoveries: Vec<(Value, Path)> = Vec::new();
        for (current_id, paths) in &self.right_paths {
            let neighbors = self.get_neighbors_with_edges(current_id)?;
            self.edges_traversed += neighbors.len();
            
            for (neighbor_id, edge) in neighbors {
                if self.right_visited.contains(&neighbor_id) {
                    continue;
                }

                self.right_visited.insert(neighbor_id.clone());
                self.nodes_visited += 1;

                let edge_clone = edge.clone();
                for path in paths {
                    let storage = safe_lock(&*self.get_storage())
                        .expect("MultiShortestPathExecutor storage lock should not be poisoned");

                    if let Ok(Some(dst_vertex)) = storage.get_node(&neighbor_id) {
                        let mut new_path = path.clone();
                        new_path.steps.push(Step {
                            dst: Box::new(dst_vertex),
                            edge: Box::new(edge_clone.clone()),
                        });
                        new_discoveries.push((neighbor_id.clone(), new_path));
                    }
                }
            }
        }

        for (vid, path) in new_discoveries {
            self.right_paths
                .entry(vid)
                .or_insert_with(Vec::new)
                .push(path);
        }

        Ok(())
    }

    fn conjunct_paths(&mut self) {
        let meet_points: Vec<(Value, Vec<Path>, Vec<Path>)> = self
            .left_paths
            .iter()
            .filter_map(|(left_vid, left_path_list)| {
                self.right_paths
                    .get(left_vid)
                    .map(|right_path_list| {
                        (
                            left_vid.clone(),
                            left_path_list.clone(),
                            right_path_list.clone(),
                        )
                    })
            })
            .collect();

        for (_meet_vid, left_list, right_list) in meet_points {
            for left_path in left_list {
                for right_path in &right_list {
                    if self.has_duplicate_edges(&left_path) {
                        continue;
                    }
                    if self.has_duplicate_edges(right_path) {
                        continue;
                    }

                    let mut combined_path = left_path.clone();
                    for step in right_path.steps.iter().rev() {
                        combined_path.steps.push(step.clone());
                    }
                    
                    if !self.has_duplicate_edges(&combined_path) {
                        self.result_paths.push(combined_path);
                    }
                }
            }
        }
    }
}

impl<S: StorageEngine + Send + 'static> InputExecutor<S> for MultiShortestPathExecutor<S> {
    fn set_input(&mut self, input: ExecutorEnum<S>) {
        self.input_executor = Some(Box::new(input));
    }

    fn get_input(&self) -> Option<&ExecutorEnum<S>> {
        self.input_executor.as_deref()
    }
}

#[async_trait]
impl<S: StorageEngine + Send + Sync + 'static> Executor<S> for MultiShortestPathExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        if self.left_start_vertices.is_empty() || self.right_target_vertices.is_empty() {
            return Ok(ExecutionResult::Paths(vec![]));
        }

        if self.current_step > self.max_steps {
            return Ok(ExecutionResult::Paths(self.result_paths.clone()));
        }

        let _ = self.build_left_paths();
        let _ = self.build_right_paths();

        self.conjunct_paths();

        self.current_step += 1;

        if self.single_shortest && !self.result_paths.is_empty() {
            self.result_paths.sort_by(|a, b| a.steps.len().cmp(&b.steps.len()));
            self.result_paths.truncate(1);
        }

        if self.result_paths.len() > self.limit {
            self.result_paths.truncate(self.limit);
        }

        Ok(ExecutionResult::Paths(self.result_paths.clone()))
    }

    fn open(&mut self) -> DBResult<()> {
        self.base.open()?;
        self.left_visited.clear();
        self.right_visited.clear();
        self.left_paths.clear();
        self.right_paths.clear();
        self.history_left_paths.clear();
        self.history_right_paths.clear();
        self.result_paths.clear();
        self.current_step = 1;
        self.nodes_visited = 0;
        self.edges_traversed = 0;
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        self.base.close()?;
        self.left_visited.clear();
        self.right_visited.clear();
        self.result_paths.clear();
        Ok(())
    }

    fn is_open(&self) -> bool {
        self.base.is_open()
    }

    fn id(&self) -> i64 {
        self.base.id
    }

    fn name(&self) -> &str {
        &self.base.name
    }

    fn description(&self) -> &str {
        &self.base.description
    }

    fn stats(&self) -> &crate::query::executor::traits::ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::traits::ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageEngine + Send> HasStorage<S> for MultiShortestPathExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.storage.as_ref().expect("MultiShortestPathExecutor storage should be set")
    }
}
