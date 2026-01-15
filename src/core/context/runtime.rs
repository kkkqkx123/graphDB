//! 运行时上下文模块
//!
//! 存储层运行时上下文，整合自query/context/runtime_context.rs
//!
//! ## 动态分发优化说明
//! - 存储环境使用泛型参数替代动态分发以获得更好的性能
//! - 运行时上下文中的tag_schema和edge_schema保留动态分发以支持多种schema管理策略

use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::base::ContextType;
use super::traits::BaseContext;
use crate::common::base::id::{EdgeType, TagId};
use crate::core::Value;

/// 结果状态枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResultStatus {
    /// 正常结果
    Normal = 0,
    /// 非法数据
    IllegalData = -1,
    /// 被过滤掉的结果
    FilterOut = -2,
    /// 标签被过滤掉
    TagFilterOut = -3,
}

/// 属性上下文（简化版本）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PropContext {
    pub name: String,
    pub prop_type: String,
    pub nullable: bool,
}

/// 计划上下文（存储层）
/// 存储处理过程中不变的信息
#[derive(Debug, Clone)]
pub struct PlanContext<S, M, I>
where
    S: StorageEngine,
    M: SchemaManager,
    I: IndexManager,
{
    /// 存储环境引用
    pub storage_env: Arc<StorageEnv<S, M, I>>,
    /// 空间ID
    pub space_id: i32,
    /// 会话ID
    pub session_id: i64,
    /// 计划ID
    pub plan_id: i64,
    /// 顶点ID长度
    pub v_id_len: usize,
    /// 是否为整数ID
    pub is_int_id: bool,
    /// 是否为边查询
    pub is_edge: bool,
    /// 默认边版本
    pub default_edge_ver: i64,
    /// 是否被终止
    pub is_killed: bool,
}

/// 存储环境（简化版本）
/// 使用泛型参数替代动态分发以获得更好的性能
#[derive(Debug, Clone)]
pub struct StorageEnv<S, M, I>
where
    S: StorageEngine,
    M: SchemaManager,
    I: IndexManager,
{
    /// 存储引擎
    pub storage_engine: Arc<S>,
    /// Schema管理器
    pub schema_manager: Arc<M>,
    /// 索引管理器
    pub index_manager: Arc<I>,
}

/// 存储引擎trait
pub trait StorageEngine: Send + Sync + std::fmt::Debug {
    // 基本存储操作
    fn insert_node(&mut self, vertex: Vertex) -> Result<Value, StorageError>;
    fn get_node(&self, id: &Value) -> Result<Option<Vertex>, StorageError>;
    fn update_node(&mut self, vertex: Vertex) -> Result<(), StorageError>;
    fn delete_node(&mut self, id: &Value) -> Result<(), StorageError>;

    /// 全表扫描所有顶点
    fn scan_all_vertices(&self) -> Result<Vec<Vertex>, StorageError>;
    /// 按标签扫描顶点
    fn scan_vertices_by_tag(&self, tag: &str) -> Result<Vec<Vertex>, StorageError>;

    fn insert_edge(&mut self, edge: Edge) -> Result<(), StorageError>;
    fn get_edge(
        &self,
        src: &Value,
        dst: &Value,
        edge_type: &str,
    ) -> Result<Option<Edge>, StorageError>;
    fn get_node_edges(
        &self,
        node_id: &Value,
        direction: Direction,
    ) -> Result<Vec<Edge>, StorageError>;
    fn delete_edge(
        &mut self,
        src: &Value,
        dst: &Value,
        edge_type: &str,
    ) -> Result<(), StorageError>;
}

/// Schema管理器trait
pub trait SchemaManager: Send + Sync + std::fmt::Debug {
    fn get_schema(&self, name: &str) -> Option<Schema>;
    fn get_all_schemas(&self) -> Vec<Schema>;
    fn add_schema(&mut self, name: String, schema: Schema);
    fn remove_schema(&mut self, name: &str) -> bool;
}

/// 索引管理器trait
pub trait IndexManager: Send + Sync + std::fmt::Debug {
    fn create_index(&mut self, name: String, schema: Schema) -> Result<(), IndexError>;
    fn drop_index(&mut self, name: &str) -> Result<(), IndexError>;
    fn get_index(&self, name: &str) -> Option<Index>;
}

/// 运行时上下文
/// 存储处理过程中可能变化的信息
///
/// ## 动态分发说明
/// - tag_schema和edge_schema保留动态分发以支持多种schema管理策略
/// - 这是必要的设计选择，因为运行时需要支持不同的schema管理实现
#[derive(Debug, Clone)]
pub struct RuntimeContext<S, M, I>
where
    S: StorageEngine,
    M: SchemaManager,
    I: IndexManager,
{
    /// 上下文ID
    pub id: String,

    /// 计划上下文引用
    pub plan_context: Arc<PlanContext<S, M, I>>,

    /// 标签ID
    pub tag_id: TagId,
    /// 标签名称
    pub tag_name: String,
    /// 标签Schema（可选）
    pub tag_schema: Option<Arc<dyn SchemaManager>>,

    /// 边类型
    pub edge_type: EdgeType,
    /// 边名称
    pub edge_name: String,
    /// 边Schema（可选）
    pub edge_schema: Option<Arc<dyn SchemaManager>>,

    /// 列索引（用于GetNeighbors）
    pub column_idx: usize,
    /// 属性上下文列表（可选）
    pub props: Option<Vec<PropContext>>,

    /// 是否为插入操作
    pub insert: bool,
    /// 是否过滤无效结果
    pub filter_invalid_result_out: bool,
    /// 结果状态
    pub result_stat: ResultStatus,

    /// 创建时间
    pub created_at: std::time::SystemTime,

    /// 最后更新时间
    pub updated_at: std::time::SystemTime,

    /// 是否有效
    pub valid: bool,
}

