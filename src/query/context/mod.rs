//! 查询上下文模块
//!
//! 管理查询从解析、验证、规划到执行整个生命周期中的上下文信息。
//!
//! # 模块结构
//!
//! - `ast/` - AST 查询上下文
//! - `validate/` - 验证上下文
//! - `symbol/` - 符号表管理
//! - `components.rs` - 组件访问器
//!
//! # 核心类型
//!
//! - [`QueryContext`] - 查询上下文，聚合查询执行所需的所有资源
//! - [`SymbolTable`] - 符号表，管理查询变量
//!
//! # 注意
//!
//! - `RequestContext` 已迁移到 `api::session::RequestContext`
//! - `RuntimeContext` 已迁移到 `storage::RuntimeContext`

pub mod ast;

// 新的模块结构
pub mod symbol;

// 新的重构模块
pub mod components;

// 重新导出主要类型
pub use ast::*;

// 导出新的模块结构
pub use symbol::{Symbol, SymbolTable};

// 导出重构的模块
pub use components::{ComponentAccessor, QueryComponents};

// 导出核心执行状态类型（推荐）
pub use crate::query::core::{ExecutorState, RowStatus};

// ==================== QueryContext 定义 ====================

use crate::api::session::RequestContext;
use crate::query::planner::plan::ExecutionPlan;
use crate::storage::StorageClient;
use crate::storage::metadata::SchemaManager;
use crate::storage::metadata::IndexMetadataManager;
use crate::utils::{ObjectPool, IdGenerator};
use crate::core::types::CharsetInfo;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

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
///
/// # 注意
///
/// 原 `QueryExecutionContext` 已删除，变量管理功能由 `ExecutionContext` 统一负责。
/// 查询参数通过 `RequestContext` 传递。
pub struct QueryContext {
    /// 请求上下文
    rctx: Option<Arc<RequestContext>>,

    /// 执行计划
    plan: Option<Box<ExecutionPlan>>,

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

    /// 符号表
    sym_table: SymbolTable,

    /// 是否被标记为已终止
    killed: AtomicBool,
}

impl QueryContext {
    /// 创建新的查询上下文
    pub fn new() -> Self {
        Self {
            rctx: None,
            plan: None,
            schema_manager: None,
            index_metadata_manager: None,
            storage_client: None,
            charset_info: None,
            obj_pool: ObjectPool::new(1000),
            id_gen: IdGenerator::new(0),
            sym_table: SymbolTable::new(),
            killed: AtomicBool::new(false),
        }
    }

    /// 使用请求上下文创建查询上下文
    pub fn with_request_context(rctx: Arc<RequestContext>) -> Self {
        let mut ctx = Self::new();
        ctx.rctx = Some(rctx);
        ctx
    }

    /// 设置请求上下文
    pub fn set_rctx(&mut self, rctx: Arc<RequestContext>) {
        self.rctx = Some(rctx);
    }

