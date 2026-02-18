//! 集合操作输入顺序优化规则
//!
//! 优化集合操作的输入顺序，选择较小的输入作为构建表：
//! - Intersect: 选择较小的输入作为构建表

use crate::query::optimizer::plan::{OptContext, OptGroupNode, OptRule, Pattern, TransformResult, OptimizerError};
use crate::query::optimizer::rule_traits::BaseOptRule;
use crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum as Enum;
use crate::query::planner::plan::core::nodes::set_operations_node::IntersectNode;
use crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode;
use crate::query::visitor::PlanNodeVisitor;
use std::rc::Rc;
use std::cell::RefCell;

/// 集合操作输入顺序优化规则
///
/// 优化集合操作的输入顺序，提高执行效率
#[derive(Debug)]
pub struct OptimizeSetOperationInputOrderRule;

impl OptRule for OptimizeSetOperationInputOrderRule {
    fn name(&self) -> &str {
        "OptimizeSetOperationInputOrderRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> Result<Option<TransformResult>, OptimizerError> {
        let node_ref = group_node.borrow();
        let mut visitor = OptimizeSetOperationInputOrderVisitor {
            ctx,
            is_optimized: false,
            optimized_node: None,
        };

        let result = visitor.visit(&node_ref.plan_node);
        drop(node_ref);

        if result.is_optimized {
            if let Some(new_node) = result.optimized_node {
                let mut transform_result = TransformResult::new();
                transform_result.add_new_group_node(Rc::new(RefCell::new(new_node)));
                return Ok(Some(transform_result));
            }
        }
        Ok(None)
    }

    fn pattern(&self) -> Pattern {
        Pattern::new()
    }
}

impl BaseOptRule for OptimizeSetOperationInputOrderRule {}

/// 集合操作输入顺序优化访问者
struct OptimizeSetOperationInputOrderVisitor<'a> {
    is_optimized: bool,
    optimized_node: Option<OptGroupNode>,
    ctx: &'a mut OptContext,
}

impl<'a> PlanNodeVisitor for OptimizeSetOperationInputOrderVisitor<'a> {
    type Result = OptimizeSetOperationInputOrderResult;

    fn visit_default(&mut self) -> Self::Result {
        OptimizeSetOperationInputOrderResult {
            is_optimized: self.is_optimized,
            optimized_node: self.optimized_node.take(),
        }
    }

    fn visit_intersect(&mut self, node: &IntersectNode) -> Self::Result {
        if self.is_optimized {
            return self.visit_default();
        }

        let input = node.input();
        let intersect_input = node.intersect_input();
        
        let left_cost = estimate_node_cost(input);
        let right_cost = estimate_node_cost(intersect_input);
        
        if right_cost < left_cost {
            let new_node = match IntersectNode::new(
                intersect_input.clone(),
                input.clone(),
            ) {
                Ok(n) => n,
                Err(_) => {
                    return self.visit_default();
                }
            };
            
            self.is_optimized = true;
            self.optimized_node = Some(OptGroupNode::new(
                self.ctx.allocate_node_id(),
                Enum::Intersect(new_node),
            ));
            return self.visit_default();
        }
        
        self.visit_default()
    }
}

/// 集合操作输入顺序优化结果
struct OptimizeSetOperationInputOrderResult {
    is_optimized: bool,
    optimized_node: Option<OptGroupNode>,
}

