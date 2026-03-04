//! 计划节点访问者 - 用于重写子节点
//!
//! 本模块提供 ChildRewriteVisitor，用于遍历计划树并重写所有子节点。
//! 利用现有的 PlanNodeVisitor trait，消除 plan_rewriter.rs 中的重复代码。
//!
//! # 设计优势
//!
//! - 消除重复代码：统一子节点重写逻辑
//! - 保持类型安全：使用静态分发，避免动态分发开销
//! - 易于扩展：新增节点类型时只需实现对应方法
//! - 与现有架构兼容：利用已有的 PlanNodeVisitor

use crate::query::planner::plan::PlanNodeEnum;
use crate::query::planner::plan::core::nodes::plan_node_visitor::PlanNodeVisitor;
use crate::query::planner::plan::core::nodes::plan_node_traits::{BinaryInputNode, MultipleInputNode, SingleInputNode};
use crate::query::planner::rewrite::context::RewriteContext;
use crate::query::planner::rewrite::plan_rewriter::PlanRewriter;
use crate::query::planner::rewrite::result::RewriteResult;

use crate::query::planner::plan::core::nodes::aggregate_node::AggregateNode;
use crate::query::planner::plan::core::nodes::data_processing_node::{
    DedupNode, MaterializeNode, PatternApplyNode, RollUpApplyNode, UnionNode, UnwindNode,
};
use crate::query::planner::plan::core::nodes::filter_node::FilterNode;
use crate::query::planner::plan::core::nodes::graph_scan_node::{
    EdgeIndexScanNode, GetEdgesNode, GetNeighborsNode, GetVerticesNode, ScanEdgesNode,
    ScanVerticesNode,
};
use crate::query::planner::plan::core::nodes::join_node::{
    CrossJoinNode, FullOuterJoinNode, HashInnerJoinNode, HashLeftJoinNode, InnerJoinNode,
    LeftJoinNode,
};
use crate::query::planner::plan::core::nodes::project_node::ProjectNode;
use crate::query::planner::plan::core::nodes::sample_node::SampleNode;
use crate::query::planner::plan::core::nodes::set_operations_node::{IntersectNode, MinusNode};
use crate::query::planner::plan::core::nodes::sort_node::{LimitNode, SortNode, TopNNode};
use crate::query::planner::plan::core::nodes::traversal_node::{
    AppendVerticesNode, ExpandAllNode, ExpandNode, TraverseNode,
};

use crate::query::planner::plan::core::nodes::control_flow_node::{
    ArgumentNode, LoopNode, PassThroughNode, SelectNode,
};
use crate::query::planner::plan::core::nodes::start_node::StartNode;
use crate::query::planner::plan::core::nodes::tag_nodes::{
    AlterTagNode, CreateTagNode, DescTagNode, DropTagNode, ShowTagsNode,
};
use crate::query::planner::plan::core::nodes::edge_nodes::{
    AlterEdgeNode, CreateEdgeNode, DescEdgeNode, DropEdgeNode, ShowEdgesNode,
};
use crate::query::planner::plan::core::nodes::index_nodes::{
    CreateEdgeIndexNode, CreateTagIndexNode, DescEdgeIndexNode, DescTagIndexNode,
    DropEdgeIndexNode, DropTagIndexNode, RebuildEdgeIndexNode, RebuildTagIndexNode,
    ShowEdgeIndexesNode, ShowTagIndexesNode,
};
use crate::query::planner::plan::core::nodes::space_nodes::{
    CreateSpaceNode, DescSpaceNode, DropSpaceNode, ShowSpacesNode,
};
use crate::query::planner::plan::core::nodes::user_nodes::{
    AlterUserNode, ChangePasswordNode, CreateUserNode, DropUserNode,
};
use crate::query::planner::plan::core::nodes::insert_nodes::{InsertVerticesNode, InsertEdgesNode};

use crate::query::planner::plan::algorithms::{
    AllPaths, BFSShortest, IndexScan, MultiShortestPath, ShortestPath,
};

/// 子节点重写访问者
///
/// 遍历计划树并重写所有子节点，用于 PlanRewriter 的 rewrite_children 方法。
/// 利用 PlanNodeVisitor trait 实现零成本抽象。
pub struct ChildRewriteVisitor<'a> {
    ctx: &'a mut RewriteContext,
    rewriter: &'a PlanRewriter,
}

