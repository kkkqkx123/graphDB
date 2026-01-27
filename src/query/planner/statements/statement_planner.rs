//! 语句级规划器
//!
//! 提供语句级规划器的统一接口，处理完整语句的规划逻辑。
//! 架构：Planner trait -> StatementPlanner trait -> ClausePlanner
//!
//! ## 架构设计
//!
//! - **Planner**：基础 trait，定义规划器的通用接口
//! - **StatementPlanner**：语句级 trait，处理完整语句的规划
//! - **ClausePlanner**：子句级 trait，处理单个子句的规划

use crate::query::context::ast::AstContext;
use crate::query::context::execution::QueryContext;
use crate::query::planner::plan::ExecutionPlan;
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::{Planner, PlannerError};
use crate::query::validator::structs::CypherClauseKind;
use std::collections::HashMap;

/// 语句级规划器 trait
///
/// 定义语句级规划器的统一接口，封装完整语句的规划逻辑。
/// 组合多个子句规划器来完成语句的规划。
pub trait StatementPlanner: Planner {
    /// 获取语句类型
    fn statement_type(&self) -> &'static str;

    /// 获取支持的子句类型列表
    fn supported_clause_kinds(&self) -> Vec<CypherClauseKind>;

    /// 从 AST 上下文中提取子句列表
    fn extract_clauses(&self, ast_ctx: &AstContext) -> Vec<CypherClauseKind>;

    /// 创建语句规划器
    fn make_statement_planner() -> Box<dyn StatementPlanner>
    where
        Self: Sized;

    /// 使用子句规划器列表处理语句
    fn plan_with_clause_planners(
        &self,
        query_context: &mut QueryContext,
        ast_ctx: &AstContext,
        clause_planners: &[(CypherClauseKind, Box<dyn ClausePlanner>)],
    ) -> Result<ExecutionPlan, PlannerError> {
        let clauses = self.extract_clauses(ast_ctx);
        if clauses.is_empty() {
            return self.create_default_plan(ast_ctx);
        }

        let mut current_plan = self.create_initial_plan(ast_ctx)?;

        for clause_kind in clauses {
            if let Some((_, planner)) = clause_planners
                .iter()
                .find(|(kind, _)| *kind == clause_kind)
            {
                current_plan = planner.transform_clause(
                    query_context,
                    ast_ctx,
                    current_plan,
                )?;
            }
        }

        self.finalize_plan(current_plan, ast_ctx)
    }

    /// 创建初始计划
    fn create_initial_plan(&self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError>;

    /// 创建默认计划（无子句时）
    fn create_default_plan(&self, ast_ctx: &AstContext) -> Result<ExecutionPlan, PlannerError>;

    /// 完成计划生成
    fn finalize_plan(
        &self,
        plan: SubPlan,
        _ast_ctx: &AstContext,
    ) -> Result<ExecutionPlan, PlannerError> {
        let root_node = plan.root().clone();
        let mut execution_plan = ExecutionPlan::new(root_node);
        self.set_plan_id(&mut execution_plan);
        Ok(execution_plan)
    }

    /// 设置计划 ID
    fn set_plan_id(&self, plan: &mut ExecutionPlan) {
        let uuid = uuid::Uuid::new_v4();
        let uuid_bytes = uuid.as_bytes();
        let id = i64::from_ne_bytes([
            uuid_bytes[0],
            uuid_bytes[1],
            uuid_bytes[2],
            uuid_bytes[3],
            uuid_bytes[4],
            uuid_bytes[5],
            uuid_bytes[6],
            uuid_bytes[7],
        ]);
        plan.set_id(id);
    }
}

/// 子句级规划器 trait
///
/// 定义子句级规划器的统一接口，处理单个子句的规划逻辑。
pub trait ClausePlanner: std::fmt::Debug {
    /// 获取子句类型
    fn clause_kind(&self) -> CypherClauseKind;

    /// 获取规划器名称
    fn name(&self) -> &'static str;

