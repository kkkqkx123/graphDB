//! 最短路径执行器
//!
//! 使用算法模块中的具体算法实现最短路径查找
//! 负责执行器的生命周期管理和算法调度

use std::sync::Arc;

use crate::core::{Path, Value};
use crate::core::error::DBResult;
use crate::query::executor::base::{BaseExecutor, EdgeDirection, InputExecutor};
use crate::query::executor::executor_enum::ExecutorEnum;
use crate::query::executor::traits::{ExecutionResult, Executor, HasStorage};
use crate::query::QueryError;
use crate::storage::StorageClient;
use parking_lot::Mutex;

// 引入算法模块
use super::algorithms::{
    AStar, AlgorithmStats, BidirectionalBFS, Dijkstra, EdgeWeightConfig, ShortestPathAlgorithm,
    ShortestPathAlgorithmType,
};

/// 最短路径执行器
///
/// 负责管理最短路径查询的执行生命周期，并调用具体的算法实现
pub struct ShortestPathExecutor<S: StorageClient + Send + 'static> {
    base: BaseExecutor<S>,
    start_vertex_ids: Vec<Value>,
    end_vertex_ids: Vec<Value>,
    pub edge_direction: EdgeDirection,
    pub edge_types: Option<Vec<String>>,
    pub max_depth: Option<usize>,
    algorithm_type: ShortestPathAlgorithmType,
    weight_config: EdgeWeightConfig,
    input_executor: Option<Box<ExecutorEnum<S>>>,
    pub shortest_paths: Vec<Path>,
    pub nodes_visited: usize,
    pub edges_traversed: usize,
    pub execution_time_ms: u64,
    pub max_depth_reached: usize,
    pub single_shortest: bool,
    pub limit: usize,
}

impl<S: StorageClient> std::fmt::Debug for ShortestPathExecutor<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ShortestPathExecutor")
            .field("base", &"BaseExecutor")
            .field("start_vertex_ids", &self.start_vertex_ids)
            .field("end_vertex_ids", &self.end_vertex_ids)
            .field("edge_direction", &self.edge_direction)
            .field("edge_types", &self.edge_types)
            .field("max_depth", &self.max_depth)
            .field("algorithm", &self.algorithm_type)
            .field("single_shortest", &self.single_shortest)
            .field("limit", &self.limit)
            .field("shortest_paths", &self.shortest_paths)
            .field("nodes_visited", &self.nodes_visited)
            .field("edges_traversed", &self.edges_traversed)
            .finish()
    }
}

impl<S: StorageClient> ShortestPathExecutor<S> {
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
            algorithm_type: algorithm,
            weight_config: EdgeWeightConfig::Unweighted,
            input_executor: None,
            shortest_paths: Vec::new(),
            nodes_visited: 0,
            edges_traversed: 0,
            execution_time_ms: 0,
            max_depth_reached: 0,
            single_shortest: false,
            limit: std::usize::MAX,
        }
    }

    pub fn with_limits(mut self, single_shortest: bool, limit: usize) -> Self {
        self.single_shortest = single_shortest;
        self.limit = limit;
        self
    }

    pub fn with_weight_config(mut self, config: EdgeWeightConfig) -> Self {
        self.weight_config = config;
        self
    }

    pub fn get_algorithm(&self) -> ShortestPathAlgorithmType {
        self.algorithm_type.clone()
    }

    pub fn set_algorithm(&mut self, algorithm: ShortestPathAlgorithmType) {
        self.algorithm_type = algorithm;
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

    /// 执行最短路径算法
    fn execute_algorithm(&mut self) -> Result<Vec<Path>, QueryError> {
        let storage = self.base.storage.clone().expect("存储未初始化");
        let edge_types = self.edge_types.as_deref();

        match self.algorithm_type {
            ShortestPathAlgorithmType::BFS => {
                let mut algorithm = BidirectionalBFS::new(storage)
                    .with_edge_direction(self.edge_direction);
                let paths = algorithm.find_paths(
                    &self.start_vertex_ids,
                    &self.end_vertex_ids,
                    edge_types,
                    self.max_depth,
                    self.single_shortest,
                    self.limit,
                )?;
                self.update_stats(algorithm.stats());
                Ok(paths)
            }
            ShortestPathAlgorithmType::Dijkstra => {
                let mut algorithm = Dijkstra::new(storage)
                    .with_edge_direction(self.edge_direction)
                    .with_weight_config(self.weight_config.clone());
                let paths = algorithm.find_paths(
                    &self.start_vertex_ids,
                    &self.end_vertex_ids,
                    edge_types,
                    self.max_depth,
                    self.single_shortest,
                    self.limit,
                )?;
                self.update_stats(algorithm.stats());
                Ok(paths)
            }
            ShortestPathAlgorithmType::AStar => {
                let mut algorithm = AStar::new(storage)
                    .with_edge_direction(self.edge_direction);
                let paths = algorithm.find_paths(
                    &self.start_vertex_ids,
                    &self.end_vertex_ids,
                    edge_types,
                    self.max_depth,
                    self.single_shortest,
                    self.limit,
                )?;
                self.update_stats(algorithm.stats());
                Ok(paths)
            }
        }
    }

    /// 更新执行器统计信息
    fn update_stats(&mut self, algorithm_stats: &AlgorithmStats) {
        self.nodes_visited = algorithm_stats.nodes_visited;
        self.edges_traversed = algorithm_stats.edges_traversed;
        self.execution_time_ms = algorithm_stats.execution_time_ms;
    }
}

impl<S: StorageClient + Send + 'static> InputExecutor<S> for ShortestPathExecutor<S> {
    fn set_input(&mut self, input: ExecutorEnum<S>) {
        self.input_executor = Some(Box::new(input));
    }

    fn get_input(&self) -> Option<&ExecutorEnum<S>> {
        self.input_executor.as_deref()
    }
}

impl<S: StorageClient + Send + Sync + 'static> Executor<S> for ShortestPathExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        let start_time = std::time::Instant::now();

        let paths = self.execute_algorithm()?;

        self.execution_time_ms = start_time.elapsed().as_millis() as u64;
        self.shortest_paths = paths.clone();

        // 更新最大深度
        for path in &paths {
            if path.steps.len() > self.max_depth_reached {
                self.max_depth_reached = path.steps.len();
            }
        }

        Ok(ExecutionResult::Paths(paths))
    }

    fn open(&mut self) -> DBResult<()> {
        self.base.open()?;
        self.shortest_paths.clear();
        self.nodes_visited = 0;
        self.edges_traversed = 0;
        self.execution_time_ms = 0;
        self.max_depth_reached = 0;
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        self.base.close()?;
        self.shortest_paths.clear();
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

impl<S: StorageClient + Send> HasStorage<S> for ShortestPathExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.storage.as_ref().expect("存储未初始化")
    }
}
