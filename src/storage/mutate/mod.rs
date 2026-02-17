//! 存储层数据修改模块 (DML Processor)
//!
//! 提供类似 NebulaGraph 存储层的数据修改功能，包括：
//! - 顶点插入、更新、删除
//! - 边插入、更新、删除
//! - 标签删除
//! - 索引联动更新
//! - 内存锁机制

pub mod vertex_processor;
pub mod edge_processor;
pub mod lock_manager;

pub use vertex_processor::{VertexInsertProcessor, VertexUpdateProcessor, VertexDeleteProcessor, TagDeleteProcessor, TagDeleteItem, VertexUpdateItem};
pub use edge_processor::{EdgeInsertProcessor, EdgeUpdateProcessor, EdgeDeleteProcessor, EdgeInsertItem, EdgeUpdateItem, EdgeDeleteItem};
pub use lock_manager::{MemoryLockManager, LockType, LockGuard};

use crate::core::StorageError;
use std::sync::Arc;
use parking_lot::Mutex;

/// DML 操作结果
#[derive(Debug, Clone)]
pub struct DmlResult {
    pub affected_count: usize,
    pub success: bool,
    pub error_message: Option<String>,
    /// 额外统计信息，如级联删除的边数量
    pub extra_stats: Option<DmlExtraStats>,
}

/// DML 额外统计信息
#[derive(Debug, Clone)]
pub struct DmlExtraStats {
    /// 级联删除的边数量
    pub deleted_edges_count: usize,
}

impl DmlResult {
    pub fn success(count: usize) -> Self {
        Self {
            affected_count: count,
            success: true,
            error_message: None,
            extra_stats: None,
        }
    }

    pub fn success_with_stats(count: usize, deleted_edges: usize) -> Self {
        Self {
            affected_count: count,
            success: true,
            error_message: None,
            extra_stats: Some(DmlExtraStats {
                deleted_edges_count: deleted_edges,
            }),
        }
    }

    pub fn error(msg: String) -> Self {
        Self {
            affected_count: 0,
            success: false,
            error_message: Some(msg),
            extra_stats: None,
        }
    }
}

/// DML 处理器 trait
pub trait DmlProcessor: Send + Sync {
    fn execute(&mut self) -> Result<DmlResult, StorageError>;
}

/// 批量 DML 操作上下文
#[derive(Debug, Clone)]
pub struct BatchDmlContext {
    pub space_name: String,
    pub if_not_exists: bool,
    pub ignore_existed_index: bool,
}

impl Default for BatchDmlContext {
    fn default() -> Self {
        Self {
            space_name: "default".to_string(),
            if_not_exists: false,
            ignore_existed_index: false,
        }
    }
}

/// 创建内存锁管理器
pub fn create_lock_manager() -> Arc<Mutex<MemoryLockManager>> {
    Arc::new(Mutex::new(MemoryLockManager::new()))
}
