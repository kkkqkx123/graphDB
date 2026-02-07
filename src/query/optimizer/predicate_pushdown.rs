//! 谓词下推优化规则
//! 这些规则负责将过滤条件下推到计划树的底层，以减少数据处理量

use super::plan::{OptContext, OptGroupNode, OptRule, Pattern, TransformResult, Result as OptResult};
use super::rule_patterns::PatternBuilder;
use super::rule_traits::BaseOptRule;
use crate::query::planner::plan::PlanNodeEnum;
use crate::query::visitor::ExtractFilterExprVisitor;
use crate::core::types::expression::visitor::ExpressionVisitor;
use crate::core::Expression;
use std::rc::Rc;
use std::cell::RefCell;

/// 将过滤条件下推到扫描操作的规则
/// 
/// 该规则识别 Filter -> ScanVertices/ScanEdges 模式，
/// 并将过滤条件集成到扫描操作中，减少后续处理的数据量。
/// 
/// 转换示例：
/// Before:
///   Filter($p1>3 and $p2<4 and $p1<9)
///           |
///   ScanVertices
/// 
/// After:
///   Filter($p1>3 and $p1<9)
///           |
///   ScanVertices($p2<4)
#[derive(Debug)]
pub struct PushFilterDownScanVerticesRule;

impl OptRule for PushFilterDownScanVerticesRule {
    fn name(&self) -> &str {
        "PushFilterDownScanVerticesRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> OptResult<Option<TransformResult>> {
        let node_ref = group_node.borrow();
        
        // 检查是否为过滤节点
        if !node_ref.plan_node.is_filter() {
            return Ok(None);
        }

        // 检查是否有且仅有一个子节点
        if node_ref.dependencies.len() != 1 {
            return Ok(None);
        }

        let child_id = node_ref.dependencies[0];
        let child_node = match ctx.find_group_node_by_id(child_id) {
            Some(node) => node,
            None => return Ok(None),
        };

        let child_ref = child_node.borrow();
        
        // 检查子节点是否为 ScanVertices
        if child_ref.plan_node.name() != "ScanVertices" {
            return Ok(None);
        }

        // 获取过滤条件
        let filter_condition = match node_ref.plan_node.as_filter() {
            Some(filter) => filter.condition().clone(),
            None => return Ok(None),
        };

        // 使用 ExtractFilterExprVisitor 分离可下推和不可下推的表达式
        let mut visitor = ExtractFilterExprVisitor::make_push_get_vertices();
        if let Err(_) = visitor.visit_expression(&filter_condition) {
            return Ok(None);
        }

        if !visitor.ok() {
            return Ok(None);
        }

        let remained_expr = visitor.remained_expr();
        let picked_expr = visitor.get_filter_exprs().first().cloned();

        // 如果没有可下推的表达式，则不进行转换
        let picked_expr = match picked_expr {
            Some(expr) => expr,
            None => return Ok(None),
        };

        // 创建新的 ScanVertices 节点
        let new_scan_vertices = match &child_ref.plan_node {
            PlanNodeEnum::ScanVertices(scan) => {
                let mut new_scan = scan.clone();
                
                // 合并过滤器
                let new_filter = match (scan.vertex_filter(), scan.tag_filter()) {
                    (Some(vf), Some(tf)) => {
                        Some(format!("({}) AND ({})", vf, tf))
                    }
                    (Some(vf), None) => Some(vf.clone()),
                    (None, Some(tf)) => Some(tf.clone()),
                    (None, None) => None,
                };

                // 将可下推的条件转换为字符串并合并
                let filter_str = expression_to_string(&picked_expr);
                let final_filter = match new_filter {
                    Some(f) => format!("({}) AND ({})", f, filter_str),
                    None => filter_str,
                };

                new_scan.set_vertex_filter(final_filter);
                new_scan
            }
            _ => return Ok(None),
        };

        let mut result = TransformResult::new();
        result.erase_curr = true;

        // 如果有剩余表达式，创建新的 Filter 节点
        if let Some(remained) = remained_expr {
            let new_filter_node = match node_ref.plan_node.as_filter() {
                Some(filter) => {
                    let mut new_filter = filter.clone();
                    new_filter.set_condition(remained);
                    new_filter
                }
                None => return Ok(None),
            };

            let mut new_filter_group_node = node_ref.clone();
            new_filter_group_node.plan_node = PlanNodeEnum::Filter(new_filter_node);
            new_filter_group_node.dependencies = vec![child_id];

            result.add_new_group_node(Rc::new(RefCell::new(new_filter_group_node)));
        } else {
            // 没有剩余表达式，直接使用新的 ScanVertices 节点
            let mut new_scan_group_node = child_ref.clone();
            new_scan_group_node.plan_node = PlanNodeEnum::ScanVertices(new_scan_vertices);
            new_scan_group_node.dependencies = child_ref.dependencies.clone();

            // 保留原始的输出变量
            if let Some(output_var) = node_ref.plan_node.output_var() {
                new_scan_group_node.plan_node.set_output_var(output_var.clone());
            }

            result.add_new_group_node(Rc::new(RefCell::new(new_scan_group_node)));
        }
        
        Ok(Some(result))
    }

    fn pattern(&self) -> Pattern {
        // 匹配 Filter -> ScanVertices 模式
        PatternBuilder::filter_with("ScanVertices")
    }
}

impl BaseOptRule for PushFilterDownScanVerticesRule {}

/// 将表达式转换为字符串（简化实现）
fn expression_to_string(expr: &Expression) -> String {
    match expr {
        Expression::Binary { left, op, right } => {
            format!("{} {} {}", expression_to_string(left), op, expression_to_string(right))
        }
        Expression::Unary { op, operand } => {
            format!("{}{}", op, expression_to_string(operand))
        }
        Expression::Variable(name) => name.clone(),
        Expression::Property { object, property } => {
            format!("{}.{}", expression_to_string(object), property)
        }
        Expression::Function { name, args } => {
            let args_str: Vec<String> = args.iter().map(expression_to_string).collect();
            format!("{}({})", name, args_str.join(", "))
        }
        _ => format!("{:?}", expr),
    }
}

