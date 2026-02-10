use std::sync::{Arc, Mutex};

use crate::core::error::DBResult;
use crate::query::executor::base::{BaseExecutor, InputExecutor};
use crate::query::executor::executor_enum::ExecutorEnum;
use crate::query::executor::traits::{ExecutionResult, Executor, HasStorage};
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
}

impl<S: StorageClient + Send + 'static> Executor<S> for ArgumentExecutor<S> {
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
