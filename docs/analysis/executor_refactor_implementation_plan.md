# 查询执行器重构实施计划

## 🎯 重构目标

1. **安全性**：消除递归风险，确保系统稳定
2. **性能**：提升50%+执行性能，减少30%+内存使用
3. **可维护性**：简化架构，降低复杂度
4. **完整性**：实现完整的查询执行器生态

## 📋 重构原则

1. **渐进式重构**：避免一次性大规模改动
2. **向后兼容**：确保现有功能正常工作
3. **测试驱动**：每个改动都有完整测试
4. **性能基准**：持续监控性能指标

## 🛠️ 第一阶段：安全修复（第1-2周）

### 1.1 递归检测机制实现

**文件：`src/query/executor/recursion_detector.rs`**

```rust
//! 递归检测器 - 防止执行器循环引用

use std::collections::{HashSet, HashMap};
use crate::core::error::{DBError, DBResult};

/// 递归检测器
#[derive(Debug, Clone)]
pub struct RecursionDetector {
    max_depth: usize,
    visited_stack: Vec<i64>,
    visited_set: HashSet<i64>,
    recursion_path: Vec<String>,
}

impl RecursionDetector {
    /// 创建新的递归检测器
    pub fn new(max_depth: usize) -> Self {
        Self {
            max_depth,
            visited_stack: Vec::new(),
            visited_set: HashSet::new(),
            recursion_path: Vec::new(),
        }
    }

    /// 验证执行器是否会导致递归
    pub fn validate_executor(&mut self, executor_id: i64, executor_name: &str) -> DBResult<()> {
        // 检查访问深度
        if self.visited_stack.len() >= self.max_depth {
            return Err(DBError::Query(crate::query::QueryError::ExecutionError(
                format!(
                    "执行器调用深度超过最大限制 {}: 路径 {:?}",
                    self.max_depth,
                    self.get_recursion_path()
                )
            )));
        }

        // 检查循环引用
        if self.visited_set.contains(&executor_id) {
            return Err(DBError::Query(crate::query::QueryError::ExecutionError(
                format!(
                    "检测到执行器循环引用: {} (ID: {}) 在路径 {:?}",
                    executor_name,
                    executor_id,
                    self.get_recursion_path()
                )
            )));
        }

        // 记录访问
        self.visited_stack.push(executor_id);
        self.visited_set.insert(executor_id);
        self.recursion_path.push(format!("{}({})", executor_name, executor_id));

        Ok(())
    }

    /// 离开当前执行器
    pub fn leave_executor(&mut self) {
        if let Some(id) = self.visited_stack.pop() {
            self.visited_set.remove(&id);
        }
        self.recursion_path.pop();
    }

    /// 获取递归路径
    fn get_recursion_path(&self) -> Vec<String> {
        self.recursion_path.clone()
    }

    /// 重置检测器状态
    pub fn reset(&mut self) {
        self.visited_stack.clear();
        self.visited_set.clear();
        self.recursion_path.clear();
    }
}

/// 执行器验证trait
pub trait ExecutorValidator {
    fn validate_no_recursion(&self, detector: &mut RecursionDetector) -> DBResult<()>;
}
```

### 1.2 LoopExecutor安全重构

**文件：`src/query/executor/data_processing/loops.rs`**

```rust
use crate::query::executor::recursion_detector::RecursionDetector;

/// 安全的循环执行器
pub struct SafeLoopExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    body_executor: Option<Box<dyn Executor<S>>>,
    condition: Option<Expression>,
    max_iterations: usize,
    current_iteration: usize,
    recursion_detector: RecursionDetector,
}

impl<S: StorageEngine> SafeLoopExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        max_iterations: usize,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "SafeLoopExecutor".to_string(), storage),
            body_executor: None,
            condition: None,
            max_iterations,
            current_iteration: 0,
            recursion_detector: RecursionDetector::new(max_iterations),
        }
    }

    /// 安全地设置循环体执行器
    pub fn set_body_executor(&mut self, executor: Box<dyn Executor<S>>) -> DBResult<()> {
        // 验证是否会导致递归
        let mut detector = RecursionDetector::new(self.max_iterations);
        detector.validate_executor(executor.id(), executor.name())?;
        
        // 检查是否是自引用
        if executor.id() == self.base.id {
            return Err(DBError::Query(QueryError::ExecutionError(
                "循环执行器不能引用自身".to_string()
            )));
        }

        self.body_executor = Some(executor);
        Ok(())
    }

    /// 安全的循环执行
    async fn execute_loop(&mut self) -> DBResult<ExecutionResult> {
        self.current_iteration = 0;
        
        while self.should_continue_loop()? {
            if self.current_iteration >= self.max_iterations {
                return Err(DBError::Query(QueryError::ExecutionError(
                    format!("循环执行超过最大迭代次数: {}", self.max_iterations)
                )));
            }

            if let Some(ref mut body_executor) = self.body_executor {
                // 验证循环体执行器
                self.recursion_detector.validate_executor(
                    body_executor.id(),
                    body_executor.name()
                )?;

                // 执行循环体
                let result = body_executor.execute().await?;
                
                // 处理执行结果
                self.process_loop_iteration(result)?;
                
                // 离开当前执行器
                self.recursion_detector.leave_executor();
            }

            self.current_iteration += 1;
        }

        Ok(self.build_loop_result())
    }

    fn should_continue_loop(&self) -> DBResult<bool> {
        if self.current_iteration == 0 {
            return Ok(true); // 第一次总是执行
        }

        // 检查条件表达式
        if let Some(ref condition) = self.condition {
            // TODO: 评估条件表达式
            Ok(true) // 临时实现
        } else {
            Ok(false) // 无条件时只执行一次
        }
    }

    fn process_loop_iteration(&mut self, result: ExecutionResult) -> DBResult<()> {
        // 处理循环迭代结果
        // TODO: 根据具体需求实现
        Ok(())
    }

    fn build_loop_result(&self) -> ExecutionResult {
        // 构建循环最终结果
        ExecutionResult::Success
    }
}
```

