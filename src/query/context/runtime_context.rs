use crate::storage::StorageClient;
use crate::storage::metadata::SchemaManager;
use crate::storage::index::IndexManager;
use std::sync::Arc;

/// 存储环境
#[derive(Clone)]
pub struct StorageEnv {
    /// 存储引擎
    pub storage_engine: Arc<dyn StorageClient>,
    /// Schema管理器
    pub schema_manager: Arc<dyn SchemaManager>,
    /// 索引管理器
    pub index_manager: Arc<dyn IndexManager>,
}

impl std::fmt::Debug for StorageEnv {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StorageEnv")
            .field("storage_engine", &"<dyn StorageClient>")
            .field("schema_manager", &"<dyn SchemaManager>")
            .field("index_manager", &"<dyn IndexManager>")
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
    pub space_id: i32,
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
        use std::path::PathBuf;

        let storage_env = Arc::new(StorageEnv {
            storage_engine: Arc::new(crate::storage::redb_storage::DefaultStorage::new().expect("Failed to create DefaultStorage")),
            schema_manager: Arc::new(crate::storage::metadata::MemorySchemaManager::new()),
            index_manager: Arc::new(crate::storage::index::MemoryIndexManager::new(PathBuf::from("."))),
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
    pub fn space_id(&self) -> i32 {
        self.plan_context.space_id
    }
}
