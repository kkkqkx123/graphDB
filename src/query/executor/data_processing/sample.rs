//! SampleExecutor - 采样执行器
//!
//! 实现数据采样功能，支持多种采样算法

use async_trait::async_trait;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::sync::{Arc, Mutex};

use crate::core::{DataSet, Edge, Value, Vertex};
use crate::graph::expression::{EvalContext, ExpressionV1 as Expression, ExpressionEvaluator};
use crate::query::executor::base::{BaseExecutor, InputExecutor};
use crate::query::executor::traits::{ExecutorCore, ExecutorLifecycle, ExecutorMetadata, ExecutionResult, DBResult};
use crate::query::QueryError;
use crate::storage::Row;
use crate::storage::StorageEngine;

/// 采样方法
#[derive(Debug, Clone, PartialEq)]
pub enum SampleMethod {
    /// 随机采样
    Random,
    /// 水库采样（适用于流式数据）
    Reservoir,
    /// 系统采样（等间隔采样）
    Systematic,
    /// 分层采样（需要指定分层键）
    Stratified(String),
}

/// SampleExecutor - 采样执行器
///
/// 实现数据采样功能，支持多种采样算法
#[derive(Debug)]
pub struct SampleExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    input_executor: Option<Box<dyn crate::query::executor::traits::Executor<S>>>,
    count_expr: Expression, // 采样数量表达式
    method: SampleMethod,   // 采样方法
    seed: Option<u64>,      // 随机种子（用于可重现的采样）
    evaluator: ExpressionEvaluator,
}

