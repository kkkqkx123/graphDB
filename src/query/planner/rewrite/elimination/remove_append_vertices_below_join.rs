//! 移除连接下方的添加顶点操作的规则

use crate::query::planner::plan::PlanNodeEnum;
use crate::query::planner::plan::core::nodes::plan_node_traits::MultipleInputNode;
use crate::query::planner::rewrite::context::RewriteContext;
use crate::query::planner::rewrite::pattern::Pattern;
use crate::query::planner::rewrite::result::{RewriteResult, TransformResult};
use crate::query::planner::rewrite::rule::{RewriteRule, EliminationRule};

/// 移除连接下方的添加顶点操作的规则
///
/// # 转换示例
///
/// Before:
/// ```text
///   AppendVertices
///       |
///   InnerJoin
/// ```
///
/// After:
/// ```text
///   InnerJoin
/// ```
///
/// # 适用条件
///
/// - AppendVertices 节点的子节点为连接操作（InnerJoin、HashInnerJoin、HashLeftJoin）
/// - 连接操作已经包含了所需的顶点信息
#[derive(Debug)]
pub struct RemoveAppendVerticesBelowJoinRule;

impl RemoveAppendVerticesBelowJoinRule {
    /// 创建规则实例
    pub fn new() -> Self {
        Self
    }

    /// 检查节点是否为连接操作
    fn is_join_node(&self, node: &PlanNodeEnum) -> bool {
        node.is_inner_join() || node.is_hash_inner_join() || node.is_hash_left_join()
    }
}

impl Default for RemoveAppendVerticesBelowJoinRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RewriteRule for RemoveAppendVerticesBelowJoinRule {
    fn name(&self) -> &'static str {
        "RemoveAppendVerticesBelowJoinRule"
    }

    fn pattern(&self) -> Pattern {
        Pattern::new_with_name("AppendVertices")
            .with_dependency_name("InnerJoin")
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

        // 获取输入节点（MultipleInputNode 使用 inputs()）
        let inputs = append_vertices_node.inputs();
        if inputs.is_empty() {
            return Ok(None);
        }
        let input = &inputs[0];

        // 检查输入节点是否为连接操作
        if !self.is_join_node(input) {
            return Ok(None);
        }

        // 创建转换结果，用输入节点替换当前 AppendVertices 节点
        let mut result = TransformResult::new();
        result.erase_curr = true;
        result.add_new_node((**input).clone());

        Ok(Some(result))
    }
}

impl EliminationRule for RemoveAppendVerticesBelowJoinRule {
    fn can_eliminate(&self, node: &PlanNodeEnum) -> bool {
        match node {
            PlanNodeEnum::AppendVertices(n) => {
                let inputs = n.inputs();
                if inputs.is_empty() {
                    return false;
                }
                self.is_join_node(&inputs[0])
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
    fn test_remove_append_vertices_below_join_rule_name() {
        let rule = RemoveAppendVerticesBelowJoinRule::new();
        assert_eq!(rule.name(), "RemoveAppendVerticesBelowJoinRule");
    }

    #[test]
    fn test_remove_append_vertices_below_join_rule_pattern() {
        let rule = RemoveAppendVerticesBelowJoinRule::new();
        let pattern = rule.pattern();
        assert!(pattern.node.is_some());
    }
}
