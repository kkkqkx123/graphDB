//! 消除重复操作的规则
//!
//! 当 Dedup 节点的子节点本身就保证结果唯一性时，可以移除 Dedup 节点。
//!
//! # 转换示例
//!
//! Before:
//! ```text
//!   Dedup
//!       |
//!   IndexScan (索引扫描保证唯一性)
//! ```
//!
//! After:
//! ```text
//!   IndexScan
//! ```
//!
//! # 适用条件
//!
//! - Dedup 节点的子节点为 IndexScan、GetVertices 或 GetEdges
//! - 这些操作本身就保证结果的唯一性

use crate::query::planner::plan::PlanNodeEnum;
use crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode;
use crate::query::planner::plan::core::nodes::data_processing_node::DedupNode;
use crate::query::planner::rewrite::context::RewriteContext;
use crate::query::planner::rewrite::pattern::Pattern;
use crate::query::planner::rewrite::result::{RewriteResult, TransformResult};
use crate::query::planner::rewrite::rule::{RewriteRule, EliminationRule};

/// 消除重复操作的规则
///
/// 当子节点本身就保证结果唯一性时，移除 Dedup 节点
#[derive(Debug)]
pub struct DedupEliminationRule;

impl DedupEliminationRule {
    /// 创建规则实例
    pub fn new() -> Self {
        Self
    }

    /// 检查子节点是否保证唯一性
    fn child_guarantees_uniqueness(&self, child: &PlanNodeEnum) -> bool {
        // 索引扫描保证唯一性
        if child.is_index_scan() {
            return true;
        }
        
        // 根据节点类型判断
        match child {
            // 主键查询保证唯一性
            PlanNodeEnum::GetVertices(_) => true,
            PlanNodeEnum::GetEdges(_) => true,
            // 索引扫描相关节点
            PlanNodeEnum::ScanVertices(node) => {
                // 如果扫描有唯一性约束（如主键扫描）
                node.limit().map_or(false, |l| l == 1)
            }
            PlanNodeEnum::ScanEdges(node) => {
                node.limit().map_or(false, |l| l == 1)
            }
            // 其他保证唯一性的节点
            PlanNodeEnum::Start(_) => true,
            _ => false,
        }
    }
}

impl Default for DedupEliminationRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RewriteRule for DedupEliminationRule {
    fn name(&self) -> &'static str {
        "DedupEliminationRule"
    }

    fn pattern(&self) -> Pattern {
        Pattern::new_with_name("Dedup")
    }

    fn apply(
        &self,
        _ctx: &mut RewriteContext,
        node: &PlanNodeEnum,
    ) -> RewriteResult<Option<TransformResult>> {
        // 检查是否为 Dedup 节点
        let dedup_node = match node {
            PlanNodeEnum::Dedup(n) => n,
            _ => return Ok(None),
        };

        // 获取输入节点
        let input = dedup_node.input();

        // 检查子节点是否保证唯一性
        if !self.child_guarantees_uniqueness(input) {
            return Ok(None);
        }

        // 创建转换结果，用输入节点替换当前 Dedup 节点
        let mut result = TransformResult::new();
        result.erase_curr = true;
        result.add_new_node(input.clone());

        Ok(Some(result))
    }
}

impl EliminationRule for DedupEliminationRule {
    fn can_eliminate(&self, node: &PlanNodeEnum) -> bool {
        match node {
            PlanNodeEnum::Dedup(n) => {
                let input = n.input();
                self.child_guarantees_uniqueness(input)
            }
            _ => false,
        }
    }

    fn eliminate(
        &self,
        ctx: &mut RewriteContext,
        node: &PlanNodeEnum,
    ) -> RewriteResult<Option<TransformResult>> {
        self.apply(ctx, node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::planner::rewrite::rule::RewriteRule;

    #[test]
    fn test_dedup_elimination_rule_name() {
        let rule = DedupEliminationRule::new();
        assert_eq!(rule.name(), "DedupEliminationRule");
    }

    #[test]
    fn test_dedup_elimination_rule_pattern() {
        let rule = DedupEliminationRule::new();
        let pattern = rule.pattern();
        assert!(pattern.node.is_some());
    }

    #[test]
    fn test_child_guarantees_uniqueness() {
        let rule = DedupEliminationRule::new();
        
        // Start 节点保证唯一性
        let start_node = crate::query::planner::plan::core::nodes::start_node::StartNode::new();
        assert!(rule.child_guarantees_uniqueness(&PlanNodeEnum::Start(start_node)));
    }
}