### 1.3 执行器工厂安全增强

**文件：`src/query/executor/factory.rs`**

```rust
use crate::query::executor::recursion_detector::RecursionDetector;

/// 安全的执行器工厂
#[derive(Debug)]
pub struct SafeExecutorFactory<S: StorageEngine + 'static> {
    storage: Arc<Mutex<S>>,
    recursion_detector: RecursionDetector,
    executor_cache: HashMap<String, Vec<Box<dyn Executor<S>>>>,
}

impl<S: StorageEngine + 'static + std::fmt::Debug> SafeExecutorFactory<S> {
    pub fn new(storage: Arc<Mutex<S>>) -> Self {
        Self {
            storage: storage.clone(),
            recursion_detector: RecursionDetector::new(1000), // 最大深度1000
            executor_cache: HashMap::new(),
        }
    }

    /// 安全地创建执行器
    pub fn create_executor_safe(
        &mut self,
        plan_node: &PlanNodeEnum,
    ) -> Result<Box<dyn Executor<S>>, QueryError> {
        // 1. 验证计划节点安全性
        self.validate_plan_node_safety(plan_node)?;
        
        // 2. 检查执行器缓存
        let node_type = plan_node.type_name();
        if let Some(cache) = self.executor_cache.get_mut(&node_type) {
            if let Some(executor) = cache.pop() {
                return Ok(executor);
            }
        }

        // 3. 创建新的执行器
        let executor = self.create_executor_internal(plan_node)?;
        
        // 4. 验证执行器安全性
        self.validate_executor_safety(&*executor)?;
        
        Ok(executor)
    }

    fn validate_plan_node_safety(&self, plan_node: &PlanNodeEnum) -> Result<(), QueryError> {
        // 验证计划节点参数
        match plan_node {
            PlanNodeEnum::Loop(config) => {
                let max_iter = config.max_iterations.unwrap_or(1000);
                if max_iter > 10000 {
                    return Err(QueryError::ExecutionError(
                        "循环最大迭代次数不能超过10000".to_string()
                    ));
                }
            }
            PlanNodeEnum::Expand(config) => {
                let max_depth = config.max_depth.unwrap_or(10);
                if max_depth > 100 {
                    return Err(QueryError::ExecutionError(
                        "扩展最大深度不能超过100".to_string()
                    ));
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn validate_executor_safety(&mut self, executor: &dyn Executor<S>) -> Result<(), QueryError> {
        // 验证执行器本身的安全性
        self.recursion_detector.validate_executor(
            executor.id(),
            executor.name()
        )?;
        
        // 重置检测器状态
        self.recursion_detector.reset();
        
        Ok(())
    }

    fn create_executor_internal(
        &self,
        plan_node: &PlanNodeEnum,
    ) -> Result<Box<dyn Executor<S>>, QueryError> {
        // 完善的核心执行器创建逻辑
        match plan_node {
            PlanNodeEnum::Start(_) => Ok(Box::new(StartExecutor::new(0, self.storage.clone()))),
            PlanNodeEnum::ScanVertices(config) => {
                Ok(Box::new(ScanVerticesExecutor::new(
                    config.id(),
                    self.storage.clone(),
                    config.space_id(),
                    config.tag_ids().to_vec(),
                    config.props().to_vec(),
                )))
            }
            PlanNodeEnum::Filter(config) => {
                Ok(Box::new(FilterExecutor::new(
                    config.id(),
                    self.storage.clone(),
                    config.filter_expr().clone(),
                )))
            }
            PlanNodeEnum::Project(config) => {
                Ok(Box::new(ProjectExecutor::new(
                    config.id(),
                    self.storage.clone(),
                    config.columns().to_vec(),
                )))
            }
            PlanNodeEnum::Limit(config) => {
                Ok(Box::new(LimitExecutor::new(
                    config.id(),
                    self.storage.clone(),
                    config.limit(),
                    config.offset(),
                )))
            }
            PlanNodeEnum::Sort(config) => {
                Ok(Box::new(SortExecutor::new(
                    config.id(),
                    self.storage.clone(),
                    config.sort_keys().to_vec(),
                    config.limit(),
                )))
            }
            PlanNodeEnum::Aggregate(config) => {
                Ok(Box::new(AggregateExecutor::new(
                    config.id(),
                    self.storage.clone(),
                    config.group_keys().to_vec(),
                    config.aggregate_functions().to_vec(),
                )))
            }
            PlanNodeEnum::Expand(config) => {
                Ok(Box::new(ExpandExecutor::new(
                    config.id(),
                    self.storage.clone(),
                    config.edge_types().to_vec(),
                    config.direction(),
                    config.max_depth(),
                )))
            }
            PlanNodeEnum::Loop(config) => {
                // 创建安全的循环执行器
                let mut loop_executor = SafeLoopExecutor::new(
                    config.id(),
                    self.storage.clone(),
                    config.max_iterations().unwrap_or(1000),
                );
                
                // 注意：循环体执行器需要后续设置
                Ok(Box::new(loop_executor))
            }
            _ => Err(QueryError::ExecutionError(format!(
                "执行器类型待实现: {:?}",
                plan_node.type_name()
            ))),
        }
    }

    /// 回收执行器到缓存
    pub fn recycle_executor(&mut self, executor: Box<dyn Executor<S>>) {
        let executor_type = executor.name().to_string();
        
        if let Some(cache) = self.executor_cache.get_mut(&executor_type) {
            if cache.len() < 100 { // 最大缓存100个同类执行器
                cache.push(executor);
            }
        } else {
            let mut cache = Vec::new();
            cache.push(executor);
            self.executor_cache.insert(executor_type, cache);
        }
    }
}
```

