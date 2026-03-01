//! 合并获取顶点和投影操作的规则

use crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum;
use crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode;
use crate::query::planner::rewrite::context::RewriteContext;
use crate::query::planner::rewrite::pattern::Pattern;
use crate::query::planner::rewrite::result::{RewriteResult, TransformResult};
use crate::query::planner::rewrite::rule::{MergeRule, RewriteRule};

/// 合并获取顶点和投影操作的规则
///
/// # 转换示例
///
/// Before:
/// ```text
///   GetVertices
///       |
///   Project(col1)
///       |
///   ScanVertices
/// ```
///
/// After:
/// ```text
///   GetVertices(src=col1.expr)
///       |
///   ScanVertices
/// ```
///
/// # 适用条件
///
/// - 当前节点为GetVertices节点
/// - 子节点为Project节点
/// - Project只投影一列，且该列作为GetVertices的源
#[derive(Debug)]
pub struct MergeGetVerticesAndProjectRule;

impl MergeGetVerticesAndProjectRule {
    /// 创建规则实例
    pub fn new() -> Self {
        Self
    }
}

impl Default for MergeGetVerticesAndProjectRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RewriteRule for MergeGetVerticesAndProjectRule {
    fn name(&self) -> &'static str {
        "MergeGetVerticesAndProjectRule"
    }

    fn pattern(&self) -> Pattern {
        Pattern::new_with_name("GetVertices").with_dependency_name("Project")
    }

    fn apply(
        &self,
        _ctx: &mut RewriteContext,
        node: &PlanNodeEnum,
    ) -> RewriteResult<Option<TransformResult>> {
        // 检查是否为 GetVertices 节点
        let get_vertices = match node {
            PlanNodeEnum::GetVertices(n) => n,
            _ => return Ok(None),
        };

        // GetVertices使用MultipleInputNode，需要获取依赖
        let deps = get_vertices.dependencies();
        if deps.is_empty() {
            return Ok(None);
        }

        // 检查第一个依赖是否为Project节点
        let project_node = match deps.first().map(|d| d.as_ref()) {
            Some(PlanNodeEnum::Project(n)) => n,
            _ => return Ok(None),
        };

        // 检查Project是否只投影一列
        let columns = project_node.columns();
        if columns.len() != 1 {
            return Ok(None);
        }

        // 获取Project的输入作为新的输入
        let project_input = project_node.input().clone();

        // 创建新的GetVertices节点
        let mut new_get_vertices = get_vertices.clone();

        // 更新源引用为Project列的表达式
        let src_expr = columns[0].expression.clone();
        if let Some(expr_meta) = src_expr.expression() {
            new_get_vertices.set_src_ref(expr_meta.inner().clone());
        }

        // 清除原有依赖并设置新的输入
        new_get_vertices.deps_mut().clear();
        new_get_vertices.deps_mut().push(Box::new(project_input));

        let mut result = TransformResult::new();
        result.erase_curr = true;
        result.add_new_node(PlanNodeEnum::GetVertices(new_get_vertices));

        Ok(Some(result))
    }
}

impl MergeRule for MergeGetVerticesAndProjectRule {
    fn can_merge(&self, parent: &PlanNodeEnum, child: &PlanNodeEnum) -> bool {
        parent.is_get_vertices() && child.is_project()
    }

    fn create_merged_node(
        &self,
        ctx: &mut RewriteContext,
        parent: &PlanNodeEnum,
        _child: &PlanNodeEnum,
    ) -> RewriteResult<Option<TransformResult>> {
        self.apply(ctx, parent)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Expression, YieldColumn};
    use crate::query::planner::plan::core::nodes::graph_scan_node::GetVerticesNode;
    use crate::query::planner::plan::core::nodes::project_node::ProjectNode;
    use crate::query::planner::plan::core::nodes::start_node::StartNode;

    #[test]
    fn test_rule_name() {
        let rule = MergeGetVerticesAndProjectRule::new();
        assert_eq!(rule.name(), "MergeGetVerticesAndProjectRule");
    }

    #[test]
    fn test_rule_pattern() {
        let rule = MergeGetVerticesAndProjectRule::new();
        let pattern = rule.pattern();
        assert!(pattern.node.is_some());
    }

    #[test]
    fn test_merge_get_vertices_and_project() {
        use std::sync::Arc;
        use crate::core::types::expression::ExpressionMeta;
        use crate::core::types::expression::ExpressionContext;
        
        // 创建起始节点
        let start = PlanNodeEnum::Start(StartNode::new());

        // 创建表达式上下文
        let expr_ctx = Arc::new(ExpressionContext::new());

        // 创建Project节点，投影一列
        let vid_expr = Expression::Variable("vid".to_string());
        let vid_meta = ExpressionMeta::new(vid_expr);
        let vid_id = expr_ctx.register_expression(vid_meta);
        let vid_ctx_expr = ContextualExpression::new(vid_id, expr_ctx);
        
        let columns = vec![YieldColumn {
            expression: vid_ctx_expr,
            alias: "v".to_string(),
            is_matched: false,
        }];
        let project = ProjectNode::new(start, columns).expect("创建ProjectNode失败");
        let project_node = PlanNodeEnum::Project(project);

        // 创建GetVertices节点
        let get_vertices = GetVerticesNode::new(1, "v");
        let mut get_vertices_node = PlanNodeEnum::GetVertices(get_vertices);

        // 手动设置依赖关系
        if let PlanNodeEnum::GetVertices(ref mut gv) = get_vertices_node {
            gv.deps_mut().clear();
            gv.deps_mut().push(Box::new(project_node));
        }

        // 应用规则
        let rule = MergeGetVerticesAndProjectRule::new();
        let mut ctx = RewriteContext::new();
        let result = rule.apply(&mut ctx, &get_vertices_node).expect("应用规则失败");

        assert!(
            result.is_some(),
            "应该成功合并GetVertices和Project节点"
        );
    }
}
