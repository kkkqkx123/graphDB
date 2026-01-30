//! 核心查询上下文模块
//!
//! 提供最小化的核心查询上下文，只包含查询执行必需的核心状态。
//! 采用Facade模式拆分QueryContext的职责。

use crate::query::context::execution::{ExecutionContext, ExecutionPlan};
use crate::query::context::validate::ValidationContext;
use crate::query::context::SymbolTable;
use crate::query::context::components::QueryComponents;
use crate::query::context::request_context::RequestContext;
use crate::core::Value;
use crate::graph::utils::IdGenerator;
use crate::utils::ObjectPool;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// 核心查询上下文
///
/// 最小化的查询上下文，只包含：
/// - 验证上下文（ValidationContext）
/// - 执行上下文（ExecutionContext）
/// - 执行计划（ExecutionPlan）
/// - 符号表（SymbolTable）
/// - ID生成器（IdGenerator）
/// - 组件访问器（QueryComponents）
/// - 请求上下文（RequestContext）
/// - 查询终止控制（killed）
///
/// 这个结构体可以在没有外部依赖的情况下创建和测试。
pub struct CoreQueryContext {
    /// 验证上下文
    vctx: ValidationContext,

    /// 执行上下文
    ectx: ExecutionContext,

    /// 执行计划
    plan: Option<ExecutionPlan>,

    /// 符号表
    sym_table: SymbolTable,

    /// ID生成器
    id_gen: IdGenerator,

    /// 对象池
    obj_pool: ObjectPool<Box<dyn std::any::Any>>,

    /// 组件访问器
    components: Option<QueryComponents>,

    /// 请求上下文引用
    rctx: Option<Arc<RequestContext>>,

    /// 查询终止标志
    killed: Arc<AtomicBool>,
}

impl CoreQueryContext {
    /// 创建新的核心查询上下文
    pub fn new() -> Self {
        Self {
            vctx: ValidationContext::new(),
            ectx: ExecutionContext::new(),
            plan: None,
            sym_table: SymbolTable::new(),
            id_gen: IdGenerator::new(0),
            obj_pool: ObjectPool::with_capacity(1000, 1000),
            components: None,
            rctx: None,
            killed: Arc::new(AtomicBool::new(false)),
        }
    }

    /// 获取验证上下文（不可变引用）
    pub fn vctx(&self) -> &ValidationContext {
        &self.vctx
    }

    /// 获取验证上下文（可变引用）
    pub fn vctx_mut(&mut self) -> &mut ValidationContext {
        &mut self.vctx
    }

    /// 获取执行上下文（不可变引用）
    pub fn ectx(&self) -> &ExecutionContext {
        &self.ectx
    }

    /// 获取执行上下文（可变引用）
    pub fn ectx_mut(&mut self) -> &mut ExecutionContext {
        &mut self.ectx
    }

    /// 获取执行计划
    pub fn plan(&self) -> Option<&ExecutionPlan> {
        self.plan.as_ref()
    }

    /// 获取可变执行计划
    pub fn plan_mut(&mut self) -> Option<&mut ExecutionPlan> {
        self.plan.as_mut()
    }

    /// 获取可变执行计划（不推荐使用，返回Option以避免panic）
    #[deprecated(since = "0.1.0", note = "使用 plan_mut() 替代")]
    pub fn plan_option_mut(&mut self) -> &mut Option<ExecutionPlan> {
        &mut self.plan
    }

    /// 设置执行计划
    pub fn set_plan(&mut self, plan: ExecutionPlan) {
        self.plan = Some(plan);
    }

    /// 获取符号表
    pub fn sym_table(&self) -> &SymbolTable {
        &self.sym_table
    }

    /// 获取可变符号表
    pub fn sym_table_mut(&mut self) -> &mut SymbolTable {
        &mut self.sym_table
    }

    /// 生成ID
    pub fn gen_id(&self) -> i64 {
        self.id_gen.id()
    }

    /// 获取当前ID值
    pub fn current_id(&self) -> i64 {
        self.id_gen.current_value()
    }

    /// 获取对象池
    pub fn obj_pool(&self) -> &ObjectPool<Box<dyn std::any::Any>> {
        &self.obj_pool
    }

    /// 获取可变对象池
    pub fn obj_pool_mut(&mut self) -> &mut ObjectPool<Box<dyn std::any::Any>> {
        &mut self.obj_pool
    }

    /// 设置变量值
    pub fn set_variable(&mut self, name: String, value: Value) {
        self.ectx.set_value(name, value);
    }

    /// 获取变量值
    pub fn get_variable(&self, name: &str) -> Option<&Value> {
        self.ectx.get_value(name)
    }

    /// 设置组件访问器
    pub fn set_components(&mut self, components: QueryComponents) {
        self.components = Some(components);
    }

    /// 获取组件访问器（不可变引用）
    pub fn components(&self) -> Option<&QueryComponents> {
        self.components.as_ref()
    }

    /// 获取组件访问器（可变引用）
    pub fn components_mut(&mut self) -> Option<&mut QueryComponents> {
        self.components.as_mut()
    }

    /// 设置请求上下文引用
    pub fn set_request_context(&mut self, rctx: Arc<RequestContext>) {
        self.rctx = Some(rctx);
    }

    /// 获取请求上下文引用（不可变引用）
    pub fn request_context(&self) -> Option<&Arc<RequestContext>> {
        self.rctx.as_ref()
    }

    /// 终止查询
    pub fn kill(&self) {
        self.killed.store(true, Ordering::SeqCst);
    }

    /// 检查查询是否已被终止
    pub fn is_killed(&self) -> bool {
        self.killed.load(Ordering::SeqCst)
    }

    /// 重置上下文
    pub fn reset(&mut self) {
        self.plan = None;
        self.id_gen.reset(0);
        self.obj_pool = ObjectPool::with_capacity(1000, 1000);
        self.ectx.clear();
        self.killed = Arc::new(AtomicBool::new(false));
    }
}

impl Default for CoreQueryContext {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for CoreQueryContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CoreQueryContext")
            .field("vctx", &"ValidationContext")
            .field("ectx", &"ExecutionContext")
            .field("plan", &self.plan.is_some())
            .field("sym_table", &self.sym_table)
            .field("id_gen", &self.id_gen)
            .field("obj_pool_size", &self.obj_pool.size())
            .field("components", &self.components.is_some())
            .field("rctx", &self.rctx.is_some())
            .field("killed", &self.is_killed())
            .finish()
    }
}

impl Clone for CoreQueryContext {
    fn clone(&self) -> Self {
        Self {
            vctx: self.vctx.clone(),
            ectx: self.ectx.clone(),
            plan: self.plan.clone(),
            sym_table: self.sym_table.clone(),
            id_gen: self.id_gen.clone(),
            obj_pool: ObjectPool::with_capacity(1000, 1000),
            components: self.components.clone(),
            rctx: self.rctx.clone(),
            killed: Arc::clone(&self.killed),
        }
    }
}
