//! 运行时上下文模块 - 存储层上下文管理
//!
//! 提供查询执行过程中的存储层上下文信息，包括：
//! - 存储环境（StorageEnv）
//! - 计划上下文（PlanContext）
//! - 运行时上下文（RuntimeContext）

use crate::storage::metadata::SchemaManager;
use crate::storage::StorageClient;
use std::sync::Arc;

/// 存储环境
#[derive(Clone)]
pub struct StorageEnv<S, M>
where
    S: StorageClient,
    M: SchemaManager,
{
    /// 存储引擎
    pub storage_engine: Arc<S>,
    /// Schema管理器
    pub schema_manager: Arc<M>,
}

impl<S, M> std::fmt::Debug for StorageEnv<S, M>
where
    S: StorageClient,
    M: SchemaManager,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StorageEnv")
            .field("storage_engine", &"<StorageClient>")
            .field("schema_manager", &"<SchemaManager>")
            .finish()
    }
}

/// 计划上下文（存储层）
/// 存储处理过程中不变的信息
#[derive(Debug, Clone)]
pub struct PlanContext<S, M>
where
    S: StorageClient,
    M: SchemaManager,
{
    /// 存储环境引用
    pub storage_env: Arc<StorageEnv<S, M>>,
    /// 空间ID
    pub space_id: u64,
}

/// 运行时上下文
/// 存储处理过程中可能变化的信息
#[derive(Debug, Clone)]
pub struct RuntimeContext<S, M>
where
    S: StorageClient,
    M: SchemaManager,
{
    /// 计划上下文引用
    pub plan_context: Arc<PlanContext<S, M>>,
}

impl<S, M> RuntimeContext<S, M>
where
    S: StorageClient,
    M: SchemaManager,
{
    /// 创建新的运行时上下文
    pub fn new(plan_context: Arc<PlanContext<S, M>>) -> Self {
        Self { plan_context }
    }

    /// 获取存储环境
    pub fn env(&self) -> &Arc<StorageEnv<S, M>> {
        &self.plan_context.storage_env
    }

    /// 获取空间ID
    pub fn space_id(&self) -> u64 {
        self.plan_context.space_id
    }
}

/// 使用默认存储类型的运行时上下文类型别名
pub type DefaultRuntimeContext =
    RuntimeContext<crate::storage::redb_storage::RedbStorage, crate::storage::metadata::RedbSchemaManager>;

/// 使用默认存储类型的计划上下文类型别名
pub type DefaultPlanContext =
    PlanContext<crate::storage::redb_storage::RedbStorage, crate::storage::metadata::RedbSchemaManager>;

/// 使用默认存储类型的存储环境类型别名
pub type DefaultStorageEnv =
    StorageEnv<crate::storage::redb_storage::RedbStorage, crate::storage::metadata::RedbSchemaManager>;

impl DefaultRuntimeContext {
    /// 创建简单的运行时上下文（用于不需要完整PlanContext的场景）
    pub fn new_simple() -> Arc<Self> {
        let storage = Arc::new(
            crate::storage::redb_storage::DefaultStorage::new()
                .expect("Failed to create DefaultStorage"),
        );
        let storage_env = Arc::new(StorageEnv {
            storage_engine: storage.clone(),
            schema_manager: storage.schema_manager.clone(),
        });

        let plan_context = Arc::new(PlanContext {
            storage_env,
            space_id: 0,
        });

        Arc::new(Self::new(plan_context))
    }
}
