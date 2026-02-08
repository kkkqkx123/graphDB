//! 递归检测器 - 防止执行器循环引用

use crate::core::error::{DBError, DBResult};
use std::collections::HashSet;

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
            return Err(DBError::Query(
                crate::core::error::QueryError::ExecutionError(format!(
                    "执行器调用深度超过最大限制 {}: 路径 {:?}",
                    self.max_depth,
                    self.get_recursion_path()
                )),
            ));
        }

        // 检查循环引用
        if self.visited_set.contains(&executor_id) {
            return Err(DBError::Query(
                crate::core::error::QueryError::ExecutionError(format!(
                    "检测到执行器循环引用: {} (ID: {}) 在路径 {:?}",
                    executor_name,
                    executor_id,
                    self.get_recursion_path()
                )),
            ));
        }

        // 记录访问
        self.visited_stack.push(executor_id);
        self.visited_set.insert(executor_id);
        self.recursion_path
            .push(format!("{}({})", executor_name, executor_id));

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
    pub fn get_recursion_path(&self) -> Vec<String> {
        self.recursion_path.clone()
    }

    /// 重置检测器状态
    pub fn reset(&mut self) {
        self.visited_stack.clear();
        self.visited_set.clear();
        self.recursion_path.clear();
    }

    /// 获取当前深度
    pub fn current_depth(&self) -> usize {
        self.visited_stack.len()
    }

    /// 检查执行器是否已被访问
    pub fn is_visited(&self, executor_id: i64) -> bool {
        self.visited_set.contains(&executor_id)
    }
}

/// 执行器验证trait
pub trait ExecutorValidator {
    fn validate_no_recursion(&self, detector: &mut RecursionDetector) -> DBResult<()>;
}

/// 并行计算配置
///
/// 参考nebula-graph的FLAGS_max_job_size和FLAGS_min_batch_size
#[derive(Debug, Clone)]
pub struct ParallelConfig {
    /// 最大并行任务数（参考nebula-graph的FLAGS_max_job_size）
    pub max_job_size: usize,
    /// 最小批处理大小（参考nebula-graph的FLAGS_min_batch_size）
    pub min_batch_size: usize,
    /// 是否启用并行计算
    pub enable_parallel: bool,
    /// 并行计算阈值：数据量超过此值才使用并行（参考nebula-graph的traverse_parallel_threshold_rows）
    pub parallel_threshold: usize,
    /// 单线程处理的最大数据量
    pub single_thread_limit: usize,
}

impl Default for ParallelConfig {
    fn default() -> Self {
        Self {
            max_job_size: 4,              // 默认4个并行任务
            min_batch_size: 1000,         // 最小批处理1000行
            enable_parallel: true,        // 默认启用并行
            parallel_threshold: 10000,    // 数据量超过10000行才并行
            single_thread_limit: 1000,    // 少于1000行使用单线程
        }
    }
}

impl ParallelConfig {
    /// 计算批处理大小
    ///
    /// 参考nebula-graph的Executor::getBatchSize实现
    /// batch size = max(min_batch_size, ceil(total_size / max_job_size))
    pub fn calculate_batch_size(&self, total_size: usize) -> usize {
        if total_size == 0 {
            return 0;
        }
        let batch_size_tmp = (total_size + self.max_job_size - 1) / self.max_job_size;
        batch_size_tmp.max(self.min_batch_size)
    }

    /// 判断是否使用并行计算
    pub fn should_use_parallel(&self, total_size: usize) -> bool {
        self.enable_parallel && total_size >= self.parallel_threshold
    }

    /// 创建适合小数据量的配置
    pub fn for_small_data() -> Self {
        Self {
            max_job_size: 2,
            min_batch_size: 100,
            enable_parallel: true,
            parallel_threshold: 500,
            single_thread_limit: 100,
        }
    }

    /// 创建适合大数据量的配置
    pub fn for_large_data() -> Self {
        Self {
            max_job_size: 8,
            min_batch_size: 1000,
            enable_parallel: true,
            parallel_threshold: 50000,
            single_thread_limit: 1000,
        }
    }
}

/// 执行器安全配置
#[derive(Debug, Clone)]
pub struct ExecutorSafetyConfig {
    pub max_recursion_depth: usize,
    pub max_loop_iterations: usize,
    pub max_expand_depth: usize,
    pub enable_recursion_detection: bool,
    /// 并行计算配置
    pub parallel_config: ParallelConfig,
}

impl Default for ExecutorSafetyConfig {
    fn default() -> Self {
        Self {
            max_recursion_depth: 1000,
            max_loop_iterations: 10000,
            max_expand_depth: 100,
            enable_recursion_detection: true,
            parallel_config: ParallelConfig::default(),
        }
    }
}

/// 执行器安全验证器
#[derive(Debug)]
pub struct ExecutorSafetyValidator {
    config: ExecutorSafetyConfig,
    recursion_detector: RecursionDetector,
}