### 1.4 安全测试用例

**文件：`src/query/executor/tests/safety_tests.rs`**

```rust
#[cfg(test)]
mod safety_tests {
    use super::*;
    use crate::query::executor::recursion_detector::RecursionDetector;
    use crate::query::executor::SafeLoopExecutor;
    use std::sync::{Arc, Mutex};

    #[tokio::test]
    async fn test_recursion_detection() {
        let mut detector = RecursionDetector::new(10);
        
        // 正常情况
        assert!(detector.validate_executor(1, "TestExecutor").is_ok());
        assert!(detector.validate_executor(2, "AnotherExecutor").is_ok());
        
        // 循环引用检测
        assert!(detector.validate_executor(1, "TestExecutor").is_err());
    }

    #[tokio::test]
    async fn test_loop_executor_self_reference() {
        let storage = Arc::new(Mutex::new(MockStorage::new()));
        let mut loop_executor = SafeLoopExecutor::new(1, storage.clone(), 100);
        
        // 创建自引用（应该失败）
        let self_reference = Box::new(loop_executor.clone());
        let result = loop_executor.set_body_executor(self_reference);
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("不能引用自身"));
    }

    #[tokio::test]
    async fn test_max_depth_protection() {
        let mut detector = RecursionDetector::new(3);
        
        // 正常深度
        assert!(detector.validate_executor(1, "E1").is_ok());
        assert!(detector.validate_executor(2, "E2").is_ok());
        assert!(detector.validate_executor(3, "E3").is_ok());
        
        // 超过最大深度
        assert!(detector.validate_executor(4, "E4").is_err());
    }

    #[tokio::test]
    async fn test_factory_safety_validation() {
        let storage = Arc::new(Mutex::new(MockStorage::new()));
        let mut factory = SafeExecutorFactory::new(storage);
        
        // 创建危险配置（最大迭代次数过高）
        let dangerous_config = LoopConfig {
            max_iterations: Some(20000), // 超过限制
            ..Default::default()
        };
        
        let plan_node = PlanNodeEnum::Loop(dangerous_config);
        let result = factory.create_executor_safe(&plan_node);
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("不能超过10000"));
    }
}
```

## 🛠️ 第二阶段：架构重构（第3-6周）

### 2.1 统一Executor Trait重构

**文件：`src/query/executor/traits.rs`（重构后）**

