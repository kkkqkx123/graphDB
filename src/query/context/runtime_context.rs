//! 存储层运行时上下文（简化版本）
//!
//! RuntimeContext用于存储层执行节点，包含计划上下文引用和运行时可变信息
//! 针对单节点GraphDB进行了大幅简化，移除了分布式设计中的Arc<RwLock<>>同步原语
//!
//! 对应原C++版本中的RuntimeContext结构

use crate::common::id::{EdgeType, TagId};
use crate::query::core::{ExecutorState, RowStatus};
use crate::storage::StorageClient;
use crate::storage::metadata::SchemaManager;
use crate::storage::index::IndexManager;
use std::sync::Arc;
use std::time::Instant;

/// 结果状态枚举
/// 
/// 已废弃：请使用 `crate::query::core::RowStatus`
#[deprecated(since = "0.1.0", note = "请使用 crate::query::core::RowStatus")]
pub type ResultStatus = RowStatus;

/// 执行状态
///
/// 已废弃：请使用 `crate::query::core::ExecutorState`
#[deprecated(since = "0.1.0", note = "请使用 crate::query::core::ExecutorState")]
pub type ExecutionState = ExecutorState;

/// 属性上下文
#[derive(Debug, Clone)]
pub struct PropContext {
    pub name: String,
    pub prop_type: String,
    pub nullable: bool,
}

/// 计划上下文（存储层）
/// 存储处理过程中不变的信息
#[derive(Debug, Clone)]
pub struct PlanContext {
    /// 存储环境引用
    pub storage_env: Arc<StorageEnv>,
    /// 空间ID
    pub space_id: i32,
    /// 计划ID
    pub plan_id: i64,
    /// 顶点ID长度
    pub v_id_len: usize,
    /// 是否为整数ID
    pub is_int_id: bool,
    /// 是否为边查询
    pub is_edge: bool,
}

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

/// 运行时上下文（简化版本）
/// 存储处理过程中可能变化的信息
#[derive(Debug, Clone)]
pub struct RuntimeContext {
    /// 计划上下文引用
    pub plan_context: Arc<PlanContext>,

    /// 分区ID（用于分布式查询）
    pub part_id: Option<u32>,

    /// 标签信息
    pub tag_id: Option<TagId>,
    pub tag_name: Option<String>,

    /// 边信息
    pub edge_type: Option<EdgeType>,
    pub edge_name: Option<String>,

    /// 执行配置
    pub column_idx: usize,
    pub props: Option<Vec<PropContext>>,
    pub filter_invalid_result_out: bool,

    /// 执行状态
    pub state: ExecutionState,
    pub start_time: Option<Instant>,
    pub error: Option<String>,
}

impl RuntimeContext {
    /// 创建新的运行时上下文
    pub fn new(plan_context: Arc<PlanContext>) -> Self {
        Self {
            plan_context,
            part_id: None,
            tag_id: None,
            tag_name: None,
            edge_type: None,
            edge_name: None,
            column_idx: 0,
            props: None,
            filter_invalid_result_out: false,
            state: ExecutionState::Initialized,
            start_time: None,
            error: None,
        }
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
            plan_id: 0,
            v_id_len: 8,
            is_int_id: true,
            is_edge: false,
        });

        Arc::new(Self::new(plan_context))
    }

    /// 获取存储环境
    pub fn env(&self) -> &Arc<StorageEnv> {
        &self.plan_context.storage_env
    }

    /// 获取计划ID
    pub fn plan_id(&self) -> i64 {
        self.plan_context.plan_id
    }

    /// 获取计划ID（用于Arc包装的类型）
    pub fn arc_plan_id(ctx: &Arc<Self>) -> i64 {
        ctx.plan_context.plan_id
    }

    /// 获取空间ID
    pub fn space_id(&self) -> i32 {
        self.plan_context.space_id
    }

    /// 获取顶点ID长度
    pub fn v_id_len(&self) -> usize {
        self.plan_context.v_id_len
    }

    /// 检查是否为整数ID
    pub fn is_int_id(&self) -> bool {
        self.plan_context.is_int_id
    }

    /// 检查是否为边查询
    pub fn is_edge(&self) -> bool {
        self.plan_context.is_edge
    }

    /// 设置标签信息
    pub fn set_tag_info(&mut self, tag_id: TagId, tag_name: String) {
        self.tag_id = Some(tag_id);
        self.tag_name = Some(tag_name);
    }

    /// 设置边信息
    pub fn set_edge_info(&mut self, edge_type: EdgeType, edge_name: String) {
        self.edge_type = Some(edge_type);
        self.edge_name = Some(edge_name);
    }

    /// 设置属性上下文
    pub fn set_props(&mut self, props: Vec<PropContext>) {
        self.props = Some(props);
    }

    /// 设置过滤标志
    pub fn set_filter_invalid_result_out(&mut self, filter: bool) {
        self.filter_invalid_result_out = filter;
    }

    /// 设置结果状态
    pub fn set_result_stat(&mut self, _stat: ResultStatus) {
        // 简化版本中ResultStatus主要用于返回结果，不存储在上下文中
    }

    /// 重置运行时状态
    pub fn reset(&mut self) {
        self.tag_id = None;
        self.tag_name = None;
        self.edge_type = None;
        self.edge_name = None;
        self.column_idx = 0;
        self.props = None;
        self.filter_invalid_result_out = false;
        self.state = ExecutionState::Initialized;
        self.start_time = None;
        self.error = None;
    }

    /// 开始执行
    pub fn start_execution(&mut self) {
        self.state = ExecutionState::Executing;
        self.start_time = Some(Instant::now());
    }

    /// 完成执行
    pub fn complete_execution(&mut self) {
        self.state = ExecutionState::Completed;
    }

    /// 失败执行
    pub fn fail_execution(&mut self, error: String) {
        self.state = ExecutionState::Failed;
        self.error = Some(error);
    }

    /// 取消执行
    pub fn cancel_execution(&mut self) {
        self.state = ExecutionState::Cancelled;
    }

    /// 获取执行持续时间（毫秒）
    pub fn get_execution_duration_ms(&self) -> u64 {
        match (self.start_time, self.state) {
            (Some(start_time), ExecutionState::Completed) => {
                let elapsed = start_time.elapsed();
                elapsed.as_millis() as u64
            }
            (Some(start_time), _) => {
                let elapsed = start_time.elapsed();
                elapsed.as_millis() as u64
            }
            _ => 0,
        }
    }

    /// 检查是否已终止
    pub fn is_terminated(&self) -> bool {
        matches!(self.state, ExecutionState::Cancelled | ExecutionState::Failed)
    }
}