impl<'a> ChildRewriteVisitor<'a> {
    pub fn new(ctx: &'a mut RewriteContext, rewriter: &'a PlanRewriter) -> Self {
        Self { ctx, rewriter }
    }
}

/// 生成单输入节点的重写方法
macro_rules! impl_single_input_rewrite {
    ($($method:ident, $node_type:ty, $enum_variant:ident),* $(,)?) => {
        $(
            fn $method(&mut self, node: &$node_type) -> Self::Result {
                let input_node = node.input().clone_plan_node();
                let node_id = self.ctx.allocate_node_id();
                let new_input = self.rewriter.rewrite_node(self.ctx, &input_node, node_id)?;
                let mut new_node = node.clone();
                new_node.set_input(new_input);
                Ok(PlanNodeEnum::$enum_variant(new_node))
            }
        )*
    };
}

/// 生成双输入节点的重写方法
macro_rules! impl_binary_input_rewrite {
    ($($method:ident, $node_type:ty, $enum_variant:ident),* $(,)?) => {
        $(
            fn $method(&mut self, node: &$node_type) -> Self::Result {
                let left = node.left_input().clone_plan_node();
                let right = node.right_input().clone_plan_node();
                let left_id = self.ctx.allocate_node_id();
                let right_id = self.ctx.allocate_node_id();
                let new_left = self.rewriter.rewrite_node(self.ctx, &left, left_id)?;
                let new_right = self.rewriter.rewrite_node(self.ctx, &right, right_id)?;
                let mut new_node = node.clone();
                new_node.set_left_input(new_left);
                new_node.set_right_input(new_right);
                Ok(PlanNodeEnum::$enum_variant(new_node))
            }
        )*
    };
}

impl<'a> PlanNodeVisitor for ChildRewriteVisitor<'a> {
    type Result = RewriteResult<PlanNodeEnum>;

    fn visit_default(&mut self) -> RewriteResult<PlanNodeEnum> {
        todo!("visit_default should not be called")
    }

    impl_single_input_rewrite!(
        visit_filter, FilterNode, Filter,
        visit_project, ProjectNode, Project,
        visit_aggregate, AggregateNode, Aggregate,
        visit_sort, SortNode, Sort,
        visit_limit, LimitNode, Limit,
        visit_topn, TopNNode, TopN,
        visit_sample, SampleNode, Sample,
        visit_dedup, DedupNode, Dedup,
        visit_unwind, UnwindNode, Unwind,
        visit_pattern_apply, PatternApplyNode, PatternApply,
        visit_roll_up_apply, RollUpApplyNode, RollUpApply
    );

    fn visit_materialize(&mut self, node: &MaterializeNode) -> Self::Result {
        let deps: Vec<PlanNodeEnum> = node
            .dependencies()
            .iter()
            .map(|dep| dep.as_ref().clone())
            .collect();
        let mut new_deps = Vec::new();
        for dep in deps.iter() {
            let node_id = self.ctx.allocate_node_id();
            let new_dep = self.rewriter.rewrite_node(self.ctx, dep, node_id)?;
            new_deps.push(new_dep);
        }
        let mut new_node = node.clone();
        new_node.set_dependencies(new_deps);
        Ok(PlanNodeEnum::Materialize(new_node))
    }

    impl_binary_input_rewrite!(
        visit_hash_inner_join, HashInnerJoinNode, HashInnerJoin,
        visit_hash_left_join, HashLeftJoinNode, HashLeftJoin,
        visit_inner_join, InnerJoinNode, InnerJoin,
        visit_left_join, LeftJoinNode, LeftJoin,
        visit_cross_join, CrossJoinNode, CrossJoin,
        visit_full_outer_join, FullOuterJoinNode, FullOuterJoin
    );

    fn visit_union(&mut self, node: &UnionNode) -> Self::Result {
        let deps: Vec<PlanNodeEnum> = node
            .dependencies()
            .iter()
            .map(|dep| dep.as_ref().clone())
            .collect();
        let mut new_deps = Vec::new();
        for dep in deps.iter() {
            let node_id = self.ctx.allocate_node_id();
            let new_dep = self.rewriter.rewrite_node(self.ctx, dep, node_id)?;
            new_deps.push(new_dep);
        }
        let mut new_node = node.clone();
        new_node.set_dependencies(new_deps);
        Ok(PlanNodeEnum::Union(new_node))
    }

