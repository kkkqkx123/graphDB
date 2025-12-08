use std::sync::{Arc, Mutex};
use async_trait::async_trait;
use rand::seq::SliceRandom;
use rand::SeedableRng;

use crate::core::{Value, Vertex, Edge};
use crate::storage::StorageEngine;
use crate::query::QueryError;
use super::base::{Executor, ExecutionResult, ExecutionContext, BaseExecutor, InputExecutor};

// Executor for limiting the number of results
pub struct LimitExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    limit: usize,
    input_executor: Option<Box<dyn Executor<S>>>,
}

impl<S: StorageEngine> LimitExecutor<S> {
    pub fn new(
        id: usize,
        storage: Arc<Mutex<S>>,
        limit: usize,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "LimitExecutor".to_string(), storage),
            limit,
            input_executor: None,
        }
    }
}

impl<S: StorageEngine> InputExecutor<S> for LimitExecutor<S> {
    fn set_input(&mut self, input: Box<dyn Executor<S>>) {
        self.input_executor = Some(input);
    }

    fn get_input(&self) -> Option<&Box<dyn Executor<S>>> {
        self.input_executor.as_ref()
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> Executor<S> for LimitExecutor<S> {
    async fn execute(&mut self) -> Result<ExecutionResult, QueryError> {
        // Execute the input executor first if it exists
        let input_result = if let Some(ref mut input_exec) = self.input_executor {
            input_exec.execute().await?
        } else {
            // If no input executor, return empty result
            ExecutionResult::Vertices(Vec::new())
        };

        // Apply the limit to the result
        let limited_result = match input_result {
            ExecutionResult::Vertices(vertices) => {
                let limited_vertices = vertices.into_iter()
                    .take(self.limit)
                    .collect::<Vec<_>>();
                ExecutionResult::Vertices(limited_vertices)
            }
            ExecutionResult::Edges(edges) => {
                let limited_edges = edges.into_iter()
                    .take(self.limit)
                    .collect::<Vec<_>>();
                ExecutionResult::Edges(limited_edges)
            }
            ExecutionResult::Values(values) => {
                let limited_values = values.into_iter()
                    .take(self.limit)
                    .collect::<Vec<_>>();
                ExecutionResult::Values(limited_values)
            }
            ExecutionResult::Count(count) => {
                let limited_count = std::cmp::min(count, self.limit);
                ExecutionResult::Count(limited_count)
            }
            ExecutionResult::Success => ExecutionResult::Success,
        };

        Ok(limited_result)
    }

    fn open(&mut self) -> Result<(), QueryError> {
        // Initialize any resources needed for limiting
        if let Some(ref mut input_exec) = self.input_executor {
            input_exec.open()?;
        }
        Ok(())
    }

    fn close(&mut self) -> Result<(), QueryError> {
        // Clean up any resources
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

// Executor for skipping a number of results (OFFSET)
pub struct OffsetExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    offset: usize,
    input_executor: Option<Box<dyn Executor<S>>>,
}

impl<S: StorageEngine> OffsetExecutor<S> {
    pub fn new(
        id: usize,
        storage: Arc<Mutex<S>>,
        offset: usize,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "OffsetExecutor".to_string(), storage),
            offset,
            input_executor: None,
        }
    }
}

impl<S: StorageEngine> InputExecutor<S> for OffsetExecutor<S> {
    fn set_input(&mut self, input: Box<dyn Executor<S>>) {
        self.input_executor = Some(input);
    }

    fn get_input(&self) -> Option<&Box<dyn Executor<S>>> {
        self.input_executor.as_ref()
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> Executor<S> for OffsetExecutor<S> {
    async fn execute(&mut self) -> Result<ExecutionResult, QueryError> {
        // Execute the input executor first if it exists
        let input_result = if let Some(ref mut input_exec) = self.input_executor {
            input_exec.execute().await?
        } else {
            // If no input executor, return empty result
            ExecutionResult::Vertices(Vec::new())
        };

        // Apply the offset to the result
        let offset_result = match input_result {
            ExecutionResult::Vertices(vertices) => {
                let offset_vertices = vertices.into_iter()
                    .skip(self.offset)
                    .collect::<Vec<_>>();
                ExecutionResult::Vertices(offset_vertices)
            }
            ExecutionResult::Edges(edges) => {
                let offset_edges = edges.into_iter()
                    .skip(self.offset)
                    .collect::<Vec<_>>();
                ExecutionResult::Edges(offset_edges)
            }
            ExecutionResult::Values(values) => {
                let offset_values = values.into_iter()
                    .skip(self.offset)
                    .collect::<Vec<_>>();
                ExecutionResult::Values(offset_values)
            }
            ExecutionResult::Count(count) => {
                let offset_count = if count > self.offset { count - self.offset } else { 0 };
                ExecutionResult::Count(offset_count)
            }
            ExecutionResult::Success => ExecutionResult::Success,
        };

        Ok(offset_result)
    }

    fn open(&mut self) -> Result<(), QueryError> {
        // Initialize any resources needed for offsetting
        if let Some(ref mut input_exec) = self.input_executor {
            input_exec.open()?;
        }
        Ok(())
    }

    fn close(&mut self) -> Result<(), QueryError> {
        // Clean up any resources
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

// Executor for removing duplicate results
pub struct DistinctExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    input_executor: Option<Box<dyn Executor<S>>>,
}

impl<S: StorageEngine> DistinctExecutor<S> {
    pub fn new(
        id: usize,
        storage: Arc<Mutex<S>>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "DistinctExecutor".to_string(), storage),
            input_executor: None,
        }
    }
}

impl<S: StorageEngine> InputExecutor<S> for DistinctExecutor<S> {
    fn set_input(&mut self, input: Box<dyn Executor<S>>) {
        self.input_executor = Some(input);
    }

