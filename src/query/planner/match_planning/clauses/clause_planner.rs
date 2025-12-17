//! 子句规划器基类
//! 定义所有子句规划器的通用接口和trait

use crate::query::planner::match_planning::core::cypher_clause_planner::CypherClausePlanner;
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::PlannerError;
use crate::query::validator::structs::{CypherClauseContext, CypherClauseKind};

/// 子句规划器trait
///
/// 所有子句规划器都应该实现这个trait，提供统一的接口
pub trait ClausePlanner: CypherClausePlanner {
    /// 获取子句规划器的名称
    fn name(&self) -> &'static str;

    /// 获取支持的子句类型
    fn supported_clause_kind(&self) -> CypherClauseKind;

    /// 验证子句上下文是否有效
    fn validate_context(&self, clause_ctx: &CypherClauseContext) -> Result<(), PlannerError> {
        if clause_ctx.kind() != self.supported_clause_kind() {
            return Err(PlannerError::InvalidAstContext(format!(
                "Invalid clause context for {}: expected {:?}, got {:?}",
                self.name(),
                self.supported_clause_kind(),
                clause_ctx.kind()
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

impl ClausePlanner for BaseClausePlanner {
    fn name(&self) -> &'static str {
        self.name
    }

    fn supported_clause_kind(&self) -> CypherClauseKind {
        self.supported_kind
    }

    fn validate_context(&self, clause_ctx: &CypherClauseContext) -> Result<(), PlannerError> {
        if clause_ctx.kind() != self.supported_clause_kind() {
            return Err(PlannerError::InvalidAstContext(format!(
                "Invalid clause context for {}: expected {:?}, got {:?}",
                self.name(),
                self.supported_clause_kind(),
                clause_ctx.kind()
            )));
        }
        Ok(())
    }
}

impl CypherClausePlanner for BaseClausePlanner {
    fn transform(&mut self, clause_ctx: &CypherClauseContext) -> Result<SubPlan, PlannerError> {
        // 验证上下文
        self.validate_context(clause_ctx)?;

        // 基础实现只是返回一个空的计划
        // 子类应该重写这个方法
        Err(PlannerError::UnsupportedOperation(format!(
            "BaseClausePlanner does not implement transform for {}",
            self.name()
        )))
    }
}

/// 子句规划器工厂
///
/// 用于创建不同类型的子句规划器
pub struct ClausePlannerFactory;

impl ClausePlannerFactory {
    /// 创建子句规划器
    pub fn create_planner(
        clause_kind: CypherClauseKind,
    ) -> Result<Box<dyn ClausePlanner>, PlannerError> {
        match clause_kind {
            CypherClauseKind::Match => {
                // 这里应该返回MatchClausePlanner的实例
                // 但是由于循环依赖，我们暂时返回错误
                Err(PlannerError::UnsupportedOperation(
                    "MatchClausePlanner creation not implemented in factory".to_string(),
                ))
            }
            CypherClauseKind::Where => {
                // 这里应该返回WhereClausePlanner的实例
                Err(PlannerError::UnsupportedOperation(
                    "WhereClausePlanner creation not implemented in factory".to_string(),
                ))
            }
            CypherClauseKind::Return => {
                // 这里应该返回ReturnClausePlanner的实例
                Err(PlannerError::UnsupportedOperation(
                    "ReturnClausePlanner creation not implemented in factory".to_string(),
                ))
            }
            CypherClauseKind::With => {
                // 这里应该返回WithClausePlanner的实例
                Err(PlannerError::UnsupportedOperation(
                    "WithClausePlanner creation not implemented in factory".to_string(),
                ))
            }
            CypherClauseKind::OrderBy => {
                // 这里应该返回OrderByClausePlanner的实例
                Err(PlannerError::UnsupportedOperation(
                    "OrderByClausePlanner creation not implemented in factory".to_string(),
                ))
            }
            CypherClauseKind::Pagination => {
                // 这里应该返回PaginationPlanner的实例
                Err(PlannerError::UnsupportedOperation(
                    "PaginationPlanner creation not implemented in factory".to_string(),
                ))
            }
            CypherClauseKind::Unwind => {
                // 这里应该返回UnwindClausePlanner的实例
                Err(PlannerError::UnsupportedOperation(
                    "UnwindClausePlanner creation not implemented in factory".to_string(),
                ))
            }
            CypherClauseKind::Yield => {
                // 这里应该返回YieldClausePlanner的实例
                Err(PlannerError::UnsupportedOperation(
                    "YieldClausePlanner creation not implemented in factory".to_string(),
                ))
            }
        }
    }

    /// 检查是否支持指定的子句类型
    pub fn supports_clause_kind(clause_kind: CypherClauseKind) -> bool {
        matches!(
            clause_kind,
            CypherClauseKind::Match
                | CypherClauseKind::Where
                | CypherClauseKind::Return
                | CypherClauseKind::With
                | CypherClauseKind::OrderBy
                | CypherClauseKind::Pagination
                | CypherClauseKind::Unwind
                | CypherClauseKind::Yield
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::validator::structs::CypherClauseKind;

    #[test]
    fn test_base_clause_planner_new() {
        let planner = BaseClausePlanner::new("TestPlanner", CypherClauseKind::Match);
        assert_eq!(planner.name(), "TestPlanner");
        assert_eq!(planner.supported_kind(), CypherClauseKind::Match);
    }

    #[test]
    fn test_base_clause_planner_validate_context_success() {
        let planner = BaseClausePlanner::new("TestPlanner", CypherClauseKind::Match);
        let clause_ctx =
            CypherClauseContext::Match(crate::query::validator::structs::MatchClauseContext {
                paths: vec![],
                aliases_available: std::collections::HashMap::new(),
                aliases_generated: std::collections::HashMap::new(),
                where_clause: None,
                is_optional: false,
                skip: None,
                limit: None,
            });

        let result = planner.validate_context(&clause_ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_base_clause_planner_validate_context_failure() {
        let planner = BaseClausePlanner::new("TestPlanner", CypherClauseKind::Match);
        let clause_ctx = CypherClauseContext::Where(
            crate::query::validator::structs::clause_structs::WhereClauseContext {
                filter: None,
                aliases_available: std::collections::HashMap::new(),
                aliases_generated: std::collections::HashMap::new(),
                paths: vec![],
            },
        );

        let result = planner.validate_context(&clause_ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_base_clause_planner_transform() {
        let mut planner = BaseClausePlanner::new("TestPlanner", CypherClauseKind::Match);
        let clause_ctx =
            CypherClauseContext::Match(crate::query::validator::structs::MatchClauseContext {
                paths: vec![],
                aliases_available: std::collections::HashMap::new(),
                aliases_generated: std::collections::HashMap::new(),
                where_clause: None,
                is_optional: false,
                skip: None,
                limit: None,
            });

        let result = planner.transform(&clause_ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_clause_planner_factory_supports_clause_kind() {
        assert!(ClausePlannerFactory::supports_clause_kind(
            CypherClauseKind::Match
        ));
        assert!(ClausePlannerFactory::supports_clause_kind(
            CypherClauseKind::Where
        ));
        assert!(ClausePlannerFactory::supports_clause_kind(
            CypherClauseKind::Return
        ));
        assert!(ClausePlannerFactory::supports_clause_kind(
            CypherClauseKind::With
        ));
        assert!(ClausePlannerFactory::supports_clause_kind(
            CypherClauseKind::OrderBy
        ));
        assert!(ClausePlannerFactory::supports_clause_kind(
            CypherClauseKind::Pagination
        ));
        assert!(ClausePlannerFactory::supports_clause_kind(
            CypherClauseKind::Unwind
        ));
        assert!(ClausePlannerFactory::supports_clause_kind(
            CypherClauseKind::Yield
        ));
    }

    #[test]
    fn test_clause_planner_factory_create_planner() {
        let result = ClausePlannerFactory::create_planner(CypherClauseKind::Match);
        assert!(result.is_err()); // 因为还没有实现具体的规划器创建
    }
}
