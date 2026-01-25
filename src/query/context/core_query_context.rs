//! 核心查询上下文模块
//!
//! 提供最小化的核心查询上下文，只包含查询执行必需的核心状态。
//! 采用Facade模式拆分QueryContext的职责。

use crate::query::context::execution::{ExecutionContext, ExecutionPlan};
use crate::query::context::validate::ValidationContext;
use crate::query::context::SymbolTable;
use crate::core::Value;
use crate::graph::utils::IdGenerator;
use crate::utils::ObjectPool;

/// 核心查询上下文
///
/// 最小化的查询上下文，只包含：
/// - 验证上下文（ValidationContext）
/// - 执行上下文（ExecutionContext）
/// - 执行计划（ExecutionPlan）
/// - 符号表（SymbolTable）
/// - ID生成器（IdGenerator）
///
/// 这个结构体可以在没有外部依赖的情况下创建和测试。
#[derive(Debug, Clone)]
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
    obj_pool: ObjectPool<String>,
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
            obj_pool: ObjectPool::new(1000),
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
    pub fn plan_mut(&mut self) -> &mut Option<ExecutionPlan> {
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
    pub fn obj_pool(&self) -> &ObjectPool<String> {
        &self.obj_pool
    }

    /// 获取可变对象池
    pub fn obj_pool_mut(&mut self) -> &mut ObjectPool<String> {
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

    /// 重置上下文
    pub fn reset(&mut self) {
        self.plan = None;
        self.id_gen.reset(0);
        self.obj_pool = ObjectPool::new(1000);
        self.ectx.clear();
    }
}

impl Default for CoreQueryContext {
    fn default() -> Self {
        Self::new()
    }
}
