use std::sync::Arc;
use parking_lot::Mutex;

use crate::core::error::DBResult;
use crate::query::executor::base::{BaseExecutor, InputExecutor};
use crate::query::executor::executor_enum::ExecutorEnum;
use crate::query::executor::base::{ExecutionResult, Executor, HasStorage};
use crate::storage::StorageClient;

/// ArgumentExecutor - 参数执行器
///
/// 用于从另一个已执行的操作中获取命名别名
pub struct ArgumentExecutor<S: StorageClient + 'static> {
    base: BaseExecutor<S>,
    var: String,
    input_executor: Option<Box<ExecutorEnum<S>>>,
}

impl<S: StorageClient + 'static> ArgumentExecutor<S> {
    pub fn new(id: i64, storage: Arc<Mutex<S>>, var: &str) -> Self {
        Self {
            base: BaseExecutor::new(id, "ArgumentExecutor".to_string(), storage),
            var: var.to_string(),
            input_executor: None,
        }
    }

    pub fn var(&self) -> &str {
        &self.var
    }

    /// 设置变量值到执行上下文
    pub fn set_variable(&mut self, name: String, value: crate::core::Value) {
        self.base.context.set_variable(name, value);
    }

    /// 设置中间结果到执行上下文
    pub fn set_result(&mut self, name: String, result: ExecutionResult) {
        self.base.context.set_result(name, result);
    }
}

impl<S: StorageClient + Send + 'static> Executor<S> for ArgumentExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        // 首先执行输入执行器获取结果
        let _input_result = if let Some(input) = &mut self.input_executor {
            input.open()?;
            let result = input.execute()?;
            input.close()?;
            Some(result)
        } else {
            None
        };

        // 从执行上下文中获取变量值
        if let Some(var_value) = self.base.context.get_variable(&self.var) {
            // 将变量值包装为 ExecutionResult
            Ok(ExecutionResult::Values(vec![var_value.clone()]))
        } else if let Some(result) = self.base.context.get_result(&self.var) {
            // 如果变量存储在中间结果中，返回克隆的结果
            Ok(result.clone())
        } else {
            // 变量不存在，返回错误
            Err(crate::core::error::DBError::Internal(
                format!("变量 '{}' 未定义", self.var)
            ))
        }
    }

    fn open(&mut self) -> DBResult<()> {
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        Ok(())
    }

    fn is_open(&self) -> bool {
        true
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

    fn stats(&self) -> &crate::query::executor::base::ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::base::ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageClient + Send + 'static> InputExecutor<S> for ArgumentExecutor<S> {
    fn set_input(&mut self, input: ExecutorEnum<S>) {
        self.input_executor = Some(Box::new(input));
    }

    fn get_input(&self) -> Option<&ExecutorEnum<S>> {
        self.input_executor.as_ref().map(|v| &**v)
    }
}

impl<S: StorageClient + Send + 'static> HasStorage<S> for ArgumentExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.storage.as_ref().expect("Storage not set")
    }
}

/// PassThroughExecutor - 直通执行器
///
/// 用于透传情况的节点，直接传递输入数据
pub struct PassThroughExecutor<S: StorageClient + Send + 'static> {
    base: BaseExecutor<S>,
    input_executor: Option<Box<ExecutorEnum<S>>>,
}

impl<S: StorageClient> PassThroughExecutor<S> {
    pub fn new(id: i64, storage: Arc<Mutex<S>>) -> Self {
        Self {
            base: BaseExecutor::new(id, "PassThroughExecutor".to_string(), storage),
            input_executor: None,
        }
    }
}

impl<S: StorageClient + Send + 'static> Executor<S> for PassThroughExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        if let Some(input) = &mut self.input_executor {
            input.open()?;
            let result = input.execute()?;
            input.close()?;
            Ok(result)
        } else {
            Ok(ExecutionResult::Success)
        }
    }

    fn open(&mut self) -> DBResult<()> {
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        Ok(())
    }

    fn is_open(&self) -> bool {
        true
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

    fn stats(&self) -> &crate::query::executor::base::ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::base::ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageClient + Send + 'static> InputExecutor<S> for PassThroughExecutor<S> {
    fn set_input(&mut self, input: ExecutorEnum<S>) {
        self.input_executor = Some(Box::new(input));
    }

    fn get_input(&self) -> Option<&ExecutorEnum<S>> {
        self.input_executor.as_ref().map(|v| &**v)
    }
}

impl<S: StorageClient + Send + 'static> HasStorage<S> for PassThroughExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.storage.as_ref().expect("Storage not set")
    }
}

/// DataCollectExecutor - 数据收集执行器
///
/// 用于收集和聚合数据
pub struct DataCollectExecutor<S: StorageClient + Send + 'static> {
    base: BaseExecutor<S>,
    input_executor: Option<Box<ExecutorEnum<S>>>,
    collected_data: Vec<ExecutionResult>,
}

