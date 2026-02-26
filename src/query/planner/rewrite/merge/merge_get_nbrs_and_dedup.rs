//! 合并获取邻居和去重操作的规则

use crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum;
use crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode;
use crate::query::planner::rewrite::context::RewriteContext;
use crate::query::planner::rewrite::pattern::Pattern;
use crate::query::planner::rewrite::result::{RewriteResult, TransformResult};
use crate::query::planner::rewrite::rule::{MergeRule, RewriteRule};

/// 合并获取邻居和去重操作的规则
///
/// # 转换示例
///
/// Before:
/// ```text
///   GetNeighbors
///       |
///   Dedup
///       |
///   ScanVertices
/// ```
///
/// After:
/// ```text
///   GetNeighbors(dedup=true)
///       |
///   ScanVertices
/// ```
///
/// # 适用条件
///
/// - 当前节点为GetNeighbors节点
/// - 子节点为Dedup节点
/// - 可以将去重操作合并到GetNeighbors中
#[derive(Debug)]
pub struct MergeGetNbrsAndDedupRule;

impl MergeGetNbrsAndDedupRule {
    /// 创建规则实例
    pub fn new() -> Self {
        Self
    }
}

impl Default for MergeGetNbrsAndDedupRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RewriteRule for MergeGetNbrsAndDedupRule {
    fn name(&self) -> &'static str {
        "MergeGetNbrsAndDedupRule"
    }

    fn pattern(&self) -> Pattern {
        Pattern::new_with_name("GetNeighbors").with_dependency_name("Dedup")
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

        // 检查第一个依赖是否为Dedup节点
        let dedup_node = match deps.first().map(|d| d.as_ref()) {
            Some(PlanNodeEnum::Dedup(n)) => n,
            _ => return Ok(None),
        };

        // 获取Dedup的输入作为新的输入
        let dedup_input = dedup_node.input().clone();

        // 创建新的GetNeighbors节点
        let mut new_get_neighbors = get_neighbors.clone();

        // 设置去重标志
        if !new_get_neighbors.dedup() {
            new_get_neighbors.set_dedup(true);
        }

        // 清除原有依赖并设置新的输入
        new_get_neighbors.deps_mut().clear();
        new_get_neighbors.deps_mut().push(Box::new(dedup_input));

        let mut result = TransformResult::new();
        result.erase_curr = true;
        result.add_new_node(PlanNodeEnum::GetNeighbors(new_get_neighbors));

        Ok(Some(result))
    }
}

impl MergeRule for MergeGetNbrsAndDedupRule {
    fn can_merge(&self, parent: &PlanNodeEnum, child: &PlanNodeEnum) -> bool {
        parent.is_get_neighbors() && child.is_dedup()
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
    use crate::query::planner::plan::core::nodes::data_processing_node::DedupNode;
    use crate::query::planner::plan::core::nodes::graph_scan_node::GetNeighborsNode;
    use crate::query::planner::plan::core::nodes::start_node::StartNode;

    #[test]
    fn test_rule_name() {
        let rule = MergeGetNbrsAndDedupRule::new();
        assert_eq!(rule.name(), "MergeGetNbrsAndDedupRule");
    }

    #[test]
    fn test_rule_pattern() {
        let rule = MergeGetNbrsAndDedupRule::new();
        let pattern = rule.pattern();
        assert!(pattern.node.is_some());
    }

    #[test]
    fn test_merge_get_nbrs_and_dedup() {
        // 创建起始节点
        let start = PlanNodeEnum::Start(StartNode::new());

        // 创建Dedup节点
        let dedup = DedupNode::new(start).expect("创建DedupNode失败");
        let dedup_node = PlanNodeEnum::Dedup(dedup);

        // 创建GetNeighbors节点
        let get_neighbors = GetNeighborsNode::new(1, "v");
        let mut get_neighbors_node = PlanNodeEnum::GetNeighbors(get_neighbors);

        // 手动设置依赖关系
        if let PlanNodeEnum::GetNeighbors(ref mut gn) = get_neighbors_node {
            gn.deps_mut().clear();
            gn.deps_mut().push(Box::new(dedup_node));
        }

        // 应用规则
        let rule = MergeGetNbrsAndDedupRule::new();
        let mut ctx = RewriteContext::new();
        let result = rule.apply(&mut ctx, &get_neighbors_node).expect("应用规则失败");

        assert!(
            result.is_some(),
            "应该成功合并GetNeighbors和Dedup节点"
        );

        // 验证结果
        let transform_result = result.expect("Failed to apply rewrite rule");
        assert!(transform_result.erase_curr);
        assert_eq!(transform_result.new_nodes.len(), 1);

        // 验证新的GetNeighbors节点设置了dedup标志
        if let PlanNodeEnum::GetNeighbors(ref new_gn) = transform_result.new_nodes[0] {
            assert!(new_gn.dedup(), "新的GetNeighbors节点应该设置dedup标志");
        } else {
            panic!("转换结果应该是GetNeighbors节点");
        }
    }
}
