//! 基于过滤条件优化边索引扫描的规则
//!
//! 该规则识别 Filter -> EdgeIndexFullScan 模式，
//! 并将过滤条件转换为边索引范围扫描或前缀扫描。

use crate::query::optimizer::plan::{OptContext, OptGroupNode, OptRule, Pattern, TransformResult, OptimizerError};
use crate::query::optimizer::rule_traits::BaseOptRule;
use crate::core::Expression;
use crate::query::planner::plan::algorithms::IndexScan;
use crate::query::planner::plan::core::nodes::PlanNodeEnum;
use std::rc::Rc;
use std::cell::RefCell;

/// 基于过滤条件优化边索引扫描的规则
///
/// # 转换示例
///
/// Before:
/// ```text
///   Filter(e.likeness > 78)
///           |
///   EdgeIndexFullScan
/// ```
///
/// After:
/// ```text
///   EdgeIndexRangeScan(e.likeness > 78)
/// ```
///
/// # 适用条件
///
/// - 过滤条件为关系表达式（如 ==, >, < 等）
/// - 过滤条件为逻辑AND表达式
/// - 过滤条件的左侧为边属性，右侧为常量
#[derive(Debug)]
pub struct OptimizeEdgeIndexScanByFilterRule;

impl OptRule for OptimizeEdgeIndexScanByFilterRule {
    fn name(&self) -> &str {
        "OptimizeEdgeIndexScanByFilterRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> Result<Option<TransformResult>, OptimizerError> {
        let node_ref = group_node.borrow();
        
        if !node_ref.plan_node.is_filter() {
            return Ok(None);
        }

        if node_ref.dependencies.len() != 1 {
            return Ok(None);
        }

        let child_id = node_ref.dependencies[0];
        let child_node = match ctx.find_group_node_by_id(child_id) {
            Some(node) => node,
            None => return Ok(None),
        };

        let child_ref = child_node.borrow();
        
        if !child_ref.plan_node.is_index_scan() {
            return Ok(None);
        }

        let filter_condition = match node_ref.plan_node.as_filter() {
            Some(filter) => filter.condition().clone(),
            None => return Ok(None),
        };

        let index_scan = match child_ref.plan_node.as_index_scan() {
            Some(scan) => scan,
            None => return Ok(None),
        };

        if !index_scan.is_edge_scan() {
            return Ok(None);
        }

        if !can_optimize_edge_index_scan(&filter_condition) {
            return Ok(None);
        }

        let mut new_index_scan = index_scan.clone();
        update_index_scan_with_filter(&mut new_index_scan, &filter_condition);

        let mut new_index_scan_group_node = child_ref.clone();
        new_index_scan_group_node.plan_node = PlanNodeEnum::IndexScan(new_index_scan);
        new_index_scan_group_node.dependencies = child_ref.dependencies.clone();

        if let Some(output_var) = node_ref.plan_node.output_var() {
            new_index_scan_group_node.plan_node.set_output_var(output_var.to_string());
        }

        let mut result = TransformResult::new();
        result.add_new_group_node(Rc::new(RefCell::new(new_index_scan_group_node)));
        
        Ok(Some(result))
    }

    fn pattern(&self) -> Pattern {
        Pattern::new_with_name("Filter").with_dependency_name("IndexScan")
    }
}

impl BaseOptRule for OptimizeEdgeIndexScanByFilterRule {}

/// 检查是否可以优化边索引扫描
fn can_optimize_edge_index_scan(condition: &Expression) -> bool {
    match condition {
        Expression::Binary { left, op, right } => {
            let is_relational_op = matches!(
                op,
                crate::core::BinaryOperator::Equal
                    | crate::core::BinaryOperator::NotEqual
                    | crate::core::BinaryOperator::LessThan
                    | crate::core::BinaryOperator::LessThanOrEqual
                    | crate::core::BinaryOperator::GreaterThan
                    | crate::core::BinaryOperator::GreaterThanOrEqual
            );

            if is_relational_op {
                is_edge_property_expression(left) && is_constant_expression(right)
            } else if matches!(op, crate::core::BinaryOperator::And) {
                can_optimize_edge_index_scan(left) || can_optimize_edge_index_scan(right)
            } else {
                false
            }
        }
        _ => false,
    }
}