    /// 转换子句为核心计划
    fn transform_clause(
        &self,
        query_context: &mut QueryContext,
        ast_ctx: &AstContext,
        input_plan: SubPlan,
    ) -> Result<SubPlan, PlannerError>;

    /// 检查是否支持该子句
    fn supports(&self, clause_kind: CypherClauseKind) -> bool {
        clause_kind == self.clause_kind()
    }
}

/// 基础语句规划器
///
/// 提供语句规划器的通用实现
#[derive(Debug)]
pub struct BaseStatementPlanner {
    statement_type: &'static str,
    supported_clauses: Vec<CypherClauseKind>,
}

impl BaseStatementPlanner {
    pub fn new(statement_type: &'static str, supported_clauses: Vec<CypherClauseKind>) -> Self {
        Self {
            statement_type,
            supported_clauses,
        }
    }

    pub fn statement_type(&self) -> &'static str {
        self.statement_type
    }

    pub fn supported_clause_kinds(&self) -> &[CypherClauseKind] {
        &self.supported_clauses
    }
}

/// 规划器注册表
///
/// 管理和注册语句级和子句级规划器
#[derive(Debug)]
pub struct PlannerRegistry {
    statement_planners: HashMap<String, Box<dyn StatementPlanner>>,
    clause_planners: HashMap<CypherClauseKind, Box<dyn ClausePlanner>>,
}

impl Default for PlannerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl PlannerRegistry {
    pub fn new() -> Self {
        Self {
            statement_planners: HashMap::new(),
            clause_planners: HashMap::new(),
        }
    }

    pub fn register_statement_planner<S: StatementPlanner + 'static>(
        &mut self,
        statement_type: &str,
        planner: S,
    ) {
        self.statement_planners
            .insert(statement_type.to_string(), Box::new(planner));
    }

    pub fn register_clause_planner<C: ClausePlanner + 'static>(&mut self, planner: C) {
        self.clause_planners
            .insert(planner.clause_kind(), Box::new(planner));
    }

    pub fn get_statement_planner(&self, statement_type: &str) -> Option<&Box<dyn StatementPlanner>> {
        self.statement_planners.get(statement_type)
    }

    pub fn get_clause_planner(
        &self,
        clause_kind: CypherClauseKind,
    ) -> Option<&Box<dyn ClausePlanner>> {
        self.clause_planners.get(&clause_kind)
    }

    pub fn get_all_clause_planners(&self) -> Vec<(CypherClauseKind, &dyn ClausePlanner)> {
        self.clause_planners
            .iter()
            .map(|(kind, planner)| (*kind, planner.as_ref()))
            .collect()
    }
}

/// 规划阶段枚举
#[derive(Debug, Clone, PartialEq)]
pub enum PlanningPhase {
    Initial,           // 初始阶段
    PatternMatching,   // 模式匹配
    Filtering,         // 过滤
    Projection,        // 投影
    Aggregation,       // 聚合
    Sorting,           // 排序
    Pagination,        // 分页
    Finalization,      // 完成
}

/// 规划上下文
///
/// 存储规划过程中的状态信息
#[derive(Debug, Clone)]
pub struct StatementPlanningContext {
    pub phase: PlanningPhase,
    pub space_id: i32,
    pub variables: HashMap<String, String>,
    pub aliased_variables: HashMap<String, String>,
}

impl StatementPlanningContext {
    pub fn new(space_id: i32) -> Self {
        Self {
            phase: PlanningPhase::Initial,
            space_id,
            variables: HashMap::new(),
            aliased_variables: HashMap::new(),
        }
    }

    pub fn set_phase(&mut self, phase: PlanningPhase) {
        self.phase = phase;
    }

    pub fn add_variable(&mut self, name: &str, var_type: &str) {
        self.variables.insert(name.to_string(), var_type.to_string());
    }

    pub fn add_alias(&mut self, alias: &str, original: &str) {
        self.aliased_variables.insert(alias.to_string(), original.to_string());
    }

    pub fn get_variable_type(&self, name: &str) -> Option<&str> {
        self.variables.get(name).map(|s| s.as_str())
    }
}
