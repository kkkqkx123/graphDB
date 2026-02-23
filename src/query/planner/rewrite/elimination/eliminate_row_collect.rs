//! 消除冗余数据收集操作的规则

use crate::query::planner::plan::PlanNodeEnum;
use crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode;
use crate::query::planner::rewrite::context::RewriteContext;
use crate::query::planner::rewrite::pattern::Pattern;
use crate::query::planner::rewrite::result::{RewriteResult, TransformResult};
use crate::query::planner::rewrite::rule::{RewriteRule, EliminationRule};

/// 消除冗余数据收集操作的规则
///
/// # 转换示例
///
/// Before:
/// ```text
///   DataCollect(kind=kRowBasedMove)
///       |
///   ScanVertices
/// ```
///
/// After:
/// ```text
///   ScanVertices
/// ```
///
/// # 适用条件
///
/// - DataCollect 节点的 kind 为 kRowBasedMove
/// - 子节点可以直接返回结果
#[derive(Debug)]
pub struct EliminateRowCollectRule;

impl EliminateRowCollectRule {
    /// 创建规则实例
    pub fn new() -> Self {
        Self
    }
}

impl Default for EliminateRowCollectRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RewriteRule for EliminateRowCollectRule {
    fn name(&self) -> &'static str {
        "EliminateRowCollectRule"
    }

    fn pattern(&self) -> Pattern {
        Pattern::new_with_name("DataCollect")
    }

    fn apply(
        &self,
        _ctx: &mut RewriteContext,
        node: &PlanNodeEnum,
    ) -> RewriteResult<Option<TransformResult>> {
        // 检查是否为 DataCollect 节点
        let data_collect_node = match node {
            PlanNodeEnum::DataCollect(n) => n,
            _ => return Ok(None),
        };

        // 检查 collect_kind 是否为 kRowBasedMove
        if data_collect_node.collect_kind() != "kRowBasedMove" {
            return Ok(None);
        }

        // 获取输入节点
        let input = data_collect_node.input();

        // 创建转换结果，用输入节点替换当前 DataCollect 节点
        let mut result = TransformResult::new();
        result.erase_curr = true;
        result.add_new_node(input.clone());

        Ok(Some(result))
    }
}

impl EliminationRule for EliminateRowCollectRule {
    fn can_eliminate(&self, node: &PlanNodeEnum) -> bool {
        match node {
            PlanNodeEnum::DataCollect(n) => {
                n.collect_kind() == "kRowBasedMove"
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
    fn test_eliminate_row_collect_rule_name() {
        let rule = EliminateRowCollectRule::new();
        assert_eq!(rule.name(), "EliminateRowCollectRule");
    }

    #[test]
    fn test_eliminate_row_collect_rule_pattern() {
        let rule = EliminateRowCollectRule::new();
        let pattern = rule.pattern();
        assert!(pattern.node.is_some());
    }
}