    /// 获取请求上下文
    pub fn rctx(&self) -> Option<&RequestContext> {
        self.rctx.as_deref()
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
    pub fn plan(&self) -> Option<&ExecutionPlan> {
        self.plan.as_ref().map(|p| p.as_ref())
    }

    /// 获取可变执行计划
    pub fn plan_mut(&mut self) -> &mut Option<Box<ExecutionPlan>> {
        &mut self.plan
    }

    /// 设置执行计划
    pub fn set_plan(&mut self, plan: ExecutionPlan) {
        self.plan = Some(Box::new(plan));
    }

    /// 获取执行计划 ID
    pub fn plan_id(&self) -> Option<i64> {
        self.plan.as_ref().map(|p| p.id)
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
    pub fn get_storage_client(&self) -> Option<&Arc<dyn StorageClient>> {
        self.storage_client.as_ref()
    }

    /// 获取字符集信息
    pub fn get_charset_info(&self) -> Option<&CharsetInfo> {
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

    /// 获取可变符号表
    pub fn sym_table_mut(&mut self) -> &mut SymbolTable {
        &mut self.sym_table
    }

    /// 获取当前空间的 ID
    ///
    /// 注意：空间信息现在存储在 AstContext 中，此方法暂时返回 0
    /// 实际使用时请从 AstContext.space().space_id 获取
    pub fn space_id(&self) -> u64 {
        0
    }

    /// 标记为部分成功
    pub fn set_partial_success(&mut self) {
        if let Some(rctx) = &self.rctx {
            let _ = rctx.set_response_error("Partial success".to_string());
        }
        log::warn!("Query marked as partial success");
    }

    /// 标记为已终止
    pub fn mark_killed(&self) {
        self.killed.store(true, Ordering::SeqCst);
        log::info!("Query context marked as killed");
    }

    /// 检查是否被终止
    pub fn is_killed(&self) -> bool {
        self.killed.load(Ordering::SeqCst)
    }

    /// 检查参数是否存在
    ///
    /// 这仅在构建阶段有效！用于检查查询参数是否已设置
    pub fn exist_parameter(&self, param: &str) -> bool {
        if let Some(rctx) = &self.rctx {
            rctx.get_parameter(param).is_some()
        } else {
            false
        }
    }

    /// 获取请求参数
    pub fn request_params(&self) -> Option<crate::api::session::RequestParams> {
        self.rctx.as_ref().map(|r| r.request_params())
    }

    /// 重置查询上下文
    ///
    /// 清除所有状态，重置为初始状态。用于重用查询上下文对象。
    pub fn reset(&mut self) {
        self.plan = None;
        self.killed.store(false, Ordering::SeqCst);
        self.id_gen.reset(0);
        log::info!("Query context reset");
    }
}

impl Default for QueryContext {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for QueryContext {
    fn clone(&self) -> Self {
        Self {
            rctx: self.rctx.clone(),
            plan: self.plan.clone(),
            schema_manager: self.schema_manager.clone(),
            index_metadata_manager: self.index_metadata_manager.clone(),
            storage_client: self.storage_client.clone(),
            charset_info: self.charset_info.clone(),
            obj_pool: self.obj_pool.clone(),
            id_gen: IdGenerator::new(self.id_gen.current_value()),
            sym_table: self.sym_table.clone(),
            killed: AtomicBool::new(self.killed.load(Ordering::SeqCst)),
        }
    }
}

impl std::fmt::Debug for QueryContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QueryContext")
            .field("rctx", &self.rctx.is_some())
            .field("plan", &self.plan.is_some())
            .field("schema_manager", &self.schema_manager.is_some())
            .field("index_metadata_manager", &self.index_metadata_manager.is_some())
            .field("storage_client", &self.storage_client.is_some())
            .field("charset_info", &self.charset_info.is_some())
            .field("obj_pool", &self.obj_pool)
            .field("id_gen", &self.id_gen)
            .field("sym_table", &self.sym_table)
            .field("killed", &self.killed)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_context_creation() {
        let ctx = QueryContext::new();

        assert_eq!(ctx.gen_id(), 0);
        assert_eq!(ctx.gen_id(), 1);

        assert!(!ctx.exist_parameter("non_existent_param"));

        assert!(!ctx.is_killed());
        ctx.mark_killed();
        assert!(ctx.is_killed());
    }

    #[test]
    fn test_context_access() {
        let mut ctx = QueryContext::new();

        ctx.sym_table_mut()
            .new_variable("test_var")
            .expect("Failed to create new variable");
        assert!(ctx.sym_table().has_variable("test_var"));
    }

    #[test]
    fn test_with_request_context() {
        let request_ctx = Arc::new(RequestContext::with_session(
            "MATCH (n) RETURN n".to_string(),
            "test_session",
            "test_user",
            "127.0.0.1",
            0,
        ));

        let ctx = QueryContext::with_request_context(request_ctx.clone());

        assert!(ctx.rctx().is_some());
        assert_eq!(
            ctx.rctx().expect("Request context should exist").query(),
            "MATCH (n) RETURN n"
        );
    }
}