/// 检查是否为边属性表达式
fn is_edge_property_expression(expr: &Expression) -> bool {
    match expr {
        Expression::Property { object, .. } => {
            matches!(object.as_ref(), Expression::Variable(_))
        }
        _ => false,
    }
}

/// 检查是否为常量表达式
fn is_constant_expression(expr: &Expression) -> bool {
    matches!(expr, Expression::Literal(_))
}

/// 使用过滤条件更新索引扫描
fn update_index_scan_with_filter(index_scan: &mut IndexScan, condition: &Expression) {
    extract_index_limits_from_expression(condition, index_scan);
}

/// 从表达式中提取索引限制
fn extract_index_limits_from_expression(expression: &Expression, index_scan: &mut IndexScan) {
    match expression {
        Expression::Binary { left, op, right } => {
            if is_relational_operator(op) {
                if let (Some(column), Some(value)) = extract_column_and_value(left, right) {
                    let limit = create_index_limit(op, column, value);
                    index_scan.scan_limits.push(limit);
                }
            } else if matches!(op, crate::core::BinaryOperator::And) {
                extract_index_limits_from_expression(left, index_scan);
                extract_index_limits_from_expression(right, index_scan);
            }
        }
        _ => {}
    }
}

/// 检查是否是关系操作符
fn is_relational_operator(op: &crate::core::BinaryOperator) -> bool {
    matches!(
        op,
        crate::core::BinaryOperator::Equal
            | crate::core::BinaryOperator::NotEqual
            | crate::core::BinaryOperator::LessThan
            | crate::core::BinaryOperator::LessThanOrEqual
            | crate::core::BinaryOperator::GreaterThan
            | crate::core::BinaryOperator::GreaterThanOrEqual
    )
}

/// 从表达式中提取列名和值
fn extract_column_and_value(
    left: &Expression,
    right: &Expression,
) -> (Option<String>, Option<String>) {
    let column = match left {
        Expression::Property { object, property } => {
            if let Expression::Variable(var_name) = object.as_ref() {
                Some(format!("{}.{}", var_name, property))
            } else {
                Some(property.clone())
            }
        }
        Expression::Variable(name) => Some(name.clone()),
        _ => None,
    };

    let value = match right {
        Expression::Literal(crate::core::Value::String(s)) => Some(s.clone()),
        Expression::Literal(crate::core::Value::Int(i)) => Some(i.to_string()),
        Expression::Literal(crate::core::Value::Float(f)) => Some(f.to_string()),
        Expression::Literal(crate::core::Value::Bool(b)) => Some(b.to_string()),
        _ => None,
    };

    (column, value)
}

/// 创建索引限制
fn create_index_limit(
    op: &crate::core::BinaryOperator,
    column: String,
    value: String,
) -> crate::query::planner::plan::algorithms::IndexLimit {
    use crate::query::planner::plan::algorithms::ScanType;

    match op {
        crate::core::BinaryOperator::Equal => crate::query::planner::plan::algorithms::IndexLimit {
            column,
            begin_value: Some(value.clone()),
            end_value: Some(value),
            include_begin: true,
            include_end: true,
            scan_type: ScanType::Unique,
        },
        crate::core::BinaryOperator::GreaterThan => {
            crate::query::planner::plan::algorithms::IndexLimit {
                column,
                begin_value: Some(value),
                end_value: None,
                include_begin: false,
                include_end: false,
                scan_type: ScanType::Range,
            }
        }
        crate::core::BinaryOperator::GreaterThanOrEqual => {
            crate::query::planner::plan::algorithms::IndexLimit {
                column,
                begin_value: Some(value),
                end_value: None,
                include_begin: true,
                include_end: false,
                scan_type: ScanType::Range,
            }
        }
        crate::core::BinaryOperator::LessThan => {
            crate::query::planner::plan::algorithms::IndexLimit {
                column,
                begin_value: None,
                end_value: Some(value),
                include_begin: false,
                include_end: false,
                scan_type: ScanType::Range,
            }
        }
        crate::core::BinaryOperator::LessThanOrEqual => {
            crate::query::planner::plan::algorithms::IndexLimit {
                column,
                begin_value: None,
                end_value: Some(value),
                include_begin: false,
                include_end: true,
                scan_type: ScanType::Range,
            }
        }
        _ => crate::query::planner::plan::algorithms::IndexLimit {
            column,
            begin_value: None,
            end_value: None,
            include_begin: false,
            include_end: false,
            scan_type: ScanType::Full,
        },
    }
}
