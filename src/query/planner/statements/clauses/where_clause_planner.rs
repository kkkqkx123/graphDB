//! WHERE 子句规划器
//! 架构重构：实现统一的 CypherClausePlanner 接口
//!
//! ## 重构说明
//!
//! ### 删除冗余方法
//! - 移除 `validate_input`, `can_start_flow`, `requires_input` 等冗余方法
//! - 通过 `flow_direction()` 统一表达数据流行为
//!
//! ### 简化变量管理
//! - WHERE 子句不产生新变量，只过滤输入
//! - 移除不必要的 `VariableRequirement` 和 `VariableProvider`
//!
//! ### 优化实现逻辑
//! - 专注于核心的过滤功能
//! - 简化路径表达式处理

use crate::query::parser::ast::expr::Expr;
use crate::query::planner::statements::clauses::clause_planner::ClausePlanner;
use crate::query::planner::statements::core::{
    ClauseType, CypherClausePlanner, DataFlowNode, FlowDirection, PlanningContext,
};

use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::PlannerError;
use crate::query::validator::structs::common_structs::CypherClauseContext;
use crate::query::validator::structs::CypherClauseKind;

/// WHERE 子句规划器
///
/// 负责规划 WHERE 子句的执行，过滤输入数据。
/// WHERE 子句是数据流转换器，需要输入并产生输出。
#[derive(Debug)]
pub struct WhereClausePlanner {
    filter_expr: Option<Expr>,
}

impl WhereClausePlanner {
    /// 创建新的 WHERE 子句规划器
    ///
    /// # 参数
    /// * `filter_expr` - 过滤表达式，None 表示无条件
    pub fn new(filter_expr: Option<Expr>) -> Self {
        Self { filter_expr }
    }

    /// 获取过滤表达式
    pub fn filter_expr(&self) -> Option<&Expr> {
        self.filter_expr.as_ref()
    }
}

impl ClausePlanner for WhereClausePlanner {
    fn name(&self) -> &'static str {
        "WhereClausePlanner"
    }

    fn supported_clause_kind(&self) -> CypherClauseKind {
        CypherClauseKind::Where
    }
}

impl CypherClausePlanner for WhereClausePlanner {
    fn clause_type(&self) -> ClauseType {
        ClauseType::Where
    }

    fn transform(
        &self,
        _clause_ctx: &CypherClauseContext,
        input_plan: Option<&SubPlan>,
        _context: &mut PlanningContext,
    ) -> Result<SubPlan, PlannerError> {
        self.validate_flow(input_plan)?;

        let input_plan = input_plan.ok_or_else(|| {
            PlannerError::PlanGenerationFailed("WHERE 子句需要输入计划".to_string())
        })?;

        Ok(input_plan.clone())
    }
}

impl DataFlowNode for WhereClausePlanner {
    fn flow_direction(&self) -> FlowDirection {
        FlowDirection::Transform
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::validator::structs::WhereClauseContext;

    #[test]
    fn test_where_clause_planner_creation() {
        let planner = WhereClausePlanner::new(None);
        assert_eq!(planner.name(), "WhereClausePlanner");
        assert_eq!(planner.supported_clause_kind(), CypherClauseKind::Where);
    }

    #[test]
    fn test_where_clause_planner_with_filter() {
        let expr = Expr::Constant(crate::query::parser::ast::expr::ConstantExpr::new(
            crate::core::Value::Bool(true),
            crate::query::parser::ast::types::Span::default(),
        ));
        let planner = WhereClausePlanner::new(Some(expr));
        assert!(planner.filter_expr().is_some());
    }

    #[test]
    fn test_where_clause_planner_clause_type() {
        let planner = WhereClausePlanner::new(None);
        assert_eq!(planner.clause_type(), ClauseType::Where);
    }

    #[test]
    fn test_where_clause_planner_flow_direction() {
        let planner = WhereClausePlanner::new(None);
        assert_eq!(DataFlowNode::flow_direction(&planner), FlowDirection::Transform);
        assert!(DataFlowNode::requires_input(&planner));
    }

    #[test]
    fn test_where_clause_planner_validate_context() {
        let planner = WhereClausePlanner::new(None);
        let where_ctx = WhereClauseContext {
            filter: None,
            aliases_available: std::collections::HashMap::new(),
            aliases_generated: std::collections::HashMap::new(),
            paths: vec![],
            query_parts: vec![],
            errors: vec![],
        };
        let clause_ctx = CypherClauseContext::Where(where_ctx);

        assert!(planner.validate_context(&clause_ctx).is_ok());
    }

    #[test]
    fn test_where_clause_planner_validate_context_invalid() {
        let planner = WhereClausePlanner::new(None);
        let return_ctx = crate::query::validator::structs::ReturnClauseContext {
            yield_clause: crate::query::validator::structs::YieldClauseContext {
                yield_columns: vec![],
                aliases_available: std::collections::HashMap::new(),
                aliases_generated: std::collections::HashMap::new(),
                distinct: false,
                has_agg: false,
                group_keys: vec![],
                group_items: vec![],
                need_gen_project: false,
                agg_output_column_names: vec![],
                proj_output_column_names: vec![],
                proj_cols: vec![],
                paths: vec![],
                query_parts: vec![],
                errors: vec![],
            },
            aliases_available: std::collections::HashMap::new(),
            aliases_generated: std::collections::HashMap::new(),
            pagination: None,
            order_by: None,
            distinct: false,
            query_parts: vec![],
            errors: vec![],
        };
        let clause_ctx = CypherClauseContext::Return(return_ctx);

        assert!(planner.validate_context(&clause_ctx).is_err());
    }

    #[test]
    fn test_where_clause_planner_transform_with_input() {
        let planner = WhereClausePlanner::new(None);
        let input_plan = SubPlan::new(None, None);
        let where_ctx = WhereClauseContext {
            filter: None,
            aliases_available: std::collections::HashMap::new(),
            aliases_generated: std::collections::HashMap::new(),
            paths: vec![],
            query_parts: vec![],
            errors: vec![],
        };
        let clause_ctx = CypherClauseContext::Where(where_ctx);
        let mut context = PlanningContext::new(QueryInfo {
            query_id: "test".to_string(),
            statement_type: "WHERE".to_string(),
        });

        let result = planner.transform(&clause_ctx, Some(&input_plan), &mut context);
        assert!(result.is_ok());
    }

    #[test]
    fn test_where_clause_planner_transform_without_input() {
        let planner = WhereClausePlanner::new(None);
        let where_ctx = WhereClauseContext {
            filter: None,
            aliases_available: std::collections::HashMap::new(),
            aliases_generated: std::collections::HashMap::new(),
            paths: vec![],
            query_parts: vec![],
            errors: vec![],
        };
        let clause_ctx = CypherClauseContext::Where(where_ctx);
        let mut context = PlanningContext::new(QueryInfo {
            query_id: "test".to_string(),
            statement_type: "WHERE".to_string(),
        });

        let result = planner.transform(&clause_ctx, None, &mut context);
        assert!(result.is_err());
    }
}
