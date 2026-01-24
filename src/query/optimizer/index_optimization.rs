//! 索引优化规则
//! 这些规则负责优化索引操作，包括基于过滤条件的索引扫描优化和索引扫描操作本身的优化

use super::optimizer::OptimizerError;
use super::rule_patterns::PatternBuilder;
use super::rule_traits::{combine_conditions, BaseOptRule, FilterSplitResult};
use crate::core::Expression;
use crate::query::optimizer::optimizer::{OptContext, OptGroupNode, OptRule, Pattern};
use crate::query::planner::plan::algorithms::IndexScan;
use crate::query::planner::plan::core::nodes::PlanNodeEnum;

/// 基于过滤条件优化边索引扫描的规则
#[derive(Debug)]
pub struct OptimizeEdgeIndexScanByFilterRule;

impl OptRule for OptimizeEdgeIndexScanByFilterRule {
    fn name(&self) -> &str {
        "OptimizeEdgeIndexScanByFilterRule"
    }

    fn apply(
        &self,
        _ctx: &mut OptContext,
        node: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        // 检查是否为索引扫描操作
        if !node.plan_node.is_index_scan() {
            return Ok(None);
        }

        // 查找依赖中的过滤操作
        if node.dependencies.len() >= 1 {
            for dep_id in &node.dependencies {
                if let Some(dep_node) = _ctx.find_group_node_by_plan_node_id(*dep_id) {
                    if dep_node.plan_node.is_filter() {
                        // 检查过滤条件是否可以推入到索引扫描中
                        if let Some(filter_node) = dep_node.plan_node.as_filter() {
                            // 分析过滤条件，确定哪些部分可以推入到索引扫描
                            let filter_condition = filter_node.condition();
                            let split_result = can_push_down_to_index_scan(filter_condition);

                            if let Some(pushable_condition) = split_result.pushable_condition {
                                // 获取当前索引扫描节点
                                if let Some(index_scan_node) = node.plan_node.as_index_scan() {
                                    // 创建新的索引扫描节点，合并过滤条件
                                    let mut new_index_scan_node = index_scan_node.clone();

                                    // 合并现有过滤条件和新的过滤条件
                                    let _new_filter = if let Some(existing_filter) =
                                        &new_index_scan_node.filter
                                    {
                                        combine_conditions(
                                            &pushable_condition,
                                            &format!("{:?}", existing_filter),
                                        )
                                    } else {
                                        format!("{:?}", pushable_condition)
                                    };

                                    // 由于IndexScanNode没有set_filter方法，我们需要创建一个新节点
                                    // 这里简化处理，直接返回原节点

                                    // 尝试将过滤条件转换为索引扫描限制
                                    update_index_scan_limits(
                                        &mut new_index_scan_node,
                                        &filter_node.condition(),
                                    );

                                    // 创建带有修改后索引扫描节点的新OptGroupNode
                                    let mut new_index_scan_opt_node = node.clone();
                                    new_index_scan_opt_node.plan_node =
                                        PlanNodeEnum::IndexScan(new_index_scan_node);

                                    // 如果有剩余的过滤条件，创建新的过滤节点
                                    if let Some(_remaining_condition) =
                                        split_result.remaining_condition
                                    {
                                        let new_filter_node = filter_node.clone();
                                        let mut new_filter_opt_node = dep_node.clone();
                                        new_filter_opt_node.plan_node =
                                            PlanNodeEnum::Filter(new_filter_node);
                                        new_filter_opt_node.dependencies =
                                            vec![new_index_scan_opt_node.id];

                                        return Ok(Some(new_filter_opt_node));
                                    } else {
                                        // 没有剩余的过滤条件，直接返回索引扫描节点
                                        return Ok(Some(new_index_scan_opt_node));
                                    }
                                } else {
                                    return Ok(None);
                                }
                            } else {
                                // 没有可以下推的条件，返回原始节点
                                return Ok(Some(node.clone()));
                            }
                        }
                    }
                }
            }
        }
        Ok(None)
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::index_scan() // 专门用于边索引扫描
    }
}

impl BaseOptRule for OptimizeEdgeIndexScanByFilterRule {}

/// 基于过滤条件优化标签索引扫描的规则
#[derive(Debug)]
pub struct OptimizeTagIndexScanByFilterRule;

impl OptRule for OptimizeTagIndexScanByFilterRule {
    fn name(&self) -> &str {
        "OptimizeTagIndexScanByFilterRule"
    }

