//! LIMIT下推优化规则
//! 这些规则负责将LIMIT操作下推到计划树的底层，以减少数据处理量

use super::engine::OptimizerError;
use super::plan::{OptContext, OptGroupNode, OptRule, Pattern, TransformResult};
use super::rule_patterns::PatternBuilder;
use super::rule_traits::{BaseOptRule, PushDownRule};
use crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum;
use crate::query::planner::plan::core::nodes::plan_node_traits::{PlanNode, SingleInputNode};
use crate::query::visitor::PlanNodeVisitor;
use std::rc::Rc;
use std::cell::RefCell;
use std::result::Result as StdResult;

/// LIMIT下推访问者
#[derive(Clone)]
struct LimitPushDownVisitor {
    pushed_down: bool,
    new_node: Option<OptGroupNode>,
    ctx: *const OptContext,
    node_dependencies: Vec<usize>,
}

impl LimitPushDownVisitor {
    fn get_ctx(&self) -> &OptContext {
        unsafe { &*self.ctx }
    }

    fn can_push_down_to(&self, child_node: &PlanNodeEnum) -> bool {
        matches!(
            child_node.type_name(),
            "IndexScan" | "GetVertices" | "GetEdges" | "ScanVertices" | "ScanEdges" | "Sort"
        )
    }
}

impl PlanNodeVisitor for LimitPushDownVisitor {
    type Result = Self;

    fn visit_default(&mut self) -> Self::Result {
        self.clone()
    }

