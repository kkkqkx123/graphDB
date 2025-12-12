//! Filter optimization rules for NebulaGraph
//! These rules optimize filter operations based on NebulaGraph's implementation

use super::optimizer::OptimizerError;
use crate::query::optimizer::optimizer::{
    MatchedResult, OptContext, OptGroupNode, OptRule, Pattern,
};
use crate::query::planner::plan::operations::data_processing_ops::Filter as FilterPlanNode;
use crate::query::planner::plan::operations::graph_scan_ops::ScanVertices;
use crate::query::planner::plan::operations::traversal_ops::Traverse;
use crate::query::planner::plan::scan_nodes::IndexScan;
use crate::query::planner::plan::{PlanNode, PlanNodeKind};
use crate::query::validator::Variable;
use std::any::Any;

// 辅助结构，用于表示过滤条件分离的结果
#[derive(Debug, Clone)]
pub struct FilterSplitResult {
    pub pushable_condition: Option<String>,  // 可以下推的条件
    pub remaining_condition: Option<String>, // 保留在Filter节点的条件
}

// 辅助函数，用于分析过滤条件是否可以下推到扫描操作
fn can_push_down_to_scan(condition: &str) -> FilterSplitResult {
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
            remaining_condition: Some(condition.to_string()),
        }
    }
}

// 辅助函数，用于分析过滤条件是否可以下推到遍历操作
fn can_push_down_to_traverse(condition: &str) -> FilterSplitResult {
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
            remaining_condition: Some(condition.to_string()),
        }
    }
}

// 辅助函数，用于合并两个过滤条件
fn combine_conditions(cond1: &str, cond2: &str) -> String {
    if cond1.is_empty() {
        cond2.to_string()
    } else if cond2.is_empty() {
        cond1.to_string()
    } else {
        format!("({}) AND ({})", cond1, cond2)
    }
}

// 辅助函数，用于检查条件是否是永真式
fn is_tautology(condition: &str) -> bool {
    // 检查一些常见的永真式
    match condition.trim() {
        "1 = 1" | "true" | "TRUE" | "True" => true,
        // 检查更复杂的永真式，如 a = a
        _ => {
            // 尝试解析表达式并检查是否为永真式
            if let Ok(expr) = parse_filter_condition(condition) {
                is_expression_tautology(&expr)
            } else {
                false
            }
        }
    }
}

// 尝试解析过滤条件为表达式
fn parse_filter_condition(
    condition: &str,
) -> Result<crate::graph::expression::expr_type::Expression, String> {
    // 这里应该使用表达式解析器，但为了简化，我们使用一个简单的实现
    // 在实际实现中，应该使用完整的表达式解析器
    Err("Expression parser not implemented".to_string())
}

// 分析表达式，确定哪些部分可以下推到扫描操作
fn analyze_expression_for_scan(
    expr: &crate::graph::expression::expr_type::Expression,
    pushable_conditions: &mut Vec<String>,
    remaining_conditions: &mut Vec<String>,
) {
    // 分析表达式
    // 通常，只涉及顶点属性的条件可以下推到ScanVertices
    match expr {
        crate::graph::expression::expr_type::Expression::BinaryOp(left, op, right) => {
            // 检查是否是AND操作
            if matches!(op, crate::graph::expression::binary::BinaryOperator::And) {
                // 递归分析左右子表达式
                analyze_expression_for_scan(left, pushable_conditions, remaining_conditions);
                analyze_expression_for_scan(right, pushable_conditions, remaining_conditions);
            } else {
                // 检查是否可以下推
                if can_push_down_expression_to_scan(expr) {
                    pushable_conditions.push(format!("{}", expr));
                } else {
                    remaining_conditions.push(format!("{}", expr));
                }
            }
        }
        _ => {
            // 检查其他类型的表达式
            if can_push_down_expression_to_scan(expr) {
                pushable_conditions.push(format!("{}", expr));
            } else {
                remaining_conditions.push(format!("{}", expr));
            }
        }
    }
}