```rust
//! 统一的执行器trait定义 - 简化架构

use crate::core::error::{DBError, DBResult};
use crate::storage::StorageEngine;
use async_trait::async_trait;
use std::sync::{Arc, Mutex};

/// 统一的执行器trait - 合并所有功能
#[async_trait]
pub trait Executor<S: StorageEngine>: Send + Sync {
    /// 核心执行方法
    async fn execute(&mut self) -> DBResult<ExecutionResult>;
    
    /// 生命周期管理 - 提供默认实现
    fn open(&mut self) -> DBResult<()> { 
        Ok(()) 
    }
    
    fn close(&mut self) -> DBResult<()> { 
        Ok(()) 
    }
    
    fn is_open(&self) -> bool { 
        true 
    }
    
    /// 元数据信息
    fn id(&self) -> i64;
    fn name(&self) -> &str;
    fn description(&self) -> &str {
        ""
    }
    
    /// 存储访问（可选实现）
    fn storage(&self) -> Option<&Arc<Mutex<S>>> {
        None
    }
    
    /// 输入执行器访问（可选实现）
    fn input(&self) -> Option<&Box<dyn Executor<S>>> {
        None
    }
    
    fn set_input(&mut self, _input: Box<dyn Executor<S>>) {
        // 默认空实现
    }
    
    /// 执行器配置（可选实现）
    fn config(&self) -> Option<&ExecutorConfig> {
        None
    }
    
    /// 执行器统计（可选实现）
    fn statistics(&self) -> Option<&ExecutorStatistics> {
        None
    }
    
    /// 重置执行器状态（可选实现）
    fn reset(&mut self) -> DBResult<()> {
        Ok(())
    }
}

/// 执行器配置
#[derive(Debug, Clone)]
pub struct ExecutorConfig {
    pub max_memory_usage: usize,
    pub timeout_ms: u64,
    pub enable_cache: bool,
    pub parallel_degree: usize,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            max_memory_usage: 1024 * 1024 * 100, // 100MB
            timeout_ms: 30000, // 30秒
            enable_cache: true,
            parallel_degree: num_cpus::get(),
        }
    }
}

/// 执行器统计
#[derive(Debug, Default, Clone)]
pub struct ExecutorStatistics {
    pub execution_count: u64,
    pub total_execution_time_ms: u64,
    pub average_execution_time_ms: u64,
    pub memory_peak_usage: usize,
    pub error_count: u64,
}

impl ExecutorStatistics {
    pub fn record_execution(&mut self, duration_ms: u64, memory_usage: usize) {
        self.execution_count += 1;
        self.total_execution_time_ms += duration_ms;
        self.average_execution_time_ms = self.total_execution_time_ms / self.execution_count;
        self.memory_peak_usage = self.memory_peak_usage.max(memory_usage);
    }
    
    pub fn record_error(&mut self) {
        self.error_count += 1;
    }
}

/// 执行结果类型 - 简化版本
#[derive(Debug, Clone)]
pub enum ExecutionResult {
    /// 成功执行，返回数据
    Values(Vec<crate::core::Value>),
    /// 成功执行，返回顶点数据
    Vertices(Vec<crate::core::Vertex>),
    /// 成功执行，返回边数据
    Edges(Vec<crate::core::Edge>),
    /// 成功执行，返回数据集
    DataSet(crate::core::DataSet),
    /// 成功执行，无数据返回
    Success,
    /// 执行错误
    Error(String),
    /// 返回计数
    Count(usize),
    /// 返回路径
    Paths(Vec<crate::core::vertex_edge_path::Path>),
    /// 返回统计信息
    Statistics(ExecutorStatistics),
}

impl ExecutionResult {
    /// 获取结果中的元素计数
    pub fn count(&self) -> usize {
        match self {
            ExecutionResult::Values(v) => v.len(),
            ExecutionResult::Vertices(v) => v.len(),
            ExecutionResult::Edges(v) => v.len(),
            ExecutionResult::DataSet(ds) => ds.rows.len(),
            ExecutionResult::Count(c) => *c,
            ExecutionResult::Success => 0,
            ExecutionResult::Error(_) => 0,
            ExecutionResult::Paths(p) => p.len(),
            ExecutionResult::Statistics(_) => 0,
        }
    }

    /// 检查结果是否成功
    pub fn is_success(&self) -> bool {
        !matches!(self, ExecutionResult::Error(_))
    }

    /// 获取错误信息
    pub fn error_message(&self) -> Option<&str> {
        match self {
            ExecutionResult::Error(msg) => Some(msg),
            _ => None,
        }
    }
}

/// 基础执行器实现 - 简化版本
#[derive(Debug)]
pub struct BaseExecutor<S: StorageEngine> {
    id: i64,
    name: String,
    description: String,
    storage: Arc<Mutex<S>>,
    config: ExecutorConfig,
    statistics: ExecutorStatistics,
    is_open: bool,
}

impl<S: StorageEngine> BaseExecutor<S> {
    pub fn new(id: i64, name: String, storage: Arc<Mutex<S>>) -> Self {
        Self {
            id,
            name: name.clone(),
            description: String::new(),
            storage,
            config: ExecutorConfig::default(),
            statistics: ExecutorStatistics::default(),
            is_open: false,
        }
    }

    pub fn with_config(mut self, config: ExecutorConfig) -> Self {
        self.config = config;
        self
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = description;
        self
    }

    /// 获取存储引擎
    pub fn storage(&self) -> &Arc<Mutex<S>> {
        &self.storage
    }

    /// 获取配置
    pub fn config(&self) -> &ExecutorConfig {
        &self.config
    }

    /// 获取统计信息
    pub fn statistics(&self) -> &ExecutorStatistics {
        &self.statistics
    }

    /// 获取可变的统计信息
    pub fn statistics_mut(&mut self) -> &mut ExecutorStatistics {
        &mut self.statistics
    }

    /// 记录执行统计
    pub fn record_execution(&mut self, duration_ms: u64, memory_usage: usize) {
        self.statistics.record_execution(duration_ms, memory_usage);
    }

    /// 记录执行错误
    pub fn record_error(&mut self) {
        self.statistics.record_error();
    }
}

// 自动实现Executor trait的宏
#[macro_export]
macro_rules! impl_base_executor {
    ($type:ty, $storage_type:ty) => {
        #[async_trait::async_trait]
        impl Executor<$storage_type> for $type {
            async fn execute(&mut self) -> DBResult<ExecutionResult> {
                self.execute().await
            }

            fn open(&mut self) -> DBResult<()> {
                self.is_open = true;
                Ok(())
            }

            fn close(&mut self) -> DBResult<()> {
                self.is_open = false;
                Ok(())
            }

            fn is_open(&self) -> bool {
                self.is_open
            }

            fn id(&self) -> i64 {
                self.id
            }

            fn name(&self) -> &str {
                &self.name
            }

            fn description(&self) -> &str {
                &self.description
            }

            fn storage(&self) -> Option<&Arc<Mutex<$storage_type>>> {
                Some(&self.storage)
            }

            fn config(&self) -> Option<&ExecutorConfig> {
                Some(&self.config)
            }

            fn statistics(&self) -> Option<&ExecutorStatistics> {
                Some(&self.statistics)
            }
        }
    };
}

/// 执行器构建器模式
pub struct ExecutorBuilder<S: StorageEngine> {
    id: i64,
    name: String,
    description: String,
    storage: Arc<Mutex<S>>,
    config: ExecutorConfig,
}

impl<S: StorageEngine> ExecutorBuilder<S> {
    pub fn new(id: i64, name: String, storage: Arc<Mutex<S>>) -> Self {
        Self {
            id,
            name,
            description: String::new(),
            storage,
            config: ExecutorConfig::default(),
        }
    }

    pub fn description(mut self, desc: String) -> Self {
        self.description = desc;
        self
    }

    pub fn config(mut self, config: ExecutorConfig) -> Self {
        self.config = config;
        self
    }

    pub fn build(self) -> BaseExecutor<S> {
        BaseExecutor {
            id: self.id,
            name: self.name,
            description: self.description,
            storage: self.storage,
            config: self.config,
            statistics: ExecutorStatistics::default(),
            is_open: false,
        }
    }
}
```