impl<S: StorageEngine> SampleExecutor<S> {
    pub fn new(
        id: usize,
        storage: Arc<Mutex<S>>,
        count_expr: Expression,
        method: SampleMethod,
        seed: Option<u64>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "SampleExecutor".to_string(), storage),
            input_executor: None,
            count_expr,
            method,
            seed,
            evaluator: ExpressionEvaluator,
        }
    }

    /// 执行采样操作
    async fn execute_sample(
        &mut self,
        input: ExecutionResult,
    ) -> Result<ExecutionResult, QueryError> {
        // 创建评估上下文
        let context = EvalContext::new();

        // 评估采样数量
        let count_value = self
            .evaluator
            .evaluate(&self.count_expr, &context)
            .map_err(|e| QueryError::ExpressionError(e.to_string()))?;

        let count = self.extract_count(&count_value)?;

        match input {
            ExecutionResult::Values(values) => {
                let sampled_values = self.sample_values(values, count).await?;
                Ok(ExecutionResult::Values(sampled_values))
            }
            ExecutionResult::Vertices(vertices) => {
                let sampled_vertices = self.sample_vertices(vertices, count).await?;
                Ok(ExecutionResult::Vertices(sampled_vertices))
            }
            ExecutionResult::Edges(edges) => {
                let sampled_edges = self.sample_edges(edges, count).await?;
                Ok(ExecutionResult::Edges(sampled_edges))
            }
            ExecutionResult::DataSet(dataset) => {
                let sampled_dataset = self.sample_dataset(dataset, count).await?;
                Ok(ExecutionResult::DataSet(sampled_dataset))
            }
            _ => Ok(input),
        }
    }

    /// 从值中提取采样数量
    fn extract_count(&self, value: &Value) -> Result<usize, QueryError> {
        match value {
            Value::Int(n) => {
                if *n < 0 {
                    return Err(QueryError::ExecutionError(
                        "Sample count cannot be negative".to_string(),
                    ));
                }
                Ok(*n as usize)
            }
            Value::Float(f) => {
                if *f < 0.0 {
                    return Err(QueryError::ExecutionError(
                        "Sample count cannot be negative".to_string(),
                    ));
                }
                Ok(*f as usize)
            }
            _ => Err(QueryError::ExecutionError(
                "Sample count must be a number".to_string(),
            )),
        }
    }

    /// 值采样
    async fn sample_values(
        &self,
        values: Vec<Value>,
        count: usize,
    ) -> Result<Vec<Value>, QueryError> {
        if count >= values.len() {
            return Ok(values);
        }

        match &self.method {
            SampleMethod::Random => self.random_sample(values, count).await,
            SampleMethod::Reservoir => self.reservoir_sample(values, count).await,
            SampleMethod::Systematic => self.systematic_sample(values, count).await,
            SampleMethod::Stratified(_) => {
                // 对于简单值，分层采样退化为随机采样
                self.random_sample(values, count).await
            }
        }
    }

    /// 顶点采样
    async fn sample_vertices(
        &self,
        vertices: Vec<Vertex>,
        count: usize,
    ) -> Result<Vec<Vertex>, QueryError> {
        if count >= vertices.len() {
            return Ok(vertices);
        }

        match &self.method {
            SampleMethod::Random => self.random_sample(vertices, count).await,
            SampleMethod::Reservoir => self.reservoir_sample(vertices, count).await,
            SampleMethod::Systematic => self.systematic_sample(vertices, count).await,
            SampleMethod::Stratified(strata_key) => {
                self.stratified_sample_vertices(vertices, count, strata_key)
                    .await
            }
        }
    }

    /// 边采样
    async fn sample_edges(&self, edges: Vec<Edge>, count: usize) -> Result<Vec<Edge>, QueryError> {
        if count >= edges.len() {
            return Ok(edges);
        }

        match &self.method {
            SampleMethod::Random => self.random_sample(edges, count).await,
            SampleMethod::Reservoir => self.reservoir_sample(edges, count).await,
            SampleMethod::Systematic => self.systematic_sample(edges, count).await,
            SampleMethod::Stratified(strata_key) => {
                self.stratified_sample_edges(edges, count, strata_key).await
            }
        }
    }

    /// 数据集采样
    async fn sample_dataset(&self, dataset: DataSet, count: usize) -> Result<DataSet, QueryError> {
        if count >= dataset.rows.len() {
            return Ok(dataset);
        }

        let sampled_rows = match &self.method {
            SampleMethod::Random => self.random_sample(dataset.rows, count).await?,
            SampleMethod::Reservoir => self.reservoir_sample(dataset.rows, count).await?,
            SampleMethod::Systematic => self.systematic_sample(dataset.rows, count).await?,
            SampleMethod::Stratified(strata_key) => {
                self.stratified_sample_rows(dataset.rows, &dataset.col_names, count, strata_key)
                    .await?
            }
        };

        Ok(DataSet {
            col_names: dataset.col_names,
            rows: sampled_rows,
        })
    }

    /// 随机采样
    async fn random_sample<T>(&self, items: Vec<T>, count: usize) -> Result<Vec<T>, QueryError>
    where
        T: Clone,
    {
        let mut rng = match self.seed {
            Some(seed) => StdRng::seed_from_u64(seed),
            None => StdRng::from_entropy(),
        };

        let mut indices: Vec<usize> = (0..items.len()).collect();

        // Fisher-Yates 洗牌算法
        for i in (1..items.len()).rev() {
            let j = rng.gen_range(0..=i);
            indices.swap(i, j);
        }

        let sampled_indices = indices.into_iter().take(count).collect::<Vec<_>>();
        let mut sampled_items = Vec::with_capacity(count);

        for &index in &sampled_indices {
            sampled_items.push(items[index].clone());
        }

        Ok(sampled_items)
    }

    /// 水库采样
    async fn reservoir_sample<T>(&self, items: Vec<T>, count: usize) -> Result<Vec<T>, QueryError>
    where
        T: Clone,
    {
        let mut rng = match self.seed {
            Some(seed) => StdRng::seed_from_u64(seed),
            None => StdRng::from_entropy(),
        };

        let mut reservoir = Vec::with_capacity(count);

        // 填充水库
        for (i, item) in items.iter().enumerate() {
            if i < count {
                reservoir.push(item.clone());
            } else {
                // 以 count/i 的概率替换
                let j = rng.gen_range(0..=i);
                if j < count {
                    reservoir[j] = item.clone();
                }
            }
        }

        Ok(reservoir)
    }

    /// 系统采样
    async fn systematic_sample<T>(&self, items: Vec<T>, count: usize) -> Result<Vec<T>, QueryError>
    where
        T: Clone,
    {
        if items.is_empty() {
            return Ok(Vec::new());
        }

        let step = items.len() / count;
        let mut sampled_items = Vec::with_capacity(count);

        // 随机起始点
        let mut rng = match self.seed {
            Some(seed) => StdRng::seed_from_u64(seed),
            None => StdRng::from_entropy(),
        };
        let start = rng.gen_range(0..step);

        for i in 0..count {
            let index = start + i * step;
            if index < items.len() {
                sampled_items.push(items[index].clone());
            }
        }

        Ok(sampled_items)
    }

    /// 分层顶点采样
    async fn stratified_sample_vertices(
        &self,
        vertices: Vec<Vertex>,
        count: usize,
        strata_key: &str,
    ) -> Result<Vec<Vertex>, QueryError> {
        // 按分层键分组
        let mut strata = std::collections::HashMap::new();

        for vertex in vertices {
            let stratum_value = self.get_vertex_stratum_value(&vertex, strata_key);
            strata
                .entry(stratum_value)
                .or_insert_with(Vec::new)
                .push(vertex);
        }

        // 计算每层的采样数量
        let total_vertices = strata.values().map(|v| v.len()).sum::<usize>();
        let mut sampled_vertices = Vec::new();

        for (stratum_value, stratum_vertices) in strata {
            let stratum_count = (stratum_vertices.len() * count) / total_vertices;
            if stratum_count > 0 {
                let stratum_sampled = self.random_sample(stratum_vertices, stratum_count).await?;
                sampled_vertices.extend(stratum_sampled);
            }
        }

        Ok(sampled_vertices)
    }

    /// 分层边采样
    async fn stratified_sample_edges(
        &self,
        edges: Vec<Edge>,
        count: usize,
        strata_key: &str,
    ) -> Result<Vec<Edge>, QueryError> {
        // 按分层键分组
        let mut strata = std::collections::HashMap::new();

        for edge in edges {
            let stratum_value = self.get_edge_stratum_value(&edge, strata_key);
            strata
                .entry(stratum_value)
                .or_insert_with(Vec::new)
                .push(edge);
        }

        // 计算每层的采样数量
        let total_edges = strata.values().map(|v| v.len()).sum::<usize>();
        let mut sampled_edges = Vec::new();

        for (stratum_value, stratum_edges) in strata {
            let stratum_count = (stratum_edges.len() * count) / total_edges;
            if stratum_count > 0 {
                let stratum_sampled = self.random_sample(stratum_edges, stratum_count).await?;
                sampled_edges.extend(stratum_sampled);
            }
        }

        Ok(sampled_edges)
    }

    /// 分层行采样
    async fn stratified_sample_rows(
        &self,
        rows: Vec<Row>,
        col_names: &[String],
        count: usize,
        strata_key: &str,
    ) -> Result<Vec<Row>, QueryError> {
        // 找到分层键的列索引
        let strata_index = col_names.iter().position(|name| name == strata_key);

        let strata_index = match strata_index {
            Some(index) => index,
            None => {
                // 如果找不到分层键，退化为随机采样
                return self.random_sample(rows, count).await;
            }
        };

        // 按分层键分组
        let mut strata = std::collections::HashMap::new();

        for row in rows {
            let stratum_value = if let Some(value) = row.get(strata_index) {
                format!("{:?}", value)
            } else {
                "NULL".to_string()
            };
            strata
                .entry(stratum_value)
                .or_insert_with(Vec::new)
                .push(row);
        }

        // 计算每层的采样数量
        let total_rows = strata.values().map(|v| v.len()).sum::<usize>();
        let mut sampled_rows = Vec::new();

        for (stratum_value, stratum_rows) in strata {
            let stratum_count = (stratum_rows.len() * count) / total_rows;
            if stratum_count > 0 {
                let stratum_sampled = self.random_sample(stratum_rows, stratum_count).await?;
                sampled_rows.extend(stratum_sampled);
            }
        }

        Ok(sampled_rows)
    }

    /// 获取顶点的分层值
    fn get_vertex_stratum_value(&self, vertex: &Vertex, strata_key: &str) -> String {
        if strata_key == "id" {
            format!("{:?}", vertex.vid)
        } else {
            // 在顶点的标签中查找属性
            for tag in &vertex.tags {
                if let Some(value) = tag.properties.get(strata_key) {
                    return format!("{:?}", value);
                }
            }
            "NULL".to_string()
        }
    }

    /// 获取边的分层值
    fn get_edge_stratum_value(&self, edge: &Edge, strata_key: &str) -> String {
        match strata_key {
            "src" => format!("{:?}", edge.src),
            "dst" => format!("{:?}", edge.dst),
            "type" => edge.edge_type.clone(),
            "rank" => format!("{:?}", edge.ranking),
            _ => {
                if let Some(value) = edge.props.get(strata_key) {
                    format!("{:?}", value)
                } else {
                    "NULL".to_string()
                }
            }
        }
    }
}

