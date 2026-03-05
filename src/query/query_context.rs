//! 查询上下文
//!
//! 管理查询从解析、验证、规划到执行整个生命周期中的上下文信息。
//!
//! # 重构说明
//!
//! 表达式上下文已合并到 Ast 中，不再在 QueryContext 中单独存储。
//! 通过 ValidatedStatement 访问表达式上下文。
//!
//! # 架构优化
//!
//! QueryContext 现在由多个专门的上下文组成：
//! - QueryRequestContext: 查询请求上下文（会话信息、请求参数）
//! - QueryExecutionState: 查询执行状态（执行计划、终止标志）
//! - QueryResourceContext: 查询资源上下文（对象池、ID 生成器、符号表）
//! - QuerySpaceContext: 查询空间上下文（空间信息、字符集）

use std::sync::Arc;

use crate::query::execution::{
    QueryExecutionState, QueryResourceContext, QuerySpaceContext,
};
use crate::query::query_request_context::QueryRequestContext;

/// 查询上下文
///
/// 每个查询请求的上下文，从查询引擎接收到查询请求时创建。
/// 该上下文对象对解析器、规划器、优化器和执行器可见。
///
/// # 职责
///
/// - 持有查询请求上下文（会话信息、请求参数）
/// - 持有查询执行状态（执行计划、终止标志）
/// - 持有查询资源上下文（对象池、ID 生成器、符号表）
/// - 持有查询空间上下文（空间信息、字符集）
///
/// # 设计变更
///
/// - 使用组合模式，将 QueryContext 拆分为多个专门的上下文
/// - 删除 expr_context 字段，表达式上下文现在存储在 Ast 中
/// - 删除 Clone 实现，强制使用 Arc<QueryContext>
/// - 删除 validation_info 字段，验证信息现在只存储在 ValidatedStatement 中
pub struct QueryContext {
    /// 查询请求上下文
    rctx: Arc<QueryRequestContext>,

    /// 查询执行状态
    execution_state: QueryExecutionState,

    /// 查询资源上下文
    resource_context: QueryResourceContext,

    /// 查询空间上下文
    space_context: QuerySpaceContext,
}

impl QueryContext {
    /// 创建新的查询上下文
    pub fn new(rctx: Arc<QueryRequestContext>) -> Self {
        Self {
            rctx,
            execution_state: QueryExecutionState::new(),
            resource_context: QueryResourceContext::new(),
            space_context: QuerySpaceContext::new(),
        }
    }

    /// 创建用于验证的临时上下文
    ///
    /// 这是一个便捷方法，用于在验证阶段创建临时的 QueryContext。
    ///
    /// # 参数
    /// - `query_text`: 查询文本
    ///
    /// # 示例
    ///
    /// ```rust
    /// use crate::query::QueryContext;
    ///
    /// let qctx = QueryContext::new_for_validation("MATCH (n) RETURN n".to_string());
    /// ```
    pub fn new_for_validation(query_text: String) -> Self {
        let rctx = Arc::new(QueryRequestContext::new(query_text));
        Self::new(rctx)
    }

    /// 创建用于规划的临时上下文
    ///
    /// 这是一个便捷方法，用于在规划阶段创建临时的 QueryContext。
    ///
    /// # 参数
    /// - `query_text`: 查询文本
    ///
    /// # 示例
    ///
    /// ```rust
    /// use crate::query::QueryContext;
    ///
    /// let qctx = QueryContext::new_for_planning("MATCH (n) RETURN n".to_string());
    /// ```
    pub fn new_for_planning(query_text: String) -> Self {
        let rctx = Arc::new(QueryRequestContext::new(query_text));
        Self::new(rctx)
    }

    /// 从各个组件创建查询上下文（供 Builder 使用）
    pub(crate) fn from_components(
        rctx: Arc<QueryRequestContext>,
        execution_state: QueryExecutionState,
        resource_context: QueryResourceContext,
        space_context: QuerySpaceContext,
    ) -> Self {
        Self {
            rctx,
            execution_state,
            resource_context,
            space_context,
        }
    }

    /// 创建构建器
    pub fn builder(rctx: Arc<QueryRequestContext>) -> crate::query::query_context_builder::QueryContextBuilder {
        crate::query::query_context_builder::QueryContextBuilder::new(rctx)
    }

    /// 获取查询请求上下文
    pub fn request_context(&self) -> &QueryRequestContext {
        &self.rctx
    }

    /// 获取查询请求上下文的 Arc 引用
    pub fn request_context_arc(&self) -> Arc<QueryRequestContext> {
        self.rctx.clone()
    }

