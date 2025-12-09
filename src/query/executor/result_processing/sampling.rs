use async_trait::async_trait;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use std::sync::{Arc, Mutex};

use crate::core::{Edge, Value, Vertex};
use crate::query::QueryError;
use crate::storage::StorageEngine;
use crate::query::executor::base::{BaseExecutor, ExecutionContext, ExecutionResult, Executor, InputExecutor};

/// SampleExecutor - 采样执行器
///
/// 从结果集中随机采样指定数量的项目，支持指定随机种子以保证重现性
pub struct SampleExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    sample_size: usize,
    seed: Option<u64>, // 用于可重现的采样
    input_executor: Option<Box<dyn Executor<S>>>,
}

impl<S: StorageEngine> SampleExecutor<S> {
    pub fn new(id: usize, storage: Arc<Mutex<S>>, sample_size: usize) -> Self {
        Self {
            base: BaseExecutor::new(id, "SampleExecutor".to_string(), storage),
            sample_size,
            seed: None,
            input_executor: None,
        }
    }

    pub fn with_seed(id: usize, storage: Arc<Mutex<S>>, sample_size: usize, seed: u64) -> Self {
        Self {
            base: BaseExecutor::new(id, "SampleExecutor".to_string(), storage),
            sample_size,
            seed: Some(seed),
            input_executor: None,
        }
    }
}

impl<S: StorageEngine> InputExecutor<S> for SampleExecutor<S> {
    fn set_input(&mut self, input: Box<dyn Executor<S>>) {
        self.input_executor = Some(input);
    }

    fn get_input(&self) -> Option<&Box<dyn Executor<S>>> {
        self.input_executor.as_ref()
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> Executor<S> for SampleExecutor<S> {
    async fn execute(&mut self) -> Result<ExecutionResult, QueryError> {
        // 首先执行输入执行器（如果存在）
        let input_result = if let Some(ref mut input_exec) = self.input_executor {
            input_exec.execute().await?
        } else {
            // 如果没有输入执行器，返回空结果
            ExecutionResult::Vertices(Vec::new())
        };

        // 从结果中采样
        let sampled_result = match input_result {
            ExecutionResult::Vertices(vertices) => {
                let sample_size = std::cmp::min(self.sample_size, vertices.len());
                let mut rng = if let Some(seed) = self.seed {
                    rand::rngs::StdRng::seed_from_u64(seed)
                } else {
                    rand::rngs::StdRng::from_entropy()
                };

                let mut indices: Vec<usize> = (0..vertices.len()).collect();
                indices.shuffle(&mut rng);

                let sampled_vertices = indices
                    .into_iter()
                    .take(sample_size)
                    .map(|i| vertices[i].clone())
                    .collect::<Vec<_>>();

                ExecutionResult::Vertices(sampled_vertices)
            }
            ExecutionResult::Edges(edges) => {
                let sample_size = std::cmp::min(self.sample_size, edges.len());
                let mut rng = if let Some(seed) = self.seed {
                    rand::rngs::StdRng::seed_from_u64(seed)
                } else {
                    rand::rngs::StdRng::from_entropy()
                };

                let mut indices: Vec<usize> = (0..edges.len()).collect();
                indices.shuffle(&mut rng);

                let sampled_edges = indices
                    .into_iter()
                    .take(sample_size)
                    .map(|i| edges[i].clone())
                    .collect::<Vec<_>>();

                ExecutionResult::Edges(sampled_edges)
            }
            ExecutionResult::Values(values) => {
                let sample_size = std::cmp::min(self.sample_size, values.len());
                let mut rng = if let Some(seed) = self.seed {
                    rand::rngs::StdRng::seed_from_u64(seed)
                } else {
                    rand::rngs::StdRng::from_entropy()
                };

                let mut indices: Vec<usize> = (0..values.len()).collect();
                indices.shuffle(&mut rng);

                let sampled_values = indices
                    .into_iter()
                    .take(sample_size)
                    .map(|i| values[i].clone())
                    .collect::<Vec<_>>();

                ExecutionResult::Values(sampled_values)
            }
            ExecutionResult::Count(count) => {
                // 对于计数，我们无法真正采样，所以只返回计数
                ExecutionResult::Count(count)
            }
            ExecutionResult::Success => ExecutionResult::Success,
        };

        Ok(sampled_result)
    }

    fn open(&mut self) -> Result<(), QueryError> {
        // 初始化采样所需的任何资源
        if let Some(ref mut input_exec) = self.input_executor {
            input_exec.open()?;
        }
        Ok(())
    }

    fn close(&mut self) -> Result<(), QueryError> {
        // 清理资源
        if let Some(ref mut input_exec) = self.input_executor {
            input_exec.close()?;
        }
        Ok(())
    }

    fn id(&self) -> usize {
        self.base.id
    }

    fn name(&self) -> &str {
        &self.base.name
    }
}