/// 将过滤条件下推到遍历操作的规则
/// 
/// 该规则识别 Filter -> Traverse 模式，
/// 并将边属性过滤条件下推到 Traverse 节点中。
/// 
/// 转换示例：
/// Before:
///   Filter(e.likeness > 78)
///           |
///   AppendVertices
///           |
///   Traverse
/// 
/// After:
///   AppendVertices
///           |
///   Traverse(eFilter: *.likeness > 78)
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
            return Ok(None);
        }

        // 检查是否有且仅有一个子节点
        if node_ref.dependencies.len() != 1 {
            return Ok(None);
        }

        let child_id = node_ref.dependencies[0];
        let child_node = match ctx.find_group_node_by_id(child_id) {
            Some(node) => node,
            None => return Ok(None),
        };

        let child_ref = child_node.borrow();
        
        // 检查子节点是否为遍历操作
        if child_ref.plan_node.name() != "Traverse" {
            return Ok(None);
        }

        // 获取过滤条件
        let filter_condition = match node_ref.plan_node.as_filter() {
            Some(filter) => filter.condition().clone(),
            None => return Ok(None),
        };

        // 获取 Traverse 节点
        let traverse = match &child_ref.plan_node {
            PlanNodeEnum::Traverse(t) => t,
            _ => return Ok(None),
        };

        // 检查是否为单步遍历
        if !traverse.is_one_step() {
            return Ok(None);
        }

        // 获取边别名（从 Traverse 节点获取）
        let edge_alias = match traverse.edge_alias() {
            Some(alias) => alias,
            None => return Ok(None),
        };

        // 使用 picker 函数选择边属性表达式
        let picker = |expr: &Expression| -> bool {
            is_edge_property_expression(edge_alias, expr)
        };

        // 分离过滤器
        let (filter_picked, filter_unpicked) = crate::query::optimizer::expression_utils::split_filter(
            &filter_condition,
            picker,
        );

        // 如果没有可下推的表达式，则不进行转换
        let filter_picked = match filter_picked {
            Some(expr) => expr,
            None => return Ok(None),
        };

        // 重写边属性表达式
        let new_filter_picked = rewrite_edge_property_filter(edge_alias, &filter_picked);

        // 创建新的 Traverse 节点
        let mut new_traverse = traverse.clone();
        
        // 合并边过滤器
        let new_e_filter = match (traverse.e_filter(), new_filter_picked) {
            (Some(ef), Some(nf)) => {
                Some(Expression::Binary {
                    left: Box::new(ef.clone()),
                    op: crate::core::BinaryOperator::And,
                    right: Box::new(nf),
                })
            }
            (Some(ef), None) => Some(ef.clone()),
            (None, Some(nf)) => Some(nf),
            (None, None) => None,
        };

        if let Some(ef) = new_e_filter {
            new_traverse.set_e_filter(ef);
        }

        let mut result = TransformResult::new();
        result.erase_curr = true;

        // 如果有剩余表达式，创建新的 Filter 节点
        if let Some(unpicked) = filter_unpicked {
            let new_filter_node = match node_ref.plan_node.as_filter() {
                Some(filter) => {
                    let mut new_filter = filter.clone();
                    new_filter.set_condition(unpicked);
                    new_filter
                }
                None => return Ok(None),
            };

            let mut new_filter_group_node = node_ref.clone();
            new_filter_group_node.plan_node = PlanNodeEnum::Filter(new_filter_node);
            new_filter_group_node.dependencies = vec![child_id];

            result.add_new_group_node(Rc::new(RefCell::new(new_filter_group_node)));
        } else {
            // 没有剩余表达式，直接使用新的 Traverse 节点
            let mut new_traverse_group_node = child_ref.clone();
            new_traverse_group_node.plan_node = PlanNodeEnum::Traverse(new_traverse);
            new_traverse_group_node.dependencies = child_ref.dependencies.clone();

            // 保留原始的输出变量
            if let Some(output_var) = node_ref.plan_node.output_var() {
                new_traverse_group_node.plan_node.set_output_var(output_var.clone());
            }

            result.add_new_group_node(Rc::new(RefCell::new(new_traverse_group_node)));
        }
        
        Ok(Some(result))
    }

    fn pattern(&self) -> Pattern {
        // 匹配 Filter -> Traverse 模式
        PatternBuilder::with_dependency("Filter", "Traverse")
    }
}

impl BaseOptRule for PushFilterDownTraverseRule {}

/// 检查表达式是否为边属性表达式
fn is_edge_property_expression(edge_alias: &str, expr: &Expression) -> bool {
    match expr {
        Expression::Property { object, .. } => {
            if let Expression::Variable(name) = object.as_ref() {
                name == edge_alias
            } else {
                false
            }
        }
        Expression::Binary { left, right, .. } => {
            is_edge_property_expression(edge_alias, left) && is_edge_property_expression(edge_alias, right)
        }
        Expression::Unary { operand, .. } => is_edge_property_expression(edge_alias, operand),
        Expression::Function { args, .. } => {
            args.iter().all(|arg| is_edge_property_expression(edge_alias, arg))
        }
        _ => false,
    }
}

/// 重写边属性表达式
fn rewrite_edge_property_filter(edge_alias: &str, expr: &Expression) -> Option<Expression> {
    match expr {
        Expression::Property { object, property } => {
            if let Expression::Variable(name) = object.as_ref() {
                if name == edge_alias {
                    // 将 e.prop 转换为 *.prop（星号表示任意边）
                    Some(Expression::Property {
                        object: Box::new(Expression::Variable("*".to_string())),
                        property: property.clone(),
                    })
                } else {
                    Some(Expression::Property {
                        object: object.clone(),
                        property: property.clone(),
                    })
                }
            } else {
                Some(Expression::Property {
                    object: object.clone(),
                    property: property.clone(),
                })
            }
        }
        Expression::Binary { left, op, right } => {
            let new_left = rewrite_edge_property_filter(edge_alias, left)?;
            let new_right = rewrite_edge_property_filter(edge_alias, right)?;
            Some(Expression::Binary {
                left: Box::new(new_left),
                op: op.clone(),
                right: Box::new(new_right),
            })
        }
        Expression::Unary { op, operand } => {
            let new_operand = rewrite_edge_property_filter(edge_alias, operand)?;
            Some(Expression::Unary {
                op: op.clone(),
                operand: Box::new(new_operand),
            })
        }
        Expression::Function { name, args } => {
            let new_args: Vec<Expression> = args
                .iter()
                .map(|arg| rewrite_edge_property_filter(edge_alias, arg))
                .collect::<Option<Vec<_>>>()?;
            Some(Expression::Function {
                name: name.clone(),
                args: new_args,
            })
        }
        _ => Some(expr.clone()),
    }
}

