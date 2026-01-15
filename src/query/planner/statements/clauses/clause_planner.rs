//! 子句规划器基类
//! 定义所有子句规划器的通用接口和trait

use crate::query::planner::statements::core::cypher_clause_planner::{
    CypherClausePlanner, DataFlowNode, ClauseType, PlanningContext, FlowDirection,
};
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::PlannerError;
use crate::query::validator::structs::common_structs::CypherClauseContext;
use crate::query::validator::structs::CypherClauseKind;

/// 子句规划器trait
///
/// 所有子句规划器都应该实现这个trait，提供统一的接口
/// 这个trait扩展了CypherClausePlanner，添加了额外的功能
pub trait ClausePlanner: CypherClausePlanner {
    /// 获取子句规划器的名称
    fn name(&self) -> &'static str;

    /// 获取支持的子句类型
    fn supported_clause_kind(&self) -> CypherClauseKind;

    /// 验证子句上下文是否有效
    fn validate_context(&self, clause_ctx: &CypherClauseContext) -> Result<(), PlannerError> {
        // 将CypherClauseContext转换为CypherClauseKind进行比较
        let clause_kind = match clause_ctx {
            CypherClauseContext::Match(_) => CypherClauseKind::Match,
            CypherClauseContext::Where(_) => CypherClauseKind::Where,
            CypherClauseContext::Return(_) => CypherClauseKind::Return,
            CypherClauseContext::With(_) => CypherClauseKind::With,
            CypherClauseContext::OrderBy(_) => CypherClauseKind::OrderBy,
            CypherClauseContext::Pagination(_) => CypherClauseKind::Pagination,
            CypherClauseContext::Unwind(_) => CypherClauseKind::Unwind,
            CypherClauseContext::Yield(_) => CypherClauseKind::Yield,
        };

        if clause_kind != self.supported_clause_kind() {
            return Err(PlannerError::InvalidAstContext(format!(
                "Invalid clause context for {}: expected {:?}, got {:?}",
                self.name(),
                self.supported_clause_kind(),
                clause_kind
            )));
        }
        Ok(())
    }

    /// 估算子句执行成本
    fn estimate_cost(&self, _clause_ctx: &CypherClauseContext) -> f64 {
        // 默认实现，子类可以重写
        10.0
    }

    /// 检查是否可以优化
    fn can_optimize(&self, _clause_ctx: &CypherClauseContext) -> bool {
        // 默认实现，子类可以重写
        false
    }

    /// 应用优化
    fn apply_optimization(
        &self,
        _clause_ctx: &CypherClauseContext,
        plan: SubPlan,
    ) -> Result<SubPlan, PlannerError> {
        // 默认实现，子类可以重写
        Ok(plan)
    }
}

/// 子句规划器基类
///
/// 提供子句规划器的通用实现
#[derive(Debug)]
pub struct BaseClausePlanner {
    name: &'static str,
    supported_kind: CypherClauseKind,
}

impl BaseClausePlanner {
    /// 创建新的基础子句规划器
    pub fn new(name: &'static str, supported_kind: CypherClauseKind) -> Self {
        Self {
            name,
            supported_kind,
        }
    }

    /// 获取名称
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// 获取支持的子句类型
    pub fn supported_kind(&self) -> CypherClauseKind {
        self.supported_kind
    }
}

impl DataFlowNode for BaseClausePlanner {
    fn flow_direction(&self) -> FlowDirection {
        match self.supported_kind {
            CypherClauseKind::Match => FlowDirection::Source,
            CypherClauseKind::Where => FlowDirection::Transform,
            CypherClauseKind::Return => FlowDirection::Output,
            CypherClauseKind::With => FlowDirection::Transform,
            CypherClauseKind::OrderBy => FlowDirection::Transform,
            CypherClauseKind::Pagination => FlowDirection::Transform,
            CypherClauseKind::Unwind => FlowDirection::Transform,
            CypherClauseKind::Yield => FlowDirection::Output,
        }
    }
}

impl CypherClausePlanner for BaseClausePlanner {
    fn transform(
        &self,
        _clause_ctx: &CypherClauseContext,
        _input_plan: Option<&SubPlan>,
        _context: &mut PlanningContext,
    ) -> Result<SubPlan, PlannerError> {
        Err(PlannerError::NotImplemented(
            "BaseClausePlanner::transform not implemented".to_string(),
        ))
    }

    fn clause_type(&self) -> ClauseType {
        match self.supported_kind {
            CypherClauseKind::Match => ClauseType::Match,
            CypherClauseKind::Where => ClauseType::Where,
            CypherClauseKind::Return => ClauseType::Return,
            CypherClauseKind::With => ClauseType::With,
            CypherClauseKind::OrderBy => ClauseType::OrderBy,
            CypherClauseKind::Pagination => ClauseType::Limit,
            CypherClauseKind::Unwind => ClauseType::Unwind,
            CypherClauseKind::Yield => ClauseType::Yield,
        }
    }

    fn flow_direction(&self) -> FlowDirection {
        self.clause_type().flow_direction()
    }
}

impl ClausePlanner for BaseClausePlanner {
    fn name(&self) -> &'static str {
        self.name
    }

    fn supported_clause_kind(&self) -> CypherClauseKind {
        self.supported_kind
    }
}
