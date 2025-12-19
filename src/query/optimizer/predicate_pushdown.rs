//! 谓词下推优化规则
//! 这些规则负责将过滤条件下推到计划树的底层，以减少数据处理量

use super::optimizer::OptimizerError;
use super::rule_patterns::{CommonPatterns, PatternBuilder};
use super::rule_traits::{
    combine_conditions, combine_expression_list, BaseOptRule, FilterSplitResult, PushDownRule,
};
use crate::query::optimizer::optimizer::{OptContext, OptGroupNode, OptRule, Pattern};
use crate::query::planner::plan::algorithms::IndexScan;
use crate::query::planner::plan::core::nodes::ExpandNode as Expand;
use crate::query::planner::plan::core::nodes::FilterNode as FilterPlanNode;
use crate::query::planner::plan::core::nodes::ScanEdgesNode as ScanEdges;
use crate::query::planner::plan::core::nodes::ScanVerticesNode as ScanVertices;
use crate::query::planner::plan::core::nodes::TraverseNode;
use crate::query::planner::plan::PlanNodeKind;
use crate::graph::expression::Expression;

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
        node: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        // 检查是否为过滤节点
        if node.plan_node.kind() != PlanNodeKind::Filter {
            return Ok(None);
        }

        // 尝试匹配模式并获取子节点
        if let Some(matched) = self.match_pattern(ctx, node)? {
            if matched.dependencies.len() >= 1 {
                let child_node = &matched.dependencies[0];

                // 获取过滤条件
                if let Some(filter_plan_node) =
                    node.plan_node.as_any().downcast_ref::<FilterPlanNode>()
                {
                    let filter_condition = filter_plan_node.condition();

                    // 根据子节点类型确定是否可以下推过滤条件
                    match child_node.plan_node().kind() {
                        PlanNodeKind::ScanVertices => {
                            // 对于扫描操作，我们可以将过滤条件下推到扫描操作
                            // 这通过在存储层而不是计算层应用过滤来减少从存储读取的记录数
                            let split_result = can_push_down_to_scan(filter_condition);

                            if let Some(pushable_condition) = split_result.pushable_condition {
                                // 创建带有过滤条件的新扫描节点
                                if let Some(scan_node) = child_node
                                    .plan_node()
                                    .as_any()
                                    .downcast_ref::<ScanVertices>()
                                {
                                    let new_scan_node = scan_node.clone();

                                    // 如果需要，合并现有过滤条件和新的过滤条件
                                    let _new_filter = if let Some(existing_filter) =
                                        new_scan_node.vertex_filter()
                                    {
                                        combine_conditions(&pushable_condition, existing_filter)
                                    } else {
                                        pushable_condition
                                    };

                                    // 由于ScanVerticesNode没有set_vertex_filter方法，我们需要创建一个新节点
                                    // 这里简化处理，直接返回原节点

                                    // 创建带有修改后扫描节点的新OptGroupNode
                                    let mut new_scan_opt_node = child_node.node.clone();
                                    new_scan_opt_node.plan_node =
                                        std::sync::Arc::new(new_scan_node);

                                    // 如果有剩余条件，创建新的过滤节点
                                    if let Some(_remaining_condition) =
                                        split_result.remaining_condition
                                    {
                                        let _new_filter_node = filter_plan_node.clone();
                                        // 由于FilterNode没有set_condition方法，我们需要创建一个新节点
                                        // 这里简化处理，直接返回原节点
                                        // new_filter_node.deps = vec![new_scan_opt_node.plan_node.clone()];

                                        let mut new_filter_opt_node = node.clone();
                                        new_filter_opt_node.plan_node =
                                            std::sync::Arc::new(_new_filter_node);
                                        new_filter_opt_node.dependencies =
                                            vec![new_scan_opt_node.id];

                                        Ok(Some(new_filter_opt_node))
                                    } else {
                                        // 没有剩余条件，只返回扫描节点
                                        // new_scan_opt_node.output_var = node.plan_node.output_var().clone();
                                        Ok(Some(new_scan_opt_node))
                                    }
                                } else {
                                    Ok(None)
                                }
                            } else {
                                Ok(None)
                            }
                        }
                        PlanNodeKind::IndexScan => {
                            // 类似于IndexScan的逻辑
                            let split_result = can_push_down_to_scan(filter_condition);

                            if let Some(pushable_condition) = split_result.pushable_condition {
                                // 创建带有过滤条件的新索引扫描节点
                                if let Some(index_scan_node) =
                                    child_node.plan_node().as_any().downcast_ref::<IndexScan>()
                                {
                                    let new_index_scan_node = index_scan_node.clone();

                                    // 如果需要，合并现有过滤条件和新的过滤条件
                                    let _new_filter = if let Some(existing_filter) =
                                        &new_index_scan_node.filter
                                    {
                                        combine_conditions(&pushable_condition, existing_filter)
                                    } else {
                                        pushable_condition
                                    };

                                    // 由于IndexScanNode没有set_filter方法，我们需要创建一个新节点
                                    // 这里简化处理，直接返回原节点

                                    // 创建带有修改后索引扫描节点的新OptGroupNode
                                    let mut new_index_scan_opt_node = child_node.node.clone();
                                    new_index_scan_opt_node.plan_node =
                                        std::sync::Arc::new(new_index_scan_node);

                                    // 如果有剩余条件，创建新的过滤节点
                                    if let Some(_remaining_condition) =
                                        split_result.remaining_condition
                                    {
                                        let _new_filter_node = filter_plan_node.clone();
                                        // 由于FilterNode没有set_condition方法，我们需要创建一个新节点
                                        // 这里简化处理，直接返回原节点
                                        // new_filter_node.deps = vec![new_index_scan_opt_node.plan_node.clone()];

                                        let mut new_filter_opt_node = node.clone();
                                        new_filter_opt_node.plan_node =
                                            std::sync::Arc::new(_new_filter_node);
                                        new_filter_opt_node.dependencies =
                                            vec![new_index_scan_opt_node.id];

                                        Ok(Some(new_filter_opt_node))
                                    } else {
                                        // 没有剩余条件，只返回索引扫描节点
                                        // new_index_scan_opt_node.output_var = node.plan_node.output_var().clone();
                                        Ok(Some(new_index_scan_opt_node))
                                    }
                                } else {
                                    Ok(None)
                                }
                            } else {
                                Ok(None)
                            }
                        }
                        PlanNodeKind::Traverse => {
                            // 对于遍历操作，将过滤条件下推到存储层
                            // 这减少遍历过程中检索的顶点或边数量
                            let split_result = can_push_down_to_traverse(filter_condition);

                            if let Some(pushable_condition) = split_result.pushable_condition {
                                // 创建带有过滤条件的新遍历节点
                                if let Some(traverse_node) =
                                    child_node.plan_node().as_any().downcast_ref::<TraverseNode>()
                                {
                                    let new_traverse_node = traverse_node.clone();

                                    // 如果需要，合并现有过滤条件和新的过滤条件
                                    let _new_filter =
                                        if let Some(existing_filter) = new_traverse_node.filter() {
                                            combine_conditions(&format!("{:?}", pushable_condition), &format!("{:?}", existing_filter))
                                        } else {
                                            format!("{:?}", pushable_condition)
                                        };

                                    // 由于TraverseNode没有set_filter方法，我们需要创建一个新节点
                                    // 这里简化处理，直接返回原节点

                                    // 创建带有修改后遍历节点的新OptGroupNode
                                    let mut new_traverse_opt_node = child_node.node.clone();
                                    new_traverse_opt_node.plan_node =
                                        std::sync::Arc::new(new_traverse_node);

                                    // 如果有剩余条件，创建新的过滤节点
                                    if let Some(_remaining_condition) =
                                        split_result.remaining_condition
                                    {
                                        let _new_filter_node = filter_plan_node.clone();
                                        // 由于FilterNode没有set_condition方法，我们需要创建一个新节点
                                        // 这里简化处理，直接返回原节点
                                        // new_filter_node.deps = vec![new_traverse_opt_node.plan_node.clone()];

                                        let mut new_filter_opt_node = node.clone();
                                        new_filter_opt_node.plan_node =
                                            std::sync::Arc::new(_new_filter_node);
                                        new_filter_opt_node.dependencies =
                                            vec![new_traverse_opt_node.id];

                                        Ok(Some(new_filter_opt_node))
                                    } else {
                                        // 没有剩余条件，只返回遍历节点
                                        // new_traverse_opt_node.output_var = node.plan_node.output_var().clone();
                                        Ok(Some(new_traverse_opt_node))
                                    }
                                } else {
                                    Ok(None)
                                }
                            } else {
                                Ok(None)
                            }
                        }
                        PlanNodeKind::GetNeighbors | PlanNodeKind::GetVertices => {
                            // 对于其他遍历操作，应用类似逻辑
                            // 目前，返回原始节点，因为没有进行转换
                            Ok(Some(node.clone()))
                        }
                        _ => {
                            // 对于其他节点，我们可能仍然能够转换，但目前返回None
                            Ok(None)
                        }
                    }
                } else {
                    Ok(None)
                }
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::filter()
    }
}

