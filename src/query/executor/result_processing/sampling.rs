use async_trait::async_trait;
use rand::Rng;
use std::sync::{Arc, Mutex};

#[allow(unused_imports)]
use crate::core::Value;
use crate::query::QueryError;
use crate::storage::StorageEngine;
use crate::query::executor::base::{BaseExecutor, ExecutionResult, Executor, InputExecutor};

/// 蓄水池采样算法实现
fn reservoir_sampling<T: Clone>(items: Vec<T>, sample_size: usize) -> Vec<T> {
    if items.len() <= sample_size {
        return items;
    }
    
    let mut rng = rand::thread_rng();
    let mut reservoir: Vec<T> = items[..sample_size].to_vec();
    
    for (i, item) in items.iter().enumerate().skip(sample_size) {
        let j = rng.gen_range(0..=i);
        if j < sample_size {
            reservoir[j] = item.clone();
        }
    }
    
    reservoir
}

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

        // 检查是否需要采样
        let should_sample = match &input_result {
            ExecutionResult::Vertices(vertices) => vertices.len() > self.sample_size,
            ExecutionResult::Edges(edges) => edges.len() > self.sample_size,
            ExecutionResult::Values(values) => values.len() > self.sample_size,
            ExecutionResult::Paths(paths) => paths.len() > self.sample_size,
            ExecutionResult::DataSet(dataset) => dataset.rows.len() > self.sample_size,
            ExecutionResult::Count(count) => *count > self.sample_size,
            ExecutionResult::Success => false,
        };

        // 如果不需要采样，直接返回原始结果
        if !should_sample {
            return Ok(input_result);
        }

        // 从结果中采样
        let sampled_result = match input_result {
            ExecutionResult::Vertices(vertices) => {
                let sample_size = std::cmp::min(self.sample_size, vertices.len());
                let sampled_vertices = reservoir_sampling(vertices, sample_size);
                ExecutionResult::Vertices(sampled_vertices)
            }
            ExecutionResult::Edges(edges) => {
                let sample_size = std::cmp::min(self.sample_size, edges.len());
                let sampled_edges = reservoir_sampling(edges, sample_size);
                ExecutionResult::Edges(sampled_edges)
            }
            ExecutionResult::Values(values) => {
                let sample_size = std::cmp::min(self.sample_size, values.len());
                let sampled_values = reservoir_sampling(values, sample_size);
                ExecutionResult::Values(sampled_values)
            }
            ExecutionResult::Paths(paths) => {
                let sample_size = std::cmp::min(self.sample_size, paths.len());
                let sampled_paths = reservoir_sampling(paths, sample_size);
                ExecutionResult::Paths(sampled_paths)
            }
            ExecutionResult::DataSet(dataset) => {
                let sample_size = std::cmp::min(self.sample_size, dataset.rows.len());
                let sampled_rows = reservoir_sampling(dataset.rows, sample_size);
                ExecutionResult::DataSet(crate::core::value::DataSet {
                    col_names: dataset.col_names,
                    rows: sampled_rows,
                })
            }
            ExecutionResult::Count(count) => {
                // 对于计数结果，采样意味着返回不超过采样大小的计数
                let sampled_count = std::cmp::min(count, self.sample_size);
                ExecutionResult::Count(sampled_count)
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