    /// 获取查询请求上下文（兼容旧代码）
    pub fn rctx(&self) -> &QueryRequestContext {
        &self.rctx
    }

    /// 获取执行计划
    pub fn plan(&self) -> Option<crate::query::planner::plan::ExecutionPlan> {
        self.execution_state.plan()
    }

    /// 设置执行计划
    pub fn set_plan(&mut self, plan: crate::query::planner::plan::ExecutionPlan) {
        self.execution_state.set_plan(plan);
    }

    /// 获取执行计划 ID
    pub fn plan_id(&self) -> Option<i64> {
        self.execution_state.plan_id()
    }

    /// 获取字符集信息
    pub fn charset_info(&self) -> Option<&crate::core::types::CharsetInfo> {
        self.space_context.charset_info()
    }

    /// 设置字符集信息
    pub fn set_charset_info(&mut self, charset_info: crate::core::types::CharsetInfo) {
        self.space_context.set_charset_info(charset_info);
    }

    /// 获取对象池
    pub fn obj_pool(&self) -> &crate::utils::ObjectPool<String> {
        self.resource_context.obj_pool()
    }

    /// 获取可变对象池
    pub fn obj_pool_mut(&mut self) -> &mut crate::utils::ObjectPool<String> {
        self.resource_context.obj_pool_mut()
    }

    /// 生成 ID
    pub fn gen_id(&self) -> i64 {
        self.resource_context.gen_id()
    }

    /// 获取当前 ID 值（不递增）
    pub fn current_id(&self) -> i64 {
        self.resource_context.current_id()
    }

    /// 获取符号表
    pub fn sym_table(&self) -> &crate::core::SymbolTable {
        self.resource_context.sym_table()
    }

    /// 获取符号表的 Arc 引用
    pub fn sym_table_arc(&self) -> Arc<crate::core::SymbolTable> {
        self.resource_context.sym_table_arc()
    }

    /// 获取当前空间信息
    pub fn space_info(&self) -> Option<&crate::core::types::SpaceInfo> {
        self.space_context.space_info()
    }

    /// 设置当前空间信息
    pub fn set_space_info(&mut self, space_info: crate::core::types::SpaceInfo) {
        self.space_context.set_space_info(space_info);
    }

    /// 获取当前空间的 ID
    pub fn space_id(&self) -> Option<u64> {
        self.space_context.space_id()
    }

    /// 获取当前空间的名称
    pub fn space_name(&self) -> Option<String> {
        self.space_context.space_name()
    }

    /// 标记为已终止
    pub fn mark_killed(&self) {
        self.execution_state.mark_killed();
    }

    /// 检查是否被终止
    pub fn is_killed(&self) -> bool {
        self.execution_state.is_killed()
    }

    /// 检查参数是否存在
    pub fn exist_parameter(&self, param: &str) -> bool {
        self.rctx.get_parameter(param).is_some()
    }

    /// 获取查询字符串
    pub fn query(&self) -> &str {
        &self.rctx.query
    }

    /// 获取参数
    pub fn parameters(&self) -> &std::collections::HashMap<String, crate::core::Value> {
        &self.rctx.parameters
    }

    /// 重置查询上下文
    pub fn reset(&mut self) {
        self.execution_state.reset();
        self.resource_context.reset();
        self.space_context.reset();
        log::info!("查询上下文已重置");
    }

    /// 获取查询执行状态的引用
    pub fn execution_state(&self) -> &QueryExecutionState {
        &self.execution_state
    }

    /// 获取查询执行状态的可变引用
    pub fn execution_state_mut(&mut self) -> &mut QueryExecutionState {
        &mut self.execution_state
    }

    /// 获取查询资源上下文的引用
    pub fn resource_context(&self) -> &QueryResourceContext {
        &self.resource_context
    }

    /// 获取查询资源上下文的可变引用
    pub fn resource_context_mut(&mut self) -> &mut QueryResourceContext {
        &mut self.resource_context
    }

    /// 获取查询空间上下文的引用
    pub fn space_context(&self) -> &QuerySpaceContext {
        &self.space_context
    }

    /// 获取查询空间上下文的可变引用
    pub fn space_context_mut(&mut self) -> &mut QuerySpaceContext {
        &mut self.space_context
    }
}

impl std::fmt::Debug for QueryContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QueryContext")
            .field("rctx", &self.rctx)
            .field("plan_id", &self.plan_id())
            .field("space_id", &self.space_id())
            .field("killed", &self.is_killed())
            .finish()
    }
}

impl Default for QueryContext {
    fn default() -> Self {
        Self::new(Arc::new(QueryRequestContext::default()))
    }
}
