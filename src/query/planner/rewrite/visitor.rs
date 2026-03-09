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

use crate::query::planner::plan::core::nodes::plan_node_traits::{
    BinaryInputNode, MultipleInputNode, SingleInputNode,
};
use crate::query::planner::plan::core::nodes::plan_node_visitor::PlanNodeVisitor;
use crate::query::planner::plan::PlanNodeEnum;
use crate::query::planner::rewrite::context::RewriteContext;
use crate::query::planner::rewrite::plan_rewriter::PlanRewriter;
use crate::query::planner::rewrite::result::RewriteResult;
use crate::query::validator::context::ExpressionAnalysisContext;

use crate::query::planner::plan::core::nodes::aggregate_node::AggregateNode;
use crate::query::planner::plan::core::nodes::data_processing_node::{
    AssignNode, DataCollectNode, DedupNode, MaterializeNode, PatternApplyNode, RollUpApplyNode, UnionNode, UnwindNode,
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
use crate::query::planner::plan::core::nodes::start_node::StartNode;
use crate::query::planner::plan::core::nodes::tag_nodes::{
    AlterTagNode, CreateTagNode, DescTagNode, DropTagNode, ShowTagsNode,
};
use crate::query::planner::plan::core::nodes::user_nodes::{
    AlterUserNode, ChangePasswordNode, CreateUserNode, DropUserNode,
};

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

/// 生成多输入节点（使用 dependencies）的重写方法
macro_rules! impl_multi_input_deps_rewrite {
    ($($method:ident, $node_type:ty, $enum_variant:ident),* $(,)?) => {
        $(
            fn $method(&mut self, node: &$node_type) -> Self::Result {
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
                Ok(PlanNodeEnum::$enum_variant(new_node))
            }
        )*
    };
}

/// 生成多输入节点（使用 inputs）的重写方法
macro_rules! impl_multi_input_inputs_rewrite {
    ($($method:ident, $node_type:ty, $enum_variant:ident),* $(,)?) => {
        $(
            fn $method(&mut self, node: &$node_type) -> Self::Result {
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
                Ok(PlanNodeEnum::$enum_variant(new_node))
            }
        )*
    };
}

/// 生成无输入节点的重写方法
macro_rules! impl_no_input_rewrite {
    ($($method:ident, $node_type:ty, $enum_variant:ident),* $(,)?) => {
        $(
            fn $method(&mut self, node: &$node_type) -> Self::Result {
                Ok(PlanNodeEnum::$enum_variant(node.clone()))
            }
        )*
    };
}

impl<'a> PlanNodeVisitor for ChildRewriteVisitor<'a> {
    type Result = RewriteResult<PlanNodeEnum>;

    fn visit_default(&mut self) -> RewriteResult<PlanNodeEnum> {
        unreachable!("visit_default should not be called - all node types should have specific visit methods")
    }

    impl_single_input_rewrite!(
        visit_filter,
        FilterNode,
        Filter,
        visit_project,
        ProjectNode,
        Project,
        visit_aggregate,
        AggregateNode,
        Aggregate,
        visit_sort,
        SortNode,
        Sort,
        visit_limit,
        LimitNode,
        Limit,
        visit_topn,
        TopNNode,
        TopN,
        visit_sample,
        SampleNode,
        Sample,
        visit_dedup,
        DedupNode,
        Dedup,
        visit_unwind,
        UnwindNode,
        Unwind,
        visit_pattern_apply,
        PatternApplyNode,
        PatternApply,
        visit_roll_up_apply,
        RollUpApplyNode,
        RollUpApply,
        visit_data_collect,
        DataCollectNode,
        DataCollect,
        visit_assign,
        AssignNode,
        Assign
    );

    impl_multi_input_inputs_rewrite!(
        visit_expand,
        ExpandNode,
        Expand,
        visit_expand_all,
        ExpandAllNode,
        ExpandAll,
        visit_append_vertices,
        AppendVerticesNode,
        AppendVertices,
        visit_get_vertices,
        GetVerticesNode,
        GetVertices,
        visit_get_neighbors,
        GetNeighborsNode,
        GetNeighbors
    );

    impl_binary_input_rewrite!(
        visit_hash_inner_join,
        HashInnerJoinNode,
        HashInnerJoin,
        visit_hash_left_join,
        HashLeftJoinNode,
        HashLeftJoin,
        visit_inner_join,
        InnerJoinNode,
        InnerJoin,
        visit_left_join,
        LeftJoinNode,
        LeftJoin,
        visit_cross_join,
        CrossJoinNode,
        CrossJoin,
        visit_full_outer_join,
        FullOuterJoinNode,
        FullOuterJoin,
        visit_multi_shortest_path,
        MultiShortestPath,
        MultiShortestPath,
        visit_bfs_shortest,
        BFSShortest,
        BFSShortest,
        visit_all_paths,
        AllPaths,
        AllPaths,
        visit_shortest_path,
        ShortestPath,
        ShortestPath
    );

    impl_no_input_rewrite!(
        visit_get_edges,
        GetEdgesNode,
        GetEdges,
        visit_scan_vertices,
        ScanVerticesNode,
        ScanVertices,
        visit_scan_edges,
        ScanEdgesNode,
        ScanEdges,
        visit_edge_index_scan,
        EdgeIndexScanNode,
        EdgeIndexScan,
        visit_argument,
        ArgumentNode,
        Argument,
        visit_pass_through,
        PassThroughNode,
        PassThrough,
        visit_start,
        StartNode,
        Start,
        visit_create_space,
        CreateSpaceNode,
        CreateSpace,
        visit_drop_space,
        DropSpaceNode,
        DropSpace,
        visit_desc_space,
        DescSpaceNode,
        DescSpace,
        visit_show_spaces,
        ShowSpacesNode,
        ShowSpaces,
        visit_create_tag,
        CreateTagNode,
        CreateTag,
        visit_alter_tag,
        AlterTagNode,
        AlterTag,
        visit_desc_tag,
        DescTagNode,
        DescTag,
        visit_drop_tag,
        DropTagNode,
        DropTag,
        visit_show_tags,
        ShowTagsNode,
        ShowTags,
        visit_create_edge,
        CreateEdgeNode,
        CreateEdge,
        visit_alter_edge,
        AlterEdgeNode,
        AlterEdge,
        visit_desc_edge,
        DescEdgeNode,
        DescEdge,
        visit_drop_edge,
        DropEdgeNode,
        DropEdge,
        visit_show_edges,
        ShowEdgesNode,
        ShowEdges,
        visit_create_tag_index,
        CreateTagIndexNode,
        CreateTagIndex,
        visit_drop_tag_index,
        DropTagIndexNode,
        DropTagIndex,
        visit_desc_tag_index,
        DescTagIndexNode,
        DescTagIndex,
        visit_show_tag_indexes,
        ShowTagIndexesNode,
        ShowTagIndexes,
        visit_create_edge_index,
        CreateEdgeIndexNode,
        CreateEdgeIndex,
        visit_drop_edge_index,
        DropEdgeIndexNode,
        DropEdgeIndex,
        visit_desc_edge_index,
        DescEdgeIndexNode,
        DescEdgeIndex,
        visit_show_edge_indexes,
        ShowEdgeIndexesNode,
        ShowEdgeIndexes,
        visit_rebuild_tag_index,
        RebuildTagIndexNode,
        RebuildTagIndex,
        visit_rebuild_edge_index,
        RebuildEdgeIndexNode,
        RebuildEdgeIndex,
        visit_create_user,
        CreateUserNode,
        CreateUser,
        visit_alter_user,
        AlterUserNode,
        AlterUser,
        visit_drop_user,
        DropUserNode,
        DropUser,
        visit_change_password,
        ChangePasswordNode,
        ChangePassword,
        visit_index_scan,
        IndexScan,
        IndexScan
    );

    impl_multi_input_deps_rewrite!(
        visit_materialize,
        MaterializeNode,
        Materialize,
        visit_union,
        UnionNode,
        Union,
        visit_minus,
        MinusNode,
        Minus,
        visit_intersect,
        IntersectNode,
        Intersect,
        visit_traverse,
        TraverseNode,
        Traverse
    );

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

    fn visit_select(&mut self, node: &SelectNode) -> Self::Result {
        let mut new_node = node.clone();

        if let Some(if_branch) = node.if_branch().clone() {
            let node_id = self.ctx.allocate_node_id();
            let new_if = self.rewriter.rewrite_node(self.ctx, &if_branch, node_id)?;
            new_node.set_if_branch(new_if);
        }

        if let Some(else_branch) = node.else_branch().clone() {
            let node_id = self.ctx.allocate_node_id();
            let new_else = self
                .rewriter
                .rewrite_node(self.ctx, &else_branch, node_id)?;
            new_node.set_else_branch(new_else);
        }

        Ok(PlanNodeEnum::Select(new_node))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::expression::ExpressionMeta;
    use crate::core::Expression;
    use crate::core::Value;
    use crate::query::planner::plan::core::nodes::filter_node::FilterNode;
    use crate::query::planner::plan::core::nodes::project_node::ProjectNode;
    use crate::query::planner::plan::core::nodes::start_node::StartNode;
    use std::sync::Arc;
    use ExpressionAnalysisContext;

    #[test]
    fn test_child_rewrite_visitor_single_input() {
        let ctx = Arc::new(ExpressionAnalysisContext::new());
        let expr_meta = ExpressionMeta::new(Expression::Literal(Value::Bool(true)));
        let id = ctx.register_expression(expr_meta);
        let ctx_expr = crate::core::types::ContextualExpression::new(id, ctx);

        let start = PlanNodeEnum::Start(StartNode::new());
        let project = ProjectNode::new(start.clone(), vec![]).expect("创建ProjectNode失败");
        let filter =
            FilterNode::new(PlanNodeEnum::Project(project), ctx_expr).expect("创建FilterNode失败");

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
        match result.expect("访问不应失败") {
            PlanNodeEnum::Start(_) => {}
            _ => panic!("期望 Start 节点"),
        }
    }
}