impl<S, M, I> RuntimeContext<S, M, I>
where
    S: StorageEngine,
    M: SchemaManager,
    I: IndexManager,
{
    /// 创建新的运行时上下文
    pub fn new(id: String, plan_context: Arc<PlanContext<S, M, I>>) -> Self {
        let now = std::time::SystemTime::now();
        Self {
            id,
            plan_context,
            tag_id: TagId::new(0),
            tag_name: String::new(),
            tag_schema: None,
            edge_type: EdgeType::new(0),
            edge_name: String::new(),
            edge_schema: None,
            column_idx: 0,
            props: None,
            insert: false,
            filter_invalid_result_out: false,
            result_stat: ResultStatus::Normal,
            created_at: now,
            updated_at: now,
            valid: true,
        }
    }

    /// 获取存储环境
    pub fn env(&self) -> &Arc<StorageEnv<S, M, I>> {
        &self.plan_context.storage_env
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

    /// 检查计划是否被终止
    pub fn is_plan_killed(&self) -> bool {
        self.plan_context.is_killed
    }

    /// 设置标签信息
    pub fn set_tag_info(
        &mut self,
        tag_id: TagId,
        tag_name: String,
        tag_schema: Option<Arc<dyn SchemaManager>>,
    ) {
        self.tag_id = tag_id;
        self.tag_name = tag_name;
        self.tag_schema = tag_schema;
        self.updated_at = std::time::SystemTime::now();
    }

    /// 设置边信息
    pub fn set_edge_info(
        &mut self,
        edge_type: EdgeType,
        edge_name: String,
        edge_schema: Option<Arc<dyn SchemaManager>>,
    ) {
        self.edge_type = edge_type;
        self.edge_name = edge_name;
        self.edge_schema = edge_schema;
        self.updated_at = std::time::SystemTime::now();
    }

    /// 设置属性上下文
    pub fn set_props(&mut self, props: Vec<PropContext>) {
        self.props = Some(props);
        self.updated_at = std::time::SystemTime::now();
    }

    /// 设置插入标志
    pub fn set_insert(&mut self, insert: bool) {
        self.insert = insert;
        self.updated_at = std::time::SystemTime::now();
    }

    /// 设置过滤标志
    pub fn set_filter_invalid_result_out(&mut self, filter: bool) {
        self.filter_invalid_result_out = filter;
        self.updated_at = std::time::SystemTime::now();
    }

    /// 设置结果状态
    pub fn set_result_stat(&mut self, stat: ResultStatus) {
        self.result_stat = stat;
        self.updated_at = std::time::SystemTime::now();
    }

    /// 重置运行时状态（保留计划上下文）
    pub fn reset(&mut self) {
        self.tag_id = TagId::new(0);
        self.tag_name.clear();
        self.tag_schema = None;
        self.edge_type = EdgeType::new(0);
        self.edge_name.clear();
        self.edge_schema = None;
        self.column_idx = 0;
        self.props = None;
        self.insert = false;
        self.filter_invalid_result_out = false;
        self.result_stat = ResultStatus::Normal;
        self.updated_at = std::time::SystemTime::now();
    }
}

impl<S, M, I> BaseContext for RuntimeContext<S, M, I>
where
    S: StorageEngine,
    M: SchemaManager,
    I: IndexManager,
{
    fn id(&self) -> &str {
        &self.id
    }

    fn context_type(&self) -> ContextType {
        ContextType::Runtime
    }

    fn created_at(&self) -> std::time::SystemTime {
        self.created_at
    }

    fn updated_at(&self) -> std::time::SystemTime {
        self.updated_at
    }

    fn is_valid(&self) -> bool {
        self.valid
    }

    fn touch(&mut self) {
        self.updated_at = std::time::SystemTime::now();
    }

    fn invalidate(&mut self) {
        self.valid = false;
        self.updated_at = std::time::SystemTime::now();
    }

    fn revalidate(&mut self) -> bool {
        self.valid = true;
        self.updated_at = std::time::SystemTime::now();
        true
    }

    fn parent_id(&self) -> Option<&str> {
        None
    }

    fn depth(&self) -> usize {
        2
    }
}

// 类型别名和简化定义
pub type StorageError = String;
pub type IndexError = String;
pub type Schema = crate::core::schema::Schema;
pub type Index = crate::core::schema::Schema;
pub type Vertex = crate::core::vertex_edge_path::Vertex;
pub type Edge = crate::core::vertex_edge_path::Edge;
pub type Direction = crate::core::vertex_edge_path::Direction;

// 默认存储环境类型别名，使用项目中实际的实现类型
pub type DefaultStorageEnv = StorageEnv<
    crate::storage::rocksdb_storage::RocksDBStorage,
    crate::query::context::managers::MemorySchemaManager,
    crate::query::context::managers::MemoryIndexManager,
>;

// 默认计划上下文类型别名
pub type DefaultPlanContext = PlanContext<
    crate::storage::rocksdb_storage::RocksDBStorage,
    crate::query::context::managers::MemorySchemaManager,
    crate::query::context::managers::MemoryIndexManager,
>;

// 默认运行时上下文类型别名
pub type DefaultRuntimeContext = RuntimeContext<
    crate::storage::rocksdb_storage::RocksDBStorage,
    crate::query::context::managers::MemorySchemaManager,
    crate::query::context::managers::MemoryIndexManager,
>;

// 测试运行时上下文类型别名
pub type TestRuntimeContext = RuntimeContext<
    crate::core::context::manager::MockStorageEngine,
    crate::core::context::manager::MockSchemaManager,
    crate::core::context::manager::MockIndexManager,
>;