    fn visit_minus(&mut self, node: &MinusNode) -> Self::Result {
        let deps: Vec<PlanNodeEnum> = node
            .dependencies()
            .iter()
            .map(|dep| dep.as_ref().clone())
            .collect();
        let mut new_deps = Vec::new();
        for dep in deps.iter() {
            let node_id = self.ctx.allocate_node_id();
            let new_dep = self.rewriter.rewrite_node(self.ctx, dep, node_id)?;
            new_deps.push(new_dep);
        }
        let mut new_node = node.clone();
        new_node.set_dependencies(new_deps);
        Ok(PlanNodeEnum::Minus(new_node))
    }

    fn visit_intersect(&mut self, node: &IntersectNode) -> Self::Result {
        let deps: Vec<PlanNodeEnum> = node
            .dependencies()
            .iter()
            .map(|dep| dep.as_ref().clone())
            .collect();
        let mut new_deps = Vec::new();
        for dep in deps.iter() {
            let node_id = self.ctx.allocate_node_id();
            let new_dep = self.rewriter.rewrite_node(self.ctx, dep, node_id)?;
            new_deps.push(new_dep);
        }
        let mut new_node = node.clone();
        new_node.set_dependencies(new_deps);
        Ok(PlanNodeEnum::Intersect(new_node))
    }

    fn visit_expand(&mut self, node: &ExpandNode) -> Self::Result {
        let deps: Vec<PlanNodeEnum> = node
            .inputs()
            .iter()
            .map(|dep| dep.as_ref().clone())
            .collect();
        let mut new_deps = Vec::new();
        for dep in deps.iter() {
            let node_id = self.ctx.allocate_node_id();
            let new_dep = self.rewriter.rewrite_node(self.ctx, dep, node_id)?;
            new_deps.push(new_dep);
        }
        let mut new_node = node.clone();
        *new_node.inputs_mut() = new_deps.into_iter().map(Box::new).collect();
        Ok(PlanNodeEnum::Expand(new_node))
    }

    fn visit_expand_all(&mut self, node: &ExpandAllNode) -> Self::Result {
        let deps: Vec<PlanNodeEnum> = node
            .inputs()
            .iter()
            .map(|dep| dep.as_ref().clone())
            .collect();
        let mut new_deps = Vec::new();
        for dep in deps.iter() {
            let node_id = self.ctx.allocate_node_id();
            let new_dep = self.rewriter.rewrite_node(self.ctx, dep, node_id)?;
            new_deps.push(new_dep);
        }
        let mut new_node = node.clone();
        *new_node.inputs_mut() = new_deps.into_iter().map(Box::new).collect();
        Ok(PlanNodeEnum::ExpandAll(new_node))
    }

    fn visit_traverse(&mut self, node: &TraverseNode) -> Self::Result {
        let deps: Vec<PlanNodeEnum> = node
            .dependencies()
            .iter()
            .map(|dep| dep.as_ref().clone())
            .collect();
        let mut new_deps = Vec::new();
        for dep in deps.iter() {
            let node_id = self.ctx.allocate_node_id();
            let new_dep = self.rewriter.rewrite_node(self.ctx, dep, node_id)?;
            new_deps.push(new_dep);
        }
        let mut new_node = node.clone();
        new_node.set_dependencies(new_deps);
        Ok(PlanNodeEnum::Traverse(new_node))
    }

    fn visit_append_vertices(&mut self, node: &AppendVerticesNode) -> Self::Result {
        let deps: Vec<PlanNodeEnum> = node
            .inputs()
            .iter()
            .map(|dep| dep.as_ref().clone())
            .collect();
        let mut new_deps = Vec::new();
        for dep in deps.iter() {
            let node_id = self.ctx.allocate_node_id();
            let new_dep = self.rewriter.rewrite_node(self.ctx, dep, node_id)?;
            new_deps.push(new_dep);
        }
        let mut new_node = node.clone();
        *new_node.inputs_mut() = new_deps.into_iter().map(Box::new).collect();
        Ok(PlanNodeEnum::AppendVertices(new_node))
    }

    fn visit_get_vertices(&mut self, node: &GetVerticesNode) -> Self::Result {
        let deps: Vec<PlanNodeEnum> = node
            .inputs()
            .iter()
            .map(|dep| dep.as_ref().clone())
            .collect();
        let mut new_deps = Vec::new();
        for dep in deps.iter() {
            let node_id = self.ctx.allocate_node_id();
            let new_dep = self.rewriter.rewrite_node(self.ctx, dep, node_id)?;
            new_deps.push(new_dep);
        }
        let mut new_node = node.clone();
        *new_node.inputs_mut() = new_deps.into_iter().map(Box::new).collect();
        Ok(PlanNodeEnum::GetVertices(new_node))
    }

