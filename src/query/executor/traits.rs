//! 执行器 trait 定义
//!
//! 本模块提供执行器相关的 trait 定义。
//! 注意：基础类型（ExecutorStats、ExecutionResult、BaseExecutor 等）已迁移到 base/ 模块。
//! 本模块主要保留 ResultProcessor 等结果处理相关的 trait。

use std::sync::Arc;

use crate::storage::StorageClient;
use parking_lot::Mutex;

// 从 base 模块重新导出基础类型
pub use crate::query::executor::base::{
    DBResult, ExecutionResult, Executor, ExecutorStats, HasInput, HasStorage,
};

/// 结果处理器 trait
///
/// 用于处理查询结果的执行器应实现此 trait。
pub trait ResultProcessor<S: StorageClient>: Executor<S> {
    /// 获取处理器上下文
    fn get_context(&self) -> &ResultProcessorContext<S>;

    /// 获取可变的处理器上下文
    fn get_context_mut(&mut self) -> &mut ResultProcessorContext<S>;
}

/// 结果处理器上下文
///
/// 为结果处理器提供必要的上下文信息。
#[derive(Debug, Clone)]
pub struct ResultProcessorContext<S: StorageClient> {
    /// 存储引擎引用
    pub storage: Option<Arc<Mutex<S>>>,
    /// 输入数据
    pub input: Option<ExecutionResult>,
}

impl<S: StorageClient> ResultProcessorContext<S> {
    /// 创建新的上下文
    pub fn new(storage: Arc<Mutex<S>>) -> Self {
        Self {
            storage: Some(storage),
            input: None,
        }
    }

    /// 创建不带存储的上下文
    pub fn new_without_storage() -> Self {
        Self {
            storage: None,
            input: None,
        }
    }
}

/// 基础结果处理器
///
/// 提供结果处理器的通用功能。
#[derive(Debug, Clone)]
pub struct BaseResultProcessor<S: StorageClient> {
    /// 处理器 ID
    pub id: i64,
    /// 处理器名称
    pub name: String,
    /// 处理器描述
    pub description: String,
    /// 处理器上下文
    pub context: ResultProcessorContext<S>,
}

impl<S: StorageClient> BaseResultProcessor<S> {
    /// 创建新的基础结果处理器
    pub fn new(id: i64, name: String, description: String, storage: Arc<Mutex<S>>) -> Self {
        Self {
            id,
            name,
            description,
            context: ResultProcessorContext::new(storage),
        }
    }

    /// 获取处理器 ID
    pub fn id(&self) -> i64 {
        self.id
    }

    /// 获取处理器名称
    pub fn name(&self) -> &str {
        &self.name
    }
}
