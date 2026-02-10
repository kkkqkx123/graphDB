//! 采样执行器
//!
//! 实现对查询结果的随机采样功能，支持多种采样方法

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use crate::core::error::{DBError, DBResult};
use crate::core::value::DataSet;
use crate::query::executor::base::InputExecutor;
use crate::query::executor::executor_enum::ExecutorEnum;
use crate::query::executor::result_processing::traits::{
    BaseResultProcessor, ResultProcessor, ResultProcessorContext,
};
use crate::query::executor::traits::{ExecutionResult, Executor};
use crate::storage::StorageClient;

/// 采样方法
#[derive(Debug, Clone, PartialEq)]
pub enum SampleMethod {
    /// 随机采样
    Random,
    /// 蓄水池采样（适用于流式数据）
    Reservoir,
    /// 系统采样（固定间隔）
    System,
}

/// SampleExecutor - 采样执行器
///
/// 实现对查询结果的随机采样功能
pub struct SampleExecutor<S: StorageClient + Send + 'static> {
    /// 基础处理器
    base: BaseResultProcessor<S>,
    /// 采样方法
    method: SampleMethod,
    /// 采样数量
    count: usize,
    /// 随机种子
    seed: Option<u64>,
    /// 输入执行器
    input_executor: Option<Box<ExecutorEnum<S>>>,
}