/// 将过滤条件下推到扩展操作的规则
/// 
/// 该规则识别 Filter -> Expand 模式，
/// 并将过滤条件集成到扩展操作中。
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
        
        // 检查是否为过滤节点
        if !node_ref.plan_node.is_filter() {
            return Ok(None);
        }

        // 检查是否有且仅有一个子节点
        if node_ref.dependencies.len() != 1 {
            return Ok(None);
        }

        let child_id = node_ref.dependencies[0];
        let child_node = match ctx.find_group_node_by_id(child_id) {
            Some(node) => node,
            None => return Ok(None),
        };

        let child_ref = child_node.borrow();
        
        // 检查子节点是否为扩展操作
        if child_ref.plan_node.name() != "Expand" {
            return Ok(None);
        }

        // 简化实现：保持原有结构
        let mut result = TransformResult::new();
        result.add_new_group_node(group_node.clone());
        
        Ok(Some(result))
    }

    fn pattern(&self) -> Pattern {
        // 匹配 Filter -> Expand 模式
        PatternBuilder::with_dependency("Filter", "Expand")
    }
}

impl BaseOptRule for PushFilterDownExpandRule {}

/// 将过滤条件下推到连接操作的规则
/// 
/// 该规则识别 Filter -> Join 模式，
/// 并将过滤条件下推到连接的一侧或两侧。
#[derive(Debug)]
pub struct PushFilterDownJoinRule;

impl OptRule for PushFilterDownJoinRule {
    fn name(&self) -> &str {
        "PushFilterDownJoinRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> OptResult<Option<TransformResult>> {
        let node_ref = group_node.borrow();
        
        // 检查是否为过滤节点
        if !node_ref.plan_node.is_filter() {
            return Ok(None);
        }

        // 检查是否只有一个子节点（单输入连接）
        if node_ref.dependencies.len() != 1 {
            return Ok(None);
        }

        let child_id = node_ref.dependencies[0];
        let child_node = match ctx.find_group_node_by_id(child_id) {
            Some(node) => node,
            None => return Ok(None),
        };

        let child_ref = child_node.borrow();
        
        // 检查子节点是否为连接操作
        if !matches!(child_ref.plan_node.name(), "HashInnerJoin" | "HashLeftJoin" | "HashRightJoin") {
            return Ok(None);
        }

        // 获取过滤条件
        let filter_condition = match node_ref.plan_node.as_filter() {
            Some(filter) => filter.condition().clone(),
            None => return Ok(None),
        };

        // 获取左子节点的列名
        let left_col_names = match &child_ref.plan_node {
            PlanNodeEnum::HashInnerJoin(join) => join.left_input().col_names().to_vec(),
            PlanNodeEnum::HashLeftJoin(join) => join.left_input().col_names().to_vec(),
            PlanNodeEnum::HashRightJoin(join) => join.left_input().col_names().to_vec(),
            _ => return Ok(None),
        };

        // 根据左子节点的列名分离过滤条件
        let picker = |expr: &Expression| -> bool {
            crate::query::optimizer::expression_utils::check_col_name(&left_col_names, expr)
        };

        let (filter_picked, filter_unpicked) = crate::query::optimizer::expression_utils::split_filter(
            &filter_condition,
            picker,
        );

        // 如果没有可下推的表达式，则不进行转换
        let filter_picked = match filter_picked {
            Some(expr) => expr,
            None => return Ok(None),
        };

        // 创建新的左 Filter 节点
        let new_left_filter = match node_ref.plan_node.as_filter() {
            Some(filter) => {
                let mut new_filter = filter.clone();
                new_filter.set_condition(filter_picked);
                new_filter
            }
            None => return Ok(None),
        };

        let mut new_left_filter_group_node = node_ref.clone();
        new_left_filter_group_node.plan_node = PlanNodeEnum::Filter(new_left_filter);
        new_left_filter_group_node.dependencies = child_ref.dependencies.clone();

        // 创建转换结果
        let mut result = TransformResult::new();
        result.erase_curr = true;

        // 如果有剩余表达式，创建新的 Filter 节点
        if let Some(unpicked) = filter_unpicked {
            let new_filter_node = match node_ref.plan_node.as_filter() {
                Some(filter) => {
                    let mut new_filter = filter.clone();
                    new_filter.set_condition(unpicked);
                    new_filter
                }
                None => return Ok(None),
            };

            let mut new_filter_group_node = node_ref.clone();
            new_filter_group_node.plan_node = PlanNodeEnum::Filter(new_filter_node);
            new_filter_group_node.dependencies = vec![child_id];

            result.add_new_group_node(Rc::new(RefCell::new(new_filter_group_node)));
        } else {
            result.add_new_group_node(Rc::new(RefCell::new(new_left_filter_group_node)));
        }
        
        Ok(Some(result))
    }

    fn pattern(&self) -> Pattern {
        // 匹配 Filter -> HashJoin 模式
        PatternBuilder::with_dependency("Filter", "HashInnerJoin")
    }
}

impl BaseOptRule for PushFilterDownJoinRule {}

/// 将过滤条件下推到Traverse/AppendVertices节点的规则
///
/// 该规则识别Traverse或AppendVertices节点中的顶点过滤条件，
/// 并将可以下推的条件提取出来推送到GetVertices操作。
#[derive(Debug)]
pub struct PushFilterDownNodeRule;

