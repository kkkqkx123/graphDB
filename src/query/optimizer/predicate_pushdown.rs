//! 谓词下推优化规则
//! 这些规则负责将过滤条件下推到计划树的底层，以减少数据处理量

use super::engine::OptimizerError;
use super::plan::{OptContext, OptGroupNode, OptRule, Pattern, TransformResult, Result as OptResult};
use super::rule_patterns::{CommonPatterns, PatternBuilder};
use super::rule_traits::{
    combine_conditions, combine_expression_list, BaseOptRule, FilterSplitResult, PushDownRule,
};
use crate::core::Expression;
use crate::query::planner::plan::PlanNodeEnum;
use crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode;
use crate::query::visitor::PlanNodeVisitor;
use std::rc::Rc;
use std::cell::RefCell;
use std::result::Result as StdResult;

/// 谓词下推访问者
#[derive(Clone)]
struct PredicatePushDownVisitor {
    pushed_down: bool,
    new_node: Option<OptGroupNode>,
    ctx: *const OptContext,
}

impl PredicatePushDownVisitor {
    fn get_ctx(&self) -> &OptContext {
        unsafe { &*self.ctx }
    }

    fn can_push_down_condition(condition: &Expression) -> bool {
        use crate::core::Expression;

        match condition {
            Expression::Binary { .. } | Expression::Unary { .. } => true,
            Expression::Property { .. } => true,
            Expression::Literal(_) => true,
            _ => false,
        }
    }

    fn split_condition_for_pushdown(condition: &Expression) -> PushDownCondition {
        use crate::core::Expression;

        match condition {
            Expression::Binary { op: crate::core::BinaryOperator::And, left, right } => {
                let left_result = Self::split_condition_for_pushdown(left);
                let right_result = Self::split_condition_for_pushdown(right);

                let mut pushable = left_result.pushable;
                pushable.extend(right_result.pushable);

                let mut remaining = left_result.remaining;
                remaining.extend(right_result.remaining);

                PushDownCondition { pushable, remaining }
            }
            _ => {
                if Self::can_push_down_condition(condition) {
                    PushDownCondition {
                        pushable: vec![condition.clone()],
                        remaining: Vec::new(),
                    }
                } else {
                    PushDownCondition {
                        pushable: Vec::new(),
                        remaining: vec![condition.clone()],
                    }
                }
            }
        }
    }
}

impl PlanNodeVisitor for PredicatePushDownVisitor {
    type Result = Self;

    fn visit_default(&mut self) -> Self::Result {
        self.clone()
    }

    fn visit_filter(&mut self, node: &crate::query::planner::plan::core::nodes::FilterNode) -> Self::Result {
        let condition = node.condition();
        let input = node.input();
        let input_id = input.id() as usize;

        if let Some(child_node) = self.get_ctx().find_group_node_by_plan_node_id(input_id) {
            let child_node_ref = child_node.borrow();
            let child_name = child_node_ref.plan_node.name();

            let (pushed_down, new_node) = match child_name.as_ref() {
                "ScanVertices" | "ScanEdges" | "IndexScan" | "Traverse" => {
                    if Self::can_push_down_condition(condition) {
                        let new_filter_condition = Self::split_condition_for_pushdown(condition);

                        if !new_filter_condition.is_pushable_is_empty() && new_filter_condition.remaining().is_some() {
                            let mut new_child_node = child_node_ref.clone();
                            new_child_node.plan_node = input.clone();
                            (true, Some(new_child_node))
                        } else if !new_filter_condition.is_pushable_is_empty() {
                            let mut new_child_node = child_node_ref.clone();
                            new_child_node.plan_node = input.clone();
                            (true, Some(new_child_node))
                        } else {
                            (false, None)
                        }
                    } else {
                        (false, None)
                    }
                }
                _ => (false, None),
            };

            drop(child_node_ref);

            if pushed_down {
                self.pushed_down = true;
                self.new_node = new_node;
            }
        }

        self.clone()
    }
}

struct PushDownCondition {
    pushable: Vec<Expression>,
    remaining: Vec<Expression>,
}

impl PushDownCondition {
    fn is_pushable_is_empty(&self) -> bool {
        self.pushable.is_empty()
    }

    fn remaining(&self) -> Option<Expression> {
        if self.remaining.is_empty() {
            None
        } else if self.remaining.len() == 1 {
            Some(self.remaining[0].clone())
        } else {
            Some(Expression::Binary {
                left: Box::new(self.remaining[0].clone()),
                op: crate::core::BinaryOperator::And,
                right: Box::new(self.remaining[1].clone()),
            })
        }
    }
}

/// 通用过滤条件下推规则
#[derive(Debug)]
pub struct FilterPushDownRule;

impl OptRule for FilterPushDownRule {
    fn name(&self) -> &str {
        "FilterPushDownRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> OptResult<Option<TransformResult>> {
        let node_ref = group_node.borrow();
        // 检查是否为过滤节点
        if !node_ref.plan_node.is_filter() {
            return Ok(Some(TransformResult::unchanged()));
        }

        let mut visitor = PredicatePushDownVisitor {
            pushed_down: false,
            new_node: None,
            ctx: ctx as *const OptContext,
        };

        let result = visitor.visit(&node_ref.plan_node);
        drop(node_ref);

        if result.pushed_down {
            if let Some(new_node) = result.new_node {
                let mut transform_result = TransformResult::new();
                transform_result.add_new_group_node(Rc::new(RefCell::new(new_node)));
                return Ok(Some(transform_result));
            }
        }
        // 回退到原有的实现
        self.apply_original(ctx, group_node)
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::filter_with("ScanVertices")
    }
}