### 2.2 核心执行器重构示例

**文件：`src/query/executor/result_processing/filter.rs`（重构后）**

```rust
//! 过滤执行器 - 重构版本

use async_trait::async_trait;
use std::sync::{Arc, Mutex};

use crate::core::error::{DBError, DBResult};
use crate::core::expression::Expression;
use crate::query::executor::traits::{Executor, ExecutionResult, BaseExecutor};
use crate::storage::StorageEngine;

/// 过滤执行器 - 简化实现
#[derive(Debug)]
pub struct FilterExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    filter_expr: Expression,
    input_executor: Option<Box<dyn Executor<S>>>,
}

impl<S: StorageEngine> FilterExecutor<S> {
    pub fn new(id: i64, storage: Arc<Mutex<S>>, filter_expr: Expression) -> Self {
        let base = BaseExecutor::new(id, "FilterExecutor".to_string(), storage)
            .with_description("过滤符合条件的记录".to_string());
            
        Self {
            base,
            filter_expr,
            input_executor: None,
        }
    }

    /// 执行过滤逻辑
    async fn execute_filter(&mut self) -> DBResult<ExecutionResult> {
        // 获取输入数据
        let input_data = if let Some(ref mut input) = self.input_executor {
            input.execute().await?
        } else {
            return Ok(ExecutionResult::Success);
        };

        // 过滤数据
        let filtered_data = match input_data {
            ExecutionResult::Values(values) => {
                let filtered: Vec<_> = values.into_iter()
                    .filter(|value| self.should_include(value))
                    .collect();
                ExecutionResult::Values(filtered)
            }
            ExecutionResult::Vertices(vertices) => {
                let filtered: Vec<_> = vertices.into_iter()
                    .filter(|vertex| self.should_include_vertex(vertex))
                    .collect();
                ExecutionResult::Vertices(filtered)
            }
            ExecutionResult::Edges(edges) => {
                let filtered: Vec<_> = edges.into_iter()
                    .filter(|edge| self.should_include_edge(edge))
                    .collect();
                ExecutionResult::Edges(filtered)
            }
            other => other, // 其他类型直接传递
        };

        Ok(filtered_data)
    }

    fn should_include(&self, value: &crate::core::Value) -> bool {
        // TODO: 实现表达式求值
        // 临时实现：总是包含
        true
    }

    fn should_include_vertex(&self, vertex: &crate::core::Vertex) -> bool {
        // TODO: 实现顶点过滤逻辑
        true
    }

    fn should_include_edge(&self, edge: &crate::core::Edge) -> bool {
        // TODO: 实现边过滤逻辑
        true
    }
}

#[async_trait]
impl<S: StorageEngine> Executor<S> for FilterExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        let start_time = std::time::Instant::now();
        
        // 记录执行开始
        self.base.statistics_mut().execution_count += 1;
        
        // 执行过滤
        let result = self.execute_filter().await;
        
        // 记录执行统计
        let duration = start_time.elapsed().as_millis() as u64;
        self.base.record_execution(duration, 0);
        
        result
    }

    fn id(&self) -> i64 {
        self.base.id()
    }

    fn name(&self) -> &str {
        self.base.name()
    }

    fn description(&self) -> &str {
        self.base.description()
    }

    fn input(&self) -> Option<&Box<dyn Executor<S>>> {
        self.input_executor.as_ref()
    }

    fn set_input(&mut self, input: Box<dyn Executor<S>>) {
        self.input_executor = Some(input);
    }
}

/// 过滤执行器构建器
pub struct FilterExecutorBuilder<S: StorageEngine> {
    id: i64,
    storage: Arc<Mutex<S>>,
    filter_expr: Option<Expression>,
    description: String,
}

impl<S: StorageEngine> FilterExecutorBuilder<S> {
    pub fn new(id: i64, storage: Arc<Mutex<S>>) -> Self {
        Self {
            id,
            storage,
            filter_expr: None,
            description: "FilterExecutor".to_string(),
        }
    }

    pub fn filter_expr(mut self, expr: Expression) -> Self {
        self.filter_expr = Some(expr);
        self
    }

    pub fn description(mut self, desc: String) -> Self {
        self.description = desc;
        self
    }

    pub fn build(self) -> Result<FilterExecutor<S>, String> {
        let filter_expr = self.filter_expr
            .ok_or_else(|| "过滤表达式必须设置".to_string())?;
            
        Ok(FilterExecutor::new(self.id, self.storage, filter_expr))
    }
}
```

## 🛠️ 第三阶段：性能优化（第7-9周）

### 3.1 对象池框架实现

**文件：`src/query/executor/pool.rs`**