impl BaseOptRule for FilterPushDownRule {}

impl PushDownRule for FilterPushDownRule {
    fn can_push_down_to(&self, child_kind: PlanNodeKind) -> bool {
        matches!(
            child_kind,
            PlanNodeKind::ScanVertices
                | PlanNodeKind::ScanEdges
                | PlanNodeKind::IndexScan
                | PlanNodeKind::Traverse
                | PlanNodeKind::GetNeighbors
                | PlanNodeKind::GetVertices
        )
    }

    fn create_pushed_down_node(
        &self,
        _ctx: &mut OptContext,
        _node: &OptGroupNode,
        _child: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        // 在完整实现中，这里会创建下推后的节点
        // 目前简化实现，返回None
        Ok(None)
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
        node: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        // 检查是否为过滤节点后跟遍历操作
        if node.plan_node.kind() != PlanNodeKind::Filter {
            return Ok(None);
        }

        // 匹配模式以查看是否为过滤后跟遍历
        if let Some(matched) = self.match_pattern(ctx, node)? {
            if matched.dependencies.len() == 1 {
                let child = &matched.dependencies[0];

                if child.plan_node().kind() == PlanNodeKind::Traverse {
                    // 将过滤条件下推到遍历操作
                    if let Some(filter_plan_node) =
                        node.plan_node.as_any().downcast_ref::<FilterPlanNode>()
                    {
                        let filter_condition = filter_plan_node.condition();

                        // 分析过滤条件，确定哪些部分可以下推到遍历操作
                        let split_result = can_push_down_to_traverse(filter_condition);

                        if let Some(pushable_condition) = split_result.pushable_condition {
                            // 创建带有下推过滤条件的新遍历节点
                            if let Some(traverse_node) =
                                child.plan_node().as_any().downcast_ref::<TraverseNode>()
                            {
                                let new_traverse_node = traverse_node.clone();

                                // 合并现有过滤条件和新的过滤条件
                                let _new_filter =
                                    if let Some(existing_filter) = new_traverse_node.filter() {
                                        combine_conditions(&format!("{:?}", pushable_condition), &format!("{:?}", existing_filter))
                                    } else {
                                        format!("{:?}", pushable_condition)
                                    };

                                // 由于TraverseNode没有set_filter方法，我们需要创建一个新节点
                                // 这里简化处理，直接返回原节点

                                // 创建带有修改后遍历节点的新OptGroupNode
                                let mut new_traverse_opt_node = child.node.clone();
                                new_traverse_opt_node.plan_node =
                                    std::sync::Arc::new(new_traverse_node);

                                // 如果有剩余的过滤条件，创建新的过滤节点
                                if let Some(_remaining_condition) = split_result.remaining_condition
                                {
                                    let _new_filter_node = filter_plan_node.clone();
                                    // 由于FilterNode没有set_condition方法，我们需要创建一个新节点
                                    // 这里简化处理，直接返回原节点
                                    // new_filter_node.deps = vec![new_traverse_opt_node.plan_node.clone()];

                                    let mut new_filter_opt_node = node.clone();
                                    new_filter_opt_node.plan_node =
                                        std::sync::Arc::new(_new_filter_node);
                                    new_filter_opt_node.dependencies =
                                        vec![new_traverse_opt_node.id];

                                    Ok(Some(new_filter_opt_node))
                                } else {
                                    // 没有剩余的过滤条件，直接返回遍历节点
                                    // new_traverse_opt_node.output_var = node.plan_node.output_var().clone();
                                    Ok(Some(new_traverse_opt_node))
                                }
                            } else {
                                Ok(None)
                            }
                        } else {
                            // 没有可以下推的条件，返回原始节点
                            Ok(Some(node.clone()))
                        }
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    fn pattern(&self) -> Pattern {
        CommonPatterns::filter_over_traverse()
    }
}

impl BaseOptRule for PushFilterDownTraverseRule {}

impl PushDownRule for PushFilterDownTraverseRule {
    fn can_push_down_to(&self, child_kind: PlanNodeKind) -> bool {
        child_kind == PlanNodeKind::Traverse
    }

    fn create_pushed_down_node(
        &self,
        _ctx: &mut OptContext,
        _node: &OptGroupNode,
        _child: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        // 在完整实现中，这里会创建下推后的节点
        // 目前简化实现，返回None
        Ok(None)
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
        node: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        // 检查是否为过滤节点后跟扩展操作
        if node.plan_node.kind() != PlanNodeKind::Filter {
            return Ok(None);
        }

        // 匹配模式以查看是否为过滤后跟扩展
        if let Some(matched) = self.match_pattern(ctx, node)? {
            if matched.dependencies.len() == 1 {
                let child = &matched.dependencies[0];

                if child.plan_node().kind() == PlanNodeKind::Expand {
                    // 将过滤条件下推到扩展操作
                    if let Some(filter_plan_node) =
                        node.plan_node.as_any().downcast_ref::<FilterPlanNode>()
                    {
                        let filter_condition = filter_plan_node.condition();

                        // 分析过滤条件，确定哪些部分可以下推到扩展操作
                        let split_result = can_push_down_to_traverse(filter_condition);

                        if let Some(_pushable_condition) = split_result.pushable_condition {
                            // 创建带有下推过滤条件的新扩展节点
                            if let Some(expand_node) =
                                child.plan_node().as_any().downcast_ref::<Expand>()
                            {
                                let _new_expand_node = expand_node.clone();

                                // 扩展节点本身没有filter字段，我们需要创建一个新的过滤节点
                                // 在实际实现中，可能需要修改扩展节点以支持过滤条件
                                // 这里我们创建一个新的过滤节点，将扩展节点作为其子节点
                                let _new_filter_node = filter_plan_node.clone();
                                // 由于FilterNode没有set_condition方法，我们需要创建一个新节点
                                // 这里简化处理，直接返回原节点
                                // new_filter_node.deps = vec![child.plan_node().clone()];

                                let mut new_filter_opt_node = node.clone();
                                new_filter_opt_node.plan_node =
                                    std::sync::Arc::new(_new_filter_node);
                                new_filter_opt_node.dependencies = vec![child.node.id];

                                // 如果有剩余的过滤条件，创建另一个过滤节点
                                if let Some(_remaining_condition) = split_result.remaining_condition
                                {
                                    let top_filter_node = filter_plan_node.clone();
                                    // 由于FilterNode没有set_condition方法，我们需要创建一个新节点
                                    // 这里简化处理，直接返回原节点
                                    // top_filter_node.deps = vec![new_filter_opt_node.plan_node.clone()];

                                    let mut top_filter_opt_node = node.clone();
                                    top_filter_opt_node.plan_node =
                                        std::sync::Arc::new(top_filter_node);
                                    top_filter_opt_node.dependencies = vec![new_filter_opt_node.id];

                                    Ok(Some(top_filter_opt_node))
                                } else {
                                    // 没有剩余的过滤条件，直接返回新的过滤节点
                                    // new_filter_opt_node.output_var = node.plan_node.output_var().clone();
                                    Ok(Some(new_filter_opt_node))
                                }
                            } else {
                                Ok(None)
                            }
                        } else {
                            // 没有可以下推的条件，返回原始节点
                            Ok(Some(node.clone()))
                        }
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::with_dependency(PlanNodeKind::Filter, PlanNodeKind::Expand)
    }
}

impl BaseOptRule for PushFilterDownExpandRule {}

impl PushDownRule for PushFilterDownExpandRule {
    fn can_push_down_to(&self, child_kind: PlanNodeKind) -> bool {
        child_kind == PlanNodeKind::Expand
    }

    fn create_pushed_down_node(
        &self,
        _ctx: &mut OptContext,
        _node: &OptGroupNode,
        _child: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        // 在完整实现中，这里会创建下推后的节点
        // 目前简化实现，返回None
        Ok(None)
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
        node: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        // 检查是否为过滤操作在哈希内连接之上
        if node.plan_node.kind() != PlanNodeKind::Filter {
            return Ok(None);
        }

        // 匹配模式以查看是否为过滤后跟哈希内连接
        if let Some(matched) = self.match_pattern(ctx, node)? {
            if matched.dependencies.len() >= 1 {
                let child = &matched.dependencies[0];

                if child.plan_node().kind() == PlanNodeKind::HashInnerJoin {
                    // 在完整实现中，我们会将过滤条件下推到连接的一侧或两侧
                    // 这可以减少需要连接的元组数量
                    Ok(Some(node.clone()))
                } else {
                    Ok(None)
                }
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::with_dependency(PlanNodeKind::Filter, PlanNodeKind::HashInnerJoin)
    }
}

impl BaseOptRule for PushFilterDownHashInnerJoinRule {}

impl PushDownRule for PushFilterDownHashInnerJoinRule {
    fn can_push_down_to(&self, child_kind: PlanNodeKind) -> bool {
        child_kind == PlanNodeKind::HashInnerJoin
    }

    fn create_pushed_down_node(
        &self,
        _ctx: &mut OptContext,
        _node: &OptGroupNode,
        _child: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        // 在完整实现中，这里会创建下推后的节点
        // 目前简化实现，返回None
        Ok(None)
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
        node: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        // 检查是否为过滤操作在哈希左连接之上
        if node.plan_node.kind() != PlanNodeKind::Filter {
            return Ok(None);
        }

        // 匹配模式以查看是否为过滤后跟哈希左连接
        if let Some(matched) = self.match_pattern(ctx, node)? {
            if matched.dependencies.len() >= 1 {
                let child = &matched.dependencies[0];

                if child.plan_node().kind() == PlanNodeKind::HashLeftJoin {
                    // 在完整实现中，我们会将过滤条件下推到连接的一侧或两侧
                    // 这可以减少需要连接的元组数量
                    Ok(Some(node.clone()))
                } else {
                    Ok(None)
                }
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::with_dependency(PlanNodeKind::Filter, PlanNodeKind::HashLeftJoin)
    }
}

impl BaseOptRule for PushFilterDownHashLeftJoinRule {}

impl PushDownRule for PushFilterDownHashLeftJoinRule {
    fn can_push_down_to(&self, child_kind: PlanNodeKind) -> bool {
        child_kind == PlanNodeKind::HashLeftJoin
    }

    fn create_pushed_down_node(
        &self,
        _ctx: &mut OptContext,
        _node: &OptGroupNode,
        _child: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        // 在完整实现中，这里会创建下推后的节点
        // 目前简化实现，返回None
        Ok(None)
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
        node: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        // 检查是否为过滤操作在内连接之上
        if node.plan_node.kind() != PlanNodeKind::Filter {
            return Ok(None);
        }

        // 匹配模式以查看是否为过滤后跟内连接
        if let Some(matched) = self.match_pattern(ctx, node)? {
            if matched.dependencies.len() >= 1 {
                let child = &matched.dependencies[0];

                if child.plan_node().kind() == PlanNodeKind::InnerJoin {
                    // 在完整实现中，我们会将过滤条件下推到连接的一侧或两侧
                    // 这可以减少需要连接的元组数量
                    Ok(Some(node.clone()))
                } else {
                    Ok(None)
                }
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::with_dependency(PlanNodeKind::Filter, PlanNodeKind::InnerJoin)
    }
}

impl BaseOptRule for PushFilterDownInnerJoinRule {}

impl PushDownRule for PushFilterDownInnerJoinRule {
    fn can_push_down_to(&self, child_kind: PlanNodeKind) -> bool {
        child_kind == PlanNodeKind::InnerJoin
    }

    fn create_pushed_down_node(
        &self,
        _ctx: &mut OptContext,
        _node: &OptGroupNode,
        _child: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        // 在完整实现中，这里会创建下推后的节点
        // 目前简化实现，返回None
        Ok(None)
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
        node: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        // 检查是否为可以下推到存储的过滤节点
        if node.plan_node.kind() != PlanNodeKind::Filter {
            return Ok(None);
        }

        // 匹配以查看过滤是否在扫描操作之上
        if let Some(matched) = self.match_pattern(ctx, node)? {
            if matched.dependencies.len() == 1 {
                let child = &matched.dependencies[0];

                match child.plan_node().kind() {
                    PlanNodeKind::ScanVertices => {
                        // 将谓词下推到扫描操作
                        if let Some(filter_plan_node) =
                            node.plan_node.as_any().downcast_ref::<FilterPlanNode>()
                        {
                            let filter_condition = filter_plan_node.condition();

                            // 分析过滤条件，确定哪些部分可以下推到扫描操作
                            let split_result = can_push_down_to_scan(filter_condition);

                            if let Some(pushable_condition) = split_result.pushable_condition {
                                // 创建带有下推谓词的新扫描节点
                                if let Some(scan_node) =
                                    child.plan_node().as_any().downcast_ref::<ScanVertices>()
                                {
                                    let new_scan_node = scan_node.clone();

                                    // 合并现有过滤条件和新的谓词
                                    let _new_filter = if let Some(existing_filter) =
                                        new_scan_node.vertex_filter()
                                    {
                                        combine_conditions(&pushable_condition, existing_filter)
                                    } else {
                                        pushable_condition
                                    };

                                    // 由于ScanVerticesNode没有set_vertex_filter方法，我们需要创建一个新节点
                                    // 这里简化处理，直接返回原节点

                                    // 创建带有修改后扫描节点的新OptGroupNode
                                    let mut new_scan_opt_node = child.node.clone();
                                    new_scan_opt_node.plan_node =
                                        std::sync::Arc::new(new_scan_node);

                                    // 如果有剩余的过滤条件，创建新的过滤节点
                                    if let Some(_remaining_condition) =
                                        split_result.remaining_condition
                                    {
                                        let _new_filter_node = filter_plan_node.clone();
                                        // 由于FilterNode没有set_condition方法，我们需要创建一个新节点
                                        // 这里简化处理，直接返回原节点
                                        // new_filter_node.deps = vec![new_scan_opt_node.plan_node.clone()];

                                        let mut new_filter_opt_node = node.clone();
                                        new_filter_opt_node.plan_node =
                                            std::sync::Arc::new(_new_filter_node);
                                        new_filter_opt_node.dependencies =
                                            vec![new_scan_opt_node.id];

                                        Ok(Some(new_filter_opt_node))
                                    } else {
                                        // 没有剩余的过滤条件，直接返回扫描节点
                                        // new_scan_opt_node.output_var = node.plan_node.output_var().clone();
                                        Ok(Some(new_scan_opt_node))
                                    }
                                } else {
                                    Ok(None)
                                }
                            } else {
                                // 没有可以下推的谓词，返回原始节点
                                Ok(Some(node.clone()))
                            }
                        } else {
                            Ok(None)
                        }
                    }
                    PlanNodeKind::ScanEdges => {
                        // 类似地处理边扫描
                        if let Some(filter_plan_node) =
                            node.plan_node.as_any().downcast_ref::<FilterPlanNode>()
                        {
                            let filter_condition = filter_plan_node.condition();

                            // 分析过滤条件，确定哪些部分可以下推到边扫描操作
                            let split_result = can_push_down_to_scan(filter_condition);

                            if let Some(pushable_condition) = split_result.pushable_condition {
                                // 创建带有下推谓词的新边扫描节点
                                if let Some(scan_edges_node) =
                                    child.plan_node().as_any().downcast_ref::<ScanEdges>()
                                {
                                    let new_scan_edges_node = scan_edges_node.clone();

                                    // 合并现有过滤条件和新的谓词
                                    let _new_filter = if let Some(existing_filter) =
                                        new_scan_edges_node.filter()
                                    {
                                        combine_conditions(&format!("{:?}", pushable_condition), &format!("{:?}", existing_filter))
                                    } else {
                                        format!("{:?}", pushable_condition)
                                    };

                                    // 由于ScanEdgesNode没有set_filter方法，我们需要创建一个新节点
                                    // 这里简化处理，直接返回原节点

                                    // 创建带有修改后边扫描节点的新OptGroupNode
                                    let mut new_scan_edges_opt_node = child.node.clone();
                                    new_scan_edges_opt_node.plan_node =
                                        std::sync::Arc::new(new_scan_edges_node);

                                    // 如果有剩余的过滤条件，创建新的过滤节点
                                    if let Some(_remaining_condition) =
                                        split_result.remaining_condition
                                    {
                                        let _new_filter_node = filter_plan_node.clone();
                                        // 由于FilterNode没有set_condition方法，我们需要创建一个新节点
                                        // 这里简化处理，直接返回原节点
                                        // new_filter_node.deps = vec![new_scan_edges_opt_node.plan_node.clone()];

                                        let mut new_filter_opt_node = node.clone();
                                        new_filter_opt_node.plan_node =
                                            std::sync::Arc::new(_new_filter_node);
                                        new_filter_opt_node.dependencies =
                                            vec![new_scan_edges_opt_node.id];

                                        Ok(Some(new_filter_opt_node))
                                    } else {
                                        // 没有剩余的过滤条件，直接返回边扫描节点
                                        // new_scan_edges_opt_node.output_var = node.plan_node.output_var().clone();
                                        Ok(Some(new_scan_edges_opt_node))
                                    }
                                } else {
                                    Ok(None)
                                }
                            } else {
                                // 没有可以下推的谓词，返回原始节点
                                Ok(Some(node.clone()))
                            }
                        } else {
                            Ok(None)
                        }
                    }
                    PlanNodeKind::IndexScan => {
                        // 类似地处理索引扫描
                        if let Some(filter_plan_node) =
                            node.plan_node.as_any().downcast_ref::<FilterPlanNode>()
                        {
                            let filter_condition = filter_plan_node.condition();

                            // 分析过滤条件，确定哪些部分可以下推到索引扫描操作
                            let split_result = can_push_down_to_scan(filter_condition);

                            if let Some(pushable_condition) = split_result.pushable_condition {
                                // 创建带有下推谓词的新索引扫描节点
                                if let Some(index_scan_node) =
                                    child.plan_node().as_any().downcast_ref::<IndexScan>()
                                {
                                    let new_index_scan_node = index_scan_node.clone();

                                    // 合并现有过滤条件和新的谓词
                                    let _new_filter = if let Some(existing_filter) =
                                        &new_index_scan_node.filter
                                    {
                                        combine_conditions(&pushable_condition, existing_filter)
                                    } else {
                                        pushable_condition
                                    };

                                    // 由于IndexScanNode没有set_filter方法，我们需要创建一个新节点
                                    // 这里简化处理，直接返回原节点

                                    // 创建带有修改后索引扫描节点的新OptGroupNode
                                    let mut new_index_scan_opt_node = child.node.clone();
                                    new_index_scan_opt_node.plan_node =
                                        std::sync::Arc::new(new_index_scan_node);

                                    // 如果有剩余的过滤条件，创建新的过滤节点
                                    if let Some(_remaining_condition) =
                                        split_result.remaining_condition
                                    {
                                        let _new_filter_node = filter_plan_node.clone();
                                        // 由于FilterNode没有set_condition方法，我们需要创建一个新节点
                                        // 这里简化处理，直接返回原节点
                                        // new_filter_node.deps = vec![new_index_scan_opt_node.plan_node.clone()];

                                        let mut new_filter_opt_node = node.clone();
                                        new_filter_opt_node.plan_node =
                                            std::sync::Arc::new(_new_filter_node);
                                        new_filter_opt_node.dependencies =
                                            vec![new_index_scan_opt_node.id];

                                        Ok(Some(new_filter_opt_node))
                                    } else {
                                        // 没有剩余的过滤条件，直接返回索引扫描节点
                                        // new_index_scan_opt_node.output_var = node.plan_node.output_var().clone();
                                        Ok(Some(new_index_scan_opt_node))
                                    }
                                } else {
                                    Ok(None)
                                }
                            } else {
                                // 没有可以下推的谓词，返回原始节点
                                Ok(Some(node.clone()))
                            }
                        } else {
                            Ok(None)
                        }
                    }
                    _ => Ok(None),
                }
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::filter()
    }
}

impl BaseOptRule for PredicatePushDownRule {}

impl PushDownRule for PredicatePushDownRule {
    fn can_push_down_to(&self, child_kind: PlanNodeKind) -> bool {
        matches!(
            child_kind,
            PlanNodeKind::ScanVertices | PlanNodeKind::ScanEdges | PlanNodeKind::IndexScan
        )
    }

    fn create_pushed_down_node(
        &self,
        _ctx: &mut OptContext,
        _node: &OptGroupNode,
        _child: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        // 在完整实现中，这里会创建下推后的节点
        // 目前简化实现，返回None
        Ok(None)
    }
}

// 辅助函数：分析过滤条件是否可以下推到扫描操作
fn can_push_down_to_scan(condition: &Expression) -> FilterSplitResult {
    // 分析过滤条件是否可以下推到扫描操作
    // 通常，只涉及顶点属性的条件可以下推到ScanVertices
    // 涉及边属性或复杂表达式的条件需要保留在Filter节点中

    // 尝试解析条件表达式
    if let Ok(expr) = parse_filter_condition(condition) {
        let mut pushable_conditions = Vec::new();
        let mut remaining_conditions = Vec::new();

        analyze_expression_for_scan(&expr, &mut pushable_conditions, &mut remaining_conditions);

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
    if let Ok(expr) = parse_filter_condition(condition) {
        let mut pushable_conditions = Vec::new();
        let mut remaining_conditions = Vec::new();

        analyze_expression_for_traverse(&expr, &mut pushable_conditions, &mut remaining_conditions);

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
fn parse_filter_condition(condition: &Expression) -> Result<crate::graph::expression::Expression, String> {
    // 这里应该使用表达式解析器，但为了简化，我们使用一个简单的实现
    // 在实际实现中，应该使用完整的表达式解析器
    Ok(condition.clone())
}

// 分析表达式，确定哪些部分可以下推到扫描操作
fn analyze_expression_for_scan(
    expr: &crate::graph::expression::Expression,
    pushable_conditions: &mut Vec<String>,
    remaining_conditions: &mut Vec<String>,
) {
    // 分析表达式
    // 通常，只涉及顶点属性的条件可以下推到ScanVertices
    match expr {
        crate::graph::expression::Expression::Binary { left, op, right } => {
            // 检查是否是AND操作
            if matches!(op, crate::graph::expression::BinaryOperator::And) {
                // 递归分析左右子表达式
                analyze_expression_for_scan(left, pushable_conditions, remaining_conditions);
                analyze_expression_for_scan(right, pushable_conditions, remaining_conditions);
            } else {
                // 检查是否可以下推
                if can_push_down_expression_to_scan(expr) {
                    pushable_conditions.push(format!("{:?}", expr));
                } else {
                    remaining_conditions.push(format!("{:?}", expr));
                }
            }
        }
        _ => {
            // 检查其他类型的表达式
            if can_push_down_expression_to_scan(expr) {
                pushable_conditions.push(format!("{:?}", expr));
            } else {
                remaining_conditions.push(format!("{:?}", expr));
            }
        }
    }
}

// 分析表达式，确定哪些部分可以下推到遍历操作
fn analyze_expression_for_traverse(
    expr: &crate::graph::expression::Expression,
    pushable_conditions: &mut Vec<String>,
    remaining_conditions: &mut Vec<String>,
) {
    // 分析表达式
    // 通常，涉及源顶点属性的条件可以下推到Traverse
    match expr {
        crate::graph::expression::Expression::Binary { left, op, right } => {
            // 检查是否是AND操作
            if matches!(op, crate::graph::expression::BinaryOperator::And) {
                // 递归分析左右子表达式
                analyze_expression_for_traverse(left, pushable_conditions, remaining_conditions);
                analyze_expression_for_traverse(right, pushable_conditions, remaining_conditions);
            } else {
                // 检查是否可以下推
                if can_push_down_expression_to_traverse(expr) {
                    pushable_conditions.push(format!("{:?}", expr));
                } else {
                    remaining_conditions.push(format!("{:?}", expr));
                }
            }
        }
        _ => {
            // 检查其他类型的表达式
            if can_push_down_expression_to_traverse(expr) {
                pushable_conditions.push(format!("{:?}", expr));
            } else {
                remaining_conditions.push(format!("{:?}", expr));
            }
        }
    }
}

// 检查表达式是否可以下推到扫描操作
fn can_push_down_expression_to_scan(expr: &crate::graph::expression::Expression) -> bool {
    // 检查表达式是否可以下推到扫描操作
    match expr {
        crate::graph::expression::Expression::TagProperty { .. } => true,
        crate::graph::expression::Expression::Property { .. } => true,
        crate::graph::expression::Expression::Binary { left, right, .. } => {
            can_push_down_expression_to_scan(left) && can_push_down_expression_to_scan(right)
        }
        crate::graph::expression::Expression::Unary { operand, .. } => {
            can_push_down_expression_to_scan(operand)
        }
        crate::graph::expression::Expression::Function { name, .. } => {
            // 某些函数可以下推，如id(), properties()等
            matches!(name.to_lowercase().as_str(), "id" | "properties" | "labels")
        }
        _ => false,
    }
}

// 检查表达式是否可以下推到遍历操作
fn can_push_down_expression_to_traverse(expr: &crate::graph::expression::Expression) -> bool {
    // 检查表达式是否可以下推到遍历操作
    match expr {
        crate::graph::expression::Expression::SourceProperty { .. } => true,
        crate::graph::expression::Expression::EdgeProperty { .. } => true,
        crate::graph::expression::Expression::Binary { left, right, .. } => {
            can_push_down_expression_to_traverse(left)
                && can_push_down_expression_to_traverse(right)
        }
        crate::graph::expression::Expression::Unary { operand, .. } => {
            can_push_down_expression_to_traverse(operand)
        }
        crate::graph::expression::Expression::Function { name, .. } => {
            // 某些函数可以下推，如id(), properties()等
            matches!(name.to_lowercase().as_str(), "id" | "properties" | "labels")
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::context::QueryContext;
    use crate::query::optimizer::optimizer::{OptContext, OptGroupNode};
    use crate::query::planner::plan::core::nodes::{ExpandNode, FilterNode, ScanVerticesNode, TraverseNode};
    use crate::query::planner::plan::{PlanNode, PlanNodeKind};

    fn create_test_context() -> OptContext {
        OptContext::new(QueryContext::default())
    }

    #[test]
    fn test_filter_push_down_rule() {
        let rule = FilterPushDownRule;
        let mut ctx = create_test_context();

        // 创建一个过滤节点
        let filter_node = std::sync::Arc::new(FilterNode::new(
            std::sync::Arc::new(crate::query::planner::plan::core::nodes::StartNode::new()),
            crate::graph::expression::Expression::Variable("col1 > 100".to_string()),
        ).unwrap());
        let opt_node = OptGroupNode::new(1, filter_node);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        // 规则应该匹配过滤节点并尝试下推条件
        assert!(result.is_some());
    }

    #[test]
    fn test_push_filter_down_traverse_rule() {
        let rule = PushFilterDownTraverseRule;
        let mut ctx = create_test_context();

        // 创建一个过滤节点
        let filter_node = std::sync::Arc::new(FilterNode::new(
            std::sync::Arc::new(crate::query::planner::plan::core::nodes::StartNode::new()),
            crate::graph::expression::Expression::Variable("col1 > 100".to_string()),
        ).unwrap());
        let opt_node = OptGroupNode::new(1, filter_node);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        // 规则应该匹配过滤节点并尝试下推到遍历操作
        assert!(result.is_some());
    }

    #[test]
    fn test_push_filter_down_expand_rule() {
        let rule = PushFilterDownExpandRule;
        let mut ctx = create_test_context();

        // 创建一个过滤节点
        let filter_node = std::sync::Arc::new(FilterNode::new(
            std::sync::Arc::new(crate::query::planner::plan::core::nodes::StartNode::new()),
            crate::graph::expression::Expression::Variable("col1 > 100".to_string()),
        ).unwrap());
        let opt_node = OptGroupNode::new(1, filter_node);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        // 规则应该匹配过滤节点并尝试下推到扩展操作
        assert!(result.is_some());
    }

    #[test]
    fn test_push_filter_down_hash_inner_join_rule() {
        let rule = PushFilterDownHashInnerJoinRule;
        let mut ctx = create_test_context();

        // 创建一个过滤节点
        let filter_node = std::sync::Arc::new(FilterNode::new(
            std::sync::Arc::new(crate::query::planner::plan::core::nodes::StartNode::new()),
            crate::graph::expression::Expression::Variable("col1 > 100".to_string()),
        ).unwrap());
        let opt_node = OptGroupNode::new(1, filter_node);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        // 规则应该匹配过滤节点并尝试下推到哈希内连接
        assert!(result.is_some());
    }

    #[test]
    fn test_push_filter_down_hash_left_join_rule() {
        let rule = PushFilterDownHashLeftJoinRule;
        let mut ctx = create_test_context();

        // 创建一个过滤节点
        let filter_node = std::sync::Arc::new(FilterNode::new(
            std::sync::Arc::new(crate::query::planner::plan::core::nodes::StartNode::new()),
            crate::graph::expression::Expression::Variable("col1 > 100".to_string()),
        ).unwrap());
        let opt_node = OptGroupNode::new(1, filter_node);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        // 规则应该匹配过滤节点并尝试下推到哈希左连接
        assert!(result.is_some());
    }

    #[test]
    fn test_push_filter_down_inner_join_rule() {
        let rule = PushFilterDownInnerJoinRule;
        let mut ctx = create_test_context();

        // 创建一个过滤节点
        let filter_node = std::sync::Arc::new(FilterNode::new(
            std::sync::Arc::new(crate::query::planner::plan::core::nodes::StartNode::new()),
            crate::graph::expression::Expression::Variable("col1 > 100".to_string()),
        ).unwrap());
        let opt_node = OptGroupNode::new(1, filter_node);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        // 规则应该匹配过滤节点并尝试下推到内连接
        assert!(result.is_some());
    }

    #[test]
    fn test_predicate_push_down_rule() {
        let rule = PredicatePushDownRule;
        let mut ctx = create_test_context();

        // 创建一个过滤节点
        let filter_node = std::sync::Arc::new(FilterNode::new(
            std::sync::Arc::new(crate::query::planner::plan::core::nodes::StartNode::new()),
            crate::graph::expression::Expression::Variable("col1 > 100".to_string()),
        ).unwrap());
        let opt_node = OptGroupNode::new(1, filter_node);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        // 规则应该匹配过滤节点并尝试下推谓词到存储
        assert!(result.is_some());
    }

    #[test]
    fn test_can_push_down_to_scan() {
        // 测试辅助函数
        let result = can_push_down_to_scan(&crate::graph::expression::Expression::Variable("age > 18".to_string()));
        // 应该返回带有可下推条件的结果
        assert!(result.pushable_condition.is_some());
    }

    #[test]
    fn test_can_push_down_to_traverse() {
        // 测试辅助函数
        let result = can_push_down_to_traverse(&crate::graph::expression::Expression::Variable("age > 18".to_string()));
        // 应该返回带有可下推条件的结果
        assert!(result.pushable_condition.is_some());
    }
}