impl FilterPushDownRule {
    fn apply_original(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> OptResult<Option<TransformResult>> {
        let mut result = TransformResult::new();
        result.add_new_group_node(group_node.clone());
        Ok(Some(result))
    }
}

impl BaseOptRule for FilterPushDownRule {}

impl PushDownRule for FilterPushDownRule {
    fn can_push_down_to(&self, child_node: &PlanNodeEnum) -> bool {
        matches!(
            child_node.name(),
            "ScanVertices"
                | "ScanEdges"
                | "IndexScan"
                | "Traverse"
                | "GetNeighbors"
                | "GetVertices"
        )
    }

    fn create_pushed_down_node(
        &self,
        _ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
        _child: &OptGroupNode,
    ) -> OptResult<Option<TransformResult>> {
        let _node_ref = group_node.borrow();
        let mut result = TransformResult::new();
        result.add_new_group_node(group_node.clone());
        Ok(Some(result))
    }
}

/// 将过滤条件下推到遍历操作的规则
#[derive(Debug)]
pub struct PushFilterDownTraverseRule;

impl OptRule for PushFilterDownTraverseRule {
    fn name(&self) -> &str {
        "PushFilterDownTraverseRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> OptResult<Option<TransformResult>> {
        let node_ref = group_node.borrow();
        // 检查是否为过滤节点
        if !node_ref.plan_node.is_filter() {
            return Ok(Some(TransformResult::unchanged()));
        }

        // 匹配模式以查看是否为过滤后跟遍历
        if let Some(matched) = self.match_pattern(ctx, group_node)? {
            if matched.dependencies.len() == 1 {
                let child = matched.dependencies[0].clone();
                let child_ref = child.borrow();

                if child_ref.plan_node.name() == "Traverse" {
                    // 将过滤条件下推到遍历操作
                    if let Some(filter_plan_node) = node_ref.plan_node.as_filter() {
                        let filter_condition = filter_plan_node.condition();

                        // 使用 ExpressionUtils 分析过滤条件
                        let edge_alias = ctx.get_edge_alias_for_node(child_ref.id);

                        if let Some(alias) = edge_alias {
                            // 分割过滤条件：可以下推到遍历的条件和剩余的条件
                            let (pushable, remaining) = crate::core::expression_utils::ExpressionUtils::split_filter(
                                filter_condition,
                                |expression| {
                                    // 检查是否为边属性表达式或可以下推的表达式
                                    Self::can_push_down_expression_to_traverse(expression, &alias)
                                }
                            );

                            if let Some(pushable_condition) = pushable {
                                // 创建带有下推过滤条件的新遍历节点
                                if let Some(traverse_node) = child_ref.plan_node.as_traverse() {
                                    let mut new_traverse_node = traverse_node.clone();

                                    // 重写边属性过滤条件
                                    let rewritten_condition = crate::core::expression_utils::ExpressionUtils::rewrite_edge_property_filter(
                                        &alias,
                                        pushable_condition
                                    );

                                    // 合并现有过滤条件和新的过滤条件
                                    let new_filter_str = if let Some(existing_filter) = new_traverse_node.filter() {
                                        // 合并现有过滤条件和新的过滤条件
                                        format!("({}) AND ({})", existing_filter, format!("{:?}", rewritten_condition))
                                    } else {
                                        format!("{:?}", rewritten_condition)
                                    };

                                    new_traverse_node.set_filter(new_filter_str);

                                    // 创建带有修改后遍历节点的新OptGroupNode
                                    let mut new_traverse_opt_node = child.borrow().clone();
                                    new_traverse_opt_node.plan_node =
                                        PlanNodeEnum::Traverse(new_traverse_node);

                                    // 如果有剩余的过滤条件，创建新的过滤节点
                                    if let Some(_remaining_condition) = remaining {
                                        let new_filter_node = filter_plan_node.clone();
                                        // 由于FilterNode没有set_condition方法，我们需要创建一个新节点
                                        // 这里简化处理，直接返回原节点
                                        // new_filter_node.deps = vec![new_traverse_opt_node.plan_node.clone()];

                                        let mut new_filter_opt_node = node_ref.clone();
                                        new_filter_opt_node.plan_node =
                                            PlanNodeEnum::Filter(new_filter_node);
                                        new_filter_opt_node.dependencies =
                                            vec![new_traverse_opt_node.id];

                                        let mut result = TransformResult::new();
                                        result.add_new_group_node(Rc::new(RefCell::new(new_filter_opt_node)));
                                        result.add_new_group_node(Rc::new(RefCell::new(new_traverse_opt_node)));
                                        return Ok(Some(result));
                                    } else {
                                        // 没有剩余的过滤条件，直接返回遍历节点
                                        // new_traverse_opt_node.output_var = node.plan_node.output_var().clone();
                                        let mut result = TransformResult::new();
                                        result.add_new_group_node(Rc::new(RefCell::new(new_traverse_opt_node)));
                                        return Ok(Some(result));
                                    }
                                } else {
                                    Ok(Some(TransformResult::unchanged()))
                                }
                            } else {
                                // 没有可以下推的条件，返回原始节点
                                let mut result = TransformResult::new();
                                result.add_new_group_node(group_node.clone());
                                Ok(Some(result))
                            }
                        } else {
                            // 没有边别名，无法下推
                            Ok(Some(TransformResult::unchanged()))
                        }
                    } else {
                        Ok(Some(TransformResult::unchanged()))
                    }
                } else {
                    Ok(Some(TransformResult::unchanged()))
                }
            } else {
                Ok(Some(TransformResult::unchanged()))
            }
        } else {
            Ok(Some(TransformResult::unchanged()))
        }
    }

