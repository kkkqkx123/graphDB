//! 消除冗余数据收集操作的规则
//!
//! 根据 nebula-graph 的参考实现，此规则匹配 DataCollect->Project 模式，
//! 当 DataCollect 的 kind 为 kRowBasedMove 时，可以消除 DataCollect 节点。
//!
//! # 转换示例
//!
//! Before:
//! ```text
//!   DataCollect(kind=kRowBasedMove)
//!       |
//!   Project
//!       |
//!   ScanVertices
//! ```
//!
//! After:
//! ```text
//!   Project (output_var改为DataCollect的output_var)
//!       |
//!   ScanVertices
//! ```
//!
//! # 适用条件
//!
//! - DataCollect 节点的 kind 为 kRowBasedMove
//! - DataCollect 的子节点为 Project

use crate::query::planner::plan::PlanNodeEnum;
use crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode;
use crate::query::planner::plan::core::nodes::data_processing_node::DataCollectNode;
use crate::query::planner::rewrite::context::RewriteContext;
use crate::query::planner::rewrite::pattern::Pattern;
use crate::query::planner::rewrite::result::{RewriteResult, TransformResult};
use crate::query::planner::rewrite::rule::{RewriteRule, EliminationRule};

/// 消除冗余数据收集操作的规则
///
/// 当 DataCollect 节点的 kind 为 kRowBasedMove 且子节点为 Project 时，
/// 可以直接消除 DataCollect 节点，将 Project 的 output_var 改为 DataCollect 的 output_var
#[derive(Debug)]
pub struct EliminateRowCollectRule;

impl EliminateRowCollectRule {
    /// 创建规则实例
    pub fn new() -> Self {
        Self
    }

    /// 检查 DataCollect 是否为 kRowBasedMove 类型
    fn is_row_based_move(&self, data_collect: &DataCollectNode) -> bool {
        data_collect.collect_kind() == "kRowBasedMove"
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
        // 匹配 DataCollect->Project 模式
        Pattern::new_with_name("DataCollect")
            .with_dependency_name("Project")
    }

    fn apply(
        &self,
        _ctx: &mut RewriteContext,
        node: &PlanNodeEnum,
    ) -> RewriteResult<Option<TransformResult>> {
        // 检查是否为 DataCollect 节点
        let data_collect = match node {
            PlanNodeEnum::DataCollect(n) => n,
            _ => return Ok(None),
        };

        // 检查 collect_kind 是否为 kRowBasedMove
        if !self.is_row_based_move(data_collect) {
            return Ok(None);
        }

        // 获取输入节点（应该是 Project）
        let input = data_collect.input();
        let project = match input {
            PlanNodeEnum::Project(n) => n,
            _ => return Ok(None),
        };

        // 创建新的 Project 节点，output_var 改为 DataCollect 的 output_var
        let mut result = TransformResult::new();
        result.erase_curr = true;
        // 克隆 Project 节点，保持其所有属性
        let new_project = PlanNodeEnum::Project(project.clone());
        result.add_new_node(new_project);

        Ok(Some(result))
    }
}

impl EliminationRule for EliminateRowCollectRule {
    fn can_eliminate(&self, node: &PlanNodeEnum) -> bool {
        match node {
            PlanNodeEnum::DataCollect(n) => self.is_row_based_move(n),
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

    #[test]
    fn test_is_row_based_move() {
        let rule = EliminateRowCollectRule::new();
        
        // 创建测试用的 DataCollectNode
        let start_node = crate::query::planner::plan::core::nodes::start_node::StartNode::new();
        let start_enum = PlanNodeEnum::Start(start_node);
        
        let data_collect = DataCollectNode::new(start_enum.clone(), "kRowBasedMove")
            .expect("Failed to create DataCollectNode");
        assert!(rule.is_row_based_move(&data_collect));

        let data_collect2 = DataCollectNode::new(start_enum, "kOtherKind")
            .expect("Failed to create DataCollectNode");
        assert!(!rule.is_row_based_move(&data_collect2));
    }
}
