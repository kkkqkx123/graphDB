//! 查询资源上下文
//!
//! 管理查询执行过程中需要的资源，包括对象池、ID 生成器、符号表等。

use crate::core::SymbolTable;
use crate::utils::{IdGenerator, ObjectPool};
use std::sync::Arc;

/// 查询资源上下文
///
/// 管理查询执行过程中需要的资源，包括：
/// - 对象池（用于字符串等对象的复用）
/// - ID 生成器（用于生成唯一的 ID）
/// - 符号表（用于管理符号信息）
pub struct QueryResourceContext {
    /// 对象池
    obj_pool: ObjectPool<String>,

    /// ID 生成器
    id_gen: IdGenerator,

    /// 符号表 - 使用 Arc<SymbolTable>，内部 DashMap 已提供并发安全
    sym_table: Arc<SymbolTable>,
}

impl QueryResourceContext {
    /// 创建新的资源上下文
    pub fn new() -> Self {
        Self {
            obj_pool: ObjectPool::new(1000),
            id_gen: IdGenerator::new(0),
            sym_table: Arc::new(SymbolTable::new()),
        }
    }

    /// 创建带自定义配置的资源上下文
    pub fn with_config(pool_size: usize, start_id: i64) -> Self {
        Self {
            obj_pool: ObjectPool::new(pool_size),
            id_gen: IdGenerator::new(start_id),
            sym_table: Arc::new(SymbolTable::new()),
        }
    }

    /// 获取对象池
    pub fn obj_pool(&self) -> &ObjectPool<String> {
        &self.obj_pool
    }

    /// 获取可变对象池
    pub fn obj_pool_mut(&mut self) -> &mut ObjectPool<String> {
        &mut self.obj_pool
    }

    /// 生成 ID
    pub fn gen_id(&self) -> i64 {
        self.id_gen.id()
    }

    /// 获取当前 ID 值（不递增）
    pub fn current_id(&self) -> i64 {
        self.id_gen.current_value()
    }

    /// 获取符号表
    pub fn sym_table(&self) -> &SymbolTable {
        &self.sym_table
    }

    /// 获取符号表的 Arc 引用
    pub fn sym_table_arc(&self) -> Arc<SymbolTable> {
        self.sym_table.clone()
    }

    /// 重置资源上下文
    pub fn reset(&mut self) {
        self.id_gen.reset(0);
        log::info!("查询资源上下文已重置");
    }
}

impl Default for QueryResourceContext {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for QueryResourceContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QueryResourceContext")
            .field("current_id", &self.current_id())
            .finish()
    }
}
