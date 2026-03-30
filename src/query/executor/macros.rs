//! Actuator Macro Module
//!
//! Provide a declaration macro for simplifying the implementation of the ExecutorEnum trait

/// Generate macro methods for the `ExecutorEnum` that implement the `Executor` trait
///
/// This macro automatically generates `match` statements that call the same method for all variants.
///
/// # Usage
/// ```
/// delegate_executor_method! {
///     fn method_name(&self) -> ReturnType;
/// }
/// ```
#[macro_export]
macro_rules! delegate_executor_method {
    // Immutable self: no parameters, returns a value
    ($method:ident, $return_type:ty) => {
        fn $method(&self) -> $return_type {
            match self {
                ExecutorEnum::Start(exec) => exec.$method(),
                ExecutorEnum::Base(exec) => exec.$method(),
                ExecutorEnum::GetVertices(exec) => exec.$method(),
                ExecutorEnum::GetNeighbors(exec) => exec.$method(),
                ExecutorEnum::GetProp(exec) => exec.$method(),
                ExecutorEnum::AllPaths(exec) => exec.$method(),
                ExecutorEnum::Expand(exec) => exec.$method(),
                ExecutorEnum::ExpandAll(exec) => exec.$method(),
                ExecutorEnum::Traverse(exec) => exec.$method(),
                ExecutorEnum::ShortestPath(exec) => exec.$method(),
                ExecutorEnum::InnerJoin(exec) => exec.$method(),
                ExecutorEnum::HashInnerJoin(exec) => exec.$method(),
                ExecutorEnum::LeftJoin(exec) => exec.$method(),
                ExecutorEnum::HashLeftJoin(exec) => exec.$method(),
                ExecutorEnum::FullOuterJoin(exec) => exec.$method(),
                ExecutorEnum::CrossJoin(exec) => exec.$method(),
                ExecutorEnum::Union(exec) => exec.$method(),
                ExecutorEnum::UnionAll(exec) => exec.$method(),
                ExecutorEnum::Minus(exec) => exec.$method(),
                ExecutorEnum::Intersect(exec) => exec.$method(),
                ExecutorEnum::Filter(exec) => exec.$method(),
                ExecutorEnum::Project(exec) => exec.$method(),
                ExecutorEnum::Limit(exec) => exec.$method(),
                ExecutorEnum::Sort(exec) => exec.$method(),
                ExecutorEnum::TopN(exec) => exec.$method(),
                ExecutorEnum::Sample(exec) => exec.$method(),
                ExecutorEnum::Aggregate(exec) => exec.$method(),
                ExecutorEnum::GroupBy(exec) => exec.$method(),
                ExecutorEnum::Having(exec) => exec.$method(),
                ExecutorEnum::Dedup(exec) => exec.$method(),
                ExecutorEnum::Unwind(exec) => exec.$method(),
                ExecutorEnum::Assign(exec) => exec.$method(),
                ExecutorEnum::AppendVertices(exec) => exec.$method(),
                ExecutorEnum::RollUpApply(exec) => exec.$method(),
                ExecutorEnum::PatternApply(exec) => exec.$method(),
                ExecutorEnum::Remove(exec) => exec.$method(),
                ExecutorEnum::Loop(exec) => exec.$method(),
                ExecutorEnum::ForLoop(exec) => exec.$method(),
                ExecutorEnum::WhileLoop(exec) => exec.$method(),
                ExecutorEnum::Select(exec) => exec.$method(),
                ExecutorEnum::ScanEdges(exec) => exec.$method(),
                ExecutorEnum::ScanVertices(exec) => exec.$method(),
                ExecutorEnum::IndexScan(exec) => exec.$method(),
                ExecutorEnum::Argument(exec) => exec.$method(),
                ExecutorEnum::PassThrough(exec) => exec.$method(),
                ExecutorEnum::DataCollect(exec) => exec.$method(),
                ExecutorEnum::BFSShortest(exec) => exec.$method(),
                ExecutorEnum::ShowSpaces(exec) => exec.$method(),
                ExecutorEnum::ShowTags(exec) => exec.$method(),
                ExecutorEnum::ShowEdges(exec) => exec.$method(),
                ExecutorEnum::CreateTagIndex(exec) => exec.$method(),
                ExecutorEnum::DropTagIndex(exec) => exec.$method(),
                ExecutorEnum::DescTagIndex(exec) => exec.$method(),
                ExecutorEnum::ShowTagIndexes(exec) => exec.$method(),
                ExecutorEnum::RebuildTagIndex(exec) => exec.$method(),
                ExecutorEnum::CreateEdgeIndex(exec) => exec.$method(),
                ExecutorEnum::DropEdgeIndex(exec) => exec.$method(),
                ExecutorEnum::DescEdgeIndex(exec) => exec.$method(),
                ExecutorEnum::ShowEdgeIndexes(exec) => exec.$method(),
                ExecutorEnum::RebuildEdgeIndex(exec) => exec.$method(),
                ExecutorEnum::CreateSpace(exec) => exec.$method(),
                ExecutorEnum::DropSpace(exec) => exec.$method(),
                ExecutorEnum::DescSpace(exec) => exec.$method(),
                ExecutorEnum::CreateTag(exec) => exec.$method(),
                ExecutorEnum::AlterTag(exec) => exec.$method(),
                ExecutorEnum::DescTag(exec) => exec.$method(),
                ExecutorEnum::DropTag(exec) => exec.$method(),
                ExecutorEnum::CreateEdge(exec) => exec.$method(),
                ExecutorEnum::AlterEdge(exec) => exec.$method(),
                ExecutorEnum::DescEdge(exec) => exec.$method(),
                ExecutorEnum::DropEdge(exec) => exec.$method(),
                ExecutorEnum::CreateUser(exec) => exec.$method(),
                ExecutorEnum::AlterUser(exec) => exec.$method(),
                ExecutorEnum::DropUser(exec) => exec.$method(),
                ExecutorEnum::ChangePassword(exec) => exec.$method(),
                ExecutorEnum::Analyze(exec) => exec.$method(),
            }
        }
    };

    // Variable self, no parameters, returns a value
    ($method:ident, mut $return_type:ty) => {
        fn $method(&mut self) -> $return_type {
            match self {
                ExecutorEnum::Start(exec) => exec.$method(),
                ExecutorEnum::Base(exec) => exec.$method(),
                ExecutorEnum::GetVertices(exec) => exec.$method(),
                ExecutorEnum::GetNeighbors(exec) => exec.$method(),
                ExecutorEnum::GetProp(exec) => exec.$method(),
                ExecutorEnum::AllPaths(exec) => exec.$method(),
                ExecutorEnum::Expand(exec) => exec.$method(),
                ExecutorEnum::ExpandAll(exec) => exec.$method(),
                ExecutorEnum::Traverse(exec) => exec.$method(),
                ExecutorEnum::ShortestPath(exec) => exec.$method(),
                ExecutorEnum::InnerJoin(exec) => exec.$method(),
                ExecutorEnum::HashInnerJoin(exec) => exec.$method(),
                ExecutorEnum::LeftJoin(exec) => exec.$method(),
                ExecutorEnum::HashLeftJoin(exec) => exec.$method(),
                ExecutorEnum::FullOuterJoin(exec) => exec.$method(),
                ExecutorEnum::CrossJoin(exec) => exec.$method(),
                ExecutorEnum::Union(exec) => exec.$method(),
                ExecutorEnum::UnionAll(exec) => exec.$method(),
                ExecutorEnum::Minus(exec) => exec.$method(),
                ExecutorEnum::Intersect(exec) => exec.$method(),
                ExecutorEnum::Filter(exec) => exec.$method(),
                ExecutorEnum::Project(exec) => exec.$method(),
                ExecutorEnum::Limit(exec) => exec.$method(),
                ExecutorEnum::Sort(exec) => exec.$method(),
                ExecutorEnum::TopN(exec) => exec.$method(),
                ExecutorEnum::Sample(exec) => exec.$method(),
                ExecutorEnum::Aggregate(exec) => exec.$method(),
                ExecutorEnum::GroupBy(exec) => exec.$method(),
                ExecutorEnum::Having(exec) => exec.$method(),
                ExecutorEnum::Dedup(exec) => exec.$method(),
                ExecutorEnum::Unwind(exec) => exec.$method(),
                ExecutorEnum::Assign(exec) => exec.$method(),
                ExecutorEnum::AppendVertices(exec) => exec.$method(),
                ExecutorEnum::RollUpApply(exec) => exec.$method(),
                ExecutorEnum::PatternApply(exec) => exec.$method(),
                ExecutorEnum::Remove(exec) => exec.$method(),
                ExecutorEnum::Loop(exec) => exec.$method(),
                ExecutorEnum::ForLoop(exec) => exec.$method(),
                ExecutorEnum::WhileLoop(exec) => exec.$method(),
                ExecutorEnum::Select(exec) => exec.$method(),
                ExecutorEnum::ScanEdges(exec) => exec.$method(),
                ExecutorEnum::ScanVertices(exec) => exec.$method(),
                ExecutorEnum::IndexScan(exec) => exec.$method(),
                ExecutorEnum::Argument(exec) => exec.$method(),
                ExecutorEnum::PassThrough(exec) => exec.$method(),
                ExecutorEnum::DataCollect(exec) => exec.$method(),
                ExecutorEnum::BFSShortest(exec) => exec.$method(),
                ExecutorEnum::ShowSpaces(exec) => exec.$method(),
                ExecutorEnum::ShowTags(exec) => exec.$method(),
                ExecutorEnum::ShowEdges(exec) => exec.$method(),
                ExecutorEnum::CreateTagIndex(exec) => exec.$method(),
                ExecutorEnum::DropTagIndex(exec) => exec.$method(),
                ExecutorEnum::DescTagIndex(exec) => exec.$method(),
                ExecutorEnum::ShowTagIndexes(exec) => exec.$method(),
                ExecutorEnum::RebuildTagIndex(exec) => exec.$method(),
                ExecutorEnum::CreateEdgeIndex(exec) => exec.$method(),
                ExecutorEnum::DropEdgeIndex(exec) => exec.$method(),
                ExecutorEnum::DescEdgeIndex(exec) => exec.$method(),
                ExecutorEnum::ShowEdgeIndexes(exec) => exec.$method(),
                ExecutorEnum::RebuildEdgeIndex(exec) => exec.$method(),
                ExecutorEnum::CreateSpace(exec) => exec.$method(),
                ExecutorEnum::DropSpace(exec) => exec.$method(),
                ExecutorEnum::DescSpace(exec) => exec.$method(),
                ExecutorEnum::CreateTag(exec) => exec.$method(),
                ExecutorEnum::AlterTag(exec) => exec.$method(),
                ExecutorEnum::DescTag(exec) => exec.$method(),
                ExecutorEnum::DropTag(exec) => exec.$method(),
                ExecutorEnum::CreateEdge(exec) => exec.$method(),
                ExecutorEnum::AlterEdge(exec) => exec.$method(),
                ExecutorEnum::DescEdge(exec) => exec.$method(),
                ExecutorEnum::DropEdge(exec) => exec.$method(),
                ExecutorEnum::CreateUser(exec) => exec.$method(),
                ExecutorEnum::AlterUser(exec) => exec.$method(),
                ExecutorEnum::DropUser(exec) => exec.$method(),
                ExecutorEnum::ChangePassword(exec) => exec.$method(),
                ExecutorEnum::Analyze(exec) => exec.$method(),
            }
        }
    };
}