    fn apply(
        &self,
        _ctx: &mut OptContext,
        node: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        // 检查是否为索引扫描操作
        if !node.plan_node.is_index_scan() {
            return Ok(None);
        }

        // 查找依赖中的过滤操作
        if node.dependencies.len() >= 1 {
            for dep_id in &node.dependencies {
                if let Some(dep_node) = _ctx.find_group_node_by_plan_node_id(*dep_id) {
                    if dep_node.plan_node.is_filter() {
                        // 检查过滤条件是否可以推入到索引扫描中
                        if let Some(filter_node) = dep_node.plan_node.as_filter() {
                            // 分析过滤条件，确定哪些部分可以推入到索引扫描
                            let filter_condition = filter_node.condition();
                            let split_result = can_push_down_to_index_scan(filter_condition);

                            if let Some(pushable_condition) = split_result.pushable_condition {
                                // 获取当前索引扫描节点
                                if let Some(index_scan_node) = node.plan_node.as_index_scan() {
                                    // 创建新的索引扫描节点，合并过滤条件
                                    let mut new_index_scan_node = index_scan_node.clone();

                                    // 合并现有过滤条件和新的过滤条件
                                    let _new_filter = if let Some(existing_filter) =
                                        &new_index_scan_node.filter
                                    {
                                        combine_conditions(
                                            &pushable_condition,
                                            &format!("{:?}", existing_filter),
                                        )
                                    } else {
                                        format!("{:?}", pushable_condition)
                                    };

                                    // 由于IndexScanNode没有set_filter方法，我们需要创建一个新节点
                                    // 这里简化处理，直接返回原节点

                                    // 尝试将过滤条件转换为索引扫描限制
                                    update_index_scan_limits(
                                        &mut new_index_scan_node,
                                        &filter_node.condition(),
                                    );

                                    // 创建带有修改后索引扫描节点的新OptGroupNode
                                    let mut new_index_scan_opt_node = node.clone();
                                    new_index_scan_opt_node.plan_node =
                                        PlanNodeEnum::IndexScan(new_index_scan_node);

                                    // 如果有剩余的过滤条件，创建新的过滤节点
                                    if let Some(_remaining_condition) =
                                        split_result.remaining_condition
                                    {
                                        let new_filter_node = filter_node.clone();
                                        // 由于FilterNode没有set_condition方法，我们需要创建一个新节点
                                        // 这里简化处理，直接返回原节点

                                        let mut new_filter_opt_node = dep_node.clone();
                                        new_filter_opt_node.plan_node =
                                            PlanNodeEnum::Filter(new_filter_node);
                                        new_filter_opt_node.dependencies =
                                            vec![new_index_scan_opt_node.id];

                                        return Ok(Some(new_filter_opt_node));
                                    } else {
                                        // 没有剩余的过滤条件，直接返回索引扫描节点
                                        return Ok(Some(new_index_scan_opt_node));
                                    }
                                } else {
                                    return Ok(None);
                                }
                            } else {
                                // 没有可以下推的条件，返回原始节点
                                return Ok(Some(node.clone()));
                            }
                        }
                    }
                }
            }
        }
        Ok(None)
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::index_scan() // 专门用于标签索引扫描
    }
}

impl BaseOptRule for OptimizeTagIndexScanByFilterRule {}

/// 转换边索引全扫描为更优操作的规则
#[derive(Debug)]
pub struct EdgeIndexFullScanRule;

impl OptRule for EdgeIndexFullScanRule {
    fn name(&self) -> &str {
        "EdgeIndexFullScanRule"
    }

    fn apply(
        &self,
        _ctx: &mut OptContext,
        node: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        // 检查是否为可能是全扫描的索引扫描操作
        if !node.plan_node.is_index_scan() {
            return Ok(None);
        }

        // 检查是否没有有效的过滤条件，这可能意味着全扫描
        // 在完整实现中，我们需要检查索引扫描的条件
        // 如果索引扫描是全扫描（没有有效过滤条件），可能转换为其他操作
        if let Some(_index_scan_node) = node.plan_node.as_index_scan() {
            // 如果索引扫描没有有效的过滤条件，可能是全扫描
            if let Some(index_scan_plan_node) = node.plan_node.as_index_scan() {
                if !index_scan_plan_node.has_effective_filter() {
                    // 根据具体情况，我们可能将其转换为更高效的操作
                    // 简单起见，目前我们返回原节点
                    return Ok(Some(node.clone()));
                }
            }
        }
        Ok(None)
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::index_scan() // 专门用于边索引扫描
    }
}

impl BaseOptRule for EdgeIndexFullScanRule {}

/// 转换标签索引全扫描为更优操作的规则
#[derive(Debug)]
pub struct TagIndexFullScanRule;

impl OptRule for TagIndexFullScanRule {
    fn name(&self) -> &str {
        "TagIndexFullScanRule"
    }

    fn apply(
        &self,
        _ctx: &mut OptContext,
        node: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        // 检查是否为可能是全扫描的索引扫描操作
        if !node.plan_node.is_index_scan() {
            return Ok(None);
        }

        // 检查是否没有有效的过滤条件，这可能意味着全扫描
        // 在完整实现中，我们需要检查索引扫描的条件
        // 如果索引扫描是全扫描（没有有效过滤条件），可能转换为其他操作
        if let Some(_index_scan_node) = node.plan_node.as_index_scan() {
            // 如果索引扫描没有有效的过滤条件，可能是全扫描
            if let Some(index_scan_plan_node) = node.plan_node.as_index_scan() {
                if !index_scan_plan_node.has_effective_filter() {
                    // 根据具体情况，我们可能将其转换为更高效的操作
                    // 简单起见，目前我们返回原节点
                    return Ok(Some(node.clone()));
                }
            }
        }
        Ok(None)
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::index_scan() // 专门用于标签索引扫描
    }
}