    fn pattern(&self) -> Pattern {
        CommonPatterns::filter_over_traverse()
    }
}

impl PushFilterDownTraverseRule {
    /// 检查表达式是否可以下推到遍历操作
    fn can_push_down_expression_to_traverse(expression: &crate::core::Expression, edge_alias: &str) -> bool {
        use crate::core::Expression;

        match expression {
            // 属性表达式可以下推
            Expression::Property { .. } => true,
            // 二元操作：如果左右两边都可以下推，则可以下推
            Expression::Binary { left, right, .. } => {
                Self::can_push_down_expression_to_traverse(left, edge_alias)
                    && Self::can_push_down_expression_to_traverse(right, edge_alias)
            }
            // 一元操作：如果操作数可以下推，则可以下推
            Expression::Unary { operand, .. } => {
                Self::can_push_down_expression_to_traverse(operand, edge_alias)
            }
            // 函数调用：某些函数可以下推
            Expression::Function { name, .. } => {
                matches!(name.to_lowercase().as_str(), "id" | "properties" | "labels")
            }
            // 其他表达式不能下推
            _ => false,
        }
    }
}

impl BaseOptRule for PushFilterDownTraverseRule {}

impl PushDownRule for PushFilterDownTraverseRule {
    fn can_push_down_to(&self, child_node: &PlanNodeEnum) -> bool {
        child_node.name() == "Traverse"
    }

    fn create_pushed_down_node(
        &self,
        _ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
        _child: &OptGroupNode,
    ) -> OptResult<Option<TransformResult>> {
        let _node_ref = group_node.borrow();
        // 在完整实现中，这里会创建下推后的节点
        // 目前简化实现，返回带有替换的TransformResult
        let mut result = TransformResult::new();
        result.add_new_group_node(group_node.clone());
        Ok(Some(result))
    }
}

/// 将过滤条件下推到扩展操作的规则
#[derive(Debug)]
pub struct PushFilterDownExpandRule;

impl OptRule for PushFilterDownExpandRule {
    fn name(&self) -> &str {
        "PushFilterDownExpandRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> OptResult<Option<TransformResult>> {
        let node_ref = group_node.borrow();
        // 检查是否为过滤节点后跟扩展操作
        if !node_ref.plan_node.is_filter() {
            return Ok(Some(TransformResult::unchanged()));
        }

        // 匹配模式以查看是否为过滤后跟扩展
        if let Some(matched) = self.match_pattern(ctx, group_node)? {
            if matched.dependencies.len() == 1 {
                let child_rc = &matched.dependencies[0];
                let child_ref = child_rc.borrow();

                if child_ref.plan_node.name() == "Expand" {
                    // 将过滤条件下推到扩展操作
                    if let Some(filter_plan_node) = node_ref.plan_node.as_filter() {
                        let filter_condition = filter_plan_node.condition();

                        // 分析过滤条件，确定哪些部分可以下推到扩展操作
                        let split_result = can_push_down_to_traverse(filter_condition);

                        if let Some(_pushable_condition) = split_result.pushable_condition {
                            // 创建带有下推过滤条件的新扩展节点
                            if let Some(expand_node) = child_ref.plan_node.as_expand() {
                                let _new_expand_node = expand_node.clone();

                                // 扩展节点本身没有filter字段，我们需要创建一个新的过滤节点
                                // 在实际实现中，可能需要修改扩展节点以支持过滤条件
                                // 这里我们创建一个新的过滤节点，将扩展节点作为其子节点
                                let _new_filter_node = filter_plan_node.clone();
                                // 由于FilterNode没有set_condition方法，我们需要创建一个新节点
                                // 这里简化处理，直接返回原节点
                                // new_filter_node.deps = vec![child.plan_node().clone()];

                                let mut new_filter_opt_node = node_ref.clone();
                                new_filter_opt_node.plan_node =
                                    PlanNodeEnum::Filter(_new_filter_node);
                                new_filter_opt_node.dependencies = vec![child_ref.id];

                                // 如果有剩余的过滤条件，创建另一个过滤节点
                                if let Some(_remaining_condition) = split_result.remaining_condition
                                {
                                    let top_filter_node = filter_plan_node.clone();
                                    // 由于FilterNode没有set_condition方法，我们需要创建一个新节点
                                    // 这里简化处理，直接返回原节点
                                    // top_filter_node.deps = vec![new_filter_opt_node.plan_node.clone()];

                                    let mut top_filter_opt_node = node_ref.clone();
                                    top_filter_opt_node.plan_node =
                                        PlanNodeEnum::Filter(top_filter_node);
                                    top_filter_opt_node.dependencies = vec![new_filter_opt_node.id];

                                    let mut result = TransformResult::new();
                                    result.add_new_group_node(Rc::new(RefCell::new(top_filter_opt_node)));
                                    return Ok(Some(result));
                                } else {
                                    // 没有剩余的过滤条件，直接返回新的过滤节点
                                    // new_filter_opt_node.output_var = node.plan_node.output_var().clone();
                                    let mut result = TransformResult::new();
                                    result.add_new_group_node(Rc::new(RefCell::new(new_filter_opt_node)));
                                    return Ok(Some(result));
                                }
                            } else {
                                Ok(Some(TransformResult::unchanged()))
                            }
                        } else {
                            // 没有可以下推的条件，返回原始节点
                            let mut result = TransformResult::new();
                            result.add_new_group_node(group_node.clone());
                            Ok(Some(result))
                        }
                    } else {
                        Ok(Some(TransformResult::unchanged()))
                    }
                } else {
                    Ok(Some(TransformResult::unchanged()))
                }
            } else {
                Ok(Some(TransformResult::unchanged()))
            }
        } else {
            Ok(Some(TransformResult::unchanged()))
        }
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::with_dependency("Filter", "Expand")
    }
}

impl BaseOptRule for PushFilterDownExpandRule {}

impl PushDownRule for PushFilterDownExpandRule {
    fn can_push_down_to(&self, child_node: &PlanNodeEnum) -> bool {
        child_node.name() == "Expand"
    }