impl<S: StorageEngine> InputExecutor<S> for SampleExecutor<S> {
    fn set_input(&mut self, input: Box<dyn crate::query::executor::traits::Executor<S>>) {
        self.input_executor = Some(input);
    }

    fn get_input(&self) -> Option<&Box<dyn crate::query::executor::traits::Executor<S>>> {
        self.input_executor.as_ref()
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> ExecutorCore for SampleExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        // 首先执行输入执行器（如果存在）
        let input_result = if let Some(ref mut input_exec) = self.input_executor {
            input_exec.execute().await?
        } else {
            // 如果没有输入执行器，返回空结果
            ExecutionResult::Values(Vec::new())
        };

        // 执行采样操作
        self.execute_sample(input_result).await.map_err(|e| crate::core::error::DBError::Query(crate::core::error::QueryError::ExecutionError(e.to_string())))
    }
}

impl<S: StorageEngine + Send + 'static> ExecutorLifecycle for SampleExecutor<S> {
    fn open(&mut self) -> DBResult<()> {
        // 初始化采样所需的任何资源
        if let Some(ref mut input_exec) = self.input_executor {
            input_exec.open()?;
        }
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        // 清理资源
        if let Some(ref mut input_exec) = self.input_executor {
            input_exec.close()?;
        }
        Ok(())
    }

    fn is_open(&self) -> bool {
        self.base.is_open()
    }
}