/// 估计节点的成本
fn estimate_node_cost(node: &Enum) -> f64 {
    match node {
        Enum::Start(_) => 0.0,
        Enum::ScanVertices(n) => n.cost(),
        Enum::ScanEdges(n) => n.cost(),
        Enum::GetVertices(n) => n.cost(),
        Enum::GetEdges(n) => n.cost(),
        Enum::IndexScan(n) => n.cost(),
        Enum::EdgeIndexScan(n) => n.cost(),
        Enum::Filter(n) => estimate_node_cost(n.input()),
        Enum::Project(n) => estimate_node_cost(n.input()),
        Enum::Dedup(n) => estimate_node_cost(n.input()),
        Enum::Limit(n) => estimate_node_cost(n.input()),
        Enum::Sort(n) => estimate_node_cost(n.input()),
        Enum::Aggregate(n) => estimate_node_cost(n.input()),
        Enum::Union(n) => estimate_node_cost(n.input()),
        Enum::Minus(n) => estimate_node_cost(n.input()),
        Enum::Intersect(n) => estimate_node_cost(n.input()),
        Enum::InnerJoin(n) => n.left_input().cost() + n.right_input().cost(),
        Enum::LeftJoin(n) => n.left_input().cost() + n.right_input().cost(),
        Enum::CrossJoin(n) => n.left_input().cost() + n.right_input().cost(),
        Enum::HashInnerJoin(n) => n.left_input().cost() + n.right_input().cost(),
        Enum::HashLeftJoin(n) => n.left_input().cost() + n.right_input().cost(),
        Enum::FullOuterJoin(n) => n.left_input().cost() + n.right_input().cost(),
        Enum::Expand(n) => {
            let mut total_cost = 0.0;
            for dep in n.dependencies() {
                total_cost += dep.cost();
            }
            total_cost
        }
        Enum::ExpandAll(n) => {
            let mut total_cost = 0.0;
            for dep in n.dependencies() {
                total_cost += dep.cost();
            }
            total_cost
        }
        Enum::Traverse(n) => {
            let mut total_cost = 0.0;
            for dep in n.dependencies() {
                total_cost += dep.cost();
            }
            total_cost
        }
        Enum::GetNeighbors(n) => n.cost(),
        Enum::AppendVertices(n) => n.cost(),
        Enum::Select(n) => {
            if let Some(if_branch) = n.if_branch() {
                estimate_node_cost(if_branch)
            } else {
                0.0
            }
        }
        Enum::Loop(n) => {
            if let Some(body) = n.body() {
                estimate_node_cost(body)
            } else {
                0.0
            }
        }
        Enum::Unwind(n) => estimate_node_cost(n.input()),
        Enum::Assign(n) => estimate_node_cost(n.input()),
        Enum::PatternApply(n) => n.left_input().cost() + n.right_input().cost(),
        Enum::RollUpApply(n) => n.left_input().cost() + n.right_input().cost(),
        Enum::DataCollect(n) => {
            let mut total_cost = 0.0;
            for dep in n.dependencies() {
                total_cost += dep.cost();
            }
            total_cost
        }
        Enum::TopN(n) => {
            let mut total_cost = 0.0;
            for dep in n.dependencies() {
                total_cost += dep.cost();
            }
            total_cost
        }
        Enum::BFSShortest(n) => {
            let mut total_cost = 0.0;
            for dep in &n.deps {
                total_cost += dep.cost();
            }
            total_cost
        }
        Enum::MultiShortestPath(n) => n.cost(),
        Enum::ShortestPath(n) => n.cost(),
        Enum::Sample(n) => estimate_node_cost(n.input()),
        Enum::AllPaths(n) => n.cost(),
        Enum::Argument(_) => 0.0,
        Enum::PassThrough(_) => 0.0,
        Enum::CreateSpace(_)
        | Enum::DropSpace(_)
        | Enum::DescSpace(_)
        | Enum::ShowSpaces(_)
        | Enum::CreateTag(_)
        | Enum::DropTag(_)
        | Enum::DescTag(_)
        | Enum::ShowTags(_)
        | Enum::AlterTag(_)
        | Enum::CreateEdge(_)
        | Enum::DropEdge(_)
        | Enum::DescEdge(_)
        | Enum::ShowEdges(_)
        | Enum::AlterEdge(_)
        | Enum::CreateTagIndex(_)
        | Enum::DropTagIndex(_)
        | Enum::DescTagIndex(_)
        | Enum::ShowTagIndexes(_)
        | Enum::RebuildTagIndex(_)
        | Enum::CreateEdgeIndex(_)
        | Enum::DropEdgeIndex(_)
        | Enum::DescEdgeIndex(_)
        | Enum::ShowEdgeIndexes(_)
        | Enum::RebuildEdgeIndex(_)
        | Enum::CreateUser(_)
        | Enum::DropUser(_)
        | Enum::AlterUser(_)
        | Enum::ChangePassword(_)
        | Enum::InsertVertices(_)
        | Enum::InsertEdges(_) => 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::context::execution::QueryContext;
    use crate::query::optimizer::plan::{OptContext, OptGroupNode};
    use crate::query::planner::plan::core::nodes::graph_scan_node::ScanVerticesNode;
    use std::rc::Rc;
    use std::cell::RefCell;

    fn create_test_context() -> OptContext {
        let query_context = QueryContext::new();
        OptContext::new(query_context)
    }

    #[test]
    fn test_optimize_intersect_input_order() {
        let rule = OptimizeSetOperationInputOrderRule;
        let mut ctx = create_test_context();
        
        let scan1 = Enum::ScanVertices(ScanVerticesNode::new(1));
        let scan2 = Enum::ScanVertices(ScanVerticesNode::new(1));
        
        let intersect_node = IntersectNode::new(scan1.clone(), scan2.clone()).expect("IntersectNode创建应该成功");
        let plan_node = Enum::Intersect(intersect_node);
        let opt_node = OptGroupNode::new(1, plan_node);

        let result = rule.apply(&mut ctx, &Rc::new(RefCell::new(opt_node)));
        
        assert!(result.is_ok());
    }
}