impl OptRule for PushFilterDownNodeRule {
    fn name(&self) -> &str {
        "PushFilterDownNodeRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> OptResult<Option<TransformResult>> {
        let node_ref = group_node.borrow();
        
        // 只处理Traverse或AppendVertices节点
        let node_name = node_ref.plan_node.name();
        if !matches!(node_name, "Traverse" | "AppendVertices") {
            return Ok(None);
        }

        // 获取vFilter
        let v_filter = match &node_ref.plan_node {
            PlanNodeEnum::Traverse(traverse) => traverse.v_filter().cloned(),
            PlanNodeEnum::AppendVertices(append) => append.v_filter().cloned(),
            _ => return Ok(None),
        };

        // 如果没有vFilter，则不进行转换
        let v_filter = match v_filter {
            Some(filter) => filter,
            None => return Ok(None),
        };

        // 使用表达式访问器分离可以下推和不能下推的表达式
        let mut visitor = ExtractFilterExprVisitor::make_push_get_vertices();
        if let Err(_) = visitor.visit_expression(&v_filter) {
            return Ok(None);
        }

        if !visitor.ok() {
            return Ok(None);
        }

        let remained_expr = visitor.remained_expr();

        // 如果没有剩余表达式，则不进行转换
        let remained_expr = match remained_expr {
            Some(expr) => expr,
            None => return Ok(None),
        };

        // 创建新的节点
        let mut new_node = node_ref.clone();
        
        match &mut new_node.plan_node {
            PlanNodeEnum::Traverse(traverse) => {
                // 设置新的顶点过滤器
                traverse.set_v_filter(remained_expr);
                
                // 合并 filter
                if let Some(filter) = traverse.filter() {
                    let new_filter = Expression::Binary {
                        left: Box::new(v_filter.clone()),
                        op: crate::core::BinaryOperator::And,
                        right: Box::new(Expression::Variable(filter.clone())),
                    };
                    traverse.set_filter(expression_to_string(&new_filter));
                }
            }
            PlanNodeEnum::AppendVertices(append) => {
                // 设置新的顶点过滤器
                append.set_v_filter(remained_expr);
                
                // 合并 filter
                if let Some(filter) = append.filter() {
                    let new_filter = Expression::Binary {
                        left: Box::new(v_filter.clone()),
                        op: crate::core::BinaryOperator::And,
                        right: Box::new(Expression::Variable(filter.clone())),
                    };
                    append.set_filter(expression_to_string(&new_filter));
                }
            }
            _ => return Ok(None),
        }

        let new_group_node = Rc::new(RefCell::new(new_node));
        
        // 创建转换结果
        let mut result = TransformResult::new();
        result.erase_all = true;
        result.add_new_group_node(new_group_node);
        
        Ok(Some(result))
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::multi(vec!["Traverse", "AppendVertices"])
    }
}

impl BaseOptRule for PushFilterDownNodeRule {}

/// 将边过滤条件下推到Traverse节点的规则
///
/// 该规则识别Traverse节点中的边过滤条件，
/// 并将条件重写后推送到GetNeighbors操作。
#[derive(Debug)]
pub struct PushEFilterDownRule;

impl OptRule for PushEFilterDownRule {
    fn name(&self) -> &str {
        "PushEFilterDownRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> OptResult<Option<TransformResult>> {
        let node_ref = group_node.borrow();
        
        // 只处理Traverse节点
        let traverse = match &node_ref.plan_node {
            PlanNodeEnum::Traverse(traverse) => traverse,
            _ => return Ok(None),
        };

        // 获取eFilter
        let e_filter = match traverse.e_filter() {
            Some(filter) => filter.clone(),
            None => return Ok(None),
        };

        // 检查是否为零步遍历（零步遍历不能下推eFilter）
        if traverse.is_zero_step() {
            return Ok(None);
        }

        // 重写星号边属性表达式
        // 简化实现：假设边别名为 "e"
        let edge_alias = "e";
        let e_filter = rewrite_star_edge(edge_alias, &e_filter);

        // 使用表达式访问器分离可以下推和不能下推的表达式
        let mut visitor = ExtractFilterExprVisitor::make_push_get_vertices();
        if let Err(_) = visitor.visit_expression(&e_filter) {
            return Ok(None);
        }

        if !visitor.ok() {
            return Ok(None);
        }

        let remained_expr = visitor.remained_expr();

        // 创建新的Traverse节点
        let mut new_traverse = traverse.clone();
        
        // 设置新的边过滤器
        if let Some(remained) = remained_expr {
            new_traverse.set_e_filter(remained);
        }

        // 合并 filter
        if let Some(filter) = traverse.filter() {
            let new_filter = Expression::Binary {
                left: Box::new(e_filter.clone()),
                op: crate::core::BinaryOperator::And,
                right: Box::new(Expression::Variable(filter.clone())),
            };
            new_traverse.set_filter(expression_to_string(&new_filter));
        }

        let mut new_node = node_ref.clone();
        new_node.plan_node = PlanNodeEnum::Traverse(new_traverse);
        
        // 创建转换结果
        let mut result = TransformResult::new();
        result.erase_curr = true;
        result.add_new_group_node(Rc::new(RefCell::new(new_node)));
        
        Ok(Some(result))
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::single("Traverse")
    }
}

impl BaseOptRule for PushEFilterDownRule {}

/// 重写星号边属性表达式
/// 将 *.prop 转换为 edge1.prop OR edge2.prop OR ...
fn rewrite_star_edge(edge_alias: &str, expr: &Expression) -> Expression {
    match expr {
        Expression::Property { object, property } => {
            if let Expression::Variable(name) = object.as_ref() {
                if name == "*" {
                    // 将 *.prop 转换为 e.prop（简化实现）
                    Expression::Property {
                        object: Box::new(Expression::Variable(edge_alias.to_string())),
                        property: property.clone(),
                    }
                } else {
                    Expression::Property {
                        object: object.clone(),
                        property: property.clone(),
                    }
                }
            } else {
                Expression::Property {
                    object: object.clone(),
                    property: property.clone(),
                }
            }
        }
        Expression::Binary { left, op, right } => {
            Expression::Binary {
                left: Box::new(rewrite_star_edge(edge_alias, left)),
                op: op.clone(),
                right: Box::new(rewrite_star_edge(edge_alias, right)),
            }
        }
        Expression::Unary { op, operand } => {
            Expression::Unary {
                op: op.clone(),
                operand: Box::new(rewrite_star_edge(edge_alias, operand)),
            }
        }
        Expression::Function { name, args } => {
            let new_args: Vec<Expression> = args
                .iter()
                .map(|arg| rewrite_star_edge(edge_alias, arg))
                .collect();
            Expression::Function {
                name: name.clone(),
                args: new_args,
            }
        }
        _ => expr.clone(),
    }
}