impl<S: StorageClient + Send + 'static> SampleExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        method: SampleMethod,
        count: usize,
        seed: Option<u64>,
    ) -> Self {
        let base = BaseResultProcessor::new(
            id,
            "SampleExecutor".to_string(),
            "Samples query results using various sampling methods".to_string(),
            storage,
        );

        Self {
            base,
            method,
            count,
            seed,
            input_executor: None,
        }
    }

    fn process_input(&mut self) -> DBResult<ExecutionResult> {
        if let Some(ref mut input_exec) = self.input_executor {
            let input_result = input_exec.execute()?;
            self.sample_input(input_result)
        } else if let Some(input) = &self.base.input {
            self.sample_input(input.clone())
        } else {
            Err(DBError::Query(
                crate::core::error::QueryError::ExecutionError(
                    "Sample executor requires input".to_string(),
                ),
            ))
        }
    }

    /// 对输入执行采样
    fn sample_input(&self, input: ExecutionResult) -> DBResult<ExecutionResult> {
        match input {
            ExecutionResult::DataSet(dataset) => {
                let sampled_dataset = self.sample_dataset(dataset)?;
                Ok(ExecutionResult::DataSet(sampled_dataset))
            }
            ExecutionResult::Values(values) => {
                let sampled_values = self.sample_values(values)?;
                Ok(ExecutionResult::Values(sampled_values))
            }
            ExecutionResult::Vertices(vertices) => {
                let sampled_vertices = self.sample_vertices(vertices)?;
                Ok(ExecutionResult::Vertices(sampled_vertices))
            }
            ExecutionResult::Edges(edges) => {
                let sampled_edges = self.sample_edges(edges)?;
                Ok(ExecutionResult::Edges(sampled_edges))
            }
            _ => Ok(input),
        }
    }

    /// 对数据集执行采样
    fn sample_dataset(&self, dataset: DataSet) -> DBResult<DataSet> {
        match self.method {
            SampleMethod::Random => self.random_sample_dataset(dataset),
            SampleMethod::Reservoir => self.reservoir_sample_dataset(dataset),
            SampleMethod::System => self.system_sample_dataset(dataset),
        }
    }

    /// 随机采样数据集
    fn random_sample_dataset(&self, mut dataset: DataSet) -> DBResult<DataSet> {
        if dataset.rows.len() <= self.count {
            return Ok(dataset);
        }

        let mut rng = self.create_rng();
        let mut sampled_indices = HashSet::new();

        // 随机选择不重复的索引
        while sampled_indices.len() < self.count {
            let index = rng.gen_range(0..dataset.rows.len());
            sampled_indices.insert(index);
        }

        // 提取采样的行
        let sampled_rows: Vec<_> = sampled_indices
            .into_iter()
            .map(|i| dataset.rows[i].clone())
            .collect();

        dataset.rows = sampled_rows;
        Ok(dataset)
    }

    /// 蓄水池采样数据集
    fn reservoir_sample_dataset(&self, mut dataset: DataSet) -> DBResult<DataSet> {
        if dataset.rows.len() <= self.count {
            return Ok(dataset);
        }

        let mut rng = self.create_rng();
        let mut reservoir: Vec<_> = dataset.rows.iter().take(self.count).cloned().collect();

        // 处理剩余元素
        for (i, row) in dataset.rows.iter().enumerate().skip(self.count) {
            let j = rng.gen_range(0..=i);
            if j < self.count {
                reservoir[j] = row.clone();
            }
        }

        dataset.rows = reservoir;
        Ok(dataset)
    }

    /// 系统采样数据集
    fn system_sample_dataset(&self, mut dataset: DataSet) -> DBResult<DataSet> {
        if dataset.rows.len() <= self.count {
            return Ok(dataset);
        }

        let step = dataset.rows.len() / self.count;
        let mut sampled_rows = Vec::new();

        for (i, row) in dataset.rows.iter().enumerate() {
            if i % step == 0 && sampled_rows.len() < self.count {
                sampled_rows.push(row.clone());
            }
        }

        dataset.rows = sampled_rows;
        Ok(dataset)
    }

    /// 对值列表执行采样
    fn sample_values(&self, values: Vec<crate::core::Value>) -> DBResult<Vec<crate::core::Value>> {
        if values.len() <= self.count {
            return Ok(values);
        }

        match self.method {
            SampleMethod::Random => self.random_sample_values(values),
            SampleMethod::Reservoir => self.reservoir_sample_values(values),
            SampleMethod::System => self.system_sample_values(values),
        }
    }

    /// 随机采样值列表
    fn random_sample_values(
        &self,
        values: Vec<crate::core::Value>,
    ) -> DBResult<Vec<crate::core::Value>> {
        let mut rng = self.create_rng();
        let mut sampled_indices = HashSet::new();

        while sampled_indices.len() < self.count {
            let index = rng.gen_range(0..values.len());
            sampled_indices.insert(index);
        }

        let sampled_values: Vec<_> = sampled_indices
            .into_iter()
            .map(|i| values[i].clone())
            .collect();

        Ok(sampled_values)
    }

    /// 蓄水池采样值列表
    fn reservoir_sample_values(
        &self,
        values: Vec<crate::core::Value>,
    ) -> DBResult<Vec<crate::core::Value>> {
        let mut rng = self.create_rng();
        let mut reservoir: Vec<_> = values.iter().take(self.count).cloned().collect();

        for (i, value) in values.iter().enumerate().skip(self.count) {
            let j = rng.gen_range(0..=i);
            if j < self.count {
                reservoir[j] = value.clone();
            }
        }

        Ok(reservoir)
    }

    /// 系统采样值列表
    fn system_sample_values(
        &self,
        values: Vec<crate::core::Value>,
    ) -> DBResult<Vec<crate::core::Value>> {
        let step = values.len() / self.count;
        let mut sampled_values = Vec::new();

        for (i, value) in values.iter().enumerate() {
            if i % step == 0 && sampled_values.len() < self.count {
                sampled_values.push(value.clone());
            }
        }

        Ok(sampled_values)
    }

    /// 对顶点列表执行采样
    fn sample_vertices(
        &self,
        vertices: Vec<crate::core::Vertex>,
    ) -> DBResult<Vec<crate::core::Vertex>> {
        if vertices.len() <= self.count {
            return Ok(vertices);
        }

        match self.method {
            SampleMethod::Random => self.random_sample_vertices(vertices),
            SampleMethod::Reservoir => self.reservoir_sample_vertices(vertices),
            SampleMethod::System => self.system_sample_vertices(vertices),
        }
    }

    /// 随机采样顶点列表
    fn random_sample_vertices(
        &self,
        vertices: Vec<crate::core::Vertex>,
    ) -> DBResult<Vec<crate::core::Vertex>> {
        let mut rng = self.create_rng();
        let mut sampled_indices = HashSet::new();

        while sampled_indices.len() < self.count {
            let index = rng.gen_range(0..vertices.len());
            sampled_indices.insert(index);
        }

        let sampled_vertices: Vec<_> = sampled_indices
            .into_iter()
            .map(|i| vertices[i].clone())
            .collect();

        Ok(sampled_vertices)
    }

    /// 蓄水池采样顶点列表
    fn reservoir_sample_vertices(
        &self,
        vertices: Vec<crate::core::Vertex>,
    ) -> DBResult<Vec<crate::core::Vertex>> {
        let mut rng = self.create_rng();
        let mut reservoir: Vec<_> = vertices.iter().take(self.count).cloned().collect();

        for (i, vertex) in vertices.iter().enumerate().skip(self.count) {
            let j = rng.gen_range(0..=i);
            if j < self.count {
                reservoir[j] = vertex.clone();
            }
        }

        Ok(reservoir)
    }

    /// 系统采样顶点列表
    fn system_sample_vertices(
        &self,
        vertices: Vec<crate::core::Vertex>,
    ) -> DBResult<Vec<crate::core::Vertex>> {
        let step = vertices.len() / self.count;
        let mut sampled_vertices = Vec::new();

        for (i, vertex) in vertices.iter().enumerate() {
            if i % step == 0 && sampled_vertices.len() < self.count {
                sampled_vertices.push(vertex.clone());
            }
        }

        Ok(sampled_vertices)
    }

    /// 对边列表执行采样
    fn sample_edges(&self, edges: Vec<crate::core::Edge>) -> DBResult<Vec<crate::core::Edge>> {
        if edges.len() <= self.count {
            return Ok(edges);
        }

        match self.method {
            SampleMethod::Random => self.random_sample_edges(edges),
            SampleMethod::Reservoir => self.reservoir_sample_edges(edges),
            SampleMethod::System => self.system_sample_edges(edges),
        }
    }

    /// 随机采样边列表
    fn random_sample_edges(
        &self,
        edges: Vec<crate::core::Edge>,
    ) -> DBResult<Vec<crate::core::Edge>> {
        let mut rng = self.create_rng();
        let mut sampled_indices = HashSet::new();

        while sampled_indices.len() < self.count {
            let index = rng.gen_range(0..edges.len());
            sampled_indices.insert(index);
        }

        let sampled_edges: Vec<_> = sampled_indices
            .into_iter()
            .map(|i| edges[i].clone())
            .collect();

        Ok(sampled_edges)
    }

    /// 蓄水池采样边列表
    fn reservoir_sample_edges(
        &self,
        edges: Vec<crate::core::Edge>,
    ) -> DBResult<Vec<crate::core::Edge>> {
        let mut rng = self.create_rng();
        let mut reservoir: Vec<_> = edges.iter().take(self.count).cloned().collect();

        for (i, edge) in edges.iter().enumerate().skip(self.count) {
            let j = rng.gen_range(0..=i);
            if j < self.count {
                reservoir[j] = edge.clone();
            }
        }

        Ok(reservoir)
    }

    /// 系统采样边列表
    fn system_sample_edges(
        &self,
        edges: Vec<crate::core::Edge>,
    ) -> DBResult<Vec<crate::core::Edge>> {
        let step = edges.len() / self.count;
        let mut sampled_edges = Vec::new();

        for (i, edge) in edges.iter().enumerate() {
            if i % step == 0 && sampled_edges.len() < self.count {
                sampled_edges.push(edge.clone());
            }
        }

        Ok(sampled_edges)
    }

    /// 创建随机数生成器
    fn create_rng(&self) -> StdRng {
        match self.seed {
            Some(seed) => StdRng::seed_from_u64(seed),
            None => StdRng::from_entropy(),
        }
    }
}