    fn visit_get_edges(&mut self, _node: &GetEdgesNode) -> Self::Result {
        Ok(PlanNodeEnum::GetEdges(_node.clone()))
    }

    fn visit_get_neighbors(&mut self, node: &GetNeighborsNode) -> Self::Result {
        let deps: Vec<PlanNodeEnum> = node
            .inputs()
            .iter()
            .map(|dep| dep.as_ref().clone())
            .collect();
        let mut new_deps = Vec::new();
        for dep in deps.iter() {
            let node_id = self.ctx.allocate_node_id();
            let new_dep = self.rewriter.rewrite_node(self.ctx, dep, node_id)?;
            new_deps.push(new_dep);
        }
        let mut new_node = node.clone();
        *new_node.inputs_mut() = new_deps.into_iter().map(Box::new).collect();
        Ok(PlanNodeEnum::GetNeighbors(new_node))
    }

    fn visit_scan_vertices(&mut self, _node: &ScanVerticesNode) -> Self::Result {
        Ok(PlanNodeEnum::ScanVertices(_node.clone()))
    }

    fn visit_scan_edges(&mut self, _node: &ScanEdgesNode) -> Self::Result {
        Ok(PlanNodeEnum::ScanEdges(_node.clone()))
    }

    fn visit_edge_index_scan(&mut self, _node: &EdgeIndexScanNode) -> Self::Result {
        Ok(PlanNodeEnum::EdgeIndexScan(_node.clone()))
    }

    fn visit_argument(&mut self, _node: &ArgumentNode) -> Self::Result {
        Ok(PlanNodeEnum::Argument(_node.clone()))
    }

    fn visit_loop(&mut self, node: &LoopNode) -> Self::Result {
        let body = node.body().clone();
        if let Some(body_node) = body {
            let node_id = self.ctx.allocate_node_id();
            let new_body = self.rewriter.rewrite_node(self.ctx, &body_node, node_id)?;
            let mut new_node = node.clone();
            new_node.set_body(new_body);
            Ok(PlanNodeEnum::Loop(new_node))
        } else {
            Ok(PlanNodeEnum::Loop(node.clone()))
        }
    }

    fn visit_pass_through(&mut self, _node: &PassThroughNode) -> Self::Result {
        Ok(PlanNodeEnum::PassThrough(_node.clone()))
    }

    fn visit_select(&mut self, node: &SelectNode) -> Self::Result {
        let mut new_node = node.clone();
        
        if let Some(if_branch) = node.if_branch().clone() {
            let node_id = self.ctx.allocate_node_id();
            let new_if = self.rewriter.rewrite_node(self.ctx, &if_branch, node_id)?;
            new_node.set_if_branch(new_if);
        }
        
        if let Some(else_branch) = node.else_branch().clone() {
            let node_id = self.ctx.allocate_node_id();
            let new_else = self.rewriter.rewrite_node(self.ctx, &else_branch, node_id)?;
            new_node.set_else_branch(new_else);
        }
        
        Ok(PlanNodeEnum::Select(new_node))
    }

    fn visit_start(&mut self, _node: &StartNode) -> Self::Result {
        Ok(PlanNodeEnum::Start(_node.clone()))
    }

    fn visit_create_space(&mut self, _node: &CreateSpaceNode) -> Self::Result {
        Ok(PlanNodeEnum::CreateSpace(_node.clone()))
    }

    fn visit_drop_space(&mut self, _node: &DropSpaceNode) -> Self::Result {
        Ok(PlanNodeEnum::DropSpace(_node.clone()))
    }

    fn visit_desc_space(&mut self, _node: &DescSpaceNode) -> Self::Result {
        Ok(PlanNodeEnum::DescSpace(_node.clone()))
    }

    fn visit_show_spaces(&mut self, _node: &ShowSpacesNode) -> Self::Result {
        Ok(PlanNodeEnum::ShowSpaces(_node.clone()))
    }

    fn visit_create_tag(&mut self, _node: &CreateTagNode) -> Self::Result {
        Ok(PlanNodeEnum::CreateTag(_node.clone()))
    }

    fn visit_alter_tag(&mut self, _node: &AlterTagNode) -> Self::Result {
        Ok(PlanNodeEnum::AlterTag(_node.clone()))
    }