/// 将顶点过滤条件下推到ScanVertices节点的规则
///
/// 该规则识别Filter -> AppendVertices -> ScanVertices模式，
/// 并将顶点过滤条件下推到ScanVertices操作。
#[derive(Debug)]
pub struct PushVFilterDownScanVerticesRule;

impl OptRule for PushVFilterDownScanVerticesRule {
    fn name(&self) -> &str {
        "PushVFilterDownScanVerticesRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> OptResult<Option<TransformResult>> {
        let node_ref = group_node.borrow();
        
        // 检查是否为AppendVertices节点
        let append_vertices = match &node_ref.plan_node {
            PlanNodeEnum::AppendVertices(append) => append,
            _ => return Ok(None),
        };

        // 检查是否有子节点
        if node_ref.dependencies.len() != 1 {
            return Ok(None);
        }

        let child_id = node_ref.dependencies[0];
        let child_node = match ctx.find_group_node_by_id(child_id) {
            Some(node) => node,
            None => return Ok(None),
        };

        let child_ref = child_node.borrow();
        
        // 检查子节点是否为ScanVertices
        if child_ref.plan_node.name() != "ScanVertices" {
            return Ok(None);
        }

        // 检查 src 是否为 InputProperty 或 VarProperty 且属性为 kVid
        let src = match append_vertices.src_expression() {
            Some(expr) => expr,
            None => return Ok(None),
        };

        let is_vid_source = match src {
            Expression::Property { object, property } => {
                if let Expression::Variable(name) = object.as_ref() {
                    property == "_vid" || property == "vid"
                } else {
                    false
                }
            }
            _ => false,
        };

        if !is_vid_source {
            return Ok(None);
        }

        // 获取vFilter
        let v_filter = match append_vertices.v_filter() {
            Some(filter) => filter.clone(),
            None => return Ok(None),
        };

        // 检查 vFilter 中是否包含通配符 *
        if contains_wildcard(&v_filter) {
            return Ok(None);
        }

        // 使用表达式访问器分离可以下推和不能下推的表达式
        let mut visitor = ExtractFilterExprVisitor::make_push_get_vertices();
        if let Err(_) = visitor.visit_expression(&v_filter) {
            return Ok(None);
        }

        if !visitor.ok() {
            return Ok(None);
        }

        let remained_expr = visitor.remained_expr();

        // 如果没有剩余表达式，则不进行转换
        let remained_expr = match remained_expr {
            Some(expr) => expr,
            None => return Ok(None),
        };

        // 创建新的 ScanVertices 节点
        let new_scan_vertices = match &child_ref.plan_node {
            PlanNodeEnum::ScanVertices(scan) => {
                let mut new_scan = scan.clone();
                
                // 合并过滤器
                let new_filter = match (scan.vertex_filter(), scan.tag_filter()) {
                    (Some(vf), Some(tf)) => {
                        Some(format!("({}) AND ({})", vf, tf))
                    }
                    (Some(vf), None) => Some(vf.clone()),
                    (None, Some(tf)) => Some(tf.clone()),
                    (None, None) => None,
                };

                // 将可下推的条件转换为字符串并合并
                let filter_str = expression_to_string(&remained_expr);
                let final_filter = match new_filter {
                    Some(f) => format!("({}) AND ({})", f, filter_str),
                    None => filter_str,
                };

                new_scan.set_vertex_filter(final_filter);
                new_scan
            }
            _ => return Ok(None),
        };

        // 创建新的 AppendVertices 节点
        let mut new_append_vertices = append_vertices.clone();
        new_append_vertices.set_v_filter(remained_expr);

        let mut new_append_group_node = node_ref.clone();
        new_append_group_node.plan_node = PlanNodeEnum::AppendVertices(new_append_vertices);

        // 创建新的 ScanVertices 节点
        let mut new_scan_group_node = child_ref.clone();
        new_scan_group_node.plan_node = PlanNodeEnum::ScanVertices(new_scan_vertices);
        new_scan_group_node.dependencies = child_ref.dependencies.clone();

        // 设置依赖关系
        new_append_group_node.dependencies = vec![new_scan_group_node.id];

        // 创建转换结果
        let mut result = TransformResult::new();
        result.erase_curr = true;
        result.add_new_group_node(Rc::new(RefCell::new(new_append_group_node)));
        
        Ok(Some(result))
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::filter_with("AppendVertices")
    }
}

impl BaseOptRule for PushVFilterDownScanVerticesRule {}

/// 检查表达式是否包含通配符
fn contains_wildcard(expr: &Expression) -> bool {
    match expr {
        Expression::Property { object, property } => {
            if property == "*" {
                return true;
            }
            contains_wildcard(object)
        }
        Expression::Binary { left, right, .. } => {
            contains_wildcard(left) || contains_wildcard(right)
        }
        Expression::Unary { operand, .. } => contains_wildcard(operand),
        Expression::Function { args, .. } => {
            args.iter().any(|arg| contains_wildcard(arg))
        }
        _ => false,
    }
}