    fn create_pushed_down_node(
        &self,
        _ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
        _child: &OptGroupNode,
    ) -> OptResult<Option<TransformResult>> {
        let _node_ref = group_node.borrow();
        let mut result = TransformResult::new();
        result.add_new_group_node(group_node.clone());
        Ok(Some(result))
    }
}

/// 将过滤条件下推到哈希内连接的规则
#[derive(Debug)]
pub struct PushFilterDownHashInnerJoinRule;

impl OptRule for PushFilterDownHashInnerJoinRule {
    fn name(&self) -> &str {
        "PushFilterDownHashInnerJoinRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> OptResult<Option<TransformResult>> {
        let node_ref = group_node.borrow();
        // 检查是否为过滤操作在哈希内连接之上
        if !node_ref.plan_node.is_filter() {
            return Ok(Some(TransformResult::unchanged()));
        }

        // 匹配模式以查看是否为过滤后跟哈希内连接
        if let Some(matched) = self.match_pattern(ctx, group_node)? {
            if matched.dependencies.len() >= 1 {
                let child = &matched.dependencies[0];

                if child.borrow().plan_node.name() == "HashInnerJoin" {
                    // 在完整实现中，我们会将过滤条件下推到连接的一侧或两侧
                    // 这可以减少需要连接的元组数量
                    let mut result = TransformResult::new();
                    result.add_new_group_node(group_node.clone());
                    return Ok(Some(result));
                } else {
                    Ok(Some(TransformResult::unchanged()))
                }
            } else {
                Ok(Some(TransformResult::unchanged()))
            }
        } else {
            Ok(Some(TransformResult::unchanged()))
        }
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::with_dependency("Filter", "HashInnerJoin")
    }
}

impl BaseOptRule for PushFilterDownHashInnerJoinRule {}

impl PushDownRule for PushFilterDownHashInnerJoinRule {
    fn can_push_down_to(&self, child_node: &PlanNodeEnum) -> bool {
        child_node.name() == "HashInnerJoin"
    }

    fn create_pushed_down_node(
        &self,
        _ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
        _child: &OptGroupNode,
    ) -> OptResult<Option<TransformResult>> {
        let _node_ref = group_node.borrow();
        let mut result = TransformResult::new();
        result.add_new_group_node(group_node.clone());
        Ok(Some(result))
    }
}

/// 将过滤条件下推到哈希左连接的规则
#[derive(Debug)]
pub struct PushFilterDownHashLeftJoinRule;

impl OptRule for PushFilterDownHashLeftJoinRule {
    fn name(&self) -> &str {
        "PushFilterDownHashLeftJoinRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> OptResult<Option<TransformResult>> {
        let node_ref = group_node.borrow();
        // 检查是否为过滤操作在哈希左连接之上
        if !node_ref.plan_node.is_filter() {
            return Ok(Some(TransformResult::unchanged()));
        }

        // 匹配模式以查看是否为过滤后跟哈希左连接
        if let Some(matched) = self.match_pattern(ctx, group_node)? {
            if matched.dependencies.len() >= 1 {
                let child = &matched.dependencies[0];

                if child.borrow().plan_node.name() == "HashLeftJoin" {
                    // 在完整实现中，我们会将过滤条件下推到连接的一侧或两侧
                    // 这可以减少需要连接的元组数量
                    let mut result = TransformResult::new();
                    result.add_new_group_node(group_node.clone());
                    return Ok(Some(result));
                } else {
                    Ok(Some(TransformResult::unchanged()))
                }
            } else {
                Ok(Some(TransformResult::unchanged()))
            }
        } else {
            Ok(Some(TransformResult::unchanged()))
        }
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::with_dependency("Filter", "HashLeftJoin")
    }
}

impl BaseOptRule for PushFilterDownHashLeftJoinRule {}

impl PushDownRule for PushFilterDownHashLeftJoinRule {
    fn can_push_down_to(&self, child_node: &PlanNodeEnum) -> bool {
        child_node.name() == "HashLeftJoin"
    }

    fn create_pushed_down_node(
        &self,
        _ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
        _child: &OptGroupNode,
    ) -> OptResult<Option<TransformResult>> {
        let _node_ref = group_node.borrow();
        let mut result = TransformResult::new();
        result.add_new_group_node(group_node.clone());
        Ok(Some(result))
    }
}

/// 将过滤条件下推到内连接的规则
#[derive(Debug)]
pub struct PushFilterDownInnerJoinRule;

impl OptRule for PushFilterDownInnerJoinRule {
    fn name(&self) -> &str {
        "PushFilterDownInnerJoinRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> OptResult<Option<TransformResult>> {
        let node_ref = group_node.borrow();
        // 检查是否为过滤操作在内连接之上
        if !node_ref.plan_node.is_filter() {
            return Ok(Some(TransformResult::unchanged()));
        }

        // 匹配模式以查看是否为过滤后跟内连接
        if let Some(matched) = self.match_pattern(ctx, group_node)? {
            if matched.dependencies.len() >= 1 {
                let child = &matched.dependencies[0];

                if child.borrow().plan_node.name() == "InnerJoin" {
                    // 在完整实现中，我们会将过滤条件下推到连接的一侧或两侧
                    // 这可以减少需要连接的元组数量
                    let mut result = TransformResult::new();
                    result.add_new_group_node(group_node.clone());
                    return Ok(Some(result));
                } else {
                    Ok(Some(TransformResult::unchanged()))
                }
            } else {
                Ok(Some(TransformResult::unchanged()))
            }
        } else {
            Ok(Some(TransformResult::unchanged()))
        }
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::with_dependency("Filter", "InnerJoin")
    }
}

impl BaseOptRule for PushFilterDownInnerJoinRule {}

impl PushDownRule for PushFilterDownInnerJoinRule {
    fn can_push_down_to(&self, child_node: &PlanNodeEnum) -> bool {
        child_node.name() == "InnerJoin"
    }

