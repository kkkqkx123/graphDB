//! UNWIND子句规划器
//! 处理UNWIND操作的规划
//! 负责规划UNWIND操作来展开集合

use crate::query::planner::match_planning::core::cypher_clause_planner::CypherClausePlanner;
use crate::query::planner::plan::{PlanNodeKind, SingleInputNode, SubPlan};
use crate::query::planner::planner::PlannerError;
use crate::query::validator::structs::{CypherClauseContext, CypherClauseKind};
use std::sync::Arc;

/// UNWIND子句规划器
/// 负责规划UNWIND操作来展开集合
#[derive(Debug)]
pub struct UnwindClausePlanner;

impl UnwindClausePlanner {
    pub fn new() -> Self {
        Self
    }
}

impl CypherClausePlanner for UnwindClausePlanner {
    fn transform(&mut self, clause_ctx: &CypherClauseContext) -> Result<SubPlan, PlannerError> {
        if !matches!(clause_ctx.kind(), CypherClauseKind::Unwind) {
            return Err(PlannerError::InvalidAstContext(
                "Not a valid context for UnwindClausePlanner".to_string(),
            ));
        }

        let _unwind_clause_ctx = match clause_ctx {
            CypherClauseContext::Unwind(ctx) => ctx,
            _ => {
                return Err(PlannerError::InvalidAstContext(
                    "Expected UnwindClauseContext".to_string(),
                ))
            }
        };

        // 创建UNWIND节点
        let unwind_node = Arc::new(SingleInputNode::new(
            PlanNodeKind::Unwind,
            create_empty_node()?,
        ));

        // TODO: 设置UNWIND表达式和别名
        // 这里需要根据unwind_expr和alias设置UNWIND逻辑
        // unwind_expr 是要展开的集合表达式
        // alias 是展开后每个元素的别名

        Ok(SubPlan::new(Some(unwind_node.clone()), Some(unwind_node)))
    }
}

/// 创建空节点
fn create_empty_node() -> Result<Arc<dyn crate::query::planner::plan::PlanNode>, PlannerError> {
    use crate::query::planner::plan::SingleDependencyNode;

    // 创建一个空的计划节点作为占位符
    Ok(Arc::new(SingleDependencyNode {
        id: -1,
        kind: PlanNodeKind::Start,
        dependencies: vec![],
        output_var: None,
        col_names: vec![],
        cost: 0.0,
    }))
}
