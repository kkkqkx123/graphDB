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

use crate::query::QueryContext;
use crate::query::parser::ast::Stmt;
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::{Planner, ValidatedStatement};
use crate::query::validator::structs::CypherClauseKind;
use std::sync::Arc;

/// 语句级规划器 trait
///
/// 定义语句级规划器的统一接口，封装完整语句的规划逻辑。
/// 组合多个子句规划器来完成语句的规划。
pub trait StatementPlanner: Planner {
    /// 获取语句类型
    fn statement_type(&self) -> &'static str;

    /// 获取支持的子句类型列表
    fn supported_clause_kinds(&self) -> &[CypherClauseKind];
}

/// 子句级规划器 trait
///
/// 定义子句级规划器的统一接口，处理单个子句的规划逻辑。
pub trait ClausePlanner: std::fmt::Debug {
    /// 获取子句类型
    fn clause_kind(&self) -> CypherClauseKind;

    /// 转换子句为核心计划
    fn transform_clause(
        &self,
        qctx: Arc<QueryContext>,
        stmt: &Stmt,
        input_plan: SubPlan,
    ) -> Result<SubPlan, crate::query::planner::planner::PlannerError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::parser::ast::Span;
    use crate::query::query_request_context::QueryRequestContext;
    use crate::query::planner::plan::core::nodes::StartNode;
    use crate::query::planner::plan::core::PlanNodeEnum;
    use std::collections::HashMap;

    #[derive(Debug)]
    struct MockStatementPlanner {
        stmt_type: &'static str,
        supported_kinds: Vec<CypherClauseKind>,
    }

    impl MockStatementPlanner {
        fn new(stmt_type: &'static str, supported_kinds: Vec<CypherClauseKind>) -> Self {
            Self {
                stmt_type,
                supported_kinds,
            }
        }
    }

    impl Planner for MockStatementPlanner {
        fn transform(&mut self, _validated: &ValidatedStatement, _qctx: Arc<QueryContext>) -> Result<SubPlan, crate::query::planner::planner::PlannerError> {
            let start_node = StartNode::new();
            let start_node_enum = PlanNodeEnum::Start(start_node);
            Ok(SubPlan {
                root: Some(start_node_enum.clone()),
                tail: Some(start_node_enum),
            })
        }

        fn match_planner(&self, _stmt: &Stmt) -> bool {
            true
        }
    }

    impl StatementPlanner for MockStatementPlanner {
        fn statement_type(&self) -> &'static str {
            self.stmt_type
        }

        fn supported_clause_kinds(&self) -> &[CypherClauseKind] {
            &self.supported_kinds
        }
    }

    #[derive(Debug)]
    struct MockClausePlanner {
        kind: CypherClauseKind,
    }

    impl MockClausePlanner {
        fn new(kind: CypherClauseKind) -> Self {
            Self { kind }
        }
    }

    impl ClausePlanner for MockClausePlanner {
        fn clause_kind(&self) -> CypherClauseKind {
            self.kind
        }

        fn transform_clause(
            &self,
            _qctx: Arc<QueryContext>,
            _stmt: &Stmt,
            input_plan: SubPlan,
        ) -> Result<SubPlan, crate::query::planner::planner::PlannerError> {
            Ok(input_plan)
        }
    }

    fn create_test_qctx() -> Arc<QueryContext> {
        let rctx = Arc::new(QueryRequestContext {
            session_id: None,
            user_name: None,
            space_name: None,
            query: String::new(),
            parameters: HashMap::new(),
        });
        Arc::new(QueryContext::new(rctx))
    }

    fn create_test_match_stmt() -> Stmt {
        Stmt::Match(crate::query::parser::ast::stmt::MatchStmt {
            span: Span::default(),
            patterns: vec![],
            where_clause: None,
            return_clause: None,
            order_by: None,
            limit: None,
            skip: None,
            optional: false,
        })
    }

    #[test]
    fn test_statement_planner_statement_type() {
        let planner = MockStatementPlanner::new(
            "MATCH",
            vec![CypherClauseKind::Match, CypherClauseKind::Where],
        );
        assert_eq!(planner.statement_type(), "MATCH");
    }

    #[test]
    fn test_statement_planner_supported_clause_kinds() {
        let supported_kinds = vec![CypherClauseKind::Match, CypherClauseKind::Where];
        let planner = MockStatementPlanner::new("MATCH", supported_kinds.clone());
        assert_eq!(planner.supported_clause_kinds(), &supported_kinds);
    }

    #[test]
    fn test_statement_planner_transform() {
        use crate::query::validator::ValidationInfo;

        let mut planner = MockStatementPlanner::new(
            "MATCH",
            vec![CypherClauseKind::Match],
        );
        let stmt = create_test_match_stmt();
        let qctx = create_test_qctx();

        // 创建验证后的语句
        let validation_info = ValidationInfo::new();
        let validated = ValidatedStatement::new(stmt, validation_info);

        let result = planner.transform(&validated, qctx);
        assert!(result.is_ok());
        let sub_plan = result.expect("transform should succeed");
        assert!(sub_plan.root.is_some());
        assert!(sub_plan.tail.is_some());
    }

    #[test]
    fn test_statement_planner_match_planner() {
        let planner = MockStatementPlanner::new(
            "MATCH",
            vec![CypherClauseKind::Match],
        );
        let stmt = create_test_match_stmt();
        assert!(planner.match_planner(&stmt));
    }

    #[test]
    fn test_clause_planner_clause_kind() {
        let planner = MockClausePlanner::new(CypherClauseKind::Where);
        assert_eq!(planner.clause_kind(), CypherClauseKind::Where);
    }

    #[test]
    fn test_clause_planner_transform_clause() {
        let planner = MockClausePlanner::new(CypherClauseKind::Where);
        let qctx = create_test_qctx();
        let stmt = create_test_match_stmt();

        let start_node = StartNode::new();
        let start_node_enum = PlanNodeEnum::Start(start_node.clone());
        let input_plan = SubPlan {
            root: Some(start_node_enum.clone()),
            tail: Some(start_node_enum),
        };

        let result = planner.transform_clause(qctx, &stmt, input_plan);
        assert!(result.is_ok());
    }

    #[test]
    fn test_multiple_supported_clause_kinds() {
        let supported_kinds = vec![
            CypherClauseKind::Match,
            CypherClauseKind::Where,
            CypherClauseKind::Return,
            CypherClauseKind::With,
        ];
        let planner = MockStatementPlanner::new("MATCH", supported_kinds.clone());
        assert_eq!(planner.supported_clause_kinds().len(), 4);
        assert!(planner.supported_clause_kinds().contains(&CypherClauseKind::Match));
        assert!(planner.supported_clause_kinds().contains(&CypherClauseKind::Where));
        assert!(planner.supported_clause_kinds().contains(&CypherClauseKind::Return));
        assert!(planner.supported_clause_kinds().contains(&CypherClauseKind::With));
    }

    #[test]
    fn test_clause_planner_different_kinds() {
        let where_planner = MockClausePlanner::new(CypherClauseKind::Where);
        let return_planner = MockClausePlanner::new(CypherClauseKind::Return);
        let with_planner = MockClausePlanner::new(CypherClauseKind::With);

        assert_eq!(where_planner.clause_kind(), CypherClauseKind::Where);
        assert_eq!(return_planner.clause_kind(), CypherClauseKind::Return);
        assert_eq!(with_planner.clause_kind(), CypherClauseKind::With);
    }
}
