//! 基础执行器实现
//!
//! 提供执行器的基础结构和通用功能，包括 Executor trait、HasStorage trait、HasInput trait 等。

use std::sync::Arc;
use std::time::Instant;

use crate::storage::StorageClient;
use parking_lot::Mutex;

use super::execution_context::ExecutionContext;
use super::execution_result::{ExecutionResult, DBResult};
use super::executor_stats::ExecutorStats;
use super::super::executor_enum::ExecutorEnum;

/// 统一的执行器 trait
///
/// 所有执行器必须实现的核心 trait，包含执行、生命周期和元数据功能。
pub trait Executor<S: StorageClient>: Send {
    /// 执行查询
    fn execute(&mut self) -> DBResult<ExecutionResult>;

    /// 打开执行器
    fn open(&mut self) -> DBResult<()>;

    /// 关闭执行器
    fn close(&mut self) -> DBResult<()>;

    /// 检查执行器是否已打开
    fn is_open(&self) -> bool;

    /// 获取执行器 ID
    fn id(&self) -> i64;

    /// 获取执行器名称
    fn name(&self) -> &str;

    /// 获取执行器描述
    fn description(&self) -> &str;

    /// 获取执行统计信息
    fn stats(&self) -> &ExecutorStats;

    /// 获取可变的执行统计信息
    fn stats_mut(&mut self) -> &mut ExecutorStats;

    /// 检查内存使用
    fn check_memory(&self) -> DBResult<()> {
        Ok(())
    }
}

/// 存储访问 trait
///
/// 只需要存储访问能力的执行器可以实现此 trait。
pub trait HasStorage<S: StorageClient> {
    fn get_storage(&self) -> &Arc<Mutex<S>>;
}

/// 输入访问 trait - 统一输入处理机制
///
/// 需要访问输入数据的执行器应实现此 trait。
pub trait HasInput<S: StorageClient> {
    fn get_input(&self) -> Option<&ExecutionResult>;
    fn set_input(&mut self, input: ExecutionResult);
}

/// 输入执行器 trait
///
/// 用于处理来自其他执行器的输入数据。
/// 使用 ExecutorEnum 替代 Box<dyn Executor<S>>，实现静态分发。
pub trait InputExecutor<S: StorageClient + Send + 'static> {
    fn set_input(&mut self, input: ExecutorEnum<S>);
    fn get_input(&self) -> Option<&ExecutorEnum<S>>;
}

/// 可链式执行的执行器 trait
///
/// 支持链式组合的执行器可以实现此 trait。
pub trait ChainableExecutor<S: StorageClient + Send + 'static>:
    Executor<S> + InputExecutor<S>
{
}

/// 基础执行器
///
/// 提供执行器的通用功能，包括存储访问、统计信息、生命周期管理等。
#[derive(Clone, Debug)]
pub struct BaseExecutor<S: StorageClient> {
    /// 执行器 ID
    pub id: i64,
    /// 执行器名称
    pub name: String,
    /// 执行器描述
    pub description: String,
    /// 存储引擎引用
    pub storage: Option<Arc<Mutex<S>>>,
    /// 执行上下文
    pub context: ExecutionContext,
    /// 是否已打开
    is_open: bool,
    /// 执行统计信息
    stats: ExecutorStats,
}

impl<S: StorageClient> BaseExecutor<S> {
    /// 创建新的基础执行器（带存储）
    pub fn new(id: i64, name: String, storage: Arc<Mutex<S>>) -> Self {
        Self {
            id,
            name,
            description: String::new(),
            storage: Some(storage),
            context: ExecutionContext::new(),
            is_open: false,
            stats: ExecutorStats::new(),
        }
    }

    /// 创建新的基础执行器（不带存储）
    pub fn without_storage(id: i64, name: String) -> Self {
        Self {
            id,
            name,
            description: String::new(),
            storage: None,
            context: ExecutionContext::new(),
            is_open: false,
            stats: ExecutorStats::new(),
        }
    }

    /// 创建带上下文的基础执行器
    pub fn with_context(id: i64, name: String, storage: Arc<Mutex<S>>, context: ExecutionContext) -> Self {
        Self {
            id,
            name,
            description: String::new(),
            storage: Some(storage),
            context,
            is_open: false,
            stats: ExecutorStats::new(),
        }
    }

    /// 创建带描述的基础执行器
    pub fn with_description(id: i64, name: String, description: String, storage: Arc<Mutex<S>>) -> Self {
        Self {
            id,
            name,
            description,
            storage: Some(storage),
            context: ExecutionContext::new(),
            is_open: false,
            stats: ExecutorStats::new(),
        }
    }

    /// 创建带上下文和描述的基础执行器
    pub fn with_context_and_description(
        id: i64,
        name: String,
        description: String,
        storage: Arc<Mutex<S>>,
        context: ExecutionContext,
    ) -> Self {
        Self {
            id,
            name,
            description,
            storage: Some(storage),
            context,
            is_open: false,
            stats: ExecutorStats::new(),
        }
    }

    /// 获取执行统计信息（不可变引用）
    pub fn get_stats(&self) -> &ExecutorStats {
        &self.stats
    }

    /// 获取执行统计信息（可变引用）
    pub fn get_stats_mut(&mut self) -> &mut ExecutorStats {
        &mut self.stats
    }
}

impl<S: StorageClient> HasStorage<S> for BaseExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.storage.as_ref().expect("Storage not set")
    }
}

impl<S: StorageClient> Executor<S> for BaseExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        let start = Instant::now();
        let result = Ok(ExecutionResult::Success);
        self.stats_mut().add_total_time(start.elapsed());
        result
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

    fn stats(&self) -> &ExecutorStats {
        &self.stats
    }

    fn stats_mut(&mut self) -> &mut ExecutorStats {
        &mut self.stats
    }
}

/// 开始执行器
///
/// 表示查询执行的起始点，不产生实际数据。
#[derive(Debug)]
pub struct StartExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
}

impl<S: StorageClient> StartExecutor<S> {
    /// 创建新的开始执行器
    pub fn new(id: i64) -> Self {
        Self {
            base: BaseExecutor::without_storage(id, "StartExecutor".to_string()),
        }
    }
}

impl<S: StorageClient> Executor<S> for StartExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        let start = Instant::now();
        let result = Ok(ExecutionResult::Success);
        self.base.get_stats_mut().add_total_time(start.elapsed());
        result
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

    fn stats(&self) -> &ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut ExecutorStats {
        self.base.get_stats_mut()
    }
}