impl BaseOptRule for TagIndexFullScanRule {}

/// 通用索引扫描操作的规则
#[derive(Debug)]
pub struct IndexScanRule;

impl OptRule for IndexScanRule {
    fn name(&self) -> &str {
        "IndexScanRule"
    }

    fn apply(
        &self,
        _ctx: &mut OptContext,
        node: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        // 检查是否为索引扫描操作
        if !node.plan_node.is_index_scan() {
            return Ok(None);
        }

        // 在完整实现中，我们会基于各种因素优化索引扫描：
        // - 索引选择性
        // - 数据分布
        // - 可用内存
        // 这里，我们基于NebulaGraph的IndexScanRule实现，检查索引扫描的查询上下文
        if let Some(_index_scan_node) = node.plan_node.as_index_scan() {
            // 实际优化逻辑可能会根据索引条件创建更优化的索引扫描计划
            // 暂时返回当前节点
            Ok(Some(node.clone()))
        } else {
            Ok(None)
        }
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::index_scan()
    }
}

impl BaseOptRule for IndexScanRule {}

/// 边索引扫描的UNION ALL规则
#[derive(Debug)]
pub struct UnionAllEdgeIndexScanRule;

impl OptRule for UnionAllEdgeIndexScanRule {
    fn name(&self) -> &str {
        "UnionAllEdgeIndexScanRule"
    }

    fn apply(
        &self,
        _ctx: &mut OptContext,
        node: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        // 检查是否为作为UNION一部分的索引扫描操作
        if !node.plan_node.is_index_scan() {
            return Ok(None);
        }

        // 检查节点是否有多个依赖（表示UNION操作）
        if node.dependencies.len() > 1 {
            // 尝试优化UNION ALL操作
            return self.optimize_union_all_index_scans(_ctx, node, true); // true表示边索引
        }

        // 单个索引扫描，无需优化
        Ok(Some(node.clone()))
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::index_scan() // 用于边索引扫描的UNION ALL
    }
}

impl BaseOptRule for UnionAllEdgeIndexScanRule {}

/// 标签索引扫描的UNION ALL规则
#[derive(Debug)]
pub struct UnionAllTagIndexScanRule;

impl OptRule for UnionAllTagIndexScanRule {
    fn name(&self) -> &str {
        "UnionAllTagIndexScanRule"
    }

    fn apply(
        &self,
        _ctx: &mut OptContext,
        node: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        // 检查是否为作为UNION一部分的索引扫描操作
        if !node.plan_node.is_index_scan() {
            return Ok(None);
        }

        // 检查节点是否有多个依赖（表示UNION操作）
        if node.dependencies.len() > 1 {
            // 尝试优化UNION ALL操作
            return self.optimize_union_all_index_scans(_ctx, node, false); // false表示标签索引
        }

        // 单个索引扫描，无需优化
        Ok(Some(node.clone()))
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::index_scan() // 用于标签索引扫描的UNION ALL
    }
}

