//! Executor trait 重构 - 拆分为多个小 trait
//! 
//! 这个模块将原来的 Executor trait 拆分为多个职责单一的小 trait，
//! 遵循接口隔离原则，提高代码的可维护性和可扩展性。

use crate::core::error::DBError;
use crate::storage::StorageEngine;
use async_trait::async_trait;

/// 执行核心 trait - 负责执行逻辑
#[async_trait]
pub trait ExecutorCore {
    /// 执行查询
    async fn execute(&mut self) -> DBResult<ExecutionResult>;
}

/// 生命周期管理 trait - 负责执行器的生命周期
pub trait ExecutorLifecycle {
    /// 打开执行器
    fn open(&mut self) -> DBResult<()>;
    
    /// 关闭执行器
    fn close(&mut self) -> DBResult<()>;
    
    /// 检查执行器是否已打开
    fn is_open(&self) -> bool;
}

/// 元数据 trait - 提供执行器的元信息
pub trait ExecutorMetadata {
    /// 获取执行器ID
    fn id(&self) -> usize;
    
    /// 获取执行器名称
    fn name(&self) -> &str;
    
    /// 获取执行器描述
    fn description(&self) -> &str;
}

/// 组合 Executor trait - 组合所有 Executor 相关 trait
#[async_trait]
pub trait Executor<S: StorageEngine>: ExecutorCore
                                    + ExecutorLifecycle
                                    + ExecutorMetadata
                                    + Send
                                    + Sync {
    /// 获取存储引擎引用
    fn storage(&self) -> &S;
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
            ExecutionResult::DataSet(ds) => ds.len(),
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
#[derive(Debug)]
pub struct BaseExecutor<S: StorageEngine> {
    storage: S,
    id: usize,
    name: String,
    description: String,
    is_open: bool,
}

impl<S: StorageEngine> BaseExecutor<S> {
    /// 创建新的基础执行器
    pub fn new(storage: S, id: usize, name: &str, description: &str) -> Self {
        Self {
            storage,
            id,
            name: name.to_string(),
            description: description.to_string(),
            is_open: false,
        }
    }
    
    /// 获取存储引擎的可变引用
    pub fn storage_mut(&mut self) -> &mut S {
        &mut self.storage
    }
}

impl<S: StorageEngine> ExecutorLifecycle for BaseExecutor<S> {
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
}

impl<S: StorageEngine> ExecutorMetadata for BaseExecutor<S> {
    fn id(&self) -> usize {
        self.id
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        &self.description
    }
}

/// 为现有实现提供自动派生的宏
#[macro_export]
macro_rules! impl_executor_for {
    ($type:ty, $storage_type:ty) => {
        #[async_trait::async_trait]
        impl $crate::query::executor::traits::ExecutorCore for $type {
            async fn execute(&mut self) -> $crate::query::executor::traits::DBResult<$crate::query::executor::traits::ExecutionResult> {
                self.execute().await
            }
        }
        
        impl $crate::query::executor::traits::ExecutorLifecycle for $type {
            fn open(&mut self) -> $crate::query::executor::traits::DBResult<()> {
                self.open()
            }
            
            fn close(&mut self) -> $crate::query::executor::traits::DBResult<()> {
                self.close()
            }
            
            fn is_open(&self) -> bool {
                self.is_open()
            }
        }
        
        impl $crate::query::executor::traits::ExecutorMetadata for $type {
            fn id(&self) -> usize {
                self.id()
            }
            
            fn name(&self) -> &str {
                self.name()
            }
            
            fn description(&self) -> &str {
                self.description()
            }
        }
        
        #[async_trait::async_trait]
        impl $crate::query::executor::traits::Executor<$storage_type> for $type {
            fn storage(&self) -> &$storage_type {
                self.storage()
            }
        }
    };
}