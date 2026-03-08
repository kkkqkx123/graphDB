//! 查询资源上下文
//!
//! 管理查询执行过程中需要的资源，包括 ID 生成器等。

use crate::utils::IdGenerator;

/// 查询资源上下文
///
/// 管理查询执行过程中需要的资源，包括：
/// - ID 生成器（用于生成唯一的 ID）
pub struct QueryResourceContext {
    /// ID 生成器
    id_gen: IdGenerator,
}

impl QueryResourceContext {
    /// 创建新的资源上下文
    pub fn new() -> Self {
        Self {
            id_gen: IdGenerator::new(0),
        }
    }

    /// 创建带自定义配置的资源上下文
    pub fn with_config(start_id: i64) -> Self {
        Self {
            id_gen: IdGenerator::new(start_id),
        }
    }

    /// 生成 ID
    pub fn gen_id(&self) -> i64 {
        self.id_gen.id()
    }

    /// 获取当前 ID 值（不递增）
    pub fn current_id(&self) -> i64 {
        self.id_gen.current_value()
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