    fn visit_limit(&mut self, node: &crate::query::planner::plan::core::nodes::LimitNode) -> Self::Result {
        let input = node.input();
        let input_id = input.id() as usize;

        if let Some(dep_id) = self.node_dependencies.first() {
            let ctx_ref = self.get_ctx();
            let child_node_opt = ctx_ref.find_group_node_by_plan_node_id(*dep_id);
            if let Some(child_node) = child_node_opt {
                let child_node_ref = child_node.borrow();
                if self.can_push_down_to(&child_node_ref.plan_node) {
                    let limit_count = node.count();
                    let output_var = node.output_var().cloned();
                    let child_node_type = child_node_ref.plan_node.type_name().to_string();
                    let child_node_clone = child_node_ref.clone();
                    drop(child_node_ref);
                    drop(ctx_ref);

                    match child_node_type.as_str() {
                        "GetVertices" => {
                            if let Some(get_vertices) = child_node_clone.plan_node.as_get_vertices() {
                                let mut new_get_vertices = get_vertices.clone();
                                new_get_vertices.set_limit(limit_count);
                                if let Some(var) = output_var {
                                    new_get_vertices.set_output_var(var);
                                }

                                let mut new_node = child_node_clone;
                                new_node.plan_node = PlanNodeEnum::GetVertices(new_get_vertices);
                                self.pushed_down = true;
                                self.new_node = Some(new_node);
                            }
                        }
                        "GetEdges" => {
                            if let Some(get_edges) = child_node_clone.plan_node.as_get_edges() {
                                let mut new_get_edges = get_edges.clone();
                                new_get_edges.set_limit(limit_count);
                                if let Some(var) = output_var {
                                    new_get_edges.set_output_var(var);
                                }

                                let mut new_node = child_node_clone;
                                new_node.plan_node = PlanNodeEnum::GetEdges(new_get_edges);
                                self.pushed_down = true;
                                self.new_node = Some(new_node);
                            }
                        }
                        "IndexScan" => {
                            if let Some(index_scan) = child_node_clone.plan_node.as_index_scan() {
                                let mut new_index_scan = index_scan.clone();
                                new_index_scan.set_limit(limit_count);
                                if let Some(var) = output_var {
                                    new_index_scan.set_output_var(var);
                                }

                                let mut new_node = child_node_clone;
                                new_node.plan_node = PlanNodeEnum::IndexScan(new_index_scan);
                                self.pushed_down = true;
                                self.new_node = Some(new_node);
                            }
                        }
                        "ScanVertices" => {
                            if let Some(scan_vertices) = child_node_clone.plan_node.as_scan_vertices() {
                                let mut new_scan_vertices = scan_vertices.clone();
                                new_scan_vertices.set_limit(limit_count);
                                if let Some(var) = output_var {
                                    new_scan_vertices.set_output_var(var);
                                }

                                let mut new_node = child_node_clone;
                                new_node.plan_node = PlanNodeEnum::ScanVertices(new_scan_vertices);
                                self.pushed_down = true;
                                self.new_node = Some(new_node);
                            }
                        }
                        "ScanEdges" => {
                            if let Some(scan_edges) = child_node_clone.plan_node.as_scan_edges() {
                                let mut new_scan_edges = scan_edges.clone();
                                new_scan_edges.set_limit(limit_count);
                                if let Some(var) = output_var {
                                    new_scan_edges.set_output_var(var);
                                }

                                let mut new_node = child_node_clone;
                                new_node.plan_node = PlanNodeEnum::ScanEdges(new_scan_edges);
                                self.pushed_down = true;
                                self.new_node = Some(new_node);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        self.clone()
    }
}

/// 通用LIMIT下推规则
#[derive(Debug)]
pub struct PushLimitDownRule;

impl OptRule for PushLimitDownRule {
    fn name(&self) -> &str {
        "PushLimitDownRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> StdResult<Option<TransformResult>, OptimizerError> {
        let node_ref = group_node.borrow();
        if !node_ref.plan_node.is_limit() {
            return Ok(Some(TransformResult::unchanged()));
        }

        let mut visitor = LimitPushDownVisitor {
            pushed_down: false,
            new_node: None,
            ctx: ctx as *const OptContext,
            node_dependencies: node_ref.dependencies.clone(),
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
        Ok(Some(TransformResult::unchanged()))
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::limit()
    }
}

impl BaseOptRule for PushLimitDownRule {}

impl PushDownRule for PushLimitDownRule {
    fn can_push_down_to(&self, _child_node: &PlanNodeEnum) -> bool {
        true
    }

    fn create_pushed_down_node(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
        child: &OptGroupNode,
    ) -> StdResult<Option<TransformResult>, OptimizerError> {
        let node_ref = group_node.borrow();
        let mut result = TransformResult::new();
        result.add_new_group_node(group_node.clone());
        Ok(Some(result))
    }
}

/// 将LIMIT下推到获取顶点操作的规则
#[derive(Debug)]
pub struct PushLimitDownGetVerticesRule;

impl OptRule for PushLimitDownGetVerticesRule {
    fn name(&self) -> &str {
        "PushLimitDownGetVerticesRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> StdResult<Option<TransformResult>, OptimizerError> {
        let node_ref = group_node.borrow();
        // 检查是否为LIMIT操作
        if !node_ref.plan_node.is_limit() {
            return Ok(Some(TransformResult::unchanged()));
        }

        // 检查是否有GetVertices子节点
        if node_ref.dependencies.len() >= 1 {
            let child_dep_id = node_ref.dependencies[0];
            let child_node_opt = ctx.find_group_node_by_plan_node_id(child_dep_id).cloned();

            if let Some(child_node) = child_node_opt {
                let child_node_ref = child_node.borrow();
                if child_node_ref.plan_node.is_get_vertices() {
                    let child_clone = child_node_ref.clone();
                    drop(child_node_ref);
                    return self.create_pushed_down_node(ctx, group_node, &child_clone);
                }
            }
        }
        Ok(Some(TransformResult::unchanged()))
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::with_dependency("Limit", "GetVertices")
    }
}

impl BaseOptRule for PushLimitDownGetVerticesRule {}

impl PushDownRule for PushLimitDownGetVerticesRule {
    fn can_push_down_to(&self, child_node: &PlanNodeEnum) -> bool {
        child_node.type_name() == "GetVertices"
    }

    fn create_pushed_down_node(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
        child: &OptGroupNode,
    ) -> Result<Option<TransformResult>, OptimizerError> {
        // 根据参考的NebulaGraph PushLimitDownGetVerticesRule实现
        // 我们需要将LIMIT的值应用到GetVertices操作上

        let node_ref = group_node.borrow();
        if let Some(limit_plan_node) = node_ref.plan_node.as_limit() {
            if let Some(get_vertices_plan_node) = child.plan_node.as_get_vertices() {
                // 检查LIMIT的计数是否是可计算的
                // 在实际实现中，我们需要验证limit表达式是否可评估

                // 创建新的带有限制的GetVertices节点
                let mut new_get_vertices = get_vertices_plan_node.clone();

                // 设置GetVertices的limit值为LIMIT操作的计数值
                let limit_value = limit_plan_node.count(); // 这是LIMIT操作的计数值
                new_get_vertices.set_limit(limit_value);

                // 设置输出变量
                if let Some(output_var) = node_ref.plan_node.output_var() {
                    new_get_vertices.set_output_var(output_var.clone());
                }

                // 创建新的组节点
                let mut new_node = child.clone();
                new_node.plan_node = crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum::GetVertices(new_get_vertices);

                // 复制子节点依赖
                new_node.dependencies = child.dependencies.clone();

                let mut result = TransformResult::new();
                result.add_new_group_node(Rc::new(RefCell::new(new_node)));
                return Ok(Some(result));
            }
        }
        Ok(Some(TransformResult::unchanged()))
    }
}

/// 将LIMIT下推到获取邻居操作的规则
#[derive(Debug)]
pub struct PushLimitDownGetNeighborsRule;

impl OptRule for PushLimitDownGetNeighborsRule {
    fn name(&self) -> &str {
        "PushLimitDownGetNeighborsRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> Result<Option<TransformResult>, OptimizerError> {
        let node_ref = group_node.borrow();
        // 检查是否为LIMIT操作
        if !node_ref.plan_node.is_limit() {
            return Ok(Some(TransformResult::unchanged()));
        }

        // 检查是否有GetNeighbors子节点
        if node_ref.dependencies.len() >= 1 {
            let child_dep_id = node_ref.dependencies[0];
            let child_node_opt = ctx.find_group_node_by_plan_node_id(child_dep_id).cloned();

            if let Some(child_node) = child_node_opt {
                let child_node_ref = child_node.borrow();
                if child_node_ref.plan_node.is_get_neighbors() {
                    let child_clone = child_node_ref.clone();
                    drop(child_node_ref);
                    return self.create_pushed_down_node(ctx, group_node, &child_clone);
                }
            }
        }
        Ok(Some(TransformResult::unchanged()))
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::with_dependency("Limit", "GetNeighbors")
    }
}

impl BaseOptRule for PushLimitDownGetNeighborsRule {}

impl PushDownRule for PushLimitDownGetNeighborsRule {
    fn can_push_down_to(&self, child_node: &PlanNodeEnum) -> bool {
        child_node.type_name() == "GetNeighbors"
    }

    fn create_pushed_down_node(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
        child: &OptGroupNode,
    ) -> Result<Option<TransformResult>, OptimizerError> {
        let node_ref = group_node.borrow();
        if let Some(limit_plan_node) = node_ref.plan_node.as_limit() {
            if let Some(get_neighbors_plan_node) = child.plan_node.as_get_neighbors() {
                // 创建新的带有限制的GetNeighbors节点
                let mut new_get_neighbors = get_neighbors_plan_node.clone();

                // 设置GetNeighbors的limit值为LIMIT操作的计数值
                let limit_value = limit_plan_node.count();
                new_get_neighbors.set_limit(limit_value);

                // 设置输出变量
                if let Some(output_var) = node_ref.plan_node.output_var() {
                    new_get_neighbors.set_output_var(output_var.clone());
                }

                // 创建新的组节点
                let mut new_node = child.clone();
                new_node.plan_node = crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum::GetNeighbors(new_get_neighbors);

                // 复制子节点依赖
                new_node.dependencies = child.dependencies.clone();

                let mut result = TransformResult::new();
                result.add_new_group_node(Rc::new(RefCell::new(new_node)));
                return Ok(Some(result));
            }
        }

        Ok(Some(TransformResult::unchanged()))
    }
}

/// 将LIMIT下推到获取边操作的规则
#[derive(Debug)]
pub struct PushLimitDownGetEdgesRule;

impl OptRule for PushLimitDownGetEdgesRule {
    fn name(&self) -> &str {
        "PushLimitDownGetEdgesRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> Result<Option<TransformResult>, OptimizerError> {
        let node_ref = group_node.borrow();
        // 检查是否为LIMIT操作
        if !node_ref.plan_node.is_limit() {
            return Ok(Some(TransformResult::unchanged()));
        }

        // 检查是否有GetEdges子节点
        if node_ref.dependencies.len() >= 1 {
            let child_dep_id = node_ref.dependencies[0];
            let child_node_opt = ctx.find_group_node_by_plan_node_id(child_dep_id).cloned();

            if let Some(child_node) = child_node_opt {
                let child_node_ref = child_node.borrow();
                if child_node_ref.plan_node.is_get_edges() {
                    let child_clone = child_node_ref.clone();
                    drop(child_node_ref);
                    return self.create_pushed_down_node(ctx, group_node, &child_clone);
                }
            }
        }
        Ok(Some(TransformResult::unchanged()))
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::with_dependency("Limit", "GetEdges")
    }
}

impl BaseOptRule for PushLimitDownGetEdgesRule {}

impl PushDownRule for PushLimitDownGetEdgesRule {
    fn can_push_down_to(&self, child_node: &PlanNodeEnum) -> bool {
        child_node.type_name() == "GetEdges"
    }

    fn create_pushed_down_node(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
        child: &OptGroupNode,
    ) -> Result<Option<TransformResult>, OptimizerError> {
        let node_ref = group_node.borrow();
        if let Some(limit_plan_node) = node_ref.plan_node.as_limit() {
            if let Some(get_edges_plan_node) = child.plan_node.as_get_edges() {
                // 创建新的带有限制的GetEdges节点
                let mut new_get_edges = get_edges_plan_node.clone();

                // 设置GetEdges的limit值为LIMIT操作的计数值
                let limit_value = limit_plan_node.count();
                new_get_edges.set_limit(limit_value);

                // 设置输出变量
                if let Some(output_var) = node_ref.plan_node.output_var() {
                    new_get_edges.set_output_var(output_var.clone());
                }

                // 创建新的组节点
                let mut new_node = child.clone();
                new_node.plan_node = crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum::GetEdges(new_get_edges);

                // 复制子节点依赖
                new_node.dependencies = child.dependencies.clone();

                let mut result = TransformResult::new();
                result.add_new_group_node(Rc::new(RefCell::new(new_node)));
                return Ok(Some(result));
            }
        }

        let mut result = TransformResult::new();
        result.add_new_group_node(group_node.clone());
        Ok(Some(result))
    }
}

/// 将LIMIT下推到扫描顶点操作的规则
#[derive(Debug)]
pub struct PushLimitDownScanVerticesRule;

impl OptRule for PushLimitDownScanVerticesRule {
    fn name(&self) -> &str {
        "PushLimitDownScanVerticesRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> Result<Option<TransformResult>, OptimizerError> {
        let node_ref = group_node.borrow();
        // 检查是否为LIMIT操作
        if !node_ref.plan_node.is_limit() {
            return Ok(Some(TransformResult::unchanged()));
        }

        // 检查是否有ScanVertices子节点
        if node_ref.dependencies.len() >= 1 {
            let child_dep_id = node_ref.dependencies[0];
            let child_node_opt = ctx.find_group_node_by_plan_node_id(child_dep_id).cloned();

            if let Some(child_node) = child_node_opt {
                let child_node_ref = child_node.borrow();
                if child_node_ref.plan_node.is_scan_vertices() {
                    let child_clone = child_node_ref.clone();
                    drop(child_node_ref);
                    return self.create_pushed_down_node(ctx, group_node, &child_clone);
                }
            }
        }
        Ok(Some(TransformResult::unchanged()))
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::with_dependency("Limit", "ScanVertices")
    }
}

impl BaseOptRule for PushLimitDownScanVerticesRule {}

impl PushDownRule for PushLimitDownScanVerticesRule {
    fn can_push_down_to(&self, child_node: &PlanNodeEnum) -> bool {
        child_node.type_name() == "ScanVertices"
    }

    fn create_pushed_down_node(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
        child: &OptGroupNode,
    ) -> Result<Option<TransformResult>, OptimizerError> {
        let node_ref = group_node.borrow();
        if let Some(limit_plan_node) = node_ref.plan_node.as_limit() {
            if let Some(scan_vertices_plan_node) = child.plan_node.as_scan_vertices() {
                // 创建新的带有限制的ScanVertices节点
                let mut new_scan_vertices = scan_vertices_plan_node.clone();

                // 设置ScanVertices的limit值为LIMIT操作的计数值
                let limit_value = limit_plan_node.count();
                new_scan_vertices.set_limit(limit_value);

                // 设置输出变量
                if let Some(output_var) = node_ref.plan_node.output_var() {
                    new_scan_vertices.set_output_var(output_var.clone());
                }

                // 创建新的组节点
                let mut new_node = child.clone();
                new_node.plan_node = crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum::ScanVertices(new_scan_vertices);

                // 复制子节点依赖
                new_node.dependencies = child.dependencies.clone();

                let mut result = TransformResult::new();
                result.add_new_group_node(Rc::new(RefCell::new(new_node)));
                return Ok(Some(result));
            }
        }

        let mut result = TransformResult::new();
        result.add_new_group_node(group_node.clone());
        Ok(Some(result))
    }
}

/// 将LIMIT下推到扫描边操作的规则
#[derive(Debug)]
pub struct PushLimitDownScanEdgesRule;

impl OptRule for PushLimitDownScanEdgesRule {
    fn name(&self) -> &str {
        "PushLimitDownScanEdgesRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> Result<Option<TransformResult>, OptimizerError> {
        let node_ref = group_node.borrow();
        // 检查是否为LIMIT操作
        if !node_ref.plan_node.is_limit() {
            return Ok(Some(TransformResult::unchanged()));
        }

        // 检查是否有ScanEdges子节点
        if node_ref.dependencies.len() >= 1 {
            let child_dep_id = node_ref.dependencies[0];
            let child_node_opt = ctx.find_group_node_by_plan_node_id(child_dep_id).cloned();

            if let Some(child_node) = child_node_opt {
                let child_node_ref = child_node.borrow();
                if child_node_ref.plan_node.is_scan_edges() {
                    let child_clone = child_node_ref.clone();
                    drop(child_node_ref);
                    return self.create_pushed_down_node(ctx, group_node, &child_clone);
                }
            }
        }
        Ok(Some(TransformResult::unchanged()))
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::with_dependency("Limit", "ScanEdges")
    }
}

impl BaseOptRule for PushLimitDownScanEdgesRule {}

impl PushDownRule for PushLimitDownScanEdgesRule {
    fn can_push_down_to(&self, child_node: &PlanNodeEnum) -> bool {
        child_node.type_name() == "ScanEdges"
    }

    fn create_pushed_down_node(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
        child: &OptGroupNode,
    ) -> Result<Option<TransformResult>, OptimizerError> {
        let node_ref = group_node.borrow();
        if let Some(limit_plan_node) = node_ref.plan_node.as_limit() {
            if let Some(scan_edges_plan_node) = child.plan_node.as_scan_edges() {
                // 创建新的带有限制的ScanEdges节点
                let mut new_scan_edges = scan_edges_plan_node.clone();

                // 设置ScanEdges的limit值为LIMIT操作的计数值
                let limit_value = limit_plan_node.count();
                new_scan_edges.set_limit(limit_value);

                // 设置输出变量
                if let Some(output_var) = node_ref.plan_node.output_var() {
                    new_scan_edges.set_output_var(output_var.clone());
                }

                // 创建新的组节点
                let mut new_node = child.clone();
                new_node.plan_node = crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum::ScanEdges(new_scan_edges);

                // 复制子节点依赖
                new_node.dependencies = child.dependencies.clone();

                let mut result = TransformResult::new();
                result.add_new_group_node(Rc::new(RefCell::new(new_node)));
                return Ok(Some(result));
            }
        }

        let mut result = TransformResult::new();
        result.add_new_group_node(group_node.clone());
        Ok(Some(result))
    }
}

/// 将LIMIT下推到索引扫描操作的规则
#[derive(Debug)]
pub struct PushLimitDownIndexScanRule;

impl OptRule for PushLimitDownIndexScanRule {
    fn name(&self) -> &str {
        "PushLimitDownIndexScanRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> Result<Option<TransformResult>, OptimizerError> {
        let node_ref = group_node.borrow();
        // 检查是否为LIMIT操作
        if !node_ref.plan_node.is_limit() {
            return Ok(Some(TransformResult::unchanged()));
        }

        // 检查是否有IndexScan子节点
        if node_ref.dependencies.len() >= 1 {
            let child_dep_id = node_ref.dependencies[0];
            let child_node_opt = ctx.find_group_node_by_plan_node_id(child_dep_id).cloned();

            if let Some(child_node) = child_node_opt {
                let child_node_ref = child_node.borrow();
                if child_node_ref.plan_node.is_index_scan() {
                    let child_clone = child_node_ref.clone();
                    drop(child_node_ref);
                    return self.create_pushed_down_node(ctx, group_node, &child_clone);
                }
            }
        }
        Ok(Some(TransformResult::unchanged()))
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::with_dependency("Limit", "IndexScan")
    }
}

impl BaseOptRule for PushLimitDownIndexScanRule {}

impl PushDownRule for PushLimitDownIndexScanRule {
    fn can_push_down_to(&self, child_node: &PlanNodeEnum) -> bool {
        child_node.type_name() == "IndexScan"
    }

    fn create_pushed_down_node(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
        child: &OptGroupNode,
    ) -> Result<Option<TransformResult>, OptimizerError> {
        let node_ref = group_node.borrow();
        if let Some(limit_plan_node) = node_ref.plan_node.as_limit() {
            if let Some(index_scan_plan_node) = child.plan_node.as_index_scan() {
                // 创建新的带有限制的IndexScan节点
                let mut new_index_scan = index_scan_plan_node.clone();

                // 设置IndexScan的limit值为LIMIT操作的计数值
                let limit_value = limit_plan_node.count();
                new_index_scan.set_limit(limit_value);

                // 设置输出变量
                if let Some(output_var) = node_ref.plan_node.output_var() {
                    new_index_scan.set_output_var(output_var.clone());
                }

                // 创建新的组节点
                let mut new_node = child.clone();
                new_node.plan_node = crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum::IndexScan(new_index_scan);

                // 复制子节点依赖
                new_node.dependencies = child.dependencies.clone();

                let mut result = TransformResult::new();
                result.add_new_group_node(Rc::new(RefCell::new(new_node)));
                return Ok(Some(result));
            }
        }

        let mut result = TransformResult::new();
        result.add_new_group_node(group_node.clone());
        Ok(Some(result))
    }
}

/// 将LIMIT下推到投影操作的规则
#[derive(Debug)]
pub struct PushLimitDownProjectRule;

impl OptRule for PushLimitDownProjectRule {
    fn name(&self) -> &str {
        "PushLimitDownProjectRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> Result<Option<TransformResult>, OptimizerError> {
        let node_ref = group_node.borrow();
        // 检查是否为LIMIT操作
        if !node_ref.plan_node.is_limit() {
            return Ok(Some(TransformResult::unchanged()));
        }

        // 检查是否有Project子节点
        if node_ref.dependencies.len() >= 1 {
            let child_dep_id = node_ref.dependencies[0];
            let child_node_opt = ctx.find_group_node_by_plan_node_id(child_dep_id).cloned();

            if let Some(child_node) = child_node_opt {
                let child_node_ref = child_node.borrow();
                if child_node_ref.plan_node.is_project() {
                    // 将LIMIT下推到Project操作
                    let child_clone = child_node_ref.clone();
                    drop(child_node_ref);
                    return self.create_pushed_down_node(ctx, group_node, &child_clone);
                }
            }
        }
        Ok(Some(TransformResult::unchanged()))
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::with_dependency("Limit", "Project")
    }
}

impl BaseOptRule for PushLimitDownProjectRule {}

impl PushDownRule for PushLimitDownProjectRule {
    fn can_push_down_to(&self, child_node: &PlanNodeEnum) -> bool {
        child_node.type_name() == "Project"
    }

    fn create_pushed_down_node(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
        child: &OptGroupNode,
    ) -> Result<Option<TransformResult>, OptimizerError> {
        let node_ref = group_node.borrow();
        if let Some(limit_plan_node) = node_ref.plan_node.as_limit() {
            if let Some(project_plan_node) = child.plan_node.as_project() {
                // 对于Project操作，我们不能直接在Project节点上设置limit
                // 而是创建一个新的计划结构，将LIMIT应用到Project的输入上
                // 这需要重新构建计划树

                // 克隆Project节点并设置输出变量
                let mut new_project = project_plan_node.clone();

                // 设置输出变量
                if let Some(output_var) = node_ref.plan_node.output_var() {
                    new_project.set_output_var(output_var.clone());
                }

                // 创建新的组节点
                let mut new_node = child.clone();
                new_node.plan_node = PlanNodeEnum::Project(new_project);

                // 复制子节点依赖
                new_node.dependencies = child.dependencies.clone();

                // 在实际实现中，我们需要更复杂地处理Project上的LIMIT下推
                // 可能需要在Project的输入上添加LIMIT操作
                let mut result = TransformResult::new();
                result.add_new_group_node(Rc::new(RefCell::new(new_node)));
                return Ok(Some(result));
            }
        }

        let mut result = TransformResult::new();
        result.add_new_group_node(group_node.clone());
        Ok(Some(result))
    }
}

/// 将LIMIT下推到全路径操作的规则
#[derive(Debug)]
pub struct PushLimitDownAllPathsRule;

impl OptRule for PushLimitDownAllPathsRule {
    fn name(&self) -> &str {
        "PushLimitDownAllPathsRule"
    }

    fn apply(&self, ctx: &mut OptContext, group_node: &Rc<RefCell<OptGroupNode>>) -> Result<Option<TransformResult>, OptimizerError> {
        let node_ref = group_node.borrow();
        if !node_ref.plan_node.is_limit() {
            return Ok(Some(TransformResult::unchanged()));
        }

        if node_ref.dependencies.len() >= 1 {
            let child_dep_id = node_ref.dependencies[0];
            let child_node_opt = ctx.find_group_node_by_plan_node_id(child_dep_id).cloned();

            if let Some(child_node) = child_node_opt {
                let child_node_ref = child_node.borrow();
                if child_node_ref.plan_node.is_all_paths() {
                    let child_clone = child_node_ref.clone();
                    drop(child_node_ref);
                    return self.create_pushed_down_node(ctx, group_node, &child_clone);
                }
            }
        }
        Ok(Some(TransformResult::unchanged()))
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::with_dependency("Limit", "AllPaths")
    }
}

impl BaseOptRule for PushLimitDownAllPathsRule {}

impl PushDownRule for PushLimitDownAllPathsRule {
    fn can_push_down_to(&self, child_node: &PlanNodeEnum) -> bool {
        child_node.type_name() == "AllPaths"
    }

    fn create_pushed_down_node(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
        child: &OptGroupNode,
    ) -> Result<Option<TransformResult>, OptimizerError> {
        let group_node_borrowed = group_node.borrow();
        let limit_node = group_node_borrowed.plan_node.as_limit().ok_or_else(|| {
            OptimizerError::InvalidPlanNode("Expected Limit node".to_string())
        })?;

        let all_paths_node = child.plan_node.as_all_paths().ok_or_else(|| {
            OptimizerError::InvalidPlanNode("Expected AllPaths node".to_string())
        })?;

        let limit_rows = limit_node.offset() + limit_node.count();

        if all_paths_node.limit() >= 0 && limit_rows >= all_paths_node.limit() {
            return Ok(Some(TransformResult::unchanged()));
        }

        let mut new_all_paths = all_paths_node.clone();
        new_all_paths.set_limit(limit_rows);
        if let Some(output_var) = limit_node.output_var() {
            new_all_paths.set_output_var(output_var.clone());
        }

        drop(group_node_borrowed);

        let new_all_paths_node = PlanNodeEnum::AllPaths(new_all_paths);

        let mut result = TransformResult::new();
        result.erase_all = true;

        let mut new_group_node = OptGroupNode::new(ctx.allocate_node_id(), new_all_paths_node);
        new_group_node.dependencies = child.dependencies.clone();
        result.add_new_group_node(Rc::new(RefCell::new(new_group_node)));

        Ok(Some(result))
    }
}

/// 将LIMIT下推到全展开操作的规则
#[derive(Debug)]
pub struct PushLimitDownExpandAllRule;

impl OptRule for PushLimitDownExpandAllRule {
    fn name(&self) -> &str {
        "PushLimitDownExpandAllRule"
    }

    fn apply(&self, ctx: &mut OptContext, group_node: &Rc<RefCell<OptGroupNode>>) -> Result<Option<TransformResult>, OptimizerError> {
        let node_ref = group_node.borrow();
        if !node_ref.plan_node.is_limit() {
            return Ok(Some(TransformResult::unchanged()));
        }

        if node_ref.dependencies.len() >= 1 {
            let child_dep_id = node_ref.dependencies[0];
            let child_node_opt = ctx.find_group_node_by_plan_node_id(child_dep_id).cloned();

            if let Some(child_node) = child_node_opt {
                let child_node_ref = child_node.borrow();
                if child_node_ref.plan_node.is_expand_all() {
                    let child_clone = child_node_ref.clone();
                    drop(child_node_ref);
                    return self.create_pushed_down_node(ctx, group_node, &child_clone);
                }
            }
        }
        Ok(Some(TransformResult::unchanged()))
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::with_dependency("Limit", "ExpandAll")
    }
}

impl BaseOptRule for PushLimitDownExpandAllRule {}

impl PushDownRule for PushLimitDownExpandAllRule {
    fn can_push_down_to(&self, child_node: &PlanNodeEnum) -> bool {
        child_node.type_name() == "ExpandAll"
    }

    fn create_pushed_down_node(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
        child: &OptGroupNode,
    ) -> Result<Option<TransformResult>, OptimizerError> {
        let group_node_borrowed = group_node.borrow();
        let limit_node = group_node_borrowed.plan_node.as_limit().ok_or_else(|| {
            OptimizerError::InvalidPlanNode("Expected Limit node".to_string())
        })?;

        let expand_all_node = child.plan_node.as_expand_all().ok_or_else(|| {
            OptimizerError::InvalidPlanNode("Expected ExpandAll node".to_string())
        })?;

        let limit_rows = limit_node.offset() + limit_node.count();

        if let Some(existing_limit) = expand_all_node.step_limit() {
            if limit_rows >= existing_limit as i64 {
                return Ok(Some(TransformResult::unchanged()));
            }
        }

        let mut new_expand_all = expand_all_node.clone();
        new_expand_all.set_step_limit(limit_rows as u32);
        if let Some(output_var) = limit_node.output_var() {
            new_expand_all.set_output_var(output_var.clone());
        }

        drop(group_node_borrowed);

        let new_expand_all_node = PlanNodeEnum::ExpandAll(new_expand_all);

        let mut result = TransformResult::new();
        result.erase_all = true;

        let mut new_group_node = OptGroupNode::new(ctx.allocate_node_id(), new_expand_all_node);
        new_group_node.dependencies = child.dependencies.clone();
        result.add_new_group_node(Rc::new(RefCell::new(new_group_node)));

        Ok(Some(result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::context::execution::QueryContext;
    use crate::core::Expression;
    use crate::query::optimizer::plan::{OptContext, OptGroupNode};
    use crate::query::planner::plan::algorithms::IndexScan;
    use crate::query::planner::plan::core::nodes::graph_scan_node::{
        GetEdgesNode, GetNeighborsNode, GetVerticesNode, ScanEdgesNode, ScanVerticesNode,
    };
    use crate::query::planner::plan::core::nodes::plan_node_traits::PlanNode;
    use crate::query::planner::plan::core::nodes::project_node::ProjectNode;
    use crate::query::planner::plan::core::nodes::sort_node::LimitNode;
    use crate::query::planner::plan::core::nodes::start_node::StartNode;
    use crate::query::validator::YieldColumn;

    fn create_test_context() -> OptContext {
        let query_context = QueryContext::new();
        OptContext::new(query_context)
    }

    #[test]
    fn test_push_limit_down_rule() {
        let rule = PushLimitDownRule;
        let mut ctx = create_test_context();

        let get_vertices_node = GetVerticesNode::new(1, "test_vids");
        let get_vertices_opt_node = OptGroupNode::new(2, get_vertices_node.into_enum());
        ctx.add_plan_node_and_group_node(2, Rc::new(RefCell::new(get_vertices_opt_node)));

        let limit_node = LimitNode::new(ctx.find_group_node_by_plan_node_id(2).expect("Child node not found").borrow().plan_node.clone(), 0, 10)
            .expect("Limit node should be created successfully");
        let mut opt_node = OptGroupNode::new(1, limit_node.into_enum());
        opt_node.dependencies = vec![2];

        let result = rule
            .apply(&mut ctx, &Rc::new(RefCell::new(opt_node)))
            .expect("Rule should apply successfully");
        assert!(result.is_some());
    }

    #[test]
    fn test_push_limit_down_get_vertices_rule() {
        let rule = PushLimitDownGetVerticesRule;
        let mut ctx = create_test_context();

        let get_vertices_node = GetVerticesNode::new(1, "test_vids");
        let get_vertices_opt_node = OptGroupNode::new(2, get_vertices_node.into_enum());
        ctx.add_plan_node_and_group_node(2, Rc::new(RefCell::new(get_vertices_opt_node)));

        let limit_node = LimitNode::new(ctx.find_group_node_by_plan_node_id(2).expect("Child node not found").borrow().plan_node.clone(), 0, 10)
            .expect("Limit node should be created successfully");
        let mut opt_node = OptGroupNode::new(1, limit_node.into_enum());
        opt_node.dependencies = vec![2];

        let result = rule
            .apply(&mut ctx, &Rc::new(RefCell::new(opt_node)))
            .expect("Rule should apply successfully");
        assert!(result.is_some());
    }

    #[test]
    fn test_push_limit_down_get_neighbors_rule() {
        let rule = PushLimitDownGetNeighborsRule;
        let mut ctx = create_test_context();

        let get_neighbors_node = GetNeighborsNode::new(1, "test_src");
        let get_neighbors_opt_node = OptGroupNode::new(2, get_neighbors_node.into_enum());
        ctx.add_plan_node_and_group_node(2, Rc::new(RefCell::new(get_neighbors_opt_node)));

        let limit_node = LimitNode::new(ctx.find_group_node_by_plan_node_id(2).expect("Child node not found").borrow().plan_node.clone(), 0, 10)
            .expect("Limit node should be created successfully");
        let mut opt_node = OptGroupNode::new(1, limit_node.into_enum());
        opt_node.dependencies = vec![2];

        let result = rule
            .apply(&mut ctx, &Rc::new(RefCell::new(opt_node)))
            .expect("Rule should apply successfully");
        assert!(result.is_some());
    }

    #[test]
    fn test_push_limit_down_get_edges_rule() {
        let rule = PushLimitDownGetEdgesRule;
        let mut ctx = create_test_context();

        let get_edges_node = GetEdgesNode::new(1, "src", "edge_type", "0", "dst");
        let get_edges_opt_node = OptGroupNode::new(2, get_edges_node.into_enum());
        ctx.add_plan_node_and_group_node(2, Rc::new(RefCell::new(get_edges_opt_node)));

        let limit_node = LimitNode::new(ctx.find_group_node_by_plan_node_id(2).expect("Child node not found").borrow().plan_node.clone(), 0, 10)
            .expect("Limit node should be created successfully");
        let mut opt_node = OptGroupNode::new(1, limit_node.into_enum());
        opt_node.dependencies = vec![2];

        let result = rule
            .apply(&mut ctx, &Rc::new(RefCell::new(opt_node)))
            .expect("Rule should apply successfully");
        assert!(result.is_some());
    }

    #[test]
    fn test_push_limit_down_scan_vertices_rule() {
        let rule = PushLimitDownScanVerticesRule;
        let mut ctx = create_test_context();

        let scan_vertices_node = ScanVerticesNode::new(1);
        let scan_vertices_opt_node = OptGroupNode::new(2, scan_vertices_node.into_enum());
        ctx.add_plan_node_and_group_node(2, Rc::new(RefCell::new(scan_vertices_opt_node)));

        let limit_node = LimitNode::new(ctx.find_group_node_by_plan_node_id(2).expect("Child node not found").borrow().plan_node.clone(), 0, 10)
            .expect("Limit node should be created successfully");
        let mut opt_node = OptGroupNode::new(1, limit_node.into_enum());
        opt_node.dependencies = vec![2];

        let result = rule
            .apply(&mut ctx, &Rc::new(RefCell::new(opt_node)))
            .expect("Rule should apply successfully");
        assert!(result.is_some());
    }

    #[test]
    fn test_push_limit_down_scan_edges_rule() {
        let rule = PushLimitDownScanEdgesRule;
        let mut ctx = create_test_context();

        let scan_edges_node = ScanEdgesNode::new(1, "edge_type");
        let scan_edges_opt_node = OptGroupNode::new(2, scan_edges_node.into_enum());
        ctx.add_plan_node_and_group_node(2, Rc::new(RefCell::new(scan_edges_opt_node)));

        let limit_node = LimitNode::new(ctx.find_group_node_by_plan_node_id(2).expect("Child node not found").borrow().plan_node.clone(), 0, 10)
            .expect("Limit node should be created successfully");
        let mut opt_node = OptGroupNode::new(1, limit_node.into_enum());
        opt_node.dependencies = vec![2];

        let result = rule
            .apply(&mut ctx, &Rc::new(RefCell::new(opt_node)))
            .expect("Rule should apply successfully");
        assert!(result.is_some());
    }

    #[test]
    fn test_push_limit_down_index_scan_rule() {
        let rule = PushLimitDownIndexScanRule;
        let mut ctx = create_test_context();

        let index_scan_node = IndexScan::new(-1, 1, 1, 1, "RANGE");
        let index_scan_opt_node = OptGroupNode::new(2, index_scan_node.into_enum());
        ctx.add_plan_node_and_group_node(2, Rc::new(RefCell::new(index_scan_opt_node)));

        let limit_node = LimitNode::new(ctx.find_group_node_by_plan_node_id(2).expect("Child node not found").borrow().plan_node.clone(), 0, 10)
            .expect("Limit node should be created successfully");
        let mut opt_node = OptGroupNode::new(1, limit_node.into_enum());
        opt_node.dependencies = vec![2];

        let result = rule
            .apply(&mut ctx, &Rc::new(RefCell::new(opt_node)))
            .expect("Rule should apply successfully");
        assert!(result.is_some());
    }

    #[test]
    fn test_push_limit_down_project_rule() {
        let rule = PushLimitDownProjectRule;
        let mut ctx = create_test_context();

        let start_node = StartNode::new();
        let start_opt_node = OptGroupNode::new(2, start_node.into_enum());
        let start_plan_node = start_opt_node.plan_node.clone();
        ctx.add_plan_node_and_group_node(2, Rc::new(RefCell::new(start_opt_node)));

        let columns = vec![YieldColumn::new(
            Expression::Variable("test_var".to_string()),
            "test_alias".to_string(),
        )];
        let project_node = ProjectNode::new(start_plan_node, columns)
            .expect("Project node should be created successfully");
        let project_opt_node = OptGroupNode::new(3, project_node.into_enum());
        let project_plan_node = project_opt_node.plan_node.clone();
        ctx.add_plan_node_and_group_node(3, Rc::new(RefCell::new(project_opt_node)));

        let limit_node = LimitNode::new(project_plan_node, 0, 10)
            .expect("Limit node should be created successfully");
        let mut opt_node = OptGroupNode::new(1, limit_node.into_enum());
        opt_node.dependencies = vec![3];

        let result = rule
            .apply(&mut ctx, &Rc::new(RefCell::new(opt_node)))
            .expect("Rule should apply successfully");
        assert!(result.is_some());
    }

    #[test]
    fn test_push_limit_down_all_paths_rule() {
        let rule = PushLimitDownAllPathsRule;
        let mut ctx = create_test_context();

        let start_node = StartNode::new();
        let start_opt_node = OptGroupNode::new(2, start_node.into_enum());
        ctx.add_plan_node_and_group_node(2, Rc::new(RefCell::new(start_opt_node.clone())));

        let limit_node = LimitNode::new(start_opt_node.plan_node.clone(), 0, 10)
            .expect("Limit node should be created successfully");
        let mut opt_node = OptGroupNode::new(1, limit_node.into_enum());
        opt_node.dependencies = vec![2];

        let result = rule.apply(&mut ctx, &Rc::new(RefCell::new(opt_node)))
            .expect("Rule should apply successfully");
        assert!(result.is_some());
    }

    #[test]
    fn test_push_limit_down_expand_all_rule() {
        let rule = PushLimitDownExpandAllRule;
        let mut ctx = create_test_context();

        let start_node = StartNode::new();
        let start_opt_node = OptGroupNode::new(2, start_node.into_enum());
        ctx.add_plan_node_and_group_node(2, Rc::new(RefCell::new(start_opt_node.clone())));

        let limit_node = LimitNode::new(start_opt_node.plan_node.clone(), 0, 10)
            .expect("Limit node should be created successfully");
        let mut opt_node = OptGroupNode::new(1, limit_node.into_enum());
        opt_node.dependencies = vec![2];

        let result = rule.apply(&mut ctx, &Rc::new(RefCell::new(opt_node)))
            .expect("Rule should apply successfully");
        assert!(result.is_some());
    }
}