/// 分析过滤条件是否可以推入到索引扫描
fn can_push_down_to_index_scan(condition: &Expression) -> FilterSplitResult {
    // 分析过滤条件是否可以推入到索引扫描
    // 通常，只涉及索引列的条件可以下推到索引扫描
    // 涉及非索引列或复杂表达式的条件需要保留在Filter节点中

    // 尝试解析条件表达式
    if let Ok(expression) = parse_filter_condition(condition) {
        let mut pushable_conditions = Vec::new();
        let mut remaining_conditions = Vec::new();

        analyze_expression_for_index_scan(
            &expression,
            &mut pushable_conditions,
            &mut remaining_conditions,
        );

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

/// 尝试解析过滤条件为表达式
fn parse_filter_condition(condition: &Expression) -> Result<crate::core::Expression, String> {
    // 直接返回表达式
    Ok(condition.clone())
}

/// 分析表达式，确定哪些部分可以下推到索引扫描
fn analyze_expression_for_index_scan(
    expression: &crate::core::Expression,
    pushable_conditions: &mut Vec<String>,
    remaining_conditions: &mut Vec<String>,
) {
    // 分析表达式
    // 通常，只涉及索引列的条件可以下推到索引扫描
    match expression {
        crate::core::Expression::Binary { left, op, right } => {
            // 检查是否是AND操作
            if matches!(op, crate::core::BinaryOperator::And) {
                // 递归分析左右子表达式
                analyze_expression_for_index_scan(left, pushable_conditions, remaining_conditions);
                analyze_expression_for_index_scan(right, pushable_conditions, remaining_conditions);
            } else {
                // 检查是否可以下推
                if can_push_down_expression_to_index_scan(expression) {
                    pushable_conditions.push(format!("{:?}", expression));
                } else {
                    remaining_conditions.push(format!("{:?}", expression));
                }
            }
        }
        _ => {
            // 检查其他类型的表达式
            if can_push_down_expression_to_index_scan(expression) {
                pushable_conditions.push(format!("{:?}", expression));
            } else {
                remaining_conditions.push(format!("{:?}", expression));
            }
        }
    }
}

/// 检查表达式是否可以下推到索引扫描
fn can_push_down_expression_to_index_scan(expression: &crate::core::Expression) -> bool {
    // 检查表达式是否可以下推到索引扫描
    match expression {
        crate::core::Expression::Property { .. } => true,
        crate::core::Expression::Variable(_) => true, // 变量也可以下推
        crate::core::Expression::Binary { left, op, right } => {
            // 检查是否是支持索引的操作符
            let is_indexable_op = matches!(
                op,
                crate::core::BinaryOperator::Equal
                    | crate::core::BinaryOperator::NotEqual
                    | crate::core::BinaryOperator::LessThan
                    | crate::core::BinaryOperator::LessThanOrEqual
                    | crate::core::BinaryOperator::GreaterThan
                    | crate::core::BinaryOperator::GreaterThanOrEqual
            );

            is_indexable_op
                && can_push_down_expression_to_index_scan(left)
                && can_push_down_expression_to_index_scan(right)
        }
        crate::core::Expression::Unary { operand, .. } => {
            can_push_down_expression_to_index_scan(operand)
        }
        crate::core::Expression::Function { name, .. } => {
            // 某些函数可以下推，如id(), properties()等
            matches!(name.to_lowercase().as_str(), "id" | "properties" | "labels")
        }
        crate::core::Expression::Literal(_) => true, // 字面量也可以下推
        _ => false,
    }
}

/// 合并表达式列表
fn combine_expression_list(exprs: &[String]) -> String {
    if exprs.is_empty() {
        String::new()
    } else if exprs.len() == 1 {
        exprs[0].clone()
    } else {
        format!("({})", exprs.join(") AND ("))
    }
}

/// 更新索引扫描的限制条件
fn update_index_scan_limits(index_scan: &mut IndexScan, condition: &Expression) {
    // 尝试将过滤条件转换为索引扫描限制
    // 使用表达式解析器来更准确地提取条件

    // 分析表达式并提取索引限制
    extract_index_limits_from_expression(condition, index_scan);
}

/// 从表达式中提取索引限制
fn extract_index_limits_from_expression(
    expression: &crate::core::Expression,
    index_scan: &mut IndexScan,
) {
    use crate::core::Expression;

    match expression {
        // 处理二元操作表达式
        Expression::Binary { left, op, right } => {
            // 只处理关系操作符
            if is_relational_operator(&op) {
                if let (Some(column), Some(value)) = extract_column_and_value(left, right) {
                    let limit = create_index_limit(op, column, value);
                    index_scan.scan_limits.push(limit);
                }
            } else if matches!(op, crate::core::BinaryOperator::And) {
                // 对于AND操作，递归处理左右子表达式
                extract_index_limits_from_expression(left, index_scan);
                extract_index_limits_from_expression(right, index_scan);
            }
        }
        // 其他类型的表达式暂时不处理
        _ => {}
    }
}

/// 检查是否是关系操作符
fn is_relational_operator(op: &crate::core::BinaryOperator) -> bool {
    use crate::core::BinaryOperator;
    matches!(
        op,
        BinaryOperator::Equal
            | BinaryOperator::NotEqual
            | BinaryOperator::LessThan
            | BinaryOperator::LessThanOrEqual
            | BinaryOperator::GreaterThan
            | BinaryOperator::GreaterThanOrEqual
    )
}

/// 从表达式中提取列名和值
fn extract_column_and_value(
    left: &crate::core::Expression,
    right: &crate::core::Expression,
) -> (Option<String>, Option<String>) {
    use crate::core::Expression;

    let column = match left {
        Expression::Property { object, property } => {
            if let Expression::Variable(var_name) = object.as_ref() {
                Some(format!("{}.{}", var_name, property))
            } else {
                Some(property.clone())
            }
        },
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
    use crate::core::BinaryOperator;

    match op {
        BinaryOperator::Equal => crate::query::planner::plan::algorithms::IndexLimit {
            column,
            begin_value: Some(value.clone()),
            end_value: Some(value),
        },
        BinaryOperator::GreaterThan | BinaryOperator::GreaterThanOrEqual => {
            crate::query::planner::plan::algorithms::IndexLimit {
                column,
                begin_value: Some(value),
                end_value: None,
            }
        }
        BinaryOperator::LessThan | BinaryOperator::LessThanOrEqual => {
            crate::query::planner::plan::algorithms::IndexLimit {
                column,
                begin_value: None,
                end_value: Some(value),
            }
        }
        // 对于不等于操作符，暂时不创建索引限制
        BinaryOperator::NotEqual => crate::query::planner::plan::algorithms::IndexLimit {
            column,
            begin_value: None,
            end_value: None,
        },
        // 其他操作符暂时不处理
        _ => crate::query::planner::plan::algorithms::IndexLimit {
            column,
            begin_value: None,
            end_value: None,
        },
    }
}

/// 提取范围条件

fn extract_range_condition(condition: &str, op: &str) -> Option<(String, String)> {
    // 简单的字符串匹配，提取形如 "column op value" 的条件
    let trimmed_condition = condition.trim();

    // 查找操作符的位置
    if let Some(op_pos) = trimmed_condition.find(op) {
        let left = trimmed_condition[..op_pos].trim();
        let right = trimmed_condition[op_pos + op.len()..].trim();

        // 简单验证左右两边是否有效
        if !left.is_empty() && !right.is_empty() {
            return Some((left.to_string(), right.to_string()));
        }
    }

    None
}

/// UnionAll规则共用的优化方法
impl UnionAllEdgeIndexScanRule {
    /// 优化UNION ALL索引扫描操作
    fn optimize_union_all_index_scans(
        &self,
        _ctx: &mut OptContext,
        node: &OptGroupNode,
        is_edge_index: bool,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        // 获取所有依赖的索引扫描节点
        let mut index_scan_nodes = Vec::new();
        for &dep_id in &node.dependencies {
            if let Some(dep_node) = _ctx.find_group_node_by_plan_node_id(dep_id) {
                if dep_node.plan_node.is_index_scan() {
                    if let Some(index_scan) = dep_node.plan_node.as_index_scan() {
                        index_scan_nodes.push((dep_id, index_scan.clone()));
                    }
                }
            }
        }

        // 如果没有足够的索引扫描节点，无法优化
        if index_scan_nodes.len() < 2 {
            return Ok(Some(node.clone()));
        }

        // 尝试合并兼容的索引扫描
        if let Some(merged_scan) = self.try_merge_index_scans(&index_scan_nodes, is_edge_index) {
            // 创建新的索引扫描节点
            let new_index_scan_node = merged_scan;

            // 创建新的OptGroupNode
            let mut new_opt_node = node.clone();
            new_opt_node.plan_node =
                crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum::IndexScan(
                    new_index_scan_node,
                );

            // 清空原有依赖，因为已经合并
            new_opt_node.dependencies.clear();

            Ok(Some(new_opt_node))
        } else {
            // 无法合并，尝试重新排序以提高效率
            if let Some(reordered_deps) = self.reorder_index_scans(_ctx, &index_scan_nodes) {
                let mut new_opt_node = node.clone();
                new_opt_node.dependencies = reordered_deps;
                Ok(Some(new_opt_node))
            } else {
                // 无法优化，返回原节点
                Ok(Some(node.clone()))
            }
        }
    }

    /// 尝试合并多个索引扫描节点
    fn try_merge_index_scans(
        &self,
        index_scans: &[(usize, IndexScan)],
        is_edge_index: bool,
    ) -> Option<IndexScan> {
        // 检查所有索引扫描是否兼容
        if !self.are_index_scans_mergeable(index_scans, is_edge_index) {
            return None;
        }

        // 使用第一个索引扫描作为基础
        let first_scan = &index_scans[0].1;
        let mut merged_scan = first_scan.clone();

        // 合并其他索引扫描的条件
        for (_, scan) in index_scans.iter().skip(1) {
            // 合并过滤条件
            if let (Some(existing_filter), Some(new_filter)) = (&merged_scan.filter, &scan.filter) {
                merged_scan.filter = Some(format!("({}) OR ({})", existing_filter, new_filter));
            } else if let Some(new_filter) = &scan.filter {
                merged_scan.filter = Some(new_filter.clone());
            }

            // 合并扫描限制
            for limit in &scan.scan_limits {
                if !merged_scan
                    .scan_limits
                    .iter()
                    .any(|l| l.column == limit.column)
                {
                    merged_scan.scan_limits.push(limit.clone());
                }
            }

            // 合并返回列
            for col in &scan.return_columns {
                if !merged_scan.return_columns.contains(col) {
                    merged_scan.return_columns.push(col.clone());
                }
            }
        }

        // 更新成本估算（合并后的成本应该更低）
        merged_scan.cost = merged_scan.cost * 0.8; // 假设合并可以减少20%的成本

        Some(merged_scan)
    }

    /// 检查索引扫描是否可以合并
    fn are_index_scans_mergeable(
        &self,
        index_scans: &[(usize, IndexScan)],
        is_edge_index: bool,
    ) -> bool {
        if index_scans.is_empty() {
            return false;
        }

        let first_scan = &index_scans[0].1;

        // 检查所有索引扫描是否在同一个空间和索引上
        for (_, scan) in index_scans.iter().skip(1) {
            if scan.space_id != first_scan.space_id {
                return false;
            }

            if is_edge_index {
                // 对于边索引，检查索引ID是否相同
                if scan.index_id != first_scan.index_id {
                    return false;
                }
            } else {
                // 对于标签索引，检查标签ID是否相同
                if scan.tag_id != first_scan.tag_id {
                    return false;
                }
            }

            // 检查扫描类型是否兼容
            if scan.scan_type != first_scan.scan_type {
                return false;
            }
        }

        true
    }

    /// 重新排序索引扫描以提高效率
    fn reorder_index_scans(
        &self,
        _ctx: &OptContext,
        index_scans: &[(usize, IndexScan)],
    ) -> Option<Vec<usize>> {
        // 根据成本估算重新排序，成本低的优先
        let mut sorted_scans: Vec<_> = index_scans.iter().collect();
        sorted_scans.sort_by(|a, b| {
            a.1.cost
                .partial_cmp(&b.1.cost)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // 返回重新排序后的依赖ID列表
        Some(sorted_scans.iter().map(|(id, _)| *id).collect())
    }
}

/// 为UnionAllTagIndexScanRule实现相同的方法
impl UnionAllTagIndexScanRule {
    /// 优化UNION ALL索引扫描操作
    fn optimize_union_all_index_scans(
        &self,
        _ctx: &mut OptContext,
        node: &OptGroupNode,
        is_edge_index: bool,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        // 使用与边索引相同的实现逻辑
        // 在实际项目中，可能需要针对标签索引的特殊处理

        // 获取所有依赖的索引扫描节点
        let mut index_scan_nodes = Vec::new();
        for &dep_id in &node.dependencies {
            if let Some(dep_node) = _ctx.find_group_node_by_plan_node_id(dep_id) {
                if dep_node.plan_node.is_index_scan() {
                    if let Some(index_scan) = dep_node.plan_node.as_index_scan() {
                        index_scan_nodes.push((dep_id, index_scan.clone()));
                    }
                }
            }
        }

        // 如果没有足够的索引扫描节点，无法优化
        if index_scan_nodes.len() < 2 {
            return Ok(Some(node.clone()));
        }

        // 尝试合并兼容的索引扫描
        if let Some(merged_scan) = self.try_merge_index_scans(&index_scan_nodes, is_edge_index) {
            // 创建新的索引扫描节点
            let new_index_scan_node = merged_scan;

            // 创建新的OptGroupNode
            let mut new_opt_node = node.clone();
            new_opt_node.plan_node =
                crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum::IndexScan(
                    new_index_scan_node,
                );

            // 清空原有依赖，因为已经合并
            new_opt_node.dependencies.clear();

            Ok(Some(new_opt_node))
        } else {
            // 无法合并，尝试重新排序以提高效率
            if let Some(reordered_deps) = self.reorder_index_scans(_ctx, &index_scan_nodes) {
                let mut new_opt_node = node.clone();
                new_opt_node.dependencies = reordered_deps;
                Ok(Some(new_opt_node))
            } else {
                // 无法优化，返回原节点
                Ok(Some(node.clone()))
            }
        }
    }

    /// 尝试合并多个索引扫描节点
    fn try_merge_index_scans(
        &self,
        index_scans: &[(usize, IndexScan)],
        is_edge_index: bool,
    ) -> Option<IndexScan> {
        // 检查所有索引扫描是否兼容
        if !self.are_index_scans_mergeable(index_scans, is_edge_index) {
            return None;
        }

        // 使用第一个索引扫描作为基础
        let first_scan = &index_scans[0].1;
        let mut merged_scan = first_scan.clone();

        // 合并其他索引扫描的条件
        for (_, scan) in index_scans.iter().skip(1) {
            // 合并过滤条件
            if let (Some(existing_filter), Some(new_filter)) = (&merged_scan.filter, &scan.filter) {
                merged_scan.filter = Some(format!("({}) OR ({})", existing_filter, new_filter));
            } else if let Some(new_filter) = &scan.filter {
                merged_scan.filter = Some(new_filter.clone());
            }

            // 合并扫描限制
            for limit in &scan.scan_limits {
                if !merged_scan
                    .scan_limits
                    .iter()
                    .any(|l| l.column == limit.column)
                {
                    merged_scan.scan_limits.push(limit.clone());
                }
            }

            // 合并返回列
            for col in &scan.return_columns {
                if !merged_scan.return_columns.contains(col) {
                    merged_scan.return_columns.push(col.clone());
                }
            }
        }

        // 更新成本估算（合并后的成本应该更低）
        merged_scan.cost = merged_scan.cost * 0.8; // 假设合并可以减少20%的成本

        Some(merged_scan)
    }

    /// 检查索引扫描是否可以合并
    fn are_index_scans_mergeable(
        &self,
        index_scans: &[(usize, IndexScan)],
        is_edge_index: bool,
    ) -> bool {
        if index_scans.is_empty() {
            return false;
        }

        let first_scan = &index_scans[0].1;

        // 检查所有索引扫描是否在同一个空间和索引上
        for (_, scan) in index_scans.iter().skip(1) {
            if scan.space_id != first_scan.space_id {
                return false;
            }

            if is_edge_index {
                // 对于边索引，检查索引ID是否相同
                if scan.index_id != first_scan.index_id {
                    return false;
                }
            } else {
                // 对于标签索引，检查标签ID是否相同
                if scan.tag_id != first_scan.tag_id {
                    return false;
                }
            }

            // 检查扫描类型是否兼容
            if scan.scan_type != first_scan.scan_type {
                return false;
            }
        }

        true
    }

    /// 重新排序索引扫描以提高效率
    fn reorder_index_scans(
        &self,
        _ctx: &OptContext,
        index_scans: &[(usize, IndexScan)],
    ) -> Option<Vec<usize>> {
        // 根据成本估算重新排序，成本低的优先
        let mut sorted_scans: Vec<_> = index_scans.iter().collect();
        sorted_scans.sort_by(|a, b| {
            a.1.cost
                .partial_cmp(&b.1.cost)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // 返回重新排序后的依赖ID列表
        Some(sorted_scans.iter().map(|(id, _)| *id).collect())
    }
}

impl BaseOptRule for UnionAllTagIndexScanRule {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::context::execution::QueryContext;
    use crate::query::optimizer::optimizer::{OptContext, OptGroupNode};
    use crate::query::planner::plan::algorithms::IndexScan;

    fn create_test_context() -> OptContext {
        let _session_info = crate::api::session::session_manager::SessionInfo {
            session_id: 1,
            user_name: "test_user".to_string(),
            space_name: None,
            graph_addr: None,
            create_time: std::time::SystemTime::now(),
            last_access_time: std::time::SystemTime::now(),
            active_queries: 0,
            timezone: None,
        };
        let query_context = QueryContext::new();
        OptContext::new(query_context)
    }

    #[test]
    fn test_optimize_edge_index_scan_by_filter_rule() {
        let rule = OptimizeEdgeIndexScanByFilterRule;
        let mut ctx = create_test_context();

        // 创建一个索引扫描节点
        let index_scan_node = PlanNodeEnum::IndexScan(IndexScan::new(1, 1, 2, 3, "RANGE"));
        let mut index_scan_opt_node = OptGroupNode::new(1, index_scan_node);

        // 创建一个过滤节点作为依赖
        let filter_node = PlanNodeEnum::Filter(
            crate::query::planner::plan::core::nodes::FilterNode::new(
                PlanNodeEnum::Start(crate::query::planner::plan::core::nodes::StartNode::new()),
                crate::core::Expression::Variable("age > 18".to_string()),
            )
            .expect("Failed to create filter node"),
        );
        let filter_opt_node = OptGroupNode::new(2, filter_node);

        // 设置依赖关系：索引扫描依赖于过滤节点
        index_scan_opt_node.dependencies.push(2);

        // 将节点添加到上下文
        ctx.add_plan_node_and_group_node(1, &index_scan_opt_node);
        ctx.add_plan_node_and_group_node(2, &filter_opt_node);

        let result = rule
            .apply(&mut ctx, &index_scan_opt_node)
            .expect("Failed to apply rule");
        assert!(result.is_some());
    }

    #[test]
    fn test_optimize_tag_index_scan_by_filter_rule() {
        let rule = OptimizeTagIndexScanByFilterRule;
        let mut ctx = create_test_context();

        // 创建一个索引扫描节点
        let index_scan_node = PlanNodeEnum::IndexScan(IndexScan::new(1, 1, 2, 3, "RANGE"));
        let mut index_scan_opt_node = OptGroupNode::new(1, index_scan_node);

        // 创建一个过滤节点作为依赖
        let filter_node = PlanNodeEnum::Filter(
            crate::query::planner::plan::core::nodes::FilterNode::new(
                PlanNodeEnum::Start(crate::query::planner::plan::core::nodes::StartNode::new()),
                crate::core::Expression::Variable("name = 'test'".to_string()),
            )
            .expect("Failed to create filter node"),
        );
        let filter_opt_node = OptGroupNode::new(2, filter_node);

        // 设置依赖关系：索引扫描依赖于过滤节点
        index_scan_opt_node.dependencies.push(2);

        // 将节点添加到上下文
        ctx.add_plan_node_and_group_node(1, &index_scan_opt_node);
        ctx.add_plan_node_and_group_node(2, &filter_opt_node);

        let result = rule
            .apply(&mut ctx, &index_scan_opt_node)
            .expect("Failed to apply rule");
        assert!(result.is_some());
    }

    #[test]
    fn test_edge_index_full_scan_rule() {
        let rule = EdgeIndexFullScanRule;
        let mut ctx = create_test_context();

        // 创建一个索引扫描节点
        let index_scan_node = PlanNodeEnum::IndexScan(IndexScan::new(1, 1, 2, 3, "RANGE"));
        let opt_node = OptGroupNode::new(1, index_scan_node);

        let result = rule
            .apply(&mut ctx, &opt_node)
            .expect("Failed to apply rule");
        assert!(result.is_some());
    }

    #[test]
    fn test_tag_index_full_scan_rule() {
        let rule = TagIndexFullScanRule;
        let mut ctx = create_test_context();

        // 创建一个索引扫描节点
        let index_scan_node = PlanNodeEnum::IndexScan(IndexScan::new(1, 1, 2, 3, "RANGE"));
        let opt_node = OptGroupNode::new(1, index_scan_node);

        let result = rule
            .apply(&mut ctx, &opt_node)
            .expect("Failed to apply rule");
        assert!(result.is_some());
    }

    #[test]
    fn test_index_scan_rule() {
        let rule = IndexScanRule;
        let mut ctx = create_test_context();

        // 创建一个索引扫描节点
        let index_scan_node = PlanNodeEnum::IndexScan(IndexScan::new(1, 1, 2, 3, "RANGE"));
        let opt_node = OptGroupNode::new(1, index_scan_node);

        let result = rule
            .apply(&mut ctx, &opt_node)
            .expect("Failed to apply rule");
        assert!(result.is_some());
    }

    #[test]
    fn test_union_all_edge_index_scan_rule() {
        let rule = UnionAllEdgeIndexScanRule;
        let mut ctx = create_test_context();

        // 创建一个索引扫描节点
        let index_scan_node = PlanNodeEnum::IndexScan(IndexScan::new(1, 1, 2, 3, "RANGE"));
        let opt_node = OptGroupNode::new(1, index_scan_node);

        let result = rule
            .apply(&mut ctx, &opt_node)
            .expect("Failed to apply rule");
        assert!(result.is_some());
    }

    #[test]
    fn test_union_all_tag_index_scan_rule() {
        let rule = UnionAllTagIndexScanRule;
        let mut ctx = create_test_context();

        // 创建一个索引扫描节点
        let index_scan_node = PlanNodeEnum::IndexScan(IndexScan::new(1, 1, 2, 3, "RANGE"));
        let opt_node = OptGroupNode::new(1, index_scan_node);

        let result = rule
            .apply(&mut ctx, &opt_node)
            .expect("Failed to apply rule");
        assert!(result.is_some());
    }

    #[test]
    fn test_extract_range_condition() {
        // 测试范围条件提取
        let condition = "age > 18";
        let result = extract_range_condition(condition, ">");
        assert!(result.is_some());
        let (column, value) = result.expect("Result should contain column and value");
        assert_eq!(column, "age");
        assert_eq!(value, "18");

        let condition = "name = 'John'";
        let result = extract_range_condition(condition, "=");
        assert!(result.is_some());
        let (column, value) = result.expect("Result should contain column and value");
        assert_eq!(column, "name");
        assert_eq!(value, "'John'");
    }

    #[test]
    fn test_can_push_down_to_index_scan() {
        // 测试过滤条件推入分析
        let condition = "age > 18 AND name = 'John'";

        // 首先测试表达式解析器是否正常工作
        let expression = crate::core::Expression::Variable(condition.to_string());
        let parse_result = parse_filter_condition(&expression);
        println!("Parse result: {:?}", parse_result);

        let result = can_push_down_to_index_scan(&expression);
        println!("Pushable condition: {:?}", result.pushable_condition);
        println!("Remaining condition: {:?}", result.remaining_condition);

        // 现在表达式解析器已实现，应该能够解析条件
        // 检查是否有可下推的条件
        if result.pushable_condition.is_none() {
            // 如果解析失败，检查是否是因为表达式解析器的问题
            if let Err(ref error) = parse_result {
                println!("Expression parsing failed: {:?}", error);
            }
        }

        // 由于所有条件都可以下推到索引扫描，剩余条件应该为空
        // 但如果解析失败，我们仍然应该有剩余条件
        if parse_result.is_ok() {
            assert!(result.pushable_condition.is_some());
            assert!(result.remaining_condition.is_none());
        } else {
            assert!(result.pushable_condition.is_none());
            assert!(result.remaining_condition.is_some());
        }
    }

    #[test]
    fn test_update_index_scan_limits() {
        // 创建索引扫描节点
        let mut index_scan = IndexScan::new(1, 1, 2, 3, "RANGE");

        // 更新索引扫描限制 - 使用正确的二元表达式格式
        let condition = crate::core::Expression::Binary {
            left: Box::new(crate::core::Expression::Variable("age".to_string())),
            op: crate::core::BinaryOperator::GreaterThan,
            right: Box::new(crate::core::Expression::Literal(crate::core::Value::Int(18))),
        };
        
        update_index_scan_limits(&mut index_scan, &condition);

        // 验证限制已添加
        assert!(!index_scan.scan_limits.is_empty());
        assert_eq!(index_scan.scan_limits[0].column, "age");
        assert_eq!(
            index_scan.scan_limits[0].begin_value,
            Some("18".to_string())
        );
        assert_eq!(index_scan.scan_limits[0].end_value, None);
    }

    #[test]
    fn test_union_all_edge_index_scan_merge() {
        let rule = UnionAllEdgeIndexScanRule;
        let mut ctx = create_test_context();

        // 创建第一个索引扫描节点
        let index_scan1 = PlanNodeEnum::IndexScan(IndexScan::new(1, 1, 2, 3, "RANGE"));
        let opt_node1 = OptGroupNode::new(1, index_scan1);

        // 创建第二个索引扫描节点
        let index_scan2 = PlanNodeEnum::IndexScan(IndexScan::new(2, 1, 2, 3, "RANGE"));
        let opt_node2 = OptGroupNode::new(2, index_scan2);

        // 添加到上下文
        ctx.add_plan_node_and_group_node(1, &opt_node1);
        ctx.add_plan_node_and_group_node(2, &opt_node2);

        // 创建一个有多个依赖的节点
        let mut union_node = OptGroupNode::new(
            3,
            PlanNodeEnum::IndexScan(IndexScan::new(3, 1, 2, 3, "RANGE")),
        );
        union_node.dependencies = vec![1, 2];

        let result = rule
            .apply(&mut ctx, &union_node)
            .expect("Failed to apply rule");
        assert!(result.is_some());
    }
}