/// Generate a macro for the InputExecutor trait method of ExecutorEnum
///
/// This macro is used to generate the `set_input` and `get_input` methods.
/// Support generating actual implementations for executors with input parameters, as well as default implementations for executors without input parameters.
#[macro_export]
macro_rules! delegate_input_executor_method {
    // The `set_input` method – distinguishes between actuators with and without input data
    (set_input, $input:ty) => {
        fn set_input(&mut self, input: $input) {
            match self {
                ExecutorEnum::Filter(exec) => exec.set_input(input),
                ExecutorEnum::Project(exec) => exec.set_input(input),
                ExecutorEnum::Limit(exec) => exec.set_input(input),
                ExecutorEnum::Sort(exec) => exec.set_input(input),
                ExecutorEnum::TopN(exec) => exec.set_input(input),
                ExecutorEnum::Sample(exec) => exec.set_input(input),
                ExecutorEnum::Dedup(exec) => exec.set_input(input),
                ExecutorEnum::Expand(exec) => exec.set_input(input),
                ExecutorEnum::ExpandAll(exec) => exec.set_input(input),
                ExecutorEnum::Traverse(exec) => exec.set_input(input),
                ExecutorEnum::ShortestPath(exec) => exec.set_input(input),
                ExecutorEnum::Aggregate(exec) => exec.set_input(input),
                ExecutorEnum::GroupBy(exec) => exec.set_input(input),
                ExecutorEnum::Having(exec) => exec.set_input(input),
                ExecutorEnum::Remove(exec) => exec.set_input(input),
                // No executor available – No action will be performed.
                _ => {}
            }
        }
    };

    // `get_input` method – Distinguishes between executors with and without input
    (get_input, $return_type:ty) => {
        fn get_input(&self) -> $return_type {
            match self {
                ExecutorEnum::Filter(exec) => exec.get_input(),
                ExecutorEnum::Project(exec) => exec.get_input(),
                ExecutorEnum::Limit(exec) => exec.get_input(),
                ExecutorEnum::Sort(exec) => exec.get_input(),
                ExecutorEnum::TopN(exec) => exec.get_input(),
                ExecutorEnum::Sample(exec) => exec.get_input(),
                ExecutorEnum::Dedup(exec) => exec.get_input(),
                ExecutorEnum::Expand(exec) => exec.get_input(),
                ExecutorEnum::ExpandAll(exec) => exec.get_input(),
                ExecutorEnum::Traverse(exec) => exec.get_input(),
                ExecutorEnum::ShortestPath(exec) => exec.get_input(),
                ExecutorEnum::Aggregate(exec) => exec.get_input(),
                ExecutorEnum::GroupBy(exec) => exec.get_input(),
                ExecutorEnum::Having(exec) => exec.get_input(),
                ExecutorEnum::Remove(exec) => exec.get_input(),
                // No executor available – Return None.
                _ => None,
            }
        }
    };
}

