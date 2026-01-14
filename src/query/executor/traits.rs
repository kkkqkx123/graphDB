//! Executor trait 重构 - 统一简化架构
//!
//! 这个模块提供了简化的执行器trait设计，减少动态分发，提高性能。
//! 采用组合trait的方式，提供灵活且高效的执行器接口。

use crate::core::error::DBError;
use crate::storage::StorageEngine;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// 执行器统计信息
#[derive(Debug, Clone, Default)]
pub struct ExecutorStats {
    /// 处理的行数
    pub num_rows: usize,
    /// 执行时间（微秒）
    pub exec_time_us: u64,
    /// 总时间（微秒）
    pub total_time_us: u64,
    /// 其他统计信息
    pub other_stats: HashMap<String, String>,
}

impl ExecutorStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_row(&mut self, count: usize) {
        self.num_rows += count;
    }

    pub fn add_exec_time(&mut self, duration: Duration) {
        self.exec_time_us += duration.as_micros() as u64;
    }

    pub fn add_total_time(&mut self, duration: Duration) {
        self.total_time_us += duration.as_micros() as u64;
    }

    pub fn add_stat(&mut self, key: String, value: String) {
        self.other_stats.insert(key, value);
    }

    pub fn get_stat(&self, key: &str) -> Option<&String> {
        self.other_stats.get(key)
    }
}

/// 统一的执行器trait - 核心接口
///
/// 这是所有执行器必须实现的核心trait，包含执行、生命周期和元数据功能
#[async_trait]
pub trait Executor<S: StorageEngine>: Send + Sync {
    /// 执行查询
    async fn execute(&mut self) -> DBResult<ExecutionResult>;

    /// 打开执行器
    fn open(&mut self) -> DBResult<()>;

    /// 关闭执行器
    fn close(&mut self) -> DBResult<()>;

    /// 检查执行器是否已打开
    fn is_open(&self) -> bool;

    /// 获取执行器ID
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

/// 存储访问trait - 可选功能
///
/// 只需要存储访问能力的执行器可以实现此trait
pub trait HasStorage<S: StorageEngine> {
    fn get_storage(&self) -> &Arc<Mutex<S>>;
}

/// 输入访问trait - 可选功能
///
/// 需要访问输入执行器的执行器可以实现此trait
pub trait HasInput<S: StorageEngine> {
    fn get_input(&self) -> Option<&Box<dyn Executor<S>>>;
    fn get_input_mut(&mut self) -> Option<&mut Box<dyn Executor<S>>>;
    fn set_input_impl(&mut self, input: Box<dyn Executor<S>>);
}

/// 执行结果类型
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
        }
    }
}

/// 结果类型别名
pub type DBResult<T> = Result<T, DBError>;

/// 基础执行器实现 - 提供默认的执行器行为
///
/// 提供存储、ID、名称、描述等基础功能
#[derive(Debug)]
pub struct BaseExecutor<S: StorageEngine> {
    storage: Option<Arc<Mutex<S>>>,
    id: i64,
    name: String,
    description: String,
    is_open: bool,
    stats: ExecutorStats,
}

impl<S: StorageEngine> BaseExecutor<S> {
    /// 创建新的基础执行器（带存储）
    pub fn new(storage: Arc<Mutex<S>>, id: i64, name: &str, description: &str) -> Self {
        Self {
            storage: Some(storage),
            id,
            name: name.to_string(),
            description: description.to_string(),
            is_open: false,
            stats: ExecutorStats::new(),
        }
    }

    /// 创建新的基础执行器（不带存储）
    pub fn new_without_storage(id: i64, name: &str, description: &str) -> Self {
        Self {
            storage: None,
            id,
            name: name.to_string(),
            description: description.to_string(),
            is_open: false,
            stats: ExecutorStats::new(),
        }
    }

    /// 获取存储引擎的可变引用
    pub fn storage_mut(&mut self) -> &Arc<Mutex<S>> {
        self.storage.as_ref().expect("Storage not set")
    }

    /// 设置存储引擎
    pub fn set_storage(&mut self, storage: Arc<Mutex<S>>) {
        self.storage = Some(storage);
    }

    /// 获取执行统计信息
    pub fn get_stats(&self) -> &ExecutorStats {
        &self.stats
    }

    /// 获取可变的执行统计信息
    pub fn get_stats_mut(&mut self) -> &mut ExecutorStats {
        &mut self.stats
    }
}

impl<S: StorageEngine> HasStorage<S> for BaseExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.storage.as_ref().expect("Storage not set")
    }
}