```rust
//! 执行器对象池 - 性能优化

use std::collections::HashMap;
use std::sync::{Arc, Mutex, Weak};
use std::any::Any;

use crate::core::error::{DBError, DBResult};
use crate::storage::StorageEngine;

/// 对象池配置
#[derive(Debug, Clone)]
pub struct ObjectPoolConfig {
    pub max_objects_per_type: usize,
    pub cleanup_interval_ms: u64,
    pub enable_statistics: bool,
}

impl Default for ObjectPoolConfig {
    fn default() -> Self {
        Self {
            max_objects_per_type: 100,
            cleanup_interval_ms: 60000, // 1分钟
            enable_statistics: true,
        }
    }
}

/// 对象池统计
#[derive(Debug, Default, Clone)]
pub struct PoolStatistics {
    pub total_created: u64,
    pub total_reused: u64,
    pub total_destroyed: u64,
    pub current_pooled: usize,
    pub hit_rate: f64,
}

/// 执行器对象池
pub struct ExecutorObjectPool<S: StorageEngine> {
    config: ObjectPoolConfig,
    pools: Arc<Mutex<HashMap<String, Vec<Box<dyn Any + Send>>>>>,
    statistics: Arc<Mutex<HashMap<String, PoolStatistics>>>,
    _phantom: std::marker::PhantomData<S>,
}

impl<S: StorageEngine> ExecutorObjectPool<S> {
    pub fn new(config: ObjectPoolConfig) -> Self {
        Self {
            config,
            pools: Arc::new(Mutex::new(HashMap::new())),
            statistics: Arc::new(Mutex::new(HashMap::new())),
            _phantom: std::marker::PhantomData,
        }
    }

    /// 从池中获取执行器
    pub fn acquire<E: 'static + Send>(&self, executor_type: &str) -> Option<Box<E>> {
        let mut pools = self.pools.lock().unwrap();
        
        if let Some(pool) = pools.get_mut(executor_type) {
            // 查找指定类型的执行器
            if let Some(pos) = pool.iter().position(|obj| {
                obj.as_ref().as_any().type_id() == std::any::TypeId::of::<E>()
            }) {
                let obj = pool.remove(pos);
                drop(pools); // 提前释放锁
                
                // 尝试转换为指定类型
                match obj.downcast::<E>() {
                    Ok(executor) => {
                        self.update_statistics(executor_type, true);
                        return Some(executor);
                    }
                    Err(original) => {
                        // 类型不匹配，放回池中
                        let mut pools = self.pools.lock().unwrap();
                        if let Some(pool) = pools.get_mut(executor_type) {
                            pool.push(original);
                        }
                    }
                }
            }
        }
        
        self.update_statistics(executor_type, false);
        None
    }

    /// 将执行器回收到池中
    pub fn release<E: 'static + Send>(
        &self,
        executor_type: &str,
        mut executor: Box<E>,
    ) {
        let mut pools = self.pools.lock().unwrap();
        let pool = pools.entry(executor_type.to_string()).or_insert_with(Vec::new);
        
        // 清理执行器状态
        if let Some(resettable) = executor.as_mut().as_any().downcast_mut::<dyn Resettable>() {
            let _ = resettable.reset();
        }
        
        // 检查池大小限制
        if pool.len() < self.config.max_objects_per_type {
            pool.push(executor);
        } else {
            // 超过限制，直接丢弃
            drop(executor);
            self.increment_destroyed_count(executor_type);
        }
    }

    /// 更新统计信息
    fn update_statistics(&self, executor_type: &str, hit: bool) {
        if !self.config.enable_statistics {
            return;
        }
        
        let mut stats = self.statistics.lock().unwrap();
        let stat = stats.entry(executor_type.to_string()).or_insert_with(|| {
            PoolStatistics {
                total_created: 0,
                total_reused: 0,
                total_destroyed: 0,
                current_pooled: 0,
                hit_rate: 0.0,
            }
        });
        
        if hit {
            stat.total_reused += 1;
        } else {
            stat.total_created += 1;
        }
        
        // 重新计算命中率
        let total_requests = stat.total_created + stat.total_reused;
        if total_requests > 0 {
            stat.hit_rate = (stat.total_reused as f64) / (total_requests as f64);
        }
    }

    fn increment_destroyed_count(&self, executor_type: &str) {
        if !self.config.enable_statistics {
            return;
        }
        
        let mut stats = self.statistics.lock().unwrap();
        if let Some(stat) = stats.get_mut(executor_type) {
            stat.total_destroyed += 1;
        }
    }

    /// 获取统计信息
    pub fn get_statistics(&self) -> HashMap<String, PoolStatistics> {
        self.statistics.lock().unwrap().clone()
    }

    /// 清理过期对象
    pub fn cleanup_expired(&self) {
        let mut pools = self.pools.lock().unwrap();
        
        for (executor_type, pool) in pools.iter_mut() {
            // 保留一半的对象，清理其余的
            let retain_count = pool.len() / 2;
            pool.truncate(retain_count);
            
            if let Some(stat) = self.statistics.lock().unwrap().get_mut(executor_type) {
                stat.current_pooled = pool.len();
            }
        }
    }

    /// 获取当前池化对象数量
    pub fn get_pooled_count(&self) -> usize {
        let pools = self.pools.lock().unwrap();
        pools.values().map(|pool| pool.len()).sum()
    }
}

/// 可重置trait - 支持对象池的对象必须实现
pub trait Resettable {
    fn reset(&mut self) -> DBResult<()>;
}

/// 执行器对象池管理器
pub struct PoolManager<S: StorageEngine> {
    pools: Arc<ExecutorObjectPool<S>>,
    cleanup_handle: Option<tokio::task::JoinHandle<()>>,
}

impl<S: StorageEngine> PoolManager<S> {
    pub fn new(config: ObjectPoolConfig) -> Self {
        let pools = Arc::new(ExecutorObjectPool::new(config.clone()));
        let pools_clone = pools.clone();
        
        // 启动清理任务
        let cleanup_handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                std::time::Duration::from_millis(config.cleanup_interval_ms)
            );
            
            loop {
                interval.tick().await;
                pools_clone.cleanup_expired();
            }
        });
        
        Self {
            pools,
            cleanup_handle: Some(cleanup_handle),
        }
    }

    pub fn get_pool(&self) -> Arc<ExecutorObjectPool<S>> {
        self.pools.clone()
    }

    /// 优雅关闭
    pub async fn shutdown(mut self) {
        if let Some(handle) = self.cleanup_handle.take() {
            handle.abort();
        }
    }
}
```