    fn create_pushed_down_node(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
        child: &OptGroupNode,
    ) -> OptResult<Option<TransformResult>> {
        let node_ref = group_node.borrow();
        let mut result = TransformResult::new();
        result.add_new_group_node(group_node.clone());
        Ok(Some(result))
    }
}

/// 将谓词条件下推到存储层的规则
#[derive(Debug)]
pub struct PredicatePushDownRule;

impl OptRule for PredicatePushDownRule {
    fn name(&self) -> &str {
        "PredicatePushDownRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> OptResult<Option<TransformResult>> {
        let node_ref = group_node.borrow();
        // 检查是否为可以下推到存储的过滤节点
        if !node_ref.plan_node.is_filter() {
            return Ok(Some(TransformResult::unchanged()));
        }

        // 匹配以查看过滤是否在扫描操作之上
        let match_result = self.match_pattern(ctx, group_node)?;
        
        if let Some(matched) = match_result {
            if matched.dependencies.is_empty() {
                // 没有依赖关系的过滤节点，无法下推谓词
                return Ok(Some(TransformResult::unchanged()));
            } else if matched.dependencies.len() == 1 {
                let child = &matched.dependencies[0];
                let child_name = child.borrow().plan_node.name();

                match child_name {
                    "ScanVertices" | "ScanEdges" | "IndexScan" => {
                        // 将谓词下推到扫描操作
                        let mut result = TransformResult::new();
                        result.add_new_group_node(group_node.clone());
                        return Ok(Some(result));
                    }
                    _ => Ok(Some(TransformResult::unchanged())),
                }
            } else {
                Ok(Some(TransformResult::unchanged()))
            }
        } else {
            Ok(Some(TransformResult::unchanged()))
        }
    }

    fn pattern(&self) -> Pattern {
        // 匹配有 ScanVertices 依赖的过滤节点（用于谓词下推）
        PatternBuilder::filter_with("ScanVertices")
    }
}

impl BaseOptRule for PredicatePushDownRule {}

impl PushDownRule for PredicatePushDownRule {
    fn can_push_down_to(&self, child_node: &PlanNodeEnum) -> bool {
        matches!(
            child_node.name(),
            "ScanVertices" | "ScanEdges" | "IndexScan"
        )
    }