    fn get_input(&self) -> Option<&Box<dyn Executor<S>>> {
        self.input_executor.as_ref()
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> Executor<S> for DistinctExecutor<S> {
    async fn execute(&mut self) -> Result<ExecutionResult, QueryError> {
        // Execute the input executor first if it exists
        let input_result = if let Some(ref mut input_exec) = self.input_executor {
            input_exec.execute().await?
        } else {
            // If no input executor, return empty result
            ExecutionResult::Vertices(Vec::new())
        };

        // Remove duplicates from the result
        let distinct_result = match input_result {
            ExecutionResult::Vertices(vertices) => {
                // In a real implementation, we would use a more sophisticated comparison
                // For now, we'll use vertex ID as the uniqueness criteria
                let mut seen_ids = std::collections::HashSet::new();
                let distinct_vertices = vertices.into_iter()
                    .filter(|v| seen_ids.insert(v.vid.clone()))
                    .collect::<Vec<_>>();
                ExecutionResult::Vertices(distinct_vertices)
            }
            ExecutionResult::Edges(edges) => {
                // In a real implementation, we would use a more sophisticated comparison
                // For now, we'll use a combination of edge fields as the uniqueness criteria
                let mut seen_edges = std::collections::HashSet::new();
                let distinct_edges = edges.into_iter()
                    .filter(|e| {
                        let edge_key = (e.src.clone(), e.dst.clone(), e.edge_type.clone(), e.ranking);
                        seen_edges.insert(edge_key)
                    })
                    .collect::<Vec<_>>();
                ExecutionResult::Edges(distinct_edges)
            }
            ExecutionResult::Values(values) => {
                // Remove duplicate values
                let mut seen_values = std::collections::HashSet::new();
                let distinct_values = values.into_iter()
                    .filter(|v| seen_values.insert(v.clone()))
                    .collect::<Vec<_>>();
                ExecutionResult::Values(distinct_values)
            }
            ExecutionResult::Count(count) => {
                // Count is already distinct
                ExecutionResult::Count(count)
            }
            ExecutionResult::Success => ExecutionResult::Success,
        };

        Ok(distinct_result)
    }

    fn open(&mut self) -> Result<(), QueryError> {
        // Initialize any resources needed for distinct operation
        if let Some(ref mut input_exec) = self.input_executor {
            input_exec.open()?;
        }
        Ok(())
    }

    fn close(&mut self) -> Result<(), QueryError> {
        // Clean up any resources
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

// Executor for sampling results
pub struct SampleExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    sample_size: usize,
    seed: Option<u64>, // For reproducible sampling
    input_executor: Option<Box<dyn Executor<S>>>,
}

impl<S: StorageEngine> SampleExecutor<S> {
    pub fn new(
        id: usize,
        storage: Arc<Mutex<S>>,
        sample_size: usize,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "SampleExecutor".to_string(), storage),
            sample_size,
            seed: None,
            input_executor: None,
        }
    }

    pub fn with_seed(
        id: usize,
        storage: Arc<Mutex<S>>,
        sample_size: usize,
        seed: u64,
    ) -> Self {
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
        // Execute the input executor first if it exists
        let input_result = if let Some(ref mut input_exec) = self.input_executor {
            input_exec.execute().await?
        } else {
            // If no input executor, return empty result
            ExecutionResult::Vertices(Vec::new())
        };

        // Sample from the result
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
                
                let sampled_vertices = indices.into_iter()
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
                
                let sampled_edges = indices.into_iter()
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
                
                let sampled_values = indices.into_iter()
                    .take(sample_size)
                    .map(|i| values[i].clone())
                    .collect::<Vec<_>>();
                
                ExecutionResult::Values(sampled_values)
            }
            ExecutionResult::Count(count) => {
                // For count, we can't really sample, so just return the count
                ExecutionResult::Count(count)
            }
            ExecutionResult::Success => ExecutionResult::Success,
        };

        Ok(sampled_result)
    }

    fn open(&mut self) -> Result<(), QueryError> {
        // Initialize any resources needed for sampling
        if let Some(ref mut input_exec) = self.input_executor {
            input_exec.open()?;
        }
        Ok(())
    }

    fn close(&mut self) -> Result<(), QueryError> {
        // Clean up any resources
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