/// Macro for generating the Debug trait implementation for ExecutorEnum
#[macro_export]
macro_rules! delegate_debug_fmt {
    () => {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            let name = match self {
                ExecutorEnum::Start(exec) => exec.name(),
                ExecutorEnum::Base(exec) => exec.name(),
                ExecutorEnum::GetVertices(exec) => exec.name(),
                ExecutorEnum::GetNeighbors(exec) => exec.name(),
                ExecutorEnum::GetProp(exec) => exec.name(),
                ExecutorEnum::AllPaths(exec) => exec.name(),
                ExecutorEnum::Expand(exec) => exec.name(),
                ExecutorEnum::ExpandAll(exec) => exec.name(),
                ExecutorEnum::Traverse(exec) => exec.name(),
                ExecutorEnum::ShortestPath(exec) => exec.name(),
                ExecutorEnum::InnerJoin(exec) => exec.name(),
                ExecutorEnum::HashInnerJoin(exec) => exec.name(),
                ExecutorEnum::LeftJoin(exec) => exec.name(),
                ExecutorEnum::HashLeftJoin(exec) => exec.name(),
                ExecutorEnum::FullOuterJoin(exec) => exec.name(),
                ExecutorEnum::CrossJoin(exec) => exec.name(),
                ExecutorEnum::Union(exec) => exec.name(),
                ExecutorEnum::UnionAll(exec) => exec.name(),
                ExecutorEnum::Minus(exec) => exec.name(),
                ExecutorEnum::Intersect(exec) => exec.name(),
                ExecutorEnum::Filter(exec) => exec.name(),
                ExecutorEnum::Project(exec) => exec.name(),
                ExecutorEnum::Limit(exec) => exec.name(),
                ExecutorEnum::Sort(exec) => exec.name(),
                ExecutorEnum::TopN(exec) => exec.name(),
                ExecutorEnum::Sample(exec) => exec.name(),
                ExecutorEnum::Aggregate(exec) => exec.name(),
                ExecutorEnum::GroupBy(exec) => exec.name(),
                ExecutorEnum::Having(exec) => exec.name(),
                ExecutorEnum::Dedup(exec) => exec.name(),
                ExecutorEnum::Unwind(exec) => exec.name(),
                ExecutorEnum::Assign(exec) => exec.name(),
                ExecutorEnum::AppendVertices(exec) => exec.name(),
                ExecutorEnum::RollUpApply(exec) => exec.name(),
                ExecutorEnum::PatternApply(exec) => exec.name(),
                ExecutorEnum::Remove(exec) => exec.name(),
                ExecutorEnum::Loop(exec) => exec.name(),
                ExecutorEnum::ForLoop(exec) => exec.name(),
                ExecutorEnum::WhileLoop(exec) => exec.name(),
                ExecutorEnum::Select(exec) => exec.name(),
                ExecutorEnum::ScanEdges(exec) => exec.name(),
                ExecutorEnum::ScanVertices(exec) => exec.name(),
                ExecutorEnum::IndexScan(exec) => exec.name(),
                ExecutorEnum::Argument(exec) => exec.name(),
                ExecutorEnum::PassThrough(exec) => exec.name(),
                ExecutorEnum::DataCollect(exec) => exec.name(),
                ExecutorEnum::BFSShortest(exec) => exec.name(),
                ExecutorEnum::ShowSpaces(exec) => exec.name(),
                ExecutorEnum::ShowTags(exec) => exec.name(),
                ExecutorEnum::ShowEdges(exec) => exec.name(),
                ExecutorEnum::CreateTagIndex(exec) => exec.name(),
                ExecutorEnum::DropTagIndex(exec) => exec.name(),
                ExecutorEnum::DescTagIndex(exec) => exec.name(),
                ExecutorEnum::ShowTagIndexes(exec) => exec.name(),
                ExecutorEnum::RebuildTagIndex(exec) => exec.name(),
                ExecutorEnum::CreateEdgeIndex(exec) => exec.name(),
                ExecutorEnum::DropEdgeIndex(exec) => exec.name(),
                ExecutorEnum::DescEdgeIndex(exec) => exec.name(),
                ExecutorEnum::ShowEdgeIndexes(exec) => exec.name(),
                ExecutorEnum::RebuildEdgeIndex(exec) => exec.name(),
                ExecutorEnum::CreateSpace(exec) => exec.name(),
                ExecutorEnum::DropSpace(exec) => exec.name(),
                ExecutorEnum::DescSpace(exec) => exec.name(),
                ExecutorEnum::CreateTag(exec) => exec.name(),
                ExecutorEnum::AlterTag(exec) => exec.name(),
                ExecutorEnum::DescTag(exec) => exec.name(),
                ExecutorEnum::DropTag(exec) => exec.name(),
                ExecutorEnum::CreateEdge(exec) => exec.name(),
                ExecutorEnum::AlterEdge(exec) => exec.name(),
                ExecutorEnum::DescEdge(exec) => exec.name(),
                ExecutorEnum::DropEdge(exec) => exec.name(),
                ExecutorEnum::CreateUser(exec) => exec.name(),
                ExecutorEnum::AlterUser(exec) => exec.name(),
                ExecutorEnum::DropUser(exec) => exec.name(),
                ExecutorEnum::ChangePassword(exec) => exec.name(),
                ExecutorEnum::Analyze(exec) => exec.name(),
            };
            f.write_str(&format!(
                "{}({})",
                std::any::type_name::<Self>()
                    .split("::")
                    .last()
                    .unwrap_or("ExecutorEnum"),
                name
            ))
        }
    };
}