### 3.2 批处理执行器实现

**文件：`src/query/executor/batch.rs`**

```rust
//! 批处理执行器 - 异步性能优化

use async_trait::async_trait;
use futures::future::join_all;
use std::sync::Arc;
use std::time::Instant;

use crate::core::error::{DBError, DBResult};
use crate::query::executor::traits::{Executor, ExecutionResult};
use crate::storage::StorageEngine;

/// 批处理配置
#[derive(Debug, Clone)]
pub struct BatchConfig {
    pub batch_size: usize,
    pub max_concurrency: usize,
    pub enable_statistics: bool,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            batch_size: 100,
            max_concurrency: num_cpus::get() * 2,
            enable_statistics: true,
        }
    }
}

/// 批处理任务
pub struct BatchTask {
    pub task_id: usize,
    pub input_data: ExecutionResult,
    pub priority: u8,
}

/// 批处理执行器
pub struct BatchExecutor<S: StorageEngine> {
    config: BatchConfig,
    tasks: Vec<BatchTask>,
    results: Vec<Option<ExecutionResult>>,
    statistics: BatchStatistics,
}

#[derive(Debug, Default)]
pub struct BatchStatistics {
    pub total_tasks: usize,
    pub completed_tasks: usize,
    pub failed_tasks: usize,
    pub total_execution_time_ms: u64,
    pub average_task_time_ms: u64,
}

impl<S: StorageEngine> BatchExecutor<S> {
    pub fn new(config: BatchConfig) -> Self {
        Self {
            config,
            tasks: Vec::new(),
            results: Vec::new(),
            statistics: BatchStatistics::default(),
        }
    }

    /// 添加批处理任务
    pub fn add_task(&mut self, task: BatchTask) {
        self.tasks.push(task);
        self.results.push(None);
    }

    /// 执行批处理任务
    pub async fn execute_batch(&mut self) -> Vec<DBResult<ExecutionResult>> {
        let start_time = Instant::now();
        self.statistics.total_tasks = self.tasks.len();

        // 按优先级排序
        self.tasks.sort_by_key(|task| std::cmp::Reverse(task.priority));

        // 分批执行
        let mut all_results = Vec::new();
        
        for chunk in self.tasks.chunks(self.config.batch_size) {
            let chunk_results = self.execute_chunk(chunk).await;
            all_results.extend(chunk_results);
        }

        // 更新统计信息
        self.statistics.total_execution_time_ms = start_time.elapsed().as_millis() as u64;
        if self.statistics.total_tasks > 0 {
            self.statistics.average_task_time_ms = 
                self.statistics.total_execution_time_ms / (self.statistics.total_tasks as u64);
        }

        all_results
    }

    /// 执行一个批次
    async fn execute_chunk(&mut self, chunk: &[BatchTask]) -> Vec<DBResult<ExecutionResult>> {
        let mut futures = Vec::new();

        // 限制并发度
        let semaphore = Arc::new(tokio::sync::Semaphore::new(self.config.max_concurrency));

        for task in chunk {
            let semaphore = semaphore.clone();
            let task_id = task.task_id;
            let input_data = task.input_data.clone();
            
            let future = async move {
                let _permit = semaphore.acquire().await.unwrap();
                
                let start_time = Instant::now();
                
                // 执行具体的批处理逻辑
                let result = self.process_task(task_id, input_data).await;
                
                let execution_time = start_time.elapsed().as_millis() as u64;
                
                (task_id, result, execution_time)
            };
            
            futures.push(future);
        }

        // 等待所有任务完成
        let results = join_all(futures).await;
        
        // 更新统计信息
        for (task_id, result, execution_time) in results {
            match &result {
                Ok(_) => self.statistics.completed_tasks += 1,
                Err(_) => self.statistics.failed_tasks += 1,
            }
            
            // 存储结果
            if let Some(result_slot) = self.results.get_mut(task_id) {
                *result_slot = Some(result.clone().unwrap_or(ExecutionResult::Success));
            }
        }

        // 提取结果
        chunk.iter()
            .map(|task| {
                self.results.get(task.task_id)
                    .and_then(|r| r.clone())
                    .map(Ok)
                    .unwrap_or_else(|| Ok(ExecutionResult::Success))
            })
            .collect()
    }

    /// 处理单个任务
    async fn process_task(
        &self,
        task_id: usize,
        input_data: ExecutionResult,
    ) -> DBResult<ExecutionResult> {
        // TODO: 根据具体业务需求实现
        // 这里只是一个示例实现
        
        match input_data {
            ExecutionResult::Values(values) => {
                // 模拟批处理操作
                let processed_values: Vec<_> = values.into_iter()
                    .filter(|v| self.should_process_value(v))
                    .collect();
                
                Ok(ExecutionResult::Values(processed_values))
            }
            other => Ok(other),
        }
    }

    fn should_process_value(&self, _value: &crate::core::Value) -> bool {
        // TODO: 实现具体的过滤逻辑
        true
    }

    /// 获取统计信息
    pub fn get_statistics(&self) -> &BatchStatistics {
        &self.statistics
    }

    /// 重置批处理执行器
    pub fn reset(&mut self) {
        self.tasks.clear();
        self.results.clear();
        self.statistics = BatchStatistics::default();
    }
}

/// 并行执行器包装器
pub struct ParallelExecutor<S: StorageEngine> {
    executors: Vec<Box<dyn Executor<S>>>,
    config: BatchConfig,
}

impl<S: StorageEngine> ParallelExecutor<S> {
    pub fn new(executors: Vec<Box<dyn Executor<S>>>, config: BatchConfig) -> Self {
        Self {
            executors,
            config,
        }
    }

    /// 并行执行所有执行器
    pub async fn execute_parallel(&mut self) -> Vec<DBResult<ExecutionResult>> {
        let mut batch_executor = BatchExecutor::new(self.config.clone());
        
        // 为每个执行器创建一个任务
        for (i, executor) in self.executors.iter_mut().enumerate() {
            // 这里需要克隆执行器，可能需要重新设计
            // 临时方案：直接执行并收集结果
        }
        
        // 临时实现：顺序执行
        let mut results = Vec::new();
        for executor in &mut self.executors {
            let result = executor.execute().await;
            results.push(result);
        }
        
        results
    }
}
```

