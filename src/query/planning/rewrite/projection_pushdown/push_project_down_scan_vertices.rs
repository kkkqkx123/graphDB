//! ScanVertices 投影下推优化规则
//!
//! 该规则将投影操作下推到 ScanVertices 节点，减少数据传输量。
//!
//! # 转换示例
//!
//! Before:
//! ```text
//! Project(col1, col2)
//!         |
//!     ScanVertices
//! ```
//!
//! After:
//! ```text
//! ScanVertices(col1, col2)
//! ```

use crate::core::YieldColumn;
use crate::query::planner::plan::core::nodes::ScanVerticesNode;
use crate::query::planner::plan::PlanNodeEnum;
use crate::query::planning::rewrite::context::RewriteContext;
use crate::query::planning::rewrite::pattern::Pattern;
use crate::query::planning::rewrite::result::{RewriteResult, TransformResult};
use crate::query::planning::rewrite::rule::{PushDownRule, RewriteRule};

/// ScanVertices 投影下推规则
///
/// 将投影操作下推到 ScanVertices 节点
#[derive(Debug)]
pub struct PushProjectDownScanVerticesRule;

impl PushProjectDownScanVerticesRule {
    pub fn new() -> Self {
        Self
    }

    fn can_push_down_project(project_node: &crate::query::planner::plan::core::nodes::ProjectNode) -> bool {
        !project_node.columns().is_empty()
    }

    fn create_scan_vertices_with_projection(
        &self,
        scan_node: &ScanVerticesNode,
        project_columns: &[YieldColumn],
    ) -> ScanVerticesNode {
        let col_names: Vec<String> = project_columns
            .iter()
            .map(|col| col.alias.clone())
            .collect();

        let mut new_node = scan_node.clone();
        new_node.set_col_names(col_names);
        new_node
    }
}

impl Default for PushProjectDownScanVerticesRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RewriteRule for PushProjectDownScanVerticesRule {
    fn name(&self) -> &'static str {
        "PushProjectDownScanVerticesRule"
    }

    fn pattern(&self) -> Pattern {
        Pattern::new_with_name("Project")
    }

    fn apply(
        &self,
        _ctx: &mut RewriteContext,
        node: &PlanNodeEnum,
    ) -> RewriteResult<Option<TransformResult>> {
        let project_node = match node {
            PlanNodeEnum::Project(n) => n,
            _ => return Ok(None),
        };

        if !Self::can_push_down_project(project_node) {
            return Ok(None);
        }

        let input = project_node.input();
        let scan_node = match input {
            PlanNodeEnum::ScanVertices(n) => n,
            _ => return Ok(None),
        };

        let columns = project_node.columns();
        let new_scan_node = self.create_scan_vertices_with_projection(scan_node, columns);
        let new_node = PlanNodeEnum::ScanVertices(new_scan_node);

        let mut result = TransformResult::new();
        result.erase_curr = true;
        result.add_new_node(new_node);

        Ok(Some(result))
    }
}

impl PushDownRule for PushProjectDownScanVerticesRule {
    fn can_push_down(&self, node: &PlanNodeEnum, target: &PlanNodeEnum) -> bool {
        match node {
            PlanNodeEnum::Project(project) => {
                if project.columns().is_empty() {
                    return false;
                }
                matches!(target, PlanNodeEnum::ScanVertices(_))
            }
            _ => false,
        }
    }

    fn push_down(
        &self,
        ctx: &mut RewriteContext,
        node: &PlanNodeEnum,
        _target: &PlanNodeEnum,
    ) -> RewriteResult<Option<TransformResult>> {
        self.apply(ctx, node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::ContextualExpression;
    use crate::core::{Expression, YieldColumn};
    use crate::query::planner::plan::core::nodes::{ProjectNode, ScanVerticesNode};
    use std::sync::Arc;
    use crate::query::validator::context::expression_context::ExpressionAnalysisContext;

    fn create_yield_column(expr: Expression, alias: &str) -> YieldColumn {
        let ctx = Arc::new(ExpressionAnalysisContext::new());
        let expr_meta = crate::core::types::expression::ExpressionMeta::new(expr);
        let id = ctx.register_expression(expr_meta);
        let ctx_expr = ContextualExpression::new(id, ctx);
        YieldColumn {
            expression: ctx_expr,
            alias: alias.to_string(),
            is_matched: false,
        }
    }

    #[test]
    fn test_rule_name() {
        let rule = PushProjectDownScanVerticesRule::new();
        assert_eq!(rule.name(), "PushProjectDownScanVerticesRule");
    }

    #[test]
    fn test_rule_pattern() {
        let rule = PushProjectDownScanVerticesRule::new();
        let pattern = rule.pattern();
        assert!(pattern.node.is_some());
    }

    #[test]
    fn test_apply_with_scan_vertices() {
        let rule = PushProjectDownScanVerticesRule::new();
        let mut ctx = RewriteContext::new();

        let scan_node = ScanVerticesNode::new(1);
        let scan = PlanNodeEnum::ScanVertices(scan_node);

        let columns = vec![
            create_yield_column(Expression::Variable("id".to_string()), "id"),
            create_yield_column(Expression::Variable("name".to_string()), "name"),
        ];
        let project = ProjectNode::new(scan.clone(), columns).expect("创建 ProjectNode 失败");
        let project_enum = PlanNodeEnum::Project(project);

        let result = rule.apply(&mut ctx, &project_enum).expect("应用规则失败");

        assert!(result.is_some());
        let transform = result.expect("Failed to apply rewrite rule");
        assert!(transform.erase_curr);
        assert_eq!(transform.new_nodes.len(), 1);

        match &transform.new_nodes[0] {
            PlanNodeEnum::ScanVertices(node) => {
                assert_eq!(node.col_names(), &["id", "name"]);
            }
            _ => panic!("期望 ScanVertices 节点"),
        }
    }

    #[test]
    fn test_push_down_rule_trait() {
        let rule = PushProjectDownScanVerticesRule::new();

        let scan = PlanNodeEnum::ScanVertices(ScanVerticesNode::new(1));
        let columns = vec![create_yield_column(
            Expression::Variable("test".to_string()),
            "test",
        )];
        let project = ProjectNode::new(scan.clone(), columns).expect("创建 ProjectNode 失败");
        let project_enum = PlanNodeEnum::Project(project);

        assert!(rule.can_push_down(&project_enum, &scan));
    }
}