    fn visit_desc_tag(&mut self, _node: &DescTagNode) -> Self::Result {
        Ok(PlanNodeEnum::DescTag(_node.clone()))
    }

    fn visit_drop_tag(&mut self, _node: &DropTagNode) -> Self::Result {
        Ok(PlanNodeEnum::DropTag(_node.clone()))
    }

    fn visit_show_tags(&mut self, _node: &ShowTagsNode) -> Self::Result {
        Ok(PlanNodeEnum::ShowTags(_node.clone()))
    }

    fn visit_create_edge(&mut self, _node: &CreateEdgeNode) -> Self::Result {
        Ok(PlanNodeEnum::CreateEdge(_node.clone()))
    }

    fn visit_alter_edge(&mut self, _node: &AlterEdgeNode) -> Self::Result {
        Ok(PlanNodeEnum::AlterEdge(_node.clone()))
    }

    fn visit_desc_edge(&mut self, _node: &DescEdgeNode) -> Self::Result {
        Ok(PlanNodeEnum::DescEdge(_node.clone()))
    }

    fn visit_drop_edge(&mut self, _node: &DropEdgeNode) -> Self::Result {
        Ok(PlanNodeEnum::DropEdge(_node.clone()))
    }

    fn visit_show_edges(&mut self, _node: &ShowEdgesNode) -> Self::Result {
        Ok(PlanNodeEnum::ShowEdges(_node.clone()))
    }

    fn visit_create_tag_index(&mut self, _node: &CreateTagIndexNode) -> Self::Result {
        Ok(PlanNodeEnum::CreateTagIndex(_node.clone()))
    }

    fn visit_drop_tag_index(&mut self, _node: &DropTagIndexNode) -> Self::Result {
        Ok(PlanNodeEnum::DropTagIndex(_node.clone()))
    }

    fn visit_desc_tag_index(&mut self, _node: &DescTagIndexNode) -> Self::Result {
        Ok(PlanNodeEnum::DescTagIndex(_node.clone()))
    }

    fn visit_show_tag_indexes(&mut self, _node: &ShowTagIndexesNode) -> Self::Result {
        Ok(PlanNodeEnum::ShowTagIndexes(_node.clone()))
    }

    fn visit_create_edge_index(&mut self, _node: &CreateEdgeIndexNode) -> Self::Result {
        Ok(PlanNodeEnum::CreateEdgeIndex(_node.clone()))
    }

    fn visit_drop_edge_index(&mut self, _node: &DropEdgeIndexNode) -> Self::Result {
        Ok(PlanNodeEnum::DropEdgeIndex(_node.clone()))
    }

    fn visit_desc_edge_index(&mut self, _node: &DescEdgeIndexNode) -> Self::Result {
        Ok(PlanNodeEnum::DescEdgeIndex(_node.clone()))
    }

    fn visit_show_edge_indexes(&mut self, _node: &ShowEdgeIndexesNode) -> Self::Result {
        Ok(PlanNodeEnum::ShowEdgeIndexes(_node.clone()))
    }

    fn visit_rebuild_tag_index(&mut self, _node: &RebuildTagIndexNode) -> Self::Result {
        Ok(PlanNodeEnum::RebuildTagIndex(_node.clone()))
    }

    fn visit_rebuild_edge_index(&mut self, _node: &RebuildEdgeIndexNode) -> Self::Result {
        Ok(PlanNodeEnum::RebuildEdgeIndex(_node.clone()))
    }

    fn visit_create_user(&mut self, _node: &CreateUserNode) -> Self::Result {
        Ok(PlanNodeEnum::CreateUser(_node.clone()))
    }

    fn visit_alter_user(&mut self, _node: &AlterUserNode) -> Self::Result {
        Ok(PlanNodeEnum::AlterUser(_node.clone()))
    }

    fn visit_drop_user(&mut self, _node: &DropUserNode) -> Self::Result {
        Ok(PlanNodeEnum::DropUser(_node.clone()))
    }

    fn visit_change_password(&mut self, _node: &ChangePasswordNode) -> Self::Result {
        Ok(PlanNodeEnum::ChangePassword(_node.clone()))
    }

    fn visit_index_scan(&mut self, _node: &IndexScan) -> Self::Result {
        Ok(PlanNodeEnum::IndexScan(_node.clone()))
    }