## 📊 重构验证与测试

### 性能基准测试

**文件：`src/query/executor/benches/executor_bench.rs`**

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::sync::{Arc, Mutex};

use graphdb::query::executor::*;
use graphdb::core::Value;
use graphdb::storage::MockStorage;

fn bench_filter_executor(c: &mut Criterion) {
    let storage = Arc::new(Mutex::new(MockStorage::new()));
    let mut group = c.benchmark_group("filter_executor");
    
    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let mut executor = FilterExecutor::new(
                    1,
                    storage.clone(),
                    create_test_filter_expr(),
                );
                
                let input = create_test_data(size);
                executor.set_input(Box::new(MockInputExecutor::new(input)));
                
                let result = executor.execute();
                black_box(result)
            });
        });
    }
    
    group.finish();
}

fn bench_expand_executor(c: &mut Criterion) {
    let storage = Arc::new(Mutex::new(MockStorage::new()));
    let mut group = c.benchmark_group("expand_executor");
    
    for depth in [1, 2, 3].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(depth), depth, |b, &depth| {
            b.iter(|| {
                let mut executor = ExpandExecutor::new(
                    1,
                    storage.clone(),
                    vec!["edge_type".to_string()],
                    EdgeDirection::Out,
                    Some(depth),
                );
                
                let input = create_vertex_data(100);
                executor.set_input(Box::new(MockInputExecutor::new(input)));
                
                let result = executor.execute();
                black_box(result)
            });
        });
    }
    
    group.finish();
}

criterion_group!(benches, bench_filter_executor, bench_expand_executor);
criterion_main!(benches);
```

### 内存使用测试

```rust
#[cfg(test)]
mod memory_tests {
    use super::*;
    
    #[test]
    fn test_memory_usage_with_pool() {
        let storage = Arc::new(Mutex::new(MockStorage::new()));
        let pool = Arc::new(ExecutorObjectPool::new(ObjectPoolConfig::default()));
        
        // 测试前内存快照
        let memory_before = get_memory_usage();
        
        // 执行大量查询
        for _ in 0..1000 {
            let mut executor = pool.acquire::<FilterExecutor<MockStorage>>("FilterExecutor")
                .unwrap_or_else(|| {
                    FilterExecutor::new(1, storage.clone(), create_test_expr())
                });
            
            let result = executor.execute();
            assert!(result.is_ok());
            
            // 回收到池中
            pool.release("FilterExecutor", Box::new(executor));
        }
        
        // 测试后内存快照
        let memory_after = get_memory_usage();
        
        // 内存使用应该相对稳定（对象池复用）
        assert!(memory_after < memory_before * 2);
        
        let stats = pool.get_statistics();
        assert!(stats.get("FilterExecutor").unwrap().hit_rate > 0.8);
    }
}
```

## 🎯 重构成功标准

### 性能指标
- ✅ 执行器链调用延迟 < 1.5μs（对比当前2.1μs）
- ✅ 内存分配减少50%+（对象池优化）
- ✅ 并发查询性能提升30%+
- ✅ CPU利用率提高20%+

### 质量指标
- ✅ 零递归风险（100%测试覆盖）
- ✅ 工厂模式完整实现（95%+覆盖）
- ✅ 所有执行器有完整测试
- ✅ 编译零警告，Clippy全通过

### 可维护性指标
- ✅ 代码复杂度降低30%（循环复杂度）
- ✅ 文档覆盖率100%（公共API）
- ✅ 性能基准测试自动化
- ✅ 架构文档实时更新

## 📈 重构时间表

```
第1周：安全机制实现
├── 递归检测器（2天）
├── LoopExecutor安全重构（2天）
├── 工厂安全增强（2天）
└── 安全测试用例（1天）

第2周：安全测试与验证
├── 集成测试（3天）
├── 性能基准测试（2天）
└── 安全审计（2天）

第3-4周：统一trait重构
├── traits.rs重构（3天）
├── 核心执行器迁移（7天）
├── 向后兼容性测试（2天）
└── 性能对比测试（2天）

第5-6周：动态分发优化
├── 泛型化执行器实现（7天）
├── 执行器链优化（5天）
├── 内存布局优化（2天）
└── 性能基准测试（2天）

第7-8周：对象池实现
├── 对象池框架（4天）
├── 执行器池化改造（6天）
├── 批处理执行器（4天）
└── 内存使用测试（2天）

第9周：性能调优与验证
├── 性能瓶颈分析（2天）
├── 针对性优化（3天）
├── 完整集成测试（2天）
└── 重构总结文档（2天）

总计：9周（约2个月）
```

这个实施计划提供了详细的重构步骤，从安全修复开始，逐步进行架构简化和性能优化，确保重构过程的可控性和成功率。