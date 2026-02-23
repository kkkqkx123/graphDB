//! 消除冗余添加顶点操作的规则

use crate::query::planner::plan::PlanNodeEnum;
use crate::query::planner::plan::core::nodes::plan_node_traits::MultipleInputNode;
use crate::query::planner::rewrite::context::RewriteContext;
use crate::query::planner::rewrite::pattern::Pattern;
use crate::query::planner::rewrite::result::{RewriteResult, TransformResult};
use crate::query::planner::rewrite::rule::{RewriteRule, EliminationRule};

/// 消除冗余添加顶点操作的规则
///
/// # 转换示例
///
/// Before:
/// ```text
///   AppendVertices(vids=[], tag_ids=[])
///       |
///   GetNeighbors
/// ```
///
/// After:
/// ```text
///   GetNeighbors
/// ```
///
/// # 适用条件
///
/// - AppendVertices 节点的 vids 和 tag_ids 都为空
#[derive(Debug)]
pub struct EliminateAppendVerticesRule;

impl EliminateAppendVerticesRule {
    /// 创建规则实例
    pub fn new() -> Self {
        Self
    }
}

impl Default for EliminateAppendVerticesRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RewriteRule for EliminateAppendVerticesRule {
    fn name(&self) -> &'static str {
        "EliminateAppendVerticesRule"
    }

    fn pattern(&self) -> Pattern {
        Pattern::new_with_name("AppendVertices")
    }

    fn apply(
        &self,
        _ctx: &mut RewriteContext,
        node: &PlanNodeEnum,
    ) -> RewriteResult<Option<TransformResult>> {
        // 检查是否为 AppendVertices 节点
        let append_vertices_node = match node {
            PlanNodeEnum::AppendVertices(n) => n,
            _ => return Ok(None),
        };

        // 检查 vids 和 tag_ids 是否都为空
        if !append_vertices_node.vids().is_empty() || !append_vertices_node.tag_ids().is_empty() {
            return Ok(None);
        }

        // 获取输入节点（MultipleInputNode 使用 inputs()）
        let inputs = append_vertices_node.inputs();
        if inputs.is_empty() {
            return Ok(None);
        }

        // 创建转换结果，用输入节点替换当前 AppendVertices 节点
        let mut result = TransformResult::new();
        result.erase_curr = true;
        result.add_new_node((**&inputs[0]).clone());

        Ok(Some(result))
    }
}

impl EliminationRule for EliminateAppendVerticesRule {
    fn can_eliminate(&self, node: &PlanNodeEnum) -> bool {
        match node {
            PlanNodeEnum::AppendVertices(n) => {
                n.vids().is_empty() && n.tag_ids().is_empty()
            }
            _ => false,
        }
    }

    fn eliminate(
        &self,
        _ctx: &mut RewriteContext,
        node: &PlanNodeEnum,
    ) -> RewriteResult<Option<TransformResult>> {
        self.apply(_ctx, node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eliminate_append_vertices_rule_name() {
        let rule = EliminateAppendVerticesRule::new();
        assert_eq!(rule.name(), "EliminateAppendVerticesRule");
    }

    #[test]
    fn test_eliminate_append_vertices_rule_pattern() {
        let rule = EliminateAppendVerticesRule::new();
        let pattern = rule.pattern();
        assert!(pattern.node.is_some());
    }
}
