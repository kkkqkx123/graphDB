//! 查询上下文
//!
//! 管理查询从解析、验证、规划到执行整个生命周期中的上下文信息。

use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicBool, Ordering};

use crate::query::request_context::RequestContext;
use crate::core::types::SpaceInfo;
use crate::core::SymbolTable;
use crate::query::planner::plan::ExecutionPlan;
use crate::storage::metadata::{SchemaManager, IndexMetadataManager};
use crate::storage::StorageClient;
use crate::utils::{ObjectPool, IdGenerator};
use crate::core::types::CharsetInfo;

/// 查询上下文
///
/// 每个查询请求的上下文，从查询引擎接收到查询请求时创建。
/// 该上下文对象对解析器、规划器、优化器和执行器可见。
///
/// # 职责
///
/// - 持有请求上下文（会话信息、请求参数）
/// - 持有执行计划
/// - 持有资源访问器（SchemaManager、StorageClient 等）
/// - 持有工具（对象池、ID 生成器、符号表）
/// - 持有当前空间信息
///
/// # 设计变更
///
/// - 使用 Arc<RwLock<SymbolTable>> 替代直接的 SymbolTable，支持并发访问
/// - 添加 space_info 字段，替代 AstContext 中的 space 字段
/// - 删除 Clone 实现，强制使用 Arc<QueryContext>
pub struct QueryContext {
    /// 请求上下文
    rctx: Arc<RequestContext>,

    /// 执行计划
    plan: RwLock<Option<Box<ExecutionPlan>>>,

    /// 模式管理器
    schema_manager: Option<Arc<dyn SchemaManager>>,

    /// 索引元数据管理器
    index_metadata_manager: Option<Arc<dyn IndexMetadataManager>>,

    /// 存储客户端
    storage_client: Option<Arc<dyn StorageClient>>,

    /// 字符集信息
    charset_info: Option<Box<CharsetInfo>>,

    /// 对象池
    obj_pool: ObjectPool<String>,

    /// ID 生成器
    id_gen: IdGenerator,

    /// 符号表 - 使用 Arc<SymbolTable>，内部 DashMap 已提供并发安全
    sym_table: Arc<SymbolTable>,

    /// 当前空间信息
    space_info: RwLock<Option<SpaceInfo>>,

    /// 是否被标记为已终止
    killed: AtomicBool,
}

impl QueryContext {
    /// 创建新的查询上下文
    pub fn new(rctx: Arc<RequestContext>) -> Self {
        Self {
            rctx,
            plan: RwLock::new(None),
            schema_manager: None,
            index_metadata_manager: None,
            storage_client: None,
            charset_info: None,
            obj_pool: ObjectPool::new(1000),
            id_gen: IdGenerator::new(0),
            sym_table: Arc::new(SymbolTable::new()),
            space_info: RwLock::new(None),
            killed: AtomicBool::new(false),
        }
    }

    /// 获取请求上下文
    pub fn request_context(&self) -> &RequestContext {
        &self.rctx
    }

    /// 获取请求上下文的 Arc 引用
    pub fn request_context_arc(&self) -> Arc<RequestContext> {
        self.rctx.clone()
    }

    /// 获取请求上下文（兼容旧代码）
    pub fn rctx(&self) -> &RequestContext {
        &self.rctx
    }

    /// 设置模式管理器
    pub fn set_schema_manager(&mut self, sm: Arc<dyn SchemaManager>) {
        self.schema_manager = Some(sm);
    }

    /// 设置索引元数据管理器
    pub fn set_index_metadata_manager(&mut self, imm: Arc<dyn IndexMetadataManager>) {
        self.index_metadata_manager = Some(imm);
    }

    /// 设置存储客户端
    pub fn set_storage_client(&mut self, storage: Arc<dyn StorageClient>) {
        self.storage_client = Some(storage);
    }

    /// 设置字符集信息
    pub fn set_charset_info(&mut self, charset_info: CharsetInfo) {
        self.charset_info = Some(Box::new(charset_info));
    }

    /// 获取执行计划
    pub fn plan(&self) -> Option<ExecutionPlan> {
        self.plan.read().ok()?.as_ref().map(|p| *p.clone())
    }

    /// 设置执行计划
    pub fn set_plan(&self, plan: ExecutionPlan) {
        if let Ok(mut guard) = self.plan.write() {
            *guard = Some(Box::new(plan));
        }
    }

    /// 获取执行计划 ID
    pub fn plan_id(&self) -> Option<i64> {
        self.plan.read().ok()?.as_ref().map(|p| p.id)
    }

    /// 获取模式管理器
    pub fn schema_manager(&self) -> Option<&Arc<dyn SchemaManager>> {
        self.schema_manager.as_ref()
    }

    /// 获取索引元数据管理器
    pub fn index_metadata_manager(&self) -> Option<&Arc<dyn IndexMetadataManager>> {
        self.index_metadata_manager.as_ref()
    }

    /// 获取存储客户端
    pub fn storage_client(&self) -> Option<&Arc<dyn StorageClient>> {
        self.storage_client.as_ref()
    }

    /// 获取字符集信息
    pub fn charset_info(&self) -> Option<&CharsetInfo> {
        self.charset_info.as_ref().map(|ci| ci.as_ref())
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

    /// 获取当前空间信息
    pub fn space_info(&self) -> Option<SpaceInfo> {
        self.space_info.read().ok()?.clone()
    }

    /// 设置当前空间信息
    pub fn set_space_info(&self, space_info: SpaceInfo) {
        if let Ok(mut guard) = self.space_info.write() {
            *guard = Some(space_info);
        }
    }

    /// 获取当前空间的 ID
    pub fn space_id(&self) -> Option<u64> {
        self.space_info().map(|s| s.space_id)
    }

    /// 获取当前空间的名称
    pub fn space_name(&self) -> Option<String> {
        self.space_info().map(|s| s.space_name)
    }

    /// 标记为已终止
    pub fn mark_killed(&self) {
        self.killed.store(true, Ordering::SeqCst);
        log::info!("查询上下文被标记为已终止");
    }

    /// 检查是否被终止
    pub fn is_killed(&self) -> bool {
        self.killed.load(Ordering::SeqCst)
    }

    /// 检查参数是否存在
    pub fn exist_parameter(&self, param: &str) -> bool {
        self.rctx.get_parameter(param).is_some()
    }

    /// 获取请求参数
    pub fn request_params(&self) -> crate::query::request_context::RequestParams {
        self.rctx.request_params().clone()
    }

    /// 重置查询上下文
    pub fn reset(&self) {
        if let Ok(mut guard) = self.plan.write() {
            *guard = None;
        }
        self.killed.store(false, Ordering::SeqCst);
        self.id_gen.reset(0);
        log::info!("查询上下文已重置");
    }
}

impl std::fmt::Debug for QueryContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QueryContext")
            .field("rctx", &self.rctx)
            .field("plan_id", &self.plan_id())
            .field("space_id", &self.space_id())
            .field("killed", &self.killed)
            .finish()
    }
}

impl Default for QueryContext {
    fn default() -> Self {
        Self::new(Arc::new(RequestContext::default()))
    }
}
