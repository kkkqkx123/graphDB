//! 合并获取邻居和投影操作的规则

use crate::core::Expression;
use crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum;
use crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode;
use crate::query::planner::rewrite::context::RewriteContext;
use crate::query::planner::rewrite::pattern::Pattern;
use crate::query::planner::rewrite::result::{RewriteResult, TransformResult};
use crate::query::planner::rewrite::rule::{MergeRule, RewriteRule};

/// 合并获取邻居和投影操作的规则
///
/// # 转换示例
///
/// Before:
/// ```text
///   GetNeighbors
///       |
///   Project(col1)
///       |
///   ScanVertices
/// ```
///
/// After:
/// ```text
///   GetNeighbors(src=col1.expr)
///       |
///   ScanVertices
/// ```
///
/// # 适用条件
///
/// - 当前节点为GetNeighbors节点
/// - 子节点为Project节点
/// - Project只投影一列，且该列作为GetNeighbors的源
#[derive(Debug)]
pub struct MergeGetNbrsAndProjectRule;

impl MergeGetNbrsAndProjectRule {
    /// 创建规则实例
    pub fn new() -> Self {
        Self
    }
}

impl Default for MergeGetNbrsAndProjectRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RewriteRule for MergeGetNbrsAndProjectRule {
    fn name(&self) -> &'static str {
        "MergeGetNbrsAndProjectRule"
    }

    fn pattern(&self) -> Pattern {
        Pattern::new_with_name("GetNeighbors").with_dependency_name("Project")
    }

    fn apply(
        &self,
        _ctx: &mut RewriteContext,
        node: &PlanNodeEnum,
    ) -> RewriteResult<Option<TransformResult>> {
        // 检查是否为 GetNeighbors 节点
        let get_neighbors = match node {
            PlanNodeEnum::GetNeighbors(n) => n,
            _ => return Ok(None),
        };

        // GetNeighbors使用MultipleInputNode，需要获取依赖
        let deps = get_neighbors.dependencies();
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

        // 创建新的GetNeighbors节点
        let mut new_get_neighbors = get_neighbors.clone();

        // 更新源引用为Project列的表达式
        let src_expr = columns[0].expression.clone();
        if let Expression::Variable(name) = &src_expr {
            new_get_neighbors.set_src_vids(name.clone());
        }

        // 清除原有依赖并设置新的输入
        new_get_neighbors.deps_mut().clear();
        new_get_neighbors.deps_mut().push(Box::new(project_input));

        let mut result = TransformResult::new();
        result.erase_curr = true;
        result.add_new_node(PlanNodeEnum::GetNeighbors(new_get_neighbors));

        Ok(Some(result))
    }
}

impl MergeRule for MergeGetNbrsAndProjectRule {
    fn can_merge(&self, parent: &PlanNodeEnum, child: &PlanNodeEnum) -> bool {
        parent.is_get_neighbors() && child.is_project()
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
    use crate::query::planner::plan::core::nodes::graph_scan_node::GetNeighborsNode;
    use crate::query::planner::plan::core::nodes::project_node::ProjectNode;
    use crate::query::planner::plan::core::nodes::start_node::StartNode;

    #[test]
    fn test_rule_name() {
        let rule = MergeGetNbrsAndProjectRule::new();
        assert_eq!(rule.name(), "MergeGetNbrsAndProjectRule");
    }

    #[test]
    fn test_rule_pattern() {
        let rule = MergeGetNbrsAndProjectRule::new();
        let pattern = rule.pattern();
        assert!(pattern.node.is_some());
    }

    #[test]
    fn test_merge_get_nbrs_and_project() {
        // 创建起始节点
        let start = PlanNodeEnum::Start(StartNode::new());

        // 创建Project节点，投影一列
        let columns = vec![YieldColumn {
            expression: Expression::Variable("vid".to_string()),
            alias: "v".to_string(),
            is_matched: false,
        }];
        let project = ProjectNode::new(start, columns).expect("创建ProjectNode失败");
        let project_node = PlanNodeEnum::Project(project);

        // 创建GetNeighbors节点
        let get_neighbors = GetNeighborsNode::new(1, "v");
        let mut get_neighbors_node = PlanNodeEnum::GetNeighbors(get_neighbors);

        // 手动设置依赖关系
        if let PlanNodeEnum::GetNeighbors(ref mut gn) = get_neighbors_node {
            gn.deps_mut().clear();
            gn.deps_mut().push(Box::new(project_node));
        }

        // 应用规则
        let rule = MergeGetNbrsAndProjectRule::new();
        let mut ctx = RewriteContext::new();
        let result = rule.apply(&mut ctx, &get_neighbors_node).expect("应用规则失败");

        assert!(
            result.is_some(),
            "应该成功合并GetNeighbors和Project节点"
        );
    }
}