impl<S: StorageClient> DataCollectExecutor<S> {
    pub fn new(id: i64, storage: Arc<Mutex<S>>) -> Self {
        Self {
            base: BaseExecutor::new(id, "DataCollectExecutor".to_string(), storage),
            input_executor: None,
            collected_data: Vec::new(),
        }
    }

    pub fn collected_data(&self) -> &[ExecutionResult] {
        &self.collected_data
    }
}

impl<S: StorageClient + Send + 'static> Executor<S> for DataCollectExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        self.collected_data.clear();

        if let Some(input) = &mut self.input_executor {
            input.open()?;
            let result = input.execute()?;
            input.close()?;
            self.collected_data.push(result);
        }

        Ok(ExecutionResult::Success)
    }

    fn open(&mut self) -> DBResult<()> {
        self.collected_data.clear();
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        Ok(())
    }

    fn is_open(&self) -> bool {
        true
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

    fn stats(&self) -> &crate::query::executor::base::ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::base::ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageClient + Send + 'static> InputExecutor<S> for DataCollectExecutor<S> {
    fn set_input(&mut self, input: ExecutorEnum<S>) {
        self.input_executor = Some(Box::new(input));
    }

    fn get_input(&self) -> Option<&ExecutorEnum<S>> {
        self.input_executor.as_ref().map(|e| e.as_ref())
    }
}

impl<S: StorageClient + Send + 'static> HasStorage<S> for DataCollectExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.storage.as_ref().expect("Storage not set")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::test_mock::MockStorage;
    use crate::core::Value;

    #[test]
    fn test_argument_executor_creation() {
        let storage = Arc::new(Mutex::new(MockStorage));
        let executor = ArgumentExecutor::<MockStorage>::new(1, storage, "test_var");
        assert_eq!(executor.id(), 1);
        assert_eq!(executor.var(), "test_var");
        assert_eq!(executor.name(), "ArgumentExecutor");
    }

    #[test]
    fn test_argument_executor_with_variable() {
        let storage = Arc::new(Mutex::new(MockStorage));
        let mut executor = ArgumentExecutor::<MockStorage>::new(1, storage, "my_var");
        
        // 设置变量值
        executor.set_variable("my_var".to_string(), Value::String("test_value".to_string()));
        
        // 执行并验证结果
        executor.open().expect("打开执行器失败");
        let result = executor.execute().expect("执行失败");
        executor.close().expect("关闭执行器失败");
        
        match result {
            ExecutionResult::Values(values) => {
                assert_eq!(values.len(), 1);
                assert_eq!(values[0], Value::String("test_value".to_string()));
            }
            _ => panic!("预期返回 Values 结果，但得到 {:?}", result),
        }
    }

    #[test]
    fn test_argument_executor_with_result() {
        let storage = Arc::new(Mutex::new(MockStorage));
        let mut executor = ArgumentExecutor::<MockStorage>::new(1, storage, "my_result");
        
        // 设置中间结果
        let test_result = ExecutionResult::Values(vec![Value::Int(42)]);
        executor.set_result("my_result".to_string(), test_result.clone());
        
        // 执行并验证结果
        executor.open().expect("打开执行器失败");
        let result = executor.execute().expect("执行失败");
        executor.close().expect("关闭执行器失败");
        
        match result {
            ExecutionResult::Values(values) => {
                assert_eq!(values.len(), 1);
                assert_eq!(values[0], Value::Int(42));
            }
            _ => panic!("预期返回 Values 结果，但得到 {:?}", result),
        }
    }

    #[test]
    fn test_argument_executor_variable_not_found() {
        let storage = Arc::new(Mutex::new(MockStorage));
        let mut executor = ArgumentExecutor::<MockStorage>::new(1, storage, "undefined_var");
        
        // 执行时应该返回错误，因为变量未定义
        executor.open().expect("打开执行器失败");
        let result = executor.execute();
        executor.close().expect("关闭执行器失败");
        
        assert!(result.is_err(), "当变量未定义时应该返回错误");
    }

    #[test]
    fn test_pass_through_executor_creation() {
        let storage = Arc::new(Mutex::new(MockStorage));
        let executor = PassThroughExecutor::<MockStorage>::new(1, storage);
        assert_eq!(executor.id(), 1);
        assert_eq!(executor.name(), "PassThroughExecutor");
    }

    #[test]
    fn test_data_collect_executor_creation() {
        let storage = Arc::new(Mutex::new(MockStorage));
        let executor = DataCollectExecutor::<MockStorage>::new(1, storage);
        assert_eq!(executor.id(), 1);
        assert_eq!(executor.name(), "DataCollectExecutor");
        assert!(executor.collected_data().is_empty());
    }
}