impl ExecutorSafetyValidator {
    /// 创建新的安全验证器
    pub fn new(config: ExecutorSafetyConfig) -> Self {
        Self {
            recursion_detector: RecursionDetector::new(config.max_recursion_depth),
            config,
        }
    }

    /// 使用默认配置创建验证器
    pub fn default() -> Self {
        Self::new(ExecutorSafetyConfig::default())
    }

    /// 验证执行器链的安全性
    pub fn validate_executor_chain(
        &mut self,
        executor_id: i64,
        executor_name: &str,
    ) -> DBResult<()> {
        if self.config.enable_recursion_detection {
            self.recursion_detector
                .validate_executor(executor_id, executor_name)?;
        }
        Ok(())
    }

    /// 验证循环配置
    pub fn validate_loop_config(&self, max_iterations: Option<usize>) -> DBResult<()> {
        if let Some(iterations) = max_iterations {
            if iterations > self.config.max_loop_iterations {
                return Err(DBError::Query(
                    crate::core::error::QueryError::ExecutionError(format!(
                        "循环最大迭代次数 {} 超过限制 {}",
                        iterations, self.config.max_loop_iterations
                    )),
                ));
            }
        }
        Ok(())
    }

    /// 验证扩展配置
    pub fn validate_expand_config(&self, max_depth: Option<usize>) -> DBResult<()> {
        if let Some(depth) = max_depth {
            if depth > self.config.max_expand_depth {
                return Err(DBError::Query(
                    crate::core::error::QueryError::ExecutionError(format!(
                        "扩展最大深度 {} 超过限制 {}",
                        depth, self.config.max_expand_depth
                    )),
                ));
            }
        }
        Ok(())
    }

    /// 重置验证器状态
    pub fn reset(&mut self) {
        self.recursion_detector.reset();
    }

    /// 获取当前递归深度
    pub fn current_depth(&self) -> usize {
        self.recursion_detector.current_depth()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recursion_detection() {
        let mut detector = RecursionDetector::new(10);

        // 正常情况
        assert!(detector.validate_executor(1, "TestExecutor").is_ok());
        assert!(detector.validate_executor(2, "AnotherExecutor").is_ok());

        // 循环引用检测
        assert!(detector.validate_executor(1, "TestExecutor").is_err());
    }

    #[test]
    fn test_max_depth_protection() {
        let mut detector = RecursionDetector::new(3);

        // 正常深度
        assert!(detector.validate_executor(1, "E1").is_ok());
        assert!(detector.validate_executor(2, "E2").is_ok());
        assert!(detector.validate_executor(3, "E3").is_ok());

        // 超过最大深度
        assert!(detector.validate_executor(4, "E4").is_err());
    }

    #[test]
    fn test_leave_executor() {
        let mut detector = RecursionDetector::new(10);

        // 进入执行器
        assert!(detector.validate_executor(1, "E1").is_ok());
        assert_eq!(detector.current_depth(), 1);

        // 离开执行器
        detector.leave_executor();
        assert_eq!(detector.current_depth(), 0);

        // 可以重新进入
        assert!(detector.validate_executor(1, "E1").is_ok());
    }

    #[test]
    fn test_reset_detector() {
        let mut detector = RecursionDetector::new(3);

        // 进入多个执行器
        assert!(detector.validate_executor(1, "E1").is_ok());
        assert!(detector.validate_executor(2, "E2").is_ok());

        // 重置
        detector.reset();

        // 现在应该可以重新进入
        assert!(detector.validate_executor(1, "E1").is_ok());
        assert!(detector.validate_executor(2, "E2").is_ok());
    }

    #[test]
    fn test_safety_validator_loop_config() {
        let validator = ExecutorSafetyValidator::default();

        // 正常配置
        assert!(validator.validate_loop_config(Some(100)).is_ok());
        assert!(validator.validate_loop_config(None).is_ok());

        // 超过限制
        assert!(validator.validate_loop_config(Some(20000)).is_err());
    }

    #[test]
    fn test_safety_validator_expand_config() {
        let validator = ExecutorSafetyValidator::default();

        // 正常配置
        assert!(validator.validate_expand_config(Some(50)).is_ok());
        assert!(validator.validate_expand_config(None).is_ok());

        // 超过限制
        assert!(validator.validate_expand_config(Some(200)).is_err());
    }

    #[test]
    fn test_recursion_path_tracking() {
        let mut detector = RecursionDetector::new(10);

        detector.validate_executor(1, "E1").expect("validate_executor should succeed");
        detector.validate_executor(2, "E2").expect("validate_executor should succeed");
        detector.validate_executor(3, "E3").expect("validate_executor should succeed");

        let path = detector.get_recursion_path();
        assert_eq!(path, vec!["E1(1)", "E2(2)", "E3(3)"]);
    }
}