impl<S: StorageClient + Send + 'static> ResultProcessor<S> for SampleExecutor<S> {
    fn process(&mut self, input: ExecutionResult) -> DBResult<ExecutionResult> {
        if self.input_executor.is_none() && self.base.input.is_none() {
            <Self as ResultProcessor<S>>::set_input(self, input.clone());
        }
        self.process_input()
    }

    fn set_input(&mut self, input: ExecutionResult) {
        self.base.input = Some(input);
    }

    fn get_input(&self) -> Option<&ExecutionResult> {
        self.base.input.as_ref()
    }

    fn context(&self) -> &ResultProcessorContext {
        &self.base.context
    }

    fn set_context(&mut self, context: ResultProcessorContext) {
        self.base.context = context;
    }

    fn memory_usage(&self) -> usize {
        self.base.memory_usage
    }

    fn reset(&mut self) {
        self.base.reset_state();
    }
}

impl<S: StorageClient + Send + Sync + 'static> Executor<S> for SampleExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        let input_result = if let Some(ref mut input_exec) = self.input_executor {
            input_exec.execute()?
        } else {
            self.base
                .input
                .clone()
                .unwrap_or(ExecutionResult::DataSet(crate::core::value::DataSet::new()))
        };

        self.process(input_result)
    }

    fn open(&mut self) -> DBResult<()> {
        if let Some(ref mut input_exec) = self.input_executor {
            input_exec.open()?;
        }
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        if let Some(ref mut input_exec) = self.input_executor {
            input_exec.close()?;
        }
        Ok(())
    }

    fn is_open(&self) -> bool {
        self.base.id > 0
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

impl<S: StorageClient + Send + 'static> InputExecutor<S> for SampleExecutor<S> {
    fn set_input(&mut self, input: ExecutorEnum<S>) {
        self.input_executor = Some(Box::new(input));
    }

    fn get_input(&self) -> Option<&ExecutorEnum<S>> {
        self.input_executor.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::test_mock::MockStorage;

    #[tokio::test]
    async fn test_sample_executor_random() {
        let storage = Arc::new(Mutex::new(MockStorage));

        // 创建测试数据
        let values: Vec<crate::core::Value> =
            (1..=100).map(|i| crate::core::Value::Int(i)).collect();

        // 创建采样执行器 (随机采样10个值，使用固定种子保证可重现)
        let mut executor = SampleExecutor::new(1, storage, SampleMethod::Random, 10, Some(42));

        // 设置输入数据
        <SampleExecutor<MockStorage> as ResultProcessor<MockStorage>>::set_input(
            &mut executor,
            ExecutionResult::Values(values),
        );

        // 执行采样
        let result = executor
            .process(ExecutionResult::DataSet(DataSet::new()))
            .expect("Failed to process sample");

        // 验证结果
        match result {
            ExecutionResult::Values(sampled_values) => {
                assert_eq!(sampled_values.len(), 10);
                // 验证所有值都是有效的
                for value in &sampled_values {
                    match value {
                        crate::core::Value::Int(i) => {
                            assert!(*i >= 1 && *i <= 100);
                        }
                        _ => panic!("Expected Int values"),
                    }
                }
            }
            _ => panic!("Expected Values result"),
        }
    }

    #[tokio::test]
    async fn test_sample_executor_reservoir() {
        let storage = Arc::new(Mutex::new(MockStorage));

        // 创建测试数据
        let values: Vec<crate::core::Value> =
            (1..=100).map(|i| crate::core::Value::Int(i)).collect();

        // 创建采样执行器 (蓄水池采样5个值)
        let mut executor = SampleExecutor::new(1, storage, SampleMethod::Reservoir, 5, Some(123));

        // 设置输入数据
        <SampleExecutor<MockStorage> as ResultProcessor<MockStorage>>::set_input(
            &mut executor,
            ExecutionResult::Values(values),
        );

        // 执行采样
        let result = executor
            .process(ExecutionResult::DataSet(DataSet::new()))
            .expect("Failed to process sample");

        // 验证结果
        match result {
            ExecutionResult::Values(sampled_values) => {
                assert_eq!(sampled_values.len(), 5);
            }
            _ => panic!("Expected Values result"),
        }
    }
}