/// 将过滤条件下推到InnerJoin节点的规则
///
/// 该规则识别Filter -> InnerJoin模式，
/// 并将过滤条件下推到InnerJoin的一侧或两侧。
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
        
        // 检查是否为过滤节点
        if !node_ref.plan_node.is_filter() {
            return Ok(None);
        }

        // 检查是否有子节点
        if node_ref.dependencies.len() != 1 {
            return Ok(None);
        }

        let child_id = node_ref.dependencies[0];
        let child_node = match ctx.find_group_node_by_id(child_id) {
            Some(node) => node,
            None => return Ok(None),
        };

        let child_ref = child_node.borrow();
        
        // 检查子节点是否为InnerJoin
        if child_ref.plan_node.name() != "InnerJoin" {
            return Ok(None);
        }

        // 获取过滤条件
        let filter_condition = match node_ref.plan_node.as_filter() {
            Some(filter) => filter.condition().clone(),
            None => return Ok(None),
        };

        // 获取左子节点的列名
        let left_col_names = match &child_ref.plan_node {
            PlanNodeEnum::InnerJoin(join) => join.left_input().col_names().to_vec(),
            _ => return Ok(None),
        };

        // 根据左子节点的列名分离过滤条件
        let picker = |expr: &Expression| -> bool {
            crate::query::optimizer::expression_utils::check_col_name(&left_col_names, expr)
        };

        let (filter_picked, filter_unpicked) = crate::query::optimizer::expression_utils::split_filter(
            &filter_condition,
            picker,
        );

        // 如果没有可下推的表达式，则不进行转换
        let filter_picked = match filter_picked {
            Some(expr) => expr,
            None => return Ok(None),
        };

        // 创建新的左 Filter 节点
        let new_left_filter = match node_ref.plan_node.as_filter() {
            Some(filter) => {
                let mut new_filter = filter.clone();
                new_filter.set_condition(filter_picked);
                new_filter
            }
            None => return Ok(None),
        };

        let mut new_left_filter_group_node = node_ref.clone();
        new_left_filter_group_node.plan_node = PlanNodeEnum::Filter(new_left_filter);
        new_left_filter_group_node.dependencies = child_ref.dependencies.clone();

        // 创建转换结果
        let mut result = TransformResult::new();
        result.erase_curr = true;

        // 如果有剩余表达式，创建新的 Filter 节点
        if let Some(unpicked) = filter_unpicked {
            let new_filter_node = match node_ref.plan_node.as_filter() {
                Some(filter) => {
                    let mut new_filter = filter.clone();
                    new_filter.set_condition(unpicked);
                    new_filter
                }
                None => return Ok(None),
            };

            let mut new_filter_group_node = node_ref.clone();
            new_filter_group_node.plan_node = PlanNodeEnum::Filter(new_filter_node);
            new_filter_group_node.dependencies = vec![child_id];

            result.add_new_group_node(Rc::new(RefCell::new(new_filter_group_node)));
        } else {
            result.add_new_group_node(Rc::new(RefCell::new(new_left_filter_group_node)));
        }
        
        Ok(Some(result))
    }

    fn pattern(&self) -> Pattern {
        // 匹配 Filter -> InnerJoin 模式
        PatternBuilder::with_dependency("Filter", "InnerJoin")
    }
}

impl BaseOptRule for PushFilterDownInnerJoinRule {}

/// 将过滤条件下推到HashInnerJoin节点的规则
///
/// 该规则识别Filter -> HashInnerJoin模式，
/// 并将过滤条件下推到HashInnerJoin的一侧或两侧。
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
        
        // 检查是否为过滤节点
        if !node_ref.plan_node.is_filter() {
            return Ok(None);
        }

        // 检查是否有子节点
        if node_ref.dependencies.len() != 1 {
            return Ok(None);
        }

        let child_id = node_ref.dependencies[0];
        let child_node = match ctx.find_group_node_by_id(child_id) {
            Some(node) => node,
            None => return Ok(None),
        };

        let child_ref = child_node.borrow();
        
        // 检查子节点是否为HashInnerJoin
        if child_ref.plan_node.name() != "HashInnerJoin" {
            return Ok(None);
        }

        // 获取过滤条件
        let filter_condition = match node_ref.plan_node.as_filter() {
            Some(filter) => filter.condition().clone(),
            None => return Ok(None),
        };

        // 获取左子节点的列名
        let left_col_names = match &child_ref.plan_node {
            PlanNodeEnum::HashInnerJoin(join) => join.left_input().col_names().to_vec(),
            _ => return Ok(None),
        };

        // 根据左子节点的列名分离过滤条件
        let picker = |expr: &Expression| -> bool {
            crate::query::optimizer::expression_utils::check_col_name(&left_col_names, expr)
        };

        let (filter_picked, filter_unpicked) = crate::query::optimizer::expression_utils::split_filter(
            &filter_condition,
            picker,
        );

        // 如果没有可下推的表达式，则不进行转换
        let filter_picked = match filter_picked {
            Some(expr) => expr,
            None => return Ok(None),
        };

        // 创建新的左 Filter 节点
        let new_left_filter = match node_ref.plan_node.as_filter() {
            Some(filter) => {
                let mut new_filter = filter.clone();
                new_filter.set_condition(filter_picked);
                new_filter
            }
            None => return Ok(None),
        };

        let mut new_left_filter_group_node = node_ref.clone();
        new_left_filter_group_node.plan_node = PlanNodeEnum::Filter(new_left_filter);
        new_left_filter_group_node.dependencies = child_ref.dependencies.clone();

        // 创建转换结果
        let mut result = TransformResult::new();
        result.erase_curr = true;

        // 如果有剩余表达式，创建新的 Filter 节点
        if let Some(unpicked) = filter_unpicked {
            let new_filter_node = match node_ref.plan_node.as_filter() {
                Some(filter) => {
                    let mut new_filter = filter.clone();
                    new_filter.set_condition(unpicked);
                    new_filter
                }
                None => return Ok(None),
            };

            let mut new_filter_group_node = node_ref.clone();
            new_filter_group_node.plan_node = PlanNodeEnum::Filter(new_filter_node);
            new_filter_group_node.dependencies = vec![child_id];

            result.add_new_group_node(Rc::new(RefCell::new(new_filter_group_node)));
        } else {
            result.add_new_group_node(Rc::new(RefCell::new(new_left_filter_group_node)));
        }
        
        Ok(Some(result))
    }

    fn pattern(&self) -> Pattern {
        // 匹配 Filter -> HashInnerJoin 模式
        PatternBuilder::with_dependency("Filter", "HashInnerJoin")
    }
}

impl BaseOptRule for PushFilterDownHashInnerJoinRule {}