// 分析表达式，确定哪些部分可以下推到遍历操作
fn analyze_expression_for_traverse(
    expr: &crate::graph::expression::expr_type::Expression,
    pushable_conditions: &mut Vec<String>,
    remaining_conditions: &mut Vec<String>,
) {
    // 分析表达式
    // 通常，涉及源顶点属性的条件可以下推到Traverse
    match expr {
        crate::graph::expression::expr_type::Expression::BinaryOp(left, op, right) => {
            // 检查是否是AND操作
            if matches!(op, crate::graph::expression::binary::BinaryOperator::And) {
                // 递归分析左右子表达式
                analyze_expression_for_traverse(left, pushable_conditions, remaining_conditions);
                analyze_expression_for_traverse(right, pushable_conditions, remaining_conditions);
            } else {
                // 检查是否可以下推
                if can_push_down_expression_to_traverse(expr) {
                    pushable_conditions.push(format!("{}", expr));
                } else {
                    remaining_conditions.push(format!("{}", expr));
                }
            }
        }
        _ => {
            // 检查其他类型的表达式
            if can_push_down_expression_to_traverse(expr) {
                pushable_conditions.push(format!("{}", expr));
            } else {
                remaining_conditions.push(format!("{}", expr));
            }
        }
    }
}

// 检查表达式是否可以下推到扫描操作
fn can_push_down_expression_to_scan(
    expr: &crate::graph::expression::expr_type::Expression,
) -> bool {
    // 检查表达式是否可以下推到扫描操作
    match expr {
        crate::graph::expression::expr_type::Expression::TagProperty { .. } => true,
        crate::graph::expression::expr_type::Expression::Property(_) => true,
        crate::graph::expression::expr_type::Expression::BinaryOp(left, _, right) => {
            can_push_down_expression_to_scan(left) && can_push_down_expression_to_scan(right)
        }
        crate::graph::expression::expr_type::Expression::UnaryOp(_, operand) => {
            can_push_down_expression_to_scan(operand)
        }
        crate::graph::expression::expr_type::Expression::Function(name, _) => {
            // 某些函数可以下推，如id(), properties()等
            matches!(name.to_lowercase().as_str(), "id" | "properties" | "labels")
        }
        _ => false,
    }
}

// 检查表达式是否可以下推到遍历操作
fn can_push_down_expression_to_traverse(
    expr: &crate::graph::expression::expr_type::Expression,
) -> bool {
    // 检查表达式是否可以下推到遍历操作
    match expr {
        crate::graph::expression::expr_type::Expression::SourceProperty { .. } => true,
        crate::graph::expression::expr_type::Expression::EdgeProperty { .. } => true,
        crate::graph::expression::expr_type::Expression::BinaryOp(left, _, right) => {
            can_push_down_expression_to_traverse(left)
                && can_push_down_expression_to_traverse(right)
        }
        crate::graph::expression::expr_type::Expression::UnaryOp(_, operand) => {
            can_push_down_expression_to_traverse(operand)
        }
        crate::graph::expression::expr_type::Expression::Function(name, _) => {
            // 某些函数可以下推，如id(), properties()等
            matches!(name.to_lowercase().as_str(), "id" | "properties" | "labels")
        }
        _ => false,
    }
}

// 合并表达式列表
fn combine_expression_list(exprs: &[String]) -> String {
    if exprs.is_empty() {
        String::new()
    } else if exprs.len() == 1 {
        exprs[0].clone()
    } else {
        format!("({})", exprs.join(") AND ("))
    }
}

// 检查表达式是否为永真式
fn is_expression_tautology(expr: &crate::graph::expression::expr_type::Expression) -> bool {
    match expr {
        crate::graph::expression::expr_type::Expression::BinaryOp(left, op, right) => {
            match op {
                crate::graph::expression::binary::BinaryOperator::Eq => {
                    // 检查是否是 a = a 形式
                    format!("{}", left) == format!("{}", right)
                }
                crate::graph::expression::binary::BinaryOperator::And => {
                    // 检查是否两个子表达式都是永真式
                    is_expression_tautology(left) && is_expression_tautology(right)
                }
                crate::graph::expression::binary::BinaryOperator::Or => {
                    // 检查是否至少有一个子表达式是永真式
                    is_expression_tautology(left) || is_expression_tautology(right)
                }
                _ => false,
            }
        }
        crate::graph::expression::expr_type::Expression::Constant(value) => match value {
            crate::core::Value::Bool(true) => true,
            _ => false,
        },
        _ => false,
    }
}

