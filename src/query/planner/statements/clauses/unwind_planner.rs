//! UNWIND 子句规划器
use crate::query::QueryContext;
use crate::query::parser::ast::Stmt;
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::PlannerError;
use crate::query::planner::statements::statement_planner::ClausePlanner;
use crate::query::validator::structs::CypherClauseKind;
use std::sync::Arc;

#[derive(Debug)]
pub struct UnwindClausePlanner {}

impl UnwindClausePlanner {
    pub fn new() -> Self {
        Self {}
    }
}

impl ClausePlanner for UnwindClausePlanner {
    fn clause_kind(&self) -> CypherClauseKind {
        CypherClauseKind::Unwind
    }

    fn transform_clause(
        &self,
        _qctx: Arc<QueryContext>,
        _stmt: &Stmt,
        input_plan: SubPlan,
    ) -> Result<SubPlan, PlannerError> {
        // UNWIND 子句的实现
        // 注意：这里需要根据实际的 AST 结构来实现 UNWIND 逻辑
        // 目前直接返回输入计划作为占位符
        Ok(input_plan)
    }
}

impl Default for UnwindClausePlanner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unwind_clause_planner_creation() {
        let planner = UnwindClausePlanner::new();
        assert_eq!(planner.clause_kind(), CypherClauseKind::Unwind);
    }
}