impl<S: StorageEngine + Send + 'static> ExecutorMetadata for SampleExecutor<S> {
    fn id(&self) -> usize {
        self.base.id()
    }

    fn name(&self) -> &str {
        self.base.name()
    }

    fn description(&self) -> &str {
        self.base.description()
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> crate::query::executor::traits::Executor<S> for SampleExecutor<S> {
    fn storage(&self) -> &S {
        self.base.storage()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::value::NullType;
    use std::sync::{Arc, Mutex};

    // 模拟存储引擎
    struct MockStorage;

    impl StorageEngine for MockStorage {
        fn insert_node(
            &mut self,
            _vertex: crate::core::vertex_edge_path::Vertex,
        ) -> Result<crate::core::Value, crate::storage::StorageError> {
            Ok(crate::core::Value::Null(NullType::NaN))
        }

        fn get_node(
            &self,
            _id: &crate::core::Value,
        ) -> Result<Option<crate::core::vertex_edge_path::Vertex>, crate::storage::StorageError>
        {
            Ok(None)
        }

        fn update_node(
            &mut self,
            _vertex: crate::core::vertex_edge_path::Vertex,
        ) -> Result<(), crate::storage::StorageError> {
            Ok(())
        }

        fn delete_node(
            &mut self,
            _id: &crate::core::Value,
        ) -> Result<(), crate::storage::StorageError> {
            Ok(())
        }

        fn insert_edge(
            &mut self,
            _edge: crate::core::vertex_edge_path::Edge,
        ) -> Result<(), crate::storage::StorageError> {
            Ok(())
        }

        fn get_edge(
            &self,
            _src: &crate::core::Value,
            _dst: &crate::core::Value,
            _edge_type: &str,
        ) -> Result<Option<crate::core::vertex_edge_path::Edge>, crate::storage::StorageError>
        {
            Ok(None)
        }

        fn get_node_edges(
            &self,
            _node_id: &crate::core::Value,
            _direction: crate::core::vertex_edge_path::Direction,
        ) -> Result<Vec<crate::core::vertex_edge_path::Edge>, crate::storage::StorageError>
        {
            Ok(Vec::new())
        }

        fn delete_edge(
            &mut self,
            _src: &crate::core::Value,
            _dst: &crate::core::Value,
            _edge_type: &str,
        ) -> Result<(), crate::storage::StorageError> {
            Ok(())
        }

        fn begin_transaction(&mut self) -> Result<u64, crate::storage::StorageError> {
            Ok(1)
        }

        fn commit_transaction(&mut self, _tx_id: u64) -> Result<(), crate::storage::StorageError> {
            Ok(())
        }

        fn rollback_transaction(
            &mut self,
            _tx_id: u64,
        ) -> Result<(), crate::storage::StorageError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_sample_executor_random() {
        let storage = Arc::new(Mutex::new(MockStorage));

        let count_expr = Expression::Constant(Value::Int(3));

        let mut executor = SampleExecutor::new(
            1,
            storage,
            count_expr,
            SampleMethod::Random,
            Some(42), // 固定种子以确保可重现性
        );

        // 设置测试数据
        let test_data = vec![
            Value::Int(1),
            Value::Int(2),
            Value::Int(3),
            Value::Int(4),
            Value::Int(5),
        ];

        let input_result = ExecutionResult::Values(test_data);

        // 创建模拟输入执行器
        struct MockInputExecutor {
            result: ExecutionResult,
        }

        #[async_trait]
        impl<S: StorageEngine + Send + 'static> ExecutorCore for MockInputExecutor {
            async fn execute(&mut self) -> DBResult<ExecutionResult> {
                Ok(self.result.clone())
            }
        }

        impl<S: StorageEngine + Send + 'static> ExecutorLifecycle for MockInputExecutor {
            fn open(&mut self) -> DBResult<()> {
                Ok(())
            }
            fn close(&mut self) -> DBResult<()> {
                Ok(())
            }
            fn is_open(&self) -> bool {
                true
            }
        }

        impl<S: StorageEngine + Send + 'static> ExecutorMetadata for MockInputExecutor {
            fn id(&self) -> usize {
                0
            }
            fn name(&self) -> &str {
                "MockInputExecutor"
            }
            fn description(&self) -> &str {
                "Mock input executor for testing"
            }
        }

        #[async_trait]
        impl<S: StorageEngine + Send + 'static> crate::query::executor::traits::Executor<S> for MockInputExecutor {
            fn storage(&self) -> &S {
                panic!("MockInputExecutor does not have storage")
            }
        }

        let mut input_executor = MockInputExecutor {
            result: input_result,
        };

        executor.set_input(Box::new(input_executor));

        // 执行采样
        let result = executor.execute().await.unwrap();

        // 验证结果
        match result {
            ExecutionResult::Values(values) => {
                assert_eq!(values.len(), 3); // 应该采样3个值
                                             // 验证所有值都是原始数据中的值
                for value in &values {
                    assert!(matches!(value, Value::Int(1..=5)));
                }
            }
            _ => panic!("Expected Values result"),
        }
    }

    #[tokio::test]
    async fn test_sample_executor_systematic() {
        let storage = Arc::new(Mutex::new(MockStorage));

        let count_expr = Expression::Constant(Value::Int(2));

        let mut executor = SampleExecutor::new(
            1,
            storage,
            count_expr,
            SampleMethod::Systematic,
            Some(42), // 固定种子以确保可重现性
        );

        // 设置测试数据
        let test_data = vec![
            Value::Int(1),
            Value::Int(2),
            Value::Int(3),
            Value::Int(4),
            Value::Int(5),
            Value::Int(6),
        ];

        let input_result = ExecutionResult::Values(test_data);

        // 创建模拟输入执行器
        struct MockInputExecutor {
            result: ExecutionResult,
        }

        #[async_trait]
        impl<S: StorageEngine + Send + 'static> ExecutorCore for MockInputExecutor {
            async fn execute(&mut self) -> DBResult<ExecutionResult> {
                Ok(self.result.clone())
            }
        }

        impl<S: StorageEngine + Send + 'static> ExecutorLifecycle for MockInputExecutor {
            fn open(&mut self) -> DBResult<()> {
                Ok(())
            }
            fn close(&mut self) -> DBResult<()> {
                Ok(())
            }
            fn is_open(&self) -> bool {
                true
            }
        }

        impl<S: StorageEngine + Send + 'static> ExecutorMetadata for MockInputExecutor {
            fn id(&self) -> usize {
                0
            }
            fn name(&self) -> &str {
                "MockInputExecutor"
            }
            fn description(&self) -> &str {
                "Mock input executor for testing"
            }
        }

        #[async_trait]
        impl<S: StorageEngine + Send + 'static> crate::query::executor::traits::Executor<S> for MockInputExecutor {
            fn storage(&self) -> &S {
                panic!("MockInputExecutor does not have storage")
            }
        }

        let mut input_executor = MockInputExecutor {
            result: input_result,
        };

        executor.set_input(Box::new(input_executor));

        // 执行采样
        let result = executor.execute().await.unwrap();

        // 验证结果
        match result {
            ExecutionResult::Values(values) => {
                assert_eq!(values.len(), 2); // 应该采样2个值
                                             // 系统采样应该有规律性
                for value in &values {
                    assert!(matches!(value, Value::Int(1..=6)));
                }
            }
            _ => panic!("Expected Values result"),
        }
    }
}