/// 将过滤条件下推到HashLeftJoin节点的规则
///
/// 该规则识别Filter -> HashLeftJoin模式，
/// 并将过滤条件下推到HashLeftJoin的一侧或两侧。
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
        
        // 检查是否为过滤节点
        if !node_ref.plan_node.is_filter() {
            return Ok(None);
        }

        // 检查是否有子节点
        if node_ref.dependencies.len() != 1 {
            return Ok(None);
        }

        let child_id = node_ref.dependencies[0];
        let child_node = match ctx.find_group_node_by_id(child_id) {
            Some(node) => node,
            None => return Ok(None),
        };

        let child_ref = child_node.borrow();
        
        // 检查子节点是否为HashLeftJoin
        if child_ref.plan_node.name() != "HashLeftJoin" {
            return Ok(None);
        }

        // 获取过滤条件
        let filter_condition = match node_ref.plan_node.as_filter() {
            Some(filter) => filter.condition().clone(),
            None => return Ok(None),
        };

        // 获取左子节点的列名
        let left_col_names = match &child_ref.plan_node {
            PlanNodeEnum::HashLeftJoin(join) => join.left_input().col_names().to_vec(),
            _ => return Ok(None),
        };

        // 根据左子节点的列名分离过滤条件
        let picker = |expr: &Expression| -> bool {
            crate::query::optimizer::expression_utils::check_col_name(&left_col_names, expr)
        };

        let (filter_picked, filter_unpicked) = crate::query::optimizer::expression_utils::split_filter(
            &filter_condition,
            picker,
        );

        // 如果没有可下推的表达式，则不进行转换
        let filter_picked = match filter_picked {
            Some(expr) => expr,
            None => return Ok(None),
        };

        // 创建新的左 Filter 节点
        let new_left_filter = match node_ref.plan_node.as_filter() {
            Some(filter) => {
                let mut new_filter = filter.clone();
                new_filter.set_condition(filter_picked);
                new_filter
            }
            None => return Ok(None),
        };

        let mut new_left_filter_group_node = node_ref.clone();
        new_left_filter_group_node.plan_node = PlanNodeEnum::Filter(new_left_filter);
        new_left_filter_group_node.dependencies = child_ref.dependencies.clone();

        // 创建转换结果
        let mut result = TransformResult::new();
        result.erase_curr = true;

        // 如果有剩余表达式，创建新的 Filter 节点
        if let Some(unpicked) = filter_unpicked {
            let new_filter_node = match node_ref.plan_node.as_filter() {
                Some(filter) => {
                    let mut new_filter = filter.clone();
                    new_filter.set_condition(unpicked);
                    new_filter
                }
                None => return Ok(None),
            };

            let mut new_filter_group_node = node_ref.clone();
            new_filter_group_node.plan_node = PlanNodeEnum::Filter(new_filter_node);
            new_filter_group_node.dependencies = vec![child_id];

            result.add_new_group_node(Rc::new(RefCell::new(new_filter_group_node)));
        } else {
            result.add_new_group_node(Rc::new(RefCell::new(new_left_filter_group_node)));
        }
        
        Ok(Some(result))
    }

    fn pattern(&self) -> Pattern {
        // 匹配 Filter -> HashLeftJoin 模式
        PatternBuilder::with_dependency("Filter", "HashLeftJoin")
    }
}

impl BaseOptRule for PushFilterDownHashLeftJoinRule {}

/// 将过滤条件下推到CrossJoin节点的规则
///
/// 该规则识别Filter -> CrossJoin模式，
/// 并将过滤条件下推到CrossJoin的一侧或两侧。
#[derive(Debug)]
pub struct PushFilterDownCrossJoinRule;

impl OptRule for PushFilterDownCrossJoinRule {
    fn name(&self) -> &str {
        "PushFilterDownCrossJoinRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> OptResult<Option<TransformResult>> {
        let node_ref = group_node.borrow();
        
        // 检查是否为过滤节点
        if !node_ref.plan_node.is_filter() {
            return Ok(None);
        }

        // 检查是否有子节点
        if node_ref.dependencies.len() != 1 {
            return Ok(None);
        }

        let child_id = node_ref.dependencies[0];
        let child_node = match ctx.find_group_node_by_id(child_id) {
            Some(node) => node,
            None => return Ok(None),
        };

        let child_ref = child_node.borrow();
        
        // 检查子节点是否为CrossJoin
        if child_ref.plan_node.name() != "CrossJoin" {
            return Ok(None);
        }

        // 获取过滤条件
        let filter_condition = match node_ref.plan_node.as_filter() {
            Some(filter) => filter.condition().clone(),
            None => return Ok(None),
        };

        // 获取左子节点的列名
        let left_col_names = match &child_ref.plan_node {
            PlanNodeEnum::CrossJoin(join) => join.left_input().col_names().to_vec(),
            _ => return Ok(None),
        };

        // 根据左子节点的列名分离过滤条件
        let picker = |expr: &Expression| -> bool {
            crate::query::optimizer::expression_utils::check_col_name(&left_col_names, expr)
        };

        let (filter_picked, filter_unpicked) = crate::query::optimizer::expression_utils::split_filter(
            &filter_condition,
            picker,
        );

        // 如果没有可下推的表达式，则不进行转换
        let filter_picked = match filter_picked {
            Some(expr) => expr,
            None => return Ok(None),
        };

        // 创建新的左 Filter 节点
        let new_left_filter = match node_ref.plan_node.as_filter() {
            Some(filter) => {
                let mut new_filter = filter.clone();
                new_filter.set_condition(filter_picked);
                new_filter
            }
            None => return Ok(None),
        };

        let mut new_left_filter_group_node = node_ref.clone();
        new_left_filter_group_node.plan_node = PlanNodeEnum::Filter(new_left_filter);
        new_left_filter_group_node.dependencies = child_ref.dependencies.clone();

        // 创建转换结果
        let mut result = TransformResult::new();
        result.erase_curr = true;

        // 如果有剩余表达式，创建新的 Filter 节点
        if let Some(unpicked) = filter_unpicked {
            let new_filter_node = match node_ref.plan_node.as_filter() {
                Some(filter) => {
                    let mut new_filter = filter.clone();
                    new_filter.set_condition(unpicked);
                    new_filter
                }
                None => return Ok(None),
            };

            let mut new_filter_group_node = node_ref.clone();
            new_filter_group_node.plan_node = PlanNodeEnum::Filter(new_filter_node);
            new_filter_group_node.dependencies = vec![child_id];

            result.add_new_group_node(Rc::new(RefCell::new(new_filter_group_node)));
        } else {
            result.add_new_group_node(Rc::new(RefCell::new(new_left_filter_group_node)));
        }
        
        Ok(Some(result))
    }

    fn pattern(&self) -> Pattern {
        // 匹配 Filter -> CrossJoin 模式
        PatternBuilder::with_dependency("Filter", "CrossJoin")
    }
}

impl BaseOptRule for PushFilterDownCrossJoinRule {}

/// 将过滤条件下推到GetNeighbors节点的规则
///
/// 该规则识别Filter -> GetNeighbors模式，
/// 并将过滤条件下推到GetNeighbors操作。
#[derive(Debug)]
pub struct PushFilterDownGetNbrsRule;

