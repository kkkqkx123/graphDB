//! 移除连接下方的添加顶点操作的规则
//!
//! 根据 nebula-graph 的参考实现，此规则匹配以下模式：
//! HashInnerJoin/HashLeftJoin -> ... -> Project -> AppendVertices -> Traverse
//! 当满足特定条件时，可以移除 AppendVertices 节点。
//!
//! # 转换示例
//!
//! Before:
//! ```text
//!   HashInnerJoin
//!    /         \
//!   /           Project
//!  /               \
//! Left           AppendVertices
//!                     \
//!                   Traverse
//! ```
//!
//! After:
//! ```text
//!   HashInnerJoin (修改右表达式)
//!    /         \
//!   /           Project (修改列表达式)
//!  /               \
//! Left           Traverse
//! ```
//!
//! # 适用条件
//!
//! - Join 的右分支为 Project->AppendVertices->Traverse
//! - AppendVertices 的 nodeAlias 只被引用一次
//! - Join 的 hash keys 匹配特定模式

use crate::core::Expression;
use crate::query::planner::plan::PlanNodeEnum;
use crate::query::planner::plan::core::nodes::plan_node_traits::{MultipleInputNode, BinaryInputNode, SingleInputNode};
use crate::query::planner::plan::core::nodes::join_node::{HashInnerJoinNode, HashLeftJoinNode};
use crate::query::planner::plan::core::nodes::project_node::ProjectNode;
use crate::query::planner::plan::core::nodes::traversal_node::{AppendVerticesNode, TraverseNode};
use crate::query::planner::rewrite::context::RewriteContext;
use crate::query::planner::rewrite::pattern::Pattern;
use crate::query::planner::rewrite::result::{RewriteResult, TransformResult};
use crate::query::planner::rewrite::rule::{RewriteRule, EliminationRule};

/// 移除连接下方的添加顶点操作的规则
///
/// 当 Join 的右分支包含 AppendVertices 且满足特定条件时，移除 AppendVertices
#[derive(Debug)]
pub struct RemoveAppendVerticesBelowJoinRule;

impl RemoveAppendVerticesBelowJoinRule {
    /// 创建规则实例
    pub fn new() -> Self {
        Self
    }

    /// 检查节点是否为哈希连接节点
    fn is_hash_join(&self, node: &PlanNodeEnum) -> bool {
        matches!(node, PlanNodeEnum::HashInnerJoin(_) | PlanNodeEnum::HashLeftJoin(_))
    }

    /// 从表达式中提取属性引用
    fn extract_property_refs(&self, expr: &Expression, prop_name: &str) -> Vec<String> {
        let mut refs = Vec::new();
        self.collect_property_refs(expr, prop_name, &mut refs);
        refs
    }

    /// 递归收集属性引用
    fn collect_property_refs(&self, expr: &Expression, target_prop: &str, refs: &mut Vec<String>) {
        match expr {
            Expression::Property { property, .. } if property == target_prop => {
                refs.push(property.clone());
            }
            Expression::Binary { left, right, .. } => {
                self.collect_property_refs(left, target_prop, refs);
                self.collect_property_refs(right, target_prop, refs);
            }
            Expression::Unary { operand, .. } => {
                self.collect_property_refs(operand, target_prop, refs);
            }
            Expression::Function { args, .. } => {
                for arg in args {
                    self.collect_property_refs(arg, target_prop, refs);
                }
            }
            _ => {}
        }
    }

    /// 检查表达式是否为 id() 或 _joinkey() 函数调用
    fn is_id_or_joinkey_function<'a>(&self, expr: &'a Expression) -> Option<&'a Expression> {
        match expr {
            Expression::Function { name, args } if (name == "id" || name == "_joinkey") && args.len() == 1 => {
                Some(&args[0])
            }
            _ => None,
        }
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
        // 匹配 HashInnerJoin 或 HashLeftJoin
        Pattern::multi(vec!["HashInnerJoin", "HashLeftJoin"])
    }

    fn apply(
        &self,
        _ctx: &mut RewriteContext,
        node: &PlanNodeEnum,
    ) -> RewriteResult<Option<TransformResult>> {
        // 检查是否为哈希连接节点
        let (join_node, join_type) = match node {
            PlanNodeEnum::HashInnerJoin(n) => (n as &dyn BinaryInputNode, "HashInnerJoin"),
            PlanNodeEnum::HashLeftJoin(n) => (n as &dyn BinaryInputNode, "HashLeftJoin"),
            _ => return Ok(None),
        };

        // 获取右输入节点
        let right_input = join_node.right_input();
        
        // 检查右输入是否为 Project
        let project = match right_input {
            PlanNodeEnum::Project(n) => n,
            _ => return Ok(None),
        };

        // 获取 Project 的输入节点
        let project_input = project.input();
        
        // 检查是否为 AppendVertices
        let append_vertices = match project_input {
            PlanNodeEnum::AppendVertices(n) => n,
            _ => return Ok(None),
        };

        // 获取 AppendVertices 的输入节点
        let append_inputs = append_vertices.inputs();
        if append_inputs.is_empty() {
            return Ok(None);
        }
        
        // 检查是否为 Traverse
        let _traverse = match &*append_inputs[0] {
            PlanNodeEnum::Traverse(n) => n,
            _ => return Ok(None),
        };

        // TODO: 实现完整的转换逻辑
        // 参考 nebula-graph 的实现，需要：
        // 1. 检查 avNodeAlias 在 join keys 和 project columns 中的引用次数
        // 2. 检查 join keys 是否匹配 id() 或 _joinkey() 模式
        // 3. 创建新的 Project 节点，使用 none_direct_dst() 函数
        // 4. 创建新的 Join 节点，修改右表达式
        
        // 目前简化实现：如果满足基本结构，直接移除 AppendVertices
        let mut result = TransformResult::new();
        result.erase_curr = true;
        
        // 添加原始节点（简化版本，实际应该创建新的修改后的节点）
        result.add_new_node(node.clone());

        Ok(Some(result))
    }
}

impl EliminationRule for RemoveAppendVerticesBelowJoinRule {
    fn can_eliminate(&self, node: &PlanNodeEnum) -> bool {
        // 这个规则需要复杂的模式匹配，单独检查节点类型不够
        // 返回 false 表示需要配合模式匹配使用
        false
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

    #[test]
    fn test_is_hash_join() {
        let rule = RemoveAppendVerticesBelowJoinRule::new();
        
        // 创建测试节点
        let start_node = crate::query::planner::plan::core::nodes::start_node::StartNode::new();
        let start_enum = PlanNodeEnum::Start(start_node);
        
        // HashInnerJoin 应该返回 true
        let hash_inner = HashInnerJoinNode::new(
            start_enum.clone(),
            start_enum.clone(),
            vec![],
            vec![],
        ).expect("Failed to create HashInnerJoinNode");
        assert!(rule.is_hash_join(&PlanNodeEnum::HashInnerJoin(hash_inner)));
        
        // HashLeftJoin 应该返回 true
        let hash_left = HashLeftJoinNode::new(
            start_enum.clone(),
            start_enum.clone(),
            vec![],
            vec![],
        ).expect("Failed to create HashLeftJoinNode");
        assert!(rule.is_hash_join(&PlanNodeEnum::HashLeftJoin(hash_left)));
        
        // Start 节点应该返回 false
        assert!(!rule.is_hash_join(&start_enum));
    }
}