// A rule that pushes down filters where possible
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
        // Check if this is a filter node
        if node.plan_node.kind() != PlanNodeKind::Filter {
            return Ok(None);
        }

        // Try to match the pattern and get the child node
        if let Some(matched) = self.match_pattern(ctx, node)? {
            if matched.dependencies.len() >= 1 {
                let child_node = &matched.dependencies[0];

                // Get the filter condition from the filter node
                if let Some(filter_plan_node) =
                    node.plan_node.as_any().downcast_ref::<FilterPlanNode>()
                {
                    let filter_condition = &filter_plan_node.condition;

                    // Determine if we can push down the filter based on the child node type
                    match child_node.plan_node().kind() {
                        PlanNodeKind::ScanVertices => {
                            // For scan operations, we can push the filter condition down to the scan operation
                            // This optimization reduces the number of records read from storage
                            // by applying the filter at the storage layer rather than at the compute layer
                            let split_result = can_push_down_to_scan(filter_condition);

                            if let Some(pushable_condition) = split_result.pushable_condition {
                                // Create a new scan node with the filter condition
                                if let Some(scan_node) =
                                    child_node.plan_node.as_any().downcast_ref::<ScanVertices>()
                                {
                                    let mut new_scan_node = scan_node.clone();

                                    // Combine existing filter with the new one if needed
                                    let new_filter = if let Some(existing_filter) =
                                        &new_scan_node.vertex_filter
                                    {
                                        combine_conditions(&pushable_condition, existing_filter)
                                    } else {
                                        pushable_condition
                                    };

                                    new_scan_node.vertex_filter = Some(new_filter);

                                    // Create a new OptGroupNode with the modified scan node
                                    let mut new_scan_opt_node = child_node.clone();
                                    new_scan_opt_node.plan_node = Box::new(new_scan_node);

                                    // If there's a remaining condition, create a new filter node
                                    if let Some(remaining_condition) =
                                        split_result.remaining_condition
                                    {
                                        let mut new_filter_node = filter_plan_node.clone();
                                        new_filter_node.condition = remaining_condition;
                                        new_filter_node.deps =
                                            vec![new_scan_opt_node.plan_node.clone()];

                                        let mut new_filter_opt_node = node.clone();
                                        new_filter_opt_node.plan_node = Box::new(new_filter_node);
                                        new_filter_opt_node.dependencies =
                                            vec![new_scan_opt_node.id];

                                        Ok(Some(new_filter_opt_node))
                                    } else {
                                        // No remaining condition, just return the scan node
                                        new_scan_opt_node.output_var =
                                            node.plan_node.output_var().clone();
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
                            // Similar logic for IndexScan
                            let split_result = can_push_down_to_scan(filter_condition);

                            if let Some(pushable_condition) = split_result.pushable_condition {
                                // Create a new index scan node with the filter condition
                                if let Some(index_scan_node) =
                                    child_node.plan_node.as_any().downcast_ref::<IndexScan>()
                                {
                                    let mut new_index_scan_node = index_scan_node.clone();

                                    // Combine existing filter with the new one if needed
                                    let new_filter = if let Some(existing_filter) =
                                        &new_index_scan_node.filter
                                    {
                                        combine_conditions(&pushable_condition, existing_filter)
                                    } else {
                                        pushable_condition
                                    };

                                    new_index_scan_node.filter = Some(new_filter);

                                    // Create a new OptGroupNode with the modified index scan node
                                    let mut new_index_scan_opt_node = child_node.clone();
                                    new_index_scan_opt_node.plan_node =
                                        Box::new(new_index_scan_node);

                                    // If there's a remaining condition, create a new filter node
                                    if let Some(remaining_condition) =
                                        split_result.remaining_condition
                                    {
                                        let mut new_filter_node = filter_plan_node.clone();
                                        new_filter_node.condition = remaining_condition;
                                        new_filter_node.deps =
                                            vec![new_index_scan_opt_node.plan_node.clone()];

                                        let mut new_filter_opt_node = node.clone();
                                        new_filter_opt_node.plan_node = Box::new(new_filter_node);
                                        new_filter_opt_node.dependencies =
                                            vec![new_index_scan_opt_node.id];

                                        Ok(Some(new_filter_opt_node))
                                    } else {
                                        // No remaining condition, just return the index scan node
                                        new_index_scan_opt_node.output_var =
                                            node.plan_node.output_var().clone();
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
                            // For traversal operations, push the filter condition down to the storage layer
                            // This reduces the number of vertices or edges retrieved during traversal
                            let split_result = can_push_down_to_traverse(filter_condition);

                            if let Some(pushable_condition) = split_result.pushable_condition {
                                // Create a new traverse node with the filter condition
                                if let Some(traverse_node) =
                                    child_node.plan_node.as_any().downcast_ref::<Traverse>()
                                {
                                    let mut new_traverse_node = traverse_node.clone();

                                    // Combine existing filter with the new one if needed
                                    let new_filter =
                                        if let Some(existing_filter) = &new_traverse_node.filter {
                                            combine_conditions(&pushable_condition, existing_filter)
                                        } else {
                                            pushable_condition
                                        };

                                    new_traverse_node.filter = Some(new_filter);

                                    // Create a new OptGroupNode with the modified traverse node
                                    let mut new_traverse_opt_node = child_node.clone();
                                    new_traverse_opt_node.plan_node = Box::new(new_traverse_node);

                                    // If there's a remaining condition, create a new filter node
                                    if let Some(remaining_condition) =
                                        split_result.remaining_condition
                                    {
                                        let mut new_filter_node = filter_plan_node.clone();
                                        new_filter_node.condition = remaining_condition;
                                        new_filter_node.deps =
                                            vec![new_traverse_opt_node.plan_node.clone()];

                                        let mut new_filter_opt_node = node.clone();
                                        new_filter_opt_node.plan_node = Box::new(new_filter_node);
                                        new_filter_opt_node.dependencies =
                                            vec![new_traverse_opt_node.id];

                                        Ok(Some(new_filter_opt_node))
                                    } else {
                                        // No remaining condition, just return the traverse node
                                        new_traverse_opt_node.output_var =
                                            node.plan_node.output_var().clone();
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
                            // For other traversal operations, similar logic applies
                            // For now, return the original node as no transformation is made
                            Ok(Some(node.clone()))
                        }
                        _ => {
                            // For other nodes, we may still be able to transform, but for now return None
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
        // Pattern: Filter node
        Pattern::new(PlanNodeKind::Filter)
    }
}

// Rule for pushing filters down traverse operations
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
        // Check if this is a filter node followed by a traverse operation
        if node.plan_node.kind() != PlanNodeKind::Filter {
            return Ok(None);
        }

        // Match the pattern to see if we have a filter over traverse
        if let Some(matched) = self.match_pattern(ctx, node)? {
            if matched.dependencies.len() == 1 {
                let child = &matched.dependencies[0];

                if child.plan_node().kind() == PlanNodeKind::Traverse {
                    // 将过滤条件下推到遍历操作
                    if let Some(filter_plan_node) =
                        node.plan_node.as_any().downcast_ref::<FilterPlanNode>()
                    {
                        let filter_condition = &filter_plan_node.condition;
                        
                        // 分析过滤条件，确定哪些部分可以下推到遍历操作
                        let split_result = can_push_down_to_traverse(filter_condition);
                        
                        if let Some(pushable_condition) = split_result.pushable_condition {
                            // 创建一个新的遍历节点，包含下推的过滤条件
                            if let Some(traverse_node) =
                                child.plan_node.as_any().downcast_ref::<Traverse>()
                            {
                                let mut new_traverse_node = traverse_node.clone();
                                
                                // 合并现有过滤条件和新的过滤条件
                                let new_filter = if let Some(existing_filter) = &new_traverse_node.filter {
                                    combine_conditions(&pushable_condition, existing_filter)
                                } else {
                                    pushable_condition
                                };
                                
                                new_traverse_node.filter = Some(new_filter);
                                
                                // 创建一个新的OptGroupNode，包含修改后的遍历节点
                                let mut new_traverse_opt_node = child.clone();
                                new_traverse_opt_node.plan_node = Box::new(new_traverse_node);
                                
                                // 如果有剩余的过滤条件，创建一个新的过滤节点
                                if let Some(remaining_condition) = split_result.remaining_condition {
                                    let mut new_filter_node = filter_plan_node.clone();
                                    new_filter_node.condition = remaining_condition;
                                    new_filter_node.deps = vec![new_traverse_opt_node.plan_node.clone()];
                                    
                                    let mut new_filter_opt_node = node.clone();
                                    new_filter_opt_node.plan_node = Box::new(new_filter_node);
                                    new_filter_opt_node.dependencies = vec![new_traverse_opt_node.id];
                                    
                                    Ok(Some(new_filter_opt_node))
                                } else {
                                    // 没有剩余的过滤条件，直接返回遍历节点
                                    new_traverse_opt_node.output_var = node.plan_node.output_var().clone();
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
        Pattern::new(PlanNodeKind::Filter).with_dependency(Pattern::new(PlanNodeKind::Traverse))
    }
}

// Rule to push filters down expand operations
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
        // Check if this node is a filter with an expand as its child
        if node.plan_node.kind() != PlanNodeKind::Filter {
            return Ok(None);
        }

        // Match the pattern to see if we have a filter over expand
        if let Some(matched) = self.match_pattern(ctx, node)? {
            if matched.dependencies.len() == 1 {
                let child = &matched.dependencies[0];

                if child.plan_node().kind() == PlanNodeKind::Expand {
                    // 将过滤条件下推到扩展操作
                    if let Some(filter_plan_node) =
                        node.plan_node.as_any().downcast_ref::<FilterPlanNode>()
                    {
                        let filter_condition = &filter_plan_node.condition;
                        
                        // 分析过滤条件，确定哪些部分可以下推到扩展操作
                        let split_result = can_push_down_to_traverse(filter_condition);
                        
                        if let Some(pushable_condition) = split_result.pushable_condition {
                            // 创建一个新的扩展节点，包含下推的过滤条件
                            if let Some(expand_node) =
                                child.plan_node.as_any().downcast_ref::<crate::query::planner::plan::operations::traversal_ops::Expand>()
                            {
                                let mut new_expand_node = expand_node.clone();
                                
                                // 扩展节点本身没有filter字段，我们需要创建一个新的过滤节点
                                // 在实际实现中，可能需要修改扩展节点以支持过滤条件
                                // 这里我们创建一个新的过滤节点，将扩展节点作为其子节点
                                let mut new_filter_node = filter_plan_node.clone();
                                new_filter_node.condition = pushable_condition;
                                new_filter_node.deps = vec![child.plan_node.clone()];
                                
                                let mut new_filter_opt_node = node.clone();
                                new_filter_opt_node.plan_node = Box::new(new_filter_node);
                                new_filter_opt_node.dependencies = vec![child.id];
                                
                                // 如果有剩余的过滤条件，创建另一个过滤节点
                                if let Some(remaining_condition) = split_result.remaining_condition {
                                    let mut top_filter_node = filter_plan_node.clone();
                                    top_filter_node.condition = remaining_condition;
                                    top_filter_node.deps = vec![new_filter_opt_node.plan_node.clone()];
                                    
                                    let mut top_filter_opt_node = node.clone();
                                    top_filter_opt_node.plan_node = Box::new(top_filter_node);
                                    top_filter_opt_node.dependencies = vec![new_filter_opt_node.id];
                                    
                                    Ok(Some(top_filter_opt_node))
                                } else {
                                    // 没有剩余的过滤条件，直接返回新的过滤节点
                                    new_filter_opt_node.output_var = node.plan_node.output_var().clone();
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
        Pattern::new(PlanNodeKind::Filter).with_dependency(Pattern::new(PlanNodeKind::Expand))
    }
}

// Rule for combining multiple filters
#[derive(Debug)]
pub struct CombineFilterRule;

impl OptRule for CombineFilterRule {
    fn name(&self) -> &str {
        "CombineFilterRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        node: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        // Check if this node is a filter with another filter as dependency
        if node.plan_node.kind() != PlanNodeKind::Filter {
            return Ok(None);
        }

        // Match the pattern to see if we have a filter over another filter
        if let Some(matched) = self.match_pattern(ctx, node)? {
            if matched.dependencies.len() == 1 {
                let child = &matched.dependencies[0];

                if child.plan_node().kind() == PlanNodeKind::Filter {
                    // 将两个连续的过滤节点合并为一个
                    // Get the current filter condition
                    if let Some(top_filter) =
                        node.plan_node.as_any().downcast_ref::<FilterPlanNode>()
                    {
                        let top_condition = &top_filter.condition;

                        // Get the child filter condition
                        if let Some(child_filter) =
                            child.plan_node().as_any().downcast_ref::<FilterPlanNode>()
                        {
                            let child_condition = &child_filter.condition;

                            // 合并两个过滤条件，使用AND连接
                            let combined_condition = combine_conditions(top_condition, child_condition);
                            
                            // 创建一个新的过滤节点，包含合并后的条件
                            let mut combined_filter_node = top_filter.clone();
                            combined_filter_node.condition = combined_condition;
                            
                            // 设置子过滤节点的依赖作为新过滤节点的依赖
                            combined_filter_node.deps = child_filter.deps.clone();
                            
                            // 创建一个新的OptGroupNode
                            let mut combined_filter_opt_node = node.clone();
                            combined_filter_opt_node.plan_node = Box::new(combined_filter_node);
                            
                            // 设置依赖关系
                            if !child.dependencies.is_empty() {
                                combined_filter_opt_node.dependencies = child.dependencies.clone();
                            }
                            
                            Ok(Some(combined_filter_opt_node))
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
        } else {
            Ok(None)
        }
    }

    fn pattern(&self) -> Pattern {
        Pattern::new(PlanNodeKind::Filter).with_dependency(Pattern::new(PlanNodeKind::Filter))
    }
}

// Rule for eliminating redundant filters
#[derive(Debug)]
pub struct EliminateFilterRule;

impl OptRule for EliminateFilterRule {
    fn name(&self) -> &str {
        "EliminateFilterRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        node: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        // Check if this is a filter node that might be redundant
        if node.plan_node.kind() != PlanNodeKind::Filter {
            return Ok(None);
        }

        // Check if the filter is a tautology (always true) and can be eliminated
        if let Some(filter_plan_node) = node.plan_node.as_any().downcast_ref::<FilterPlanNode>() {
            let condition = &filter_plan_node.condition;

            // 检查条件是否为永真式
            if is_tautology(condition) {
                // 如果过滤条件是永真式，我们可以移除它，直接返回其子节点
                // 在实际实现中，我们需要获取过滤节点的子节点并返回它
                // 这里我们返回一个表示移除过滤节点的标记
                
                // 如果过滤节点有依赖，返回第一个依赖节点
                if !node.dependencies.is_empty() {
                    // 在实际实现中，我们需要获取依赖节点的引用
                    // 这里我们返回None表示需要进一步处理
                    // 在完整的实现中，应该返回子节点而不是过滤节点
                    Ok(None) // 表示需要进一步处理
                } else {
                    // 没有依赖节点，无法移除过滤节点
                    Ok(None)
                }
            } else {
                // 对于非平凡过滤条件，我们不消除它们
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    fn pattern(&self) -> Pattern {
        Pattern::new(PlanNodeKind::Filter)
    }
}

// A rule that tries to push down conditions to storage
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
        // Check if this is a filter node that can be pushed down to storage
        if node.plan_node.kind() != PlanNodeKind::Filter {
            return Ok(None);
        }

        // Match to see if the filter is on top of a scan operation
        if let Some(matched) = self.match_pattern(ctx, node)? {
            if matched.dependencies.len() == 1 {
                let child = &matched.dependencies[0];

                match child.plan_node().kind() {
                    PlanNodeKind::ScanVertices => {
                        // 将谓词下推到扫描操作
                        if let Some(filter_plan_node) =
                            node.plan_node.as_any().downcast_ref::<FilterPlanNode>()
                        {
                            let filter_condition = &filter_plan_node.condition;
                            
                            // 分析过滤条件，确定哪些部分可以下推到扫描操作
                            let split_result = can_push_down_to_scan(filter_condition);
                            
                            if let Some(pushable_condition) = split_result.pushable_condition {
                                // 创建一个新的扫描节点，包含下推的谓词
                                if let Some(scan_node) =
                                    child.plan_node.as_any().downcast_ref::<ScanVertices>()
                                {
                                    let mut new_scan_node = scan_node.clone();
                                    
                                    // 合并现有过滤条件和新的谓词
                                    let new_filter = if let Some(existing_filter) = &new_scan_node.vertex_filter {
                                        combine_conditions(&pushable_condition, existing_filter)
                                    } else {
                                        pushable_condition
                                    };
                                    
                                    new_scan_node.vertex_filter = Some(new_filter);
                                    
                                    // 创建一个新的OptGroupNode，包含修改后的扫描节点
                                    let mut new_scan_opt_node = child.clone();
                                    new_scan_opt_node.plan_node = Box::new(new_scan_node);
                                    
                                    // 如果有剩余的过滤条件，创建一个新的过滤节点
                                    if let Some(remaining_condition) = split_result.remaining_condition {
                                        let mut new_filter_node = filter_plan_node.clone();
                                        new_filter_node.condition = remaining_condition;
                                        new_filter_node.deps = vec![new_scan_opt_node.plan_node.clone()];
                                        
                                        let mut new_filter_opt_node = node.clone();
                                        new_filter_opt_node.plan_node = Box::new(new_filter_node);
                                        new_filter_opt_node.dependencies = vec![new_scan_opt_node.id];
                                        
                                        Ok(Some(new_filter_opt_node))
                                    } else {
                                        // 没有剩余的过滤条件，直接返回扫描节点
                                        new_scan_opt_node.output_var = node.plan_node.output_var().clone();
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
                            let filter_condition = &filter_plan_node.condition;
                            
                            // 分析过滤条件，确定哪些部分可以下推到边扫描操作
                            let split_result = can_push_down_to_scan(filter_condition);
                            
                            if let Some(pushable_condition) = split_result.pushable_condition {
                                // 创建一个新的边扫描节点，包含下推的谓词
                                if let Some(scan_edges_node) =
                                    child.plan_node.as_any().downcast_ref::<crate::query::planner::plan::operations::graph_scan_ops::ScanEdges>()
                                {
                                    let mut new_scan_edges_node = scan_edges_node.clone();
                                    
                                    // 合并现有过滤条件和新的谓词
                                    let new_filter = if let Some(existing_filter) = &new_scan_edges_node.filter {
                                        combine_conditions(&pushable_condition, existing_filter)
                                    } else {
                                        pushable_condition
                                    };
                                    
                                    new_scan_edges_node.filter = Some(new_filter);
                                    
                                    // 创建一个新的OptGroupNode，包含修改后的边扫描节点
                                    let mut new_scan_edges_opt_node = child.clone();
                                    new_scan_edges_opt_node.plan_node = Box::new(new_scan_edges_node);
                                    
                                    // 如果有剩余的过滤条件，创建一个新的过滤节点
                                    if let Some(remaining_condition) = split_result.remaining_condition {
                                        let mut new_filter_node = filter_plan_node.clone();
                                        new_filter_node.condition = remaining_condition;
                                        new_filter_node.deps = vec![new_scan_edges_opt_node.plan_node.clone()];
                                        
                                        let mut new_filter_opt_node = node.clone();
                                        new_filter_opt_node.plan_node = Box::new(new_filter_node);
                                        new_filter_opt_node.dependencies = vec![new_scan_edges_opt_node.id];
                                        
                                        Ok(Some(new_filter_opt_node))
                                    } else {
                                        // 没有剩余的过滤条件，直接返回边扫描节点
                                        new_scan_edges_opt_node.output_var = node.plan_node.output_var().clone();
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
                            let filter_condition = &filter_plan_node.condition;
                            
                            // 分析过滤条件，确定哪些部分可以下推到索引扫描操作
                            let split_result = can_push_down_to_scan(filter_condition);
                            
                            if let Some(pushable_condition) = split_result.pushable_condition {
                                // 创建一个新的索引扫描节点，包含下推的谓词
                                if let Some(index_scan_node) =
                                    child.plan_node.as_any().downcast_ref::<IndexScan>()
                                {
                                    let mut new_index_scan_node = index_scan_node.clone();
                                    
                                    // 合并现有过滤条件和新的谓词
                                    let new_filter = if let Some(existing_filter) = &new_index_scan_node.filter {
                                        combine_conditions(&pushable_condition, existing_filter)
                                    } else {
                                        pushable_condition
                                    };
                                    
                                    new_index_scan_node.filter = Some(new_filter);
                                    
                                    // 创建一个新的OptGroupNode，包含修改后的索引扫描节点
                                    let mut new_index_scan_opt_node = child.clone();
                                    new_index_scan_opt_node.plan_node = Box::new(new_index_scan_node);
                                    
                                    // 如果有剩余的过滤条件，创建一个新的过滤节点
                                    if let Some(remaining_condition) = split_result.remaining_condition {
                                        let mut new_filter_node = filter_plan_node.clone();
                                        new_filter_node.condition = remaining_condition;
                                        new_filter_node.deps = vec![new_index_scan_opt_node.plan_node.clone()];
                                        
                                        let mut new_filter_opt_node = node.clone();
                                        new_filter_opt_node.plan_node = Box::new(new_filter_node);
                                        new_filter_opt_node.dependencies = vec![new_index_scan_opt_node.id];
                                        
                                        Ok(Some(new_filter_opt_node))
                                    } else {
                                        // 没有剩余的过滤条件，直接返回索引扫描节点
                                        new_index_scan_opt_node.output_var = node.plan_node.output_var().clone();
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
        // Pattern: Filter node
        Pattern::new(PlanNodeKind::Filter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::context::QueryContext;
    use crate::query::optimizer::optimizer::{OptContext, OptGroupNode};
    use crate::query::planner::plan::Filter;
    use crate::query::planner::plan::{PlanNode, PlanNodeKind};

    fn create_test_context() -> OptContext {
        OptContext::new(QueryContext::default())
    }

    #[test]
    fn test_filter_push_down_rule() {
        let rule = FilterPushDownRule;
        let mut ctx = create_test_context();

        // Create a filter node
        let filter_node = Box::new(Filter::new(1, "col1 > 100"));
        let opt_node = OptGroupNode::new(1, filter_node);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        // The rule should match filter nodes and attempt to push down conditions
        assert!(result.is_some());
    }

    #[test]
    fn test_push_filter_down_traverse_rule() {
        let rule = PushFilterDownTraverseRule;
        let mut ctx = create_test_context();

        // Create a filter node
        let filter_node = Box::new(Filter::new(1, "col1 > 100"));
        let opt_node = OptGroupNode::new(1, filter_node);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        // The rule should match filter nodes and attempt to push down to traverse operations
        assert!(result.is_some());
    }

    #[test]
    fn test_push_filter_down_expand_rule() {
        let rule = PushFilterDownExpandRule;
        let mut ctx = create_test_context();

        // Create a filter node
        let filter_node = Box::new(Filter::new(1, "col1 > 100"));
        let opt_node = OptGroupNode::new(1, filter_node);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        // The rule should match filter nodes and attempt to push down to expand operations
        assert!(result.is_some());
    }

    #[test]
    fn test_combine_filter_rule() {
        let rule = CombineFilterRule;
        let mut ctx = create_test_context();

        // Create a filter node
        let filter_node = Box::new(Filter::new(1, "col1 > 100"));
        let opt_node = OptGroupNode::new(1, filter_node);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        // The rule should match filter nodes and attempt to combine sequential filters
        assert!(result.is_some());
    }

    #[test]
    fn test_eliminate_filter_rule() {
        let rule = EliminateFilterRule;
        let mut ctx = create_test_context();

        // Create a filter node with a tautology condition
        let filter_node = Box::new(Filter::new(1, "1 = 1"));
        let opt_node = OptGroupNode::new(1, filter_node);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        // The rule should recognize tautology filters and attempt to eliminate them
        assert!(result.is_some());
    }

    #[test]
    fn test_predicate_push_down_rule() {
        let rule = PredicatePushDownRule;
        let mut ctx = create_test_context();

        // Create a filter node
        let filter_node = Box::new(Filter::new(1, "col1 > 100"));
        let opt_node = OptGroupNode::new(1, filter_node);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        // The rule should match filter nodes and attempt to push down predicates to storage
        assert!(result.is_some());
    }

    #[test]
    fn test_can_push_down_to_scan() {
        // Test the helper function for analyzing filter conditions
        let result = can_push_down_to_scan("age > 18");
        // Should return a result with pushable condition
        assert!(result.pushable_condition.is_some());
    }

    #[test]
    fn test_can_push_down_to_traverse() {
        // Test the helper function for analyzing filter conditions
        let result = can_push_down_to_traverse("age > 18");
        // Should return a result with pushable condition
        assert!(result.pushable_condition.is_some());
    }

    #[test]
    fn test_combine_conditions() {
        // Test the helper function for combining conditions
        let result = combine_conditions("age > 18", "name = 'test'");
        assert_eq!(result, "(age > 18) AND (name = 'test')");
    }

    #[test]
    fn test_is_tautology() {
        // Test the helper function for checking tautologies
        assert!(is_tautology("1 = 1"));
        assert!(is_tautology("true"));
        assert!(!is_tautology("age > 18"));
    }
}