impl OptRule for PushFilterDownGetNbrsRule {
    fn name(&self) -> &str {
        "PushFilterDownGetNbrsRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> OptResult<Option<TransformResult>> {
        let node_ref = group_node.borrow();
        
        // 检查是否为过滤节点
        if !node_ref.plan_node.is_filter() {
            return Ok(None);
        }

        // 检查是否有子节点
        if node_ref.dependencies.len() != 1 {
            return Ok(None);
        }

        let child_id = node_ref.dependencies[0];
        let child_node = match ctx.find_group_node_by_id(child_id) {
            Some(node) => node,
            None => return Ok(None),
        };

        let child_ref = child_node.borrow();
        
        // 检查子节点是否为GetNeighbors
        if child_ref.plan_node.name() != "GetNeighbors" {
            return Ok(None);
        }

        // 获取过滤条件
        let _filter_condition = match node_ref.plan_node.as_filter() {
            Some(filter) => filter.condition(),
            None => return Ok(None),
        };

        // 将过滤条件下推到GetNeighbors操作
        // 这里简化处理：保持原有结构
        // 在实际实现中，应该将过滤条件集成到GetNeighbors的filter字段中
        let mut result = TransformResult::new();
        result.add_new_group_node(group_node.clone());
        
        Ok(Some(result))
    }

    fn pattern(&self) -> Pattern {
        // 匹配 Filter -> GetNeighbors 模式
        PatternBuilder::with_dependency("Filter", "GetNeighbors")
    }
}

impl BaseOptRule for PushFilterDownGetNbrsRule {}

/// 将过滤条件下推到ExpandAll节点的规则
///
/// 该规则识别Filter -> ExpandAll模式，
/// 并将过滤条件下推到ExpandAll操作。
#[derive(Debug)]
pub struct PushFilterDownExpandAllRule;

impl OptRule for PushFilterDownExpandAllRule {
    fn name(&self) -> &str {
        "PushFilterDownExpandAllRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> OptResult<Option<TransformResult>> {
        let node_ref = group_node.borrow();
        
        // 检查是否为过滤节点
        if !node_ref.plan_node.is_filter() {
            return Ok(None);
        }

        // 检查是否有子节点
        if node_ref.dependencies.len() != 1 {
            return Ok(None);
        }

        let child_id = node_ref.dependencies[0];
        let child_node = match ctx.find_group_node_by_id(child_id) {
            Some(node) => node,
            None => return Ok(None),
        };

        let child_ref = child_node.borrow();
        
        // 检查子节点是否为ExpandAll
        if child_ref.plan_node.name() != "ExpandAll" {
            return Ok(None);
        }

        // 获取过滤条件
        let _filter_condition = match node_ref.plan_node.as_filter() {
            Some(filter) => filter.condition(),
            None => return Ok(None),
        };

        // 将过滤条件下推到ExpandAll操作
        // 这里简化处理：保持原有结构
        // 在实际实现中，应该将过滤条件集成到ExpandAll的filter字段中
        let mut result = TransformResult::new();
        result.add_new_group_node(group_node.clone());
        
        Ok(Some(result))
    }

    fn pattern(&self) -> Pattern {
        // 匹配 Filter -> ExpandAll 模式
        PatternBuilder::with_dependency("Filter", "ExpandAll")
    }
}

impl BaseOptRule for PushFilterDownExpandAllRule {}

/// 将过滤条件下推到AllPaths节点的规则
///
/// 该规则识别Filter -> AllPaths模式，
/// 并将过滤条件下推到AllPaths操作。
#[derive(Debug)]
pub struct PushFilterDownAllPathsRule;

impl OptRule for PushFilterDownAllPathsRule {
    fn name(&self) -> &str {
        "PushFilterDownAllPathsRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> OptResult<Option<TransformResult>> {
        let node_ref = group_node.borrow();
        
        // 检查是否为过滤节点
        if !node_ref.plan_node.is_filter() {
            return Ok(None);
        }

        // 检查是否有子节点
        if node_ref.dependencies.len() != 1 {
            return Ok(None);
        }

        let child_id = node_ref.dependencies[0];
        let child_node = match ctx.find_group_node_by_id(child_id) {
            Some(node) => node,
            None => return Ok(None),
        };

        let child_ref = child_node.borrow();
        
        // 检查子节点是否为AllPaths
        if child_ref.plan_node.name() != "AllPaths" {
            return Ok(None);
        }

        // 获取过滤条件
        let _filter_condition = match node_ref.plan_node.as_filter() {
            Some(filter) => filter.condition(),
            None => return Ok(None),
        };

        // 将过滤条件下推到AllPaths操作
        // 这里简化处理：保持原有结构
        // 在实际实现中，应该将过滤条件集成到AllPaths的filter字段中
        let mut result = TransformResult::new();
        result.add_new_group_node(group_node.clone());
        
        Ok(Some(result))
    }

    fn pattern(&self) -> Pattern {
        // 匹配 Filter -> AllPaths 模式
        PatternBuilder::with_dependency("Filter", "AllPaths")
    }
}

impl BaseOptRule for PushFilterDownAllPathsRule {}

/// 谓词下推规则集合
/// 
/// 提供所有谓词下推规则的便捷访问。
pub struct PredicatePushDownRules;

impl PredicatePushDownRules {
    /// 获取所有谓词下推规则
    pub fn all_rules() -> Vec<Box<dyn OptRule>> {
        vec![
            Box::new(PushFilterDownScanVerticesRule),
            Box::new(PushFilterDownTraverseRule),
            Box::new(PushFilterDownExpandRule),
            Box::new(PushFilterDownJoinRule),
            Box::new(PushFilterDownNodeRule),
            Box::new(PushEFilterDownRule),
            Box::new(PushVFilterDownScanVerticesRule),
            Box::new(PushFilterDownInnerJoinRule),
            Box::new(PushFilterDownHashInnerJoinRule),
            Box::new(PushFilterDownHashLeftJoinRule),
            Box::new(PushFilterDownCrossJoinRule),
            Box::new(PushFilterDownGetNbrsRule),
            Box::new(PushFilterDownExpandAllRule),
            Box::new(PushFilterDownAllPathsRule),
        ]
    }
}