/// Macro for generating the NodeType trait implementation for ExecutorEnum
#[macro_export]
macro_rules! delegate_node_type_id {
    () => {
        fn node_type_id(&self) -> &'static str {
            match self {
                ExecutorEnum::Start(_) => "start",
                ExecutorEnum::Base(_) => "base",
                ExecutorEnum::GetVertices(_) => "get_vertices",
                ExecutorEnum::GetNeighbors(_) => "get_neighbors",
                ExecutorEnum::GetProp(_) => "get_prop",
                ExecutorEnum::AllPaths(_) => "all_paths",
                ExecutorEnum::Expand(_) => "expand",
                ExecutorEnum::ExpandAll(_) => "expand_all",
                ExecutorEnum::Traverse(_) => "traverse",
                ExecutorEnum::ShortestPath(_) => "shortest_path",
                ExecutorEnum::InnerJoin(_) => "inner_join",
                ExecutorEnum::HashInnerJoin(_) => "hash_inner_join",
                ExecutorEnum::LeftJoin(_) => "left_join",
                ExecutorEnum::HashLeftJoin(_) => "hash_left_join",
                ExecutorEnum::FullOuterJoin(_) => "full_outer_join",
                ExecutorEnum::CrossJoin(_) => "cross_join",
                ExecutorEnum::Union(_) => "union",
                ExecutorEnum::UnionAll(_) => "union_all",
                ExecutorEnum::Minus(_) => "minus",
                ExecutorEnum::Intersect(_) => "intersect",
                ExecutorEnum::Filter(_) => "filter",
                ExecutorEnum::Project(_) => "project",
                ExecutorEnum::Limit(_) => "limit",
                ExecutorEnum::Sort(_) => "sort",
                ExecutorEnum::TopN(_) => "topn",
                ExecutorEnum::Sample(_) => "sample",
                ExecutorEnum::Aggregate(_) => "aggregate",
                ExecutorEnum::GroupBy(_) => "group_by",
                ExecutorEnum::Having(_) => "having",
                ExecutorEnum::Dedup(_) => "dedup",
                ExecutorEnum::Unwind(_) => "unwind",
                ExecutorEnum::Assign(_) => "assign",
                ExecutorEnum::AppendVertices(_) => "append_vertices",
                ExecutorEnum::RollUpApply(_) => "rollup_apply",
                ExecutorEnum::PatternApply(_) => "pattern_apply",
                ExecutorEnum::Remove(_) => "remove",
                ExecutorEnum::Loop(_) => "loop",
                ExecutorEnum::ForLoop(_) => "for_loop",
                ExecutorEnum::WhileLoop(_) => "while_loop",
                ExecutorEnum::Select(_) => "select",
                ExecutorEnum::ScanEdges(_) => "scan_edges",
                ExecutorEnum::ScanVertices(_) => "scan_vertices",
                ExecutorEnum::IndexScan(_) => "index_scan",
                ExecutorEnum::Argument(_) => "argument",
                ExecutorEnum::PassThrough(_) => "pass_through",
                ExecutorEnum::DataCollect(_) => "data_collect",
                ExecutorEnum::BFSShortest(_) => "bfs_shortest",
                ExecutorEnum::ShowSpaces(_) => "show_spaces",
                ExecutorEnum::ShowTags(_) => "show_tags",
                ExecutorEnum::ShowEdges(_) => "show_edges",
                ExecutorEnum::CreateTagIndex(_) => "create_tag_index",
                ExecutorEnum::DropTagIndex(_) => "drop_tag_index",
                ExecutorEnum::DescTagIndex(_) => "desc_tag_index",
                ExecutorEnum::ShowTagIndexes(_) => "show_tag_indexes",
                ExecutorEnum::RebuildTagIndex(_) => "rebuild_tag_index",
                ExecutorEnum::CreateEdgeIndex(_) => "create_edge_index",
                ExecutorEnum::DropEdgeIndex(_) => "drop_edge_index",
                ExecutorEnum::DescEdgeIndex(_) => "desc_edge_index",
                ExecutorEnum::ShowEdgeIndexes(_) => "show_edge_indexes",
                ExecutorEnum::RebuildEdgeIndex(_) => "rebuild_edge_index",
                ExecutorEnum::CreateSpace(_) => "create_space",
                ExecutorEnum::DropSpace(_) => "drop_space",
                ExecutorEnum::DescSpace(_) => "desc_space",
                ExecutorEnum::CreateTag(_) => "create_tag",
                ExecutorEnum::AlterTag(_) => "alter_tag",
                ExecutorEnum::DescTag(_) => "desc_tag",
                ExecutorEnum::DropTag(_) => "drop_tag",
                ExecutorEnum::CreateEdge(_) => "create_edge",
                ExecutorEnum::AlterEdge(_) => "alter_edge",
                ExecutorEnum::DescEdge(_) => "desc_edge",
                ExecutorEnum::DropEdge(_) => "drop_edge",
                ExecutorEnum::CreateUser(_) => "create_user",
                ExecutorEnum::AlterUser(_) => "alter_user",
                ExecutorEnum::DropUser(_) => "drop_user",
                ExecutorEnum::ChangePassword(_) => "change_password",
                ExecutorEnum::Analyze(_) => "analyze",
            }
        }
    };
}