    fn visit_multi_shortest_path(&mut self, node: &MultiShortestPath) -> Self::Result {
        let left = node.left_input().clone_plan_node();
        let right = node.right_input().clone_plan_node();
        let left_id = self.ctx.allocate_node_id();
        let right_id = self.ctx.allocate_node_id();
        let new_left = self.rewriter.rewrite_node(self.ctx, &left, left_id)?;
        let new_right = self.rewriter.rewrite_node(self.ctx, &right, right_id)?;
        let mut new_node = node.clone();
        new_node.set_left_input(new_left);
        new_node.set_right_input(new_right);
        Ok(PlanNodeEnum::MultiShortestPath(new_node))
    }

    fn visit_bfs_shortest(&mut self, node: &BFSShortest) -> Self::Result {
        let left = node.left_input().clone_plan_node();
        let right = node.right_input().clone_plan_node();
        let left_id = self.ctx.allocate_node_id();
        let right_id = self.ctx.allocate_node_id();
        let new_left = self.rewriter.rewrite_node(self.ctx, &left, left_id)?;
        let new_right = self.rewriter.rewrite_node(self.ctx, &right, right_id)?;
        let mut new_node = node.clone();
        new_node.set_left_input(new_left);
        new_node.set_right_input(new_right);
        Ok(PlanNodeEnum::BFSShortest(new_node))
    }

    fn visit_all_paths(&mut self, node: &AllPaths) -> Self::Result {
        let left = node.left_input().clone_plan_node();
        let right = node.right_input().clone_plan_node();
        let left_id = self.ctx.allocate_node_id();
        let right_id = self.ctx.allocate_node_id();
        let new_left = self.rewriter.rewrite_node(self.ctx, &left, left_id)?;
        let new_right = self.rewriter.rewrite_node(self.ctx, &right, right_id)?;
        let mut new_node = node.clone();
        new_node.set_left_input(new_left);
        new_node.set_right_input(new_right);
        Ok(PlanNodeEnum::AllPaths(new_node))
    }

    fn visit_shortest_path(&mut self, node: &ShortestPath) -> Self::Result {
        let left = node.left_input().clone_plan_node();
        let right = node.right_input().clone_plan_node();
        let left_id = self.ctx.allocate_node_id();
        let right_id = self.ctx.allocate_node_id();
        let new_left = self.rewriter.rewrite_node(self.ctx, &left, left_id)?;
        let new_right = self.rewriter.rewrite_node(self.ctx, &right, right_id)?;
        let mut new_node = node.clone();
        new_node.set_left_input(new_left);
        new_node.set_right_input(new_right);
        Ok(PlanNodeEnum::ShortestPath(new_node))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::planner::plan::core::nodes::start_node::StartNode;
    use crate::query::planner::plan::core::nodes::filter_node::FilterNode;
    use crate::query::planner::plan::core::nodes::project_node::ProjectNode;
    use crate::core::types::ExpressionContext;
    use crate::core::types::expression::ExpressionMeta;
    use crate::core::Expression;
    use crate::core::Value;
    use std::sync::Arc;

    #[test]
    fn test_child_rewrite_visitor_single_input() {
        let ctx = Arc::new(ExpressionAnalysisContext::new());
        let expr_meta = ExpressionMeta::new(Expression::Literal(Value::Bool(true)));
        let id = ctx.register_expression(expr_meta);
        let ctx_expr = crate::core::types::ContextualExpression::new(id, ctx);

        let start = PlanNodeEnum::Start(StartNode::new());
        let project = ProjectNode::new(start.clone(), vec![]).expect("创建ProjectNode失败");
        let filter = FilterNode::new(PlanNodeEnum::Project(project), ctx_expr)
            .expect("创建FilterNode失败");

        let mut rewrite_ctx = RewriteContext::new();
        let rewriter = PlanRewriter::new();
        let mut visitor = ChildRewriteVisitor::new(&mut rewrite_ctx, &rewriter);

        let result = visitor.visit_filter(&filter);
        assert!(result.is_ok());
    }

    #[test]
    fn test_child_rewrite_visitor_leaf_node() {
        let start = StartNode::new();
        let mut rewrite_ctx = RewriteContext::new();
        let rewriter = PlanRewriter::new();
        let mut visitor = ChildRewriteVisitor::new(&mut rewrite_ctx, &rewriter);

        let result = visitor.visit_start(&start);
        assert!(result.is_ok());
        match result.unwrap() {
            PlanNodeEnum::Start(_) => {}
            _ => panic!("期望 Start 节点"),
        }
    }
}
