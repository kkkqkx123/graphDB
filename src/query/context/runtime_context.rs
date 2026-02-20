use crate::storage::StorageClient;
use crate::storage::metadata::SchemaManager;
use std::sync::Arc;

/// 存储环境
#[derive(Clone)]
pub struct StorageEnv {
    /// 存储引擎
    pub storage_engine: Arc<dyn StorageClient>,
    /// Schema管理器
    pub schema_manager: Arc<dyn SchemaManager>,
}

impl std::fmt::Debug for StorageEnv {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StorageEnv")
            .field("storage_engine", &"<dyn StorageClient>")
            .field("schema_manager", &"<dyn SchemaManager>")
            .finish()
    }
}

/// 计划上下文（存储层）
/// 存储处理过程中不变的信息
#[derive(Debug, Clone)]
pub struct PlanContext {
    /// 存储环境引用
    pub storage_env: Arc<StorageEnv>,
    /// 空间ID
    pub space_id: u64,
}

/// 运行时上下文
/// 存储处理过程中可能变化的信息
#[derive(Debug, Clone)]
pub struct RuntimeContext {
    /// 计划上下文引用
    pub plan_context: Arc<PlanContext>,
}

impl RuntimeContext {
    /// 创建新的运行时上下文
    pub fn new(plan_context: Arc<PlanContext>) -> Self {
        Self { plan_context }
    }

    /// 创建简单的运行时上下文（用于不需要完整PlanContext的场景）
    pub fn new_simple() -> Arc<Self> {
        let storage = Arc::new(crate::storage::redb_storage::DefaultStorage::new().expect("Failed to create DefaultStorage"));
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

    /// 获取存储环境
    pub fn env(&self) -> &Arc<StorageEnv> {
        &self.plan_context.storage_env
    }

    /// 获取空间ID
    pub fn space_id(&self) -> u64 {
        self.plan_context.space_id
    }
}
