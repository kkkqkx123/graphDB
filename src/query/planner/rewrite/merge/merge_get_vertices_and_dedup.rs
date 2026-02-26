//! 合并获取顶点和去重操作的规则

use crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum;
use crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode;
use crate::query::planner::rewrite::context::RewriteContext;
use crate::query::planner::rewrite::pattern::Pattern;
use crate::query::planner::rewrite::result::{RewriteResult, TransformResult};
use crate::query::planner::rewrite::rule::{MergeRule, RewriteRule};

/// 合并获取顶点和去重操作的规则
///
/// # 转换示例
///
/// Before:
/// ```text
///   GetVertices
///       |
///   Dedup
///       |
///   ScanVertices
/// ```
///
/// After:
/// ```text
///   GetVertices(dedup=true)
///       |
///   ScanVertices
/// ```
///
/// # 适用条件
///
/// - 当前节点为GetVertices节点
/// - 子节点为Dedup节点
/// - 可以将去重操作合并到GetVertices中
#[derive(Debug)]
pub struct MergeGetVerticesAndDedupRule;

impl MergeGetVerticesAndDedupRule {
    /// 创建规则实例
    pub fn new() -> Self {
        Self
    }
}

impl Default for MergeGetVerticesAndDedupRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RewriteRule for MergeGetVerticesAndDedupRule {
    fn name(&self) -> &'static str {
        "MergeGetVerticesAndDedupRule"
    }

    fn pattern(&self) -> Pattern {
        Pattern::new_with_name("GetVertices").with_dependency_name("Dedup")
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

        // 检查第一个依赖是否为Dedup节点
        let dedup_node = match deps.first().map(|d| d.as_ref()) {
            Some(PlanNodeEnum::Dedup(n)) => n,
            _ => return Ok(None),
        };

        // 获取Dedup的输入作为新的输入
        let dedup_input = dedup_node.input().clone();

        // 创建新的GetVertices节点
        let mut new_get_vertices = get_vertices.clone();

        // 设置去重标志
        if !new_get_vertices.dedup() {
            new_get_vertices.set_dedup(true);
        }

        // 清除原有依赖并设置新的输入
        new_get_vertices.deps_mut().clear();
        new_get_vertices.deps_mut().push(Box::new(dedup_input));

        let mut result = TransformResult::new();
        result.erase_curr = true;
        result.add_new_node(PlanNodeEnum::GetVertices(new_get_vertices));

        Ok(Some(result))
    }
}

impl MergeRule for MergeGetVerticesAndDedupRule {
    fn can_merge(&self, parent: &PlanNodeEnum, child: &PlanNodeEnum) -> bool {
        parent.is_get_vertices() && child.is_dedup()
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
    use crate::query::planner::plan::core::nodes::graph_scan_node::GetVerticesNode;
    use crate::query::planner::plan::core::nodes::start_node::StartNode;

    #[test]
    fn test_rule_name() {
        let rule = MergeGetVerticesAndDedupRule::new();
        assert_eq!(rule.name(), "MergeGetVerticesAndDedupRule");
    }

    #[test]
    fn test_rule_pattern() {
        let rule = MergeGetVerticesAndDedupRule::new();
        let pattern = rule.pattern();
        assert!(pattern.node.is_some());
    }

    #[test]
    fn test_merge_get_vertices_and_dedup() {
        // 创建起始节点
        let start = PlanNodeEnum::Start(StartNode::new());

        // 创建Dedup节点
        let dedup = DedupNode::new(start).expect("创建DedupNode失败");
        let dedup_node = PlanNodeEnum::Dedup(dedup);

        // 创建GetVertices节点
        let get_vertices = GetVerticesNode::new(1, "v");
        let mut get_vertices_node = PlanNodeEnum::GetVertices(get_vertices);

        // 手动设置依赖关系
        if let PlanNodeEnum::GetVertices(ref mut gv) = get_vertices_node {
            gv.deps_mut().clear();
            gv.deps_mut().push(Box::new(dedup_node));
        }

        // 应用规则
        let rule = MergeGetVerticesAndDedupRule::new();
        let mut ctx = RewriteContext::new();
        let result = rule.apply(&mut ctx, &get_vertices_node).expect("应用规则失败");

        assert!(
            result.is_some(),
            "应该成功合并GetVertices和Dedup节点"
        );

        // 验证结果
        let transform_result = result.expect("Failed to apply rewrite rule");
        assert!(transform_result.erase_curr);
        assert_eq!(transform_result.new_nodes.len(), 1);

        // 验证新的GetVertices节点设置了dedup标志
        if let PlanNodeEnum::GetVertices(ref new_gv) = transform_result.new_nodes[0] {
            assert!(new_gv.dedup(), "新的GetVertices节点应该设置dedup标志");
        } else {
            panic!("转换结果应该是GetVertices节点");
        }
    }
}