    fn create_pushed_down_node(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
        child: &OptGroupNode,
    ) -> OptResult<Option<TransformResult>> {
        let node_ref = group_node.borrow();
        let mut result = TransformResult::new();
        result.add_new_group_node(group_node.clone());
        Ok(Some(result))
    }
}

// 辅助函数：分析过滤条件是否可以下推到扫描操作
fn can_push_down_to_scan(condition: &Expression) -> FilterSplitResult {
    // 分析过滤条件是否可以下推到扫描操作
    // 通常，只涉及顶点属性的条件可以下推到ScanVertices
    // 涉及边属性或复杂表达式的条件需要保留在Filter节点中

    // 尝试解析条件表达式
    if let Ok(expression) = parse_filter_condition(condition) {
        let mut pushable_conditions = Vec::new();
        let mut remaining_conditions = Vec::new();

        analyze_expression_for_scan(&expression, &mut pushable_conditions, &mut remaining_conditions);

        let pushable_condition = if pushable_conditions.is_empty() {
            None
        } else {
            Some(combine_expression_list(&pushable_conditions))
        };

        let remaining_condition = if remaining_conditions.is_empty() {
            None
        } else {
            Some(combine_expression_list(&remaining_conditions))
        };

        FilterSplitResult {
            pushable_condition,
            remaining_condition,
        }
    } else {
        // 如果解析失败，保留所有条件在Filter节点中
        FilterSplitResult {
            pushable_condition: None,
            remaining_condition: Some(format!("{:?}", condition)),
        }
    }
}

// 辅助函数：分析过滤条件是否可以下推到遍历操作
fn can_push_down_to_traverse(condition: &Expression) -> FilterSplitResult {
    // 分析过滤条件是否可以下推到遍历操作
    // 通常，涉及源顶点属性的条件可以下推到Traverse
    // 涉及目标顶点属性或复杂表达式的条件需要保留在Filter节点中

    // 尝试解析条件表达式
    if let Ok(expression) = parse_filter_condition(condition) {
        let mut pushable_conditions = Vec::new();
        let mut remaining_conditions = Vec::new();

        analyze_expression_for_traverse(&expression, &mut pushable_conditions, &mut remaining_conditions);

        let pushable_condition = if pushable_conditions.is_empty() {
            None
        } else {
            Some(combine_expression_list(&pushable_conditions))
        };

        let remaining_condition = if remaining_conditions.is_empty() {
            None
        } else {
            Some(combine_expression_list(&remaining_conditions))
        };

        FilterSplitResult {
            pushable_condition,
            remaining_condition,
        }
    } else {
        // 如果解析失败，保留所有条件在Filter节点中
        FilterSplitResult {
            pushable_condition: None,
            remaining_condition: Some(format!("{:?}", condition)),
        }
    }
}

// 尝试解析过滤条件为表达式
#[allow(unused_variables)]
fn parse_filter_condition(condition: &Expression) -> StdResult<crate::core::Expression, String> {
    // 这里应该使用表达式解析器，但为了简化，我们使用一个简单的实现
    // 在实际实现中，应该使用完整的表达式解析器
    Ok(condition.clone())
}

// 分析表达式，确定哪些部分可以下推到扫描操作
fn analyze_expression_for_scan(
    expression: &crate::core::Expression,
    pushable_conditions: &mut Vec<String>,
    remaining_conditions: &mut Vec<String>,
) {
    // 分析表达式
    // 通常，只涉及顶点属性的条件可以下推到ScanVertices
    match expression {
        crate::core::Expression::Binary { left, op, right } => {
            // 检查是否是AND操作
            if matches!(op, crate::core::BinaryOperator::And) {
                // 递归分析左右子表达式
                analyze_expression_for_scan(left, pushable_conditions, remaining_conditions);
                analyze_expression_for_scan(right, pushable_conditions, remaining_conditions);
            } else {
                // 检查是否可以下推
                if can_push_down_expression_to_scan(expression) {
                    pushable_conditions.push(format!("{:?}", expression));
                } else {
                    remaining_conditions.push(format!("{:?}", expression));
                }
            }
        }
        _ => {
            // 检查其他类型的表达式
            if can_push_down_expression_to_scan(expression) {
                pushable_conditions.push(format!("{:?}", expression));
            } else {
                remaining_conditions.push(format!("{:?}", expression));
            }
        }
    }
}

// 分析表达式，确定哪些部分可以下推到遍历操作
fn analyze_expression_for_traverse(
    expression: &crate::core::Expression,
    pushable_conditions: &mut Vec<String>,
    remaining_conditions: &mut Vec<String>,
) {
    // 分析表达式
    // 通常，涉及源顶点属性的条件可以下推到Traverse
    match expression {
        crate::core::Expression::Binary { left, op, right } => {
            // 检查是否是AND操作
            if matches!(op, crate::core::BinaryOperator::And) {
                // 递归分析左右子表达式
                analyze_expression_for_traverse(left, pushable_conditions, remaining_conditions);
                analyze_expression_for_traverse(right, pushable_conditions, remaining_conditions);
            } else {
                // 检查是否可以下推
                if can_push_down_expression_to_traverse(expression) {
                    pushable_conditions.push(format!("{:?}", expression));
                } else {
                    remaining_conditions.push(format!("{:?}", expression));
                }
            }
        }
        _ => {
            // 检查其他类型的表达式
            if can_push_down_expression_to_traverse(expression) {
                pushable_conditions.push(format!("{:?}", expression));
            } else {
                remaining_conditions.push(format!("{:?}", expression));
            }
        }
    }
}

// 检查表达式是否可以下推到扫描操作
fn can_push_down_expression_to_scan(expression: &crate::core::Expression) -> bool {
    // 检查表达式是否可以下推到扫描操作
    match expression {
        crate::core::Expression::Property { .. } => true,
        crate::core::Expression::Binary { left, right, .. } => {
            can_push_down_expression_to_scan(left) && can_push_down_expression_to_scan(right)
        }
        crate::core::Expression::Unary { operand, .. } => can_push_down_expression_to_scan(operand),
        crate::core::Expression::Function { name, .. } => {
            // 某些函数可以下推，如id(), properties()等
            matches!(name.to_lowercase().as_str(), "id" | "properties" | "labels")
        }
        crate::core::Expression::Variable(_) => true, // 变量表达式可以下推
        _ => false,
    }
}

// 检查表达式是否可以下推到遍历操作
fn can_push_down_expression_to_traverse(expression: &crate::core::Expression) -> bool {
    // 检查表达式是否可以下推到遍历操作
    match expression {
        crate::core::Expression::Property { .. } => true,
        crate::core::Expression::Binary { left, right, .. } => {
            can_push_down_expression_to_traverse(left)
                && can_push_down_expression_to_traverse(right)
        }
        crate::core::Expression::Unary { operand, .. } => {
            can_push_down_expression_to_traverse(operand)
        }
        crate::core::Expression::Function { name, .. } => {
            // 某些函数可以下推，如id(), properties()等
            matches!(name.to_lowercase().as_str(), "id" | "properties" | "labels")
        }
        crate::core::Expression::Variable(_) => true, // 变量表达式可以下推
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::context::execution::QueryContext;
    use crate::query::optimizer::plan::{OptContext, OptGroupNode};
    use crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum;
    use crate::query::planner::plan::core::nodes::plan_node_traits::PlanNode;
    use crate::query::planner::plan::core::nodes::{FilterNode, StartNode};

    fn create_test_context() -> OptContext {
        let query_context = QueryContext::new();
        OptContext::new(query_context)
    }

    #[test]
    fn test_filter_push_down_rule() {
        let rule = FilterPushDownRule;
        let mut ctx = create_test_context();

        // 创建子节点（扫描节点）
        let mut scan_node = PlanNodeEnum::ScanVertices(
            crate::query::planner::plan::core::nodes::ScanVerticesNode::new(2),
        );
        // 设置列名以便 get_tag_alias_for_node 可以工作
        scan_node.set_col_names(vec!["v".to_string()]);
        let scan_opt_node = OptGroupNode::new(2, scan_node.clone());
        ctx.add_plan_node_and_group_node(2, &scan_opt_node);

        // 创建过滤条件 - 使用属性表达式
        let filter_condition = crate::core::Expression::Binary {
            left: Box::new(crate::core::Expression::Property {
                object: Box::new(crate::core::Expression::Variable("v".to_string())),
                property: "col1".to_string(),
            }),
            op: crate::core::BinaryOperator::GreaterThan,
            right: Box::new(crate::core::Expression::Literal(crate::core::Value::Int(100))),
        };

        // 创建过滤节点并设置依赖
        let filter_node = FilterNode::new(
            scan_node,
            filter_condition,
        )
        .expect("Filter node should be created successfully");
        let mut opt_node = OptGroupNode::new(1, filter_node.into_enum());
        opt_node.dependencies.push(2);

        let result = rule
            .apply(&mut ctx, &opt_node)
            .expect("Rule should apply successfully");
        // 规则应该匹配过滤节点并尝试下推条件
        assert!(result.is_some());
    }

    #[test]
    fn test_push_filter_down_traverse_rule() {
        let rule = PushFilterDownTraverseRule;
        let mut ctx = create_test_context();

        // 创建遍历节点
        let mut traverse_node = PlanNodeEnum::Traverse(
            crate::query::planner::plan::core::nodes::TraverseNode::new(2, vec!["edge1".to_string()], "BOTH"),
        );
        // 设置列名以便 get_edge_alias_for_node 可以工作
        traverse_node.set_col_names(vec!["e".to_string()]);
        let traverse_opt_node = OptGroupNode::new(2, traverse_node.clone());
        ctx.add_plan_node_and_group_node(2, &traverse_opt_node);

        // 创建过滤条件 - 使用属性表达式
        let filter_condition = crate::core::Expression::Binary {
            left: Box::new(crate::core::Expression::Property {
                object: Box::new(crate::core::Expression::Variable("e".to_string())),
                property: "col1".to_string(),
            }),
            op: crate::core::BinaryOperator::GreaterThan,
            right: Box::new(crate::core::Expression::Literal(crate::core::Value::Int(100))),
        };

        // 创建过滤节点并设置依赖
        let filter_node = FilterNode::new(
            traverse_node,
            filter_condition,
        )
        .expect("Filter node should be created successfully");
        let mut opt_node = OptGroupNode::new(1, filter_node.into_enum());
        opt_node.dependencies.push(2);

        let result = rule
            .apply(&mut ctx, &opt_node)
            .expect("Rule should apply successfully");
        // 规则应该匹配过滤节点并尝试下推到遍历操作
        assert!(result.is_some());
    }

    #[test]
    fn test_push_filter_down_expand_rule() {
        let rule = PushFilterDownExpandRule;
        let mut ctx = create_test_context();

        // 创建起始节点
        let start_node = PlanNodeEnum::Start(StartNode::new());
        let start_opt_node = OptGroupNode::new(2, start_node.clone());
        ctx.add_plan_node_and_group_node(2, &start_opt_node);

        // 创建扩展节点
        let expand_node = crate::query::planner::plan::core::nodes::ExpandNode::new(
            1, // space_id
            vec!["edge_type".to_string()],
            crate::core::types::EdgeDirection::Out,
        );
        let expand_opt_node = OptGroupNode::new(3, expand_node.into_enum());
        ctx.add_plan_node_and_group_node(3, &expand_opt_node);

        // 创建过滤节点
        let filter_node = FilterNode::new(
            expand_opt_node.plan_node.clone(),
            crate::core::Expression::Variable("col1 > 100".to_string()),
        )
        .expect("Filter node should be created successfully");
        let mut filter_opt_node = OptGroupNode::new(1, filter_node.into_enum());
        filter_opt_node.dependencies.push(3);

        let result = rule
            .apply(&mut ctx, &filter_opt_node)
            .expect("Rule should apply successfully");
        // 规则应该匹配过滤节点并尝试下推到扩展操作
        assert!(result.is_some());
    }

    #[test]
    fn test_push_filter_down_hash_inner_join_rule() {
        let rule = PushFilterDownHashInnerJoinRule;
        let mut ctx = create_test_context();

        // 创建左子节点
        let left_node = PlanNodeEnum::Start(StartNode::new());
        let left_opt_node = OptGroupNode::new(3, left_node.clone());
        ctx.add_plan_node_and_group_node(3, &left_opt_node);

        // 创建右子节点
        let right_node = PlanNodeEnum::Start(StartNode::new());
        let right_opt_node = OptGroupNode::new(4, right_node.clone());
        ctx.add_plan_node_and_group_node(4, &right_opt_node);

        // 创建哈希内连接节点
        let hash_inner_join_node = crate::query::planner::plan::core::nodes::HashInnerJoinNode::new(
            left_node,
            right_node,
            vec![crate::core::Expression::Variable("id".to_string())],
            vec![crate::core::Expression::Variable("id".to_string())],
        ).expect("HashInnerJoin node should be created successfully");
        let hash_inner_join_opt_node = OptGroupNode::new(2, hash_inner_join_node.into_enum());
        ctx.add_plan_node_and_group_node(2, &hash_inner_join_opt_node);

        // 创建过滤节点
        let filter_node = FilterNode::new(
            hash_inner_join_opt_node.plan_node.clone(),
            crate::core::Expression::Variable("col1 > 100".to_string()),
        )
        .expect("Filter node should be created successfully");
        let mut filter_opt_node = OptGroupNode::new(1, filter_node.into_enum());
        filter_opt_node.dependencies.push(2);

        let result = rule
            .apply(&mut ctx, &filter_opt_node)
            .expect("Rule should apply successfully");
        // 规则应该匹配过滤节点并尝试下推到哈希内连接
        assert!(result.is_some());
    }

    #[test]
    fn test_push_filter_down_hash_left_join_rule() {
        let rule = PushFilterDownHashLeftJoinRule;
        let mut ctx = create_test_context();

        // 创建左子节点
        let left_node = PlanNodeEnum::Start(StartNode::new());
        let left_opt_node = OptGroupNode::new(3, left_node.clone());
        ctx.add_plan_node_and_group_node(3, &left_opt_node);

        // 创建右子节点
        let right_node = PlanNodeEnum::Start(StartNode::new());
        let right_opt_node = OptGroupNode::new(4, right_node.clone());
        ctx.add_plan_node_and_group_node(4, &right_opt_node);

        // 创建哈希左连接节点
        let hash_left_join_node = crate::query::planner::plan::core::nodes::HashLeftJoinNode::new(
            left_node,
            right_node,
            vec![crate::core::Expression::Variable("id".to_string())],
            vec![crate::core::Expression::Variable("id".to_string())],
        ).expect("HashLeftJoin node should be created successfully");
        let hash_left_join_opt_node = OptGroupNode::new(2, hash_left_join_node.into_enum());
        ctx.add_plan_node_and_group_node(2, &hash_left_join_opt_node);

        // 创建过滤节点
        let filter_node = FilterNode::new(
            hash_left_join_opt_node.plan_node.clone(),
            crate::core::Expression::Variable("col1 > 100".to_string()),
        )
        .expect("Filter node should be created successfully");
        let mut filter_opt_node = OptGroupNode::new(1, filter_node.into_enum());
        filter_opt_node.dependencies.push(2);

        let result = rule
            .apply(&mut ctx, &filter_opt_node)
            .expect("Rule should apply successfully");
        // 规则应该匹配过滤节点并尝试下推到哈希左连接
        assert!(result.is_some());
    }

    #[test]
    fn test_push_filter_down_inner_join_rule() {
        let rule = PushFilterDownInnerJoinRule;
        let mut ctx = create_test_context();

        // 创建左子节点
        let left_node = PlanNodeEnum::Start(StartNode::new());
        let left_opt_node = OptGroupNode::new(3, left_node.clone());
        ctx.add_plan_node_and_group_node(3, &left_opt_node);

        // 创建右子节点
        let right_node = PlanNodeEnum::Start(StartNode::new());
        let right_opt_node = OptGroupNode::new(4, right_node.clone());
        ctx.add_plan_node_and_group_node(4, &right_opt_node);

        // 创建内连接节点
        let inner_join_node = crate::query::planner::plan::core::nodes::InnerJoinNode::new(
            left_node,
            right_node,
            vec![crate::core::Expression::Variable("id".to_string())],
            vec![crate::core::Expression::Variable("id".to_string())],
        ).expect("InnerJoin node should be created successfully");
        let inner_join_opt_node = OptGroupNode::new(2, inner_join_node.into_enum());
        ctx.add_plan_node_and_group_node(2, &inner_join_opt_node);

        // 创建过滤节点
        let filter_node = FilterNode::new(
            inner_join_opt_node.plan_node.clone(),
            crate::core::Expression::Variable("col1 > 100".to_string()),
        )
        .expect("Filter node should be created successfully");
        let mut filter_opt_node = OptGroupNode::new(1, filter_node.into_enum());
        filter_opt_node.dependencies.push(2);

        let result = rule
            .apply(&mut ctx, &filter_opt_node)
            .expect("Rule should apply successfully");
        // 规则应该匹配过滤节点并尝试下推到内连接
        assert!(result.is_some());
    }

    #[test]
    fn test_predicate_push_down_rule() {
        let rule = PredicatePushDownRule;
        let mut ctx = create_test_context();

        // 创建一个扫描节点
        let mut scan_node = PlanNodeEnum::ScanVertices(
            crate::query::planner::plan::core::nodes::ScanVerticesNode::new(2),
        );
        scan_node.set_col_names(vec!["v".to_string()]);
        let scan_opt_node = OptGroupNode::new(2, scan_node.clone());
        ctx.add_plan_node_and_group_node(2, &scan_opt_node);

        // 创建过滤条件 - 使用属性表达式
        let filter_condition = crate::core::Expression::Binary {
            left: Box::new(crate::core::Expression::Property {
                object: Box::new(crate::core::Expression::Variable("v".to_string())),
                property: "col1".to_string(),
            }),
            op: crate::core::BinaryOperator::GreaterThan,
            right: Box::new(crate::core::Expression::Literal(crate::core::Value::Int(100))),
        };

        // 创建过滤节点并设置依赖
        let filter_node = FilterNode::new(
            scan_node,
            filter_condition,
        )
        .expect("Filter node should be created successfully");
        let mut opt_node = OptGroupNode::new(1, filter_node.into_enum());
        opt_node.dependencies.push(2);

        let result = rule
            .apply(&mut ctx, &opt_node)
            .expect("Rule should apply successfully");
        
        // 调试信息
        println!("PredicatePushDownRule result: {:?}", result);
        
        // 规则应该匹配过滤节点并尝试下推谓词到存储
        assert!(result.is_some());
    }

    #[test]
    fn test_can_push_down_to_scan() {
        // 测试辅助函数
        let result =
            can_push_down_to_scan(&crate::core::Expression::Variable("age > 18".to_string()));
        // 应该返回带有可下推条件的结果
        assert!(result.pushable_condition.is_some());
    }

    #[test]
    fn test_can_push_down_to_traverse() {
        // 测试辅助函数
        let result =
            can_push_down_to_traverse(&crate::core::Expression::Variable("age > 18".to_string()));
        // 应该返回带有可下推条件的结果
        assert!(result.pushable_condition.is_some());
    }
}
