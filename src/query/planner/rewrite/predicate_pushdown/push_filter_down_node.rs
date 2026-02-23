//! 将过滤条件下推到Traverse/AppendVertices节点的规则
//!
//! 该规则识别 Traverse/AppendVertices 节点中的 vFilter，
//! 并将可下推的过滤条件下推到数据源。

use crate::core::Expression;
use crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum;
use crate::query::planner::plan::core::nodes::traversal_node::TraverseNode;
use crate::query::planner::plan::core::nodes::AppendVerticesNode;
use crate::query::planner::rewrite::context::RewriteContext;
use crate::query::planner::rewrite::pattern::Pattern;
use crate::query::planner::rewrite::result::{RewriteResult, TransformResult};
use crate::query::planner::rewrite::rule::{PushDownRule, RewriteRule};
use crate::query::optimizer::expression_utils::{split_filter, check_col_name};

/// 将过滤条件下推到Traverse/AppendVertices节点的规则
///
/// # 转换示例
///
/// Before:
/// ```text
///   Traverse(vFilter: v.age > 18)
/// ```
///
/// After:
/// ```text
///   Traverse(vFilter: <remained>, firstStepFilter: v.age > 18)
/// ```
///
/// # 适用条件
///
/// - Traverse 或 AppendVertices 节点存在 vFilter
/// - vFilter 可以部分下推到 firstStepFilter
#[derive(Debug)]
pub struct PushFilterDownNodeRule;

impl PushFilterDownNodeRule {
    /// 创建规则实例
    pub fn new() -> Self {
        Self
    }
}

impl Default for PushFilterDownNodeRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RewriteRule for PushFilterDownNodeRule {
    fn name(&self) -> &'static str {
        "PushFilterDownNodeRule"
    }

    fn pattern(&self) -> Pattern {
        Pattern::new_with_name("Traverse")
    }

    fn apply(
        &self,
        _ctx: &mut RewriteContext,
        node: &PlanNodeEnum,
    ) -> RewriteResult<Option<TransformResult>> {
        match node {
            PlanNodeEnum::Traverse(traverse) => {
                self.apply_to_traverse(traverse)
            }
            PlanNodeEnum::AppendVertices(append) => {
                self.apply_to_append_vertices(append)
            }
            _ => Ok(None),
        }
    }
}

impl PushFilterDownNodeRule {
    /// 应用到 Traverse 节点
    fn apply_to_traverse(
        &self,
        traverse: &TraverseNode,
    ) -> RewriteResult<Option<TransformResult>> {
        // 检查是否存在 vFilter
        let v_filter = match traverse.v_filter() {
            Some(filter) => filter,
            None => return Ok(None),
        };

        // 获取列名用于判断可下推的表达式
        let col_names = traverse.col_names().to_vec();

        // 定义选择器：检查表达式是否只涉及当前节点的列
        let picker = |expr: &Expression| -> bool {
            check_col_name(&col_names, expr)
        };

        // 分割过滤条件
        let (filter_picked, filter_remained) = split_filter(v_filter, picker);

        // 如果没有可以下推的条件，则不进行转换
        let picked = match filter_picked {
            Some(f) => f,
            None => return Ok(None),
        };

        // 创建新的 Traverse 节点
        let mut new_traverse = traverse.clone();

        // 设置 firstStepFilter
        if let Some(existing) = traverse.first_step_filter() {
            // 合并现有条件
            let combined = Expression::Binary {
                left: Box::new(picked),
                op: crate::core::types::operators::BinaryOperator::And,
                right: Box::new(existing.clone()),
            };
            new_traverse.set_first_step_filter(combined);
        } else {
            new_traverse.set_first_step_filter(picked);
        }

        // 更新 vFilter
        if let Some(remained) = filter_remained {
            new_traverse.set_v_filter(remained);
        }

        // 构建转换结果
        let mut result = TransformResult::new();
        result.erase_curr = true;
        result.add_new_node(PlanNodeEnum::Traverse(new_traverse));

        Ok(Some(result))
    }

    /// 应用到 AppendVertices 节点
    fn apply_to_append_vertices(
        &self,
        append: &AppendVerticesNode,
    ) -> RewriteResult<Option<TransformResult>> {
        // 检查是否存在 vFilter
        let v_filter = match append.v_filter() {
            Some(filter) => filter,
            None => return Ok(None),
        };

        // 获取列名用于判断可下推的表达式
        let col_names = append.col_names().to_vec();

        // 定义选择器：检查表达式是否只涉及当前节点的列
        let picker = |expr: &Expression| -> bool {
            check_col_name(&col_names, expr)
        };

        // 分割过滤条件
        let (filter_picked, filter_remained) = split_filter(v_filter, picker);

        // 如果没有可以下推的条件，则不进行转换
        let picked = match filter_picked {
            Some(f) => f,
            None => return Ok(None),
        };

        // 创建新的 AppendVertices 节点
        let mut new_append = append.clone();

        // 设置 filter
        let picked_str = match serde_json::to_string(&picked) {
            Ok(s) => s,
            Err(_) => return Ok(None),
        };

        if let Some(existing) = append.filter() {
            let combined = format!("{{\"and\": [{}, {}]}}", picked_str, existing);
            new_append.set_filter(combined);
        } else {
            new_append.set_filter(picked_str);
        }

        // 更新 vFilter
        if let Some(remained) = filter_remained {
            new_append.set_v_filter(remained);
        }

        // 构建转换结果
        let mut result = TransformResult::new();
        result.erase_curr = true;
        result.add_new_node(PlanNodeEnum::AppendVertices(new_append));

        Ok(Some(result))
    }
}

impl PushDownRule for PushFilterDownNodeRule {
    fn can_push_down(&self, node: &PlanNodeEnum, _target: &PlanNodeEnum) -> bool {
        matches!(node, PlanNodeEnum::Traverse(_) | PlanNodeEnum::AppendVertices(_))
    }

    fn push_down(
        &self,
        _ctx: &mut RewriteContext,
        node: &PlanNodeEnum,
        _target: &PlanNodeEnum,
    ) -> RewriteResult<Option<TransformResult>> {
        self.apply(_ctx, node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_name() {
        let rule = PushFilterDownNodeRule::new();
        assert_eq!(rule.name(), "PushFilterDownNodeRule");
    }

    #[test]
    fn test_rule_pattern() {
        let rule = PushFilterDownNodeRule::new();
        let pattern = rule.pattern();
        assert!(pattern.node.is_some());
    }
}
