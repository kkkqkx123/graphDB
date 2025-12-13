//! Cypher子句规划器基类
//! 定义所有Cypher子句规划器的通用接口

use crate::query::planner::plan::SubPlan;
use crate::query::validator::structs::common_structs::CypherClauseContext;

/// Cypher子句规划器trait
/// 所有Cypher子句规划器都应实现该trait
pub trait CypherClausePlanner: std::fmt::Debug {
    /// 将Cypher子句上下文转换为执行计划
    fn transform(
        &mut self,
        clause_ctx: &CypherClauseContext,
    ) -> Result<SubPlan, crate::query::planner::planner::PlannerError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::planner::plan::{PlanNodeKind, SingleInputNode, VariableDependencyNode};
    use crate::query::planner::plan::core::plan_node_traits::PlanNodeClonable;
    use crate::query::planner::planner::PlannerError;
    use crate::query::validator::structs::common_structs::CypherClauseContext;

    /// 测试用的 Cypher 子句规划器实现
    #[derive(Debug)]
    struct TestCypherClausePlanner {
        should_fail: bool,
    }

    impl TestCypherClausePlanner {
        fn new() -> Self {
            Self { should_fail: false }
        }

        fn new_with_failure() -> Self {
            Self { should_fail: true }
        }
    }

    impl CypherClausePlanner for TestCypherClausePlanner {
        fn transform(
            &mut self,
            _clause_ctx: &CypherClauseContext,
        ) -> Result<
            crate::query::planner::plan::SubPlan,
            crate::query::planner::planner::PlannerError,
        > {
            if self.should_fail {
                Err(PlannerError::PlanGenerationFailed(
                    "Test failure".to_string(),
                ))
            } else {
                // 创建一个简单的测试节点
                use std::sync::Arc;
                let test_node = SingleInputNode::new(
                    PlanNodeKind::Project,
                    Arc::new(
                        VariableDependencyNode::new(
                            PlanNodeKind::Start,
                        ),
                    ),
                );
                let subplan =
                    crate::query::planner::plan::SubPlan::new(Some(Arc::new(test_node)), None);
                Ok(subplan)
            }
        }
    }

    /// 创建测试用的 CypherClauseContext
    fn create_test_clause_context() -> CypherClauseContext {
        use crate::query::validator::structs::{MatchClauseContext, NodeInfo, Path, PathType};
        use std::collections::HashMap;

        let node_info = NodeInfo {
            alias: "n".to_string(),
            labels: vec!["Person".to_string()],
            props: None,
            anonymous: false,
            filter: None,
            tids: vec![1],
            label_props: vec![None],
        };

        let path = Path {
            alias: "p".to_string(),
            anonymous: false,
            gen_path: false,
            path_type: PathType::Default,
            node_infos: vec![node_info],
            edge_infos: vec![],
            path_build: None,
            is_pred: false,
            is_anti_pred: false,
            compare_variables: vec![],
            collect_variable: String::new(),
            roll_up_apply: false,
        };

        let match_ctx = MatchClauseContext {
            paths: vec![path],
            aliases_available: HashMap::new(),
            aliases_generated: HashMap::new(),
            where_clause: None,
            is_optional: false,
            skip: None,
            limit: None,
        };

        CypherClauseContext::Match(match_ctx)
    }

    #[test]
    fn test_cypher_clause_planner_trait_success() {
        let mut planner = TestCypherClausePlanner::new();
        let clause_ctx = create_test_clause_context();

        let result = planner.transform(&clause_ctx);

        assert!(result.is_ok());
        let subplan = result.unwrap();
        assert!(subplan.root.is_some());
    }

    #[test]
    fn test_cypher_clause_planner_trait_failure() {
        let mut planner = TestCypherClausePlanner::new_with_failure();
        let clause_ctx = create_test_clause_context();

        let result = planner.transform(&clause_ctx);

        assert!(result.is_err());
        match result.unwrap_err() {
            PlannerError::PlanGenerationFailed(msg) => {
                assert_eq!(msg, "Test failure");
            }
            _ => panic!("Expected PlanGenerationFailed error"),
        }
    }

    #[test]
    fn test_cypher_clause_planner_debug() {
        let planner = TestCypherClausePlanner::new();
        let debug_str = format!("{:?}", planner);
        assert!(debug_str.contains("TestCypherClausePlanner"));
    }

    #[test]
    fn test_subplan_creation() {
        use crate::query::planner::plan::{PlanNode, PlanNodeKind, VariableDependencyNode};

        let start_node = VariableDependencyNode::new(PlanNodeKind::Start);
        let subplan =
            crate::query::planner::plan::SubPlan::new(Some(start_node.clone_plan_node()), None);

        assert!(subplan.root.is_some());
        assert!(subplan.tail.is_none());

        if let Some(root) = &subplan.root {
            assert_eq!(root.kind(), PlanNodeKind::Start);
        }
    }

    #[test]
    fn test_subplan_from_root() {
        use crate::query::planner::plan::{PlanNode, PlanNodeKind, VariableDependencyNode};

        let start_node = VariableDependencyNode::new(PlanNodeKind::Start);
        let subplan = crate::query::planner::plan::SubPlan::from_root(start_node.clone_plan_node());

        assert!(subplan.root.is_some());
        assert!(subplan.tail.is_some());

        if let Some(root) = &subplan.root {
            assert_eq!(root.kind(), PlanNodeKind::Start);
        }

        if let Some(tail) = &subplan.tail {
            assert_eq!(tail.kind(), PlanNodeKind::Start);
        }
    }

    #[test]
    fn test_plan_node_kind() {
        let kind = PlanNodeKind::Project;
        assert_eq!(kind, PlanNodeKind::Project);

        let debug_str = format!("{:?}", kind);
        assert!(debug_str.contains("Project"));
    }

    #[test]
    fn test_planner_error() {
        let error = PlannerError::NoSuitablePlanner("Test error".to_string());
        let error_str = format!("{}", error);
        assert!(error_str.contains("No suitable planner found"));
        assert!(error_str.contains("Test error"));

        let error = PlannerError::UnsupportedOperation("Unsupported".to_string());
        let error_str = format!("{}", error);
        assert!(error_str.contains("Unsupported operation"));

        let error = PlannerError::PlanGenerationFailed("Failed".to_string());
        let error_str = format!("{}", error);
        assert!(error_str.contains("Plan generation failed"));

        let error = PlannerError::InvalidAstContext("Invalid".to_string());
        let error_str = format!("{}", error);
        assert!(error_str.contains("Invalid AST context"));
    }
}
