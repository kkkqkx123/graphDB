//! Executor Factory Master Module
//!
//! Coordinating various builders, parsers, and validators
//! Responsible for creating the corresponding executor instances based on the execution plan.

use crate::coordinator::FulltextCoordinator;
use crate::core::error::QueryError;
use crate::core::types::span::Span;
use crate::query::executor::base::ExecutionContext;
use crate::query::executor::executor_enum::ExecutorEnum;
use crate::query::executor::factory::builders::{
    AdminBuilder, ControlFlowBuilder, DataAccessBuilder, DataModificationBuilder,
    DataProcessingBuilder, JoinBuilder, SetOperationBuilder, TransformationBuilder,
    TraversalBuilder,
};
use crate::query::executor::factory::validators::RecursionDetector;
use crate::query::planning::plan::core::nodes::base::plan_node_enum::PlanNodeEnum;
use crate::query::planning::plan::core::nodes::base::plan_node_traits::PlanNode;
use crate::storage::StorageClient;
use parking_lot::Mutex;
use std::sync::Arc;

// Import security configuration type
use crate::query::executor::factory::validators::safety_validator::ExecutorSafetyConfig;

/// Actuator Factory
///
/// Responsible for coordinating the creation of executors for each sub-module
pub struct ExecutorFactory<S: StorageClient + Send + 'static> {
    pub(crate) storage: Option<Arc<Mutex<S>>>,
    pub(crate) config: ExecutorSafetyConfig,
    pub(crate) recursion_detector: RecursionDetector,
    pub(crate) fulltext_coordinator: Option<Arc<FulltextCoordinator>>,
}

impl<S: StorageClient + Send + 'static> ExecutorFactory<S> {
    /// Create a new executor factory.
    pub fn new() -> Self {
        let config = ExecutorSafetyConfig::default();
        let recursion_detector = RecursionDetector::new(config.max_recursion_depth);

        Self {
            storage: None,
            config,
            recursion_detector,
            fulltext_coordinator: None,
        }
    }

    /// Setting the storage engine
    pub fn with_storage(storage: Arc<Mutex<S>>) -> Self {
        let mut factory = Self::new();
        factory.storage = Some(storage);
        factory
    }

    /// Setting the fulltext coordinator
    pub fn with_fulltext_coordinator(coordinator: Arc<FulltextCoordinator>) -> Self {
        let mut factory = Self::new();
        factory.fulltext_coordinator = Some(coordinator);
        factory
    }

    /// Setting both storage and fulltext coordinator
    pub fn with_storage_and_coordinator(
        storage: Arc<Mutex<S>>,
        coordinator: Arc<FulltextCoordinator>,
    ) -> Self {
        let mut factory = Self::new();
        factory.storage = Some(storage);
        factory.fulltext_coordinator = Some(coordinator);
        factory
    }

    /// Set fulltext coordinator
    pub fn set_fulltext_coordinator(&mut self, coordinator: Arc<FulltextCoordinator>) {
        self.fulltext_coordinator = Some(coordinator);
    }

    /// Analyzing the lifecycle and security of execution plans
    ///
    /// Traverse the execution plan tree using DFS to detect circular references and verify security.
    pub fn analyze_plan_lifecycle(&mut self, root: &PlanNodeEnum) -> Result<(), QueryError> {
        self.recursion_detector.reset();
        self.analyze_plan_node(root, 0)?;
        Ok(())
    }

    /// Recursive analysis of a single planning node
    #[allow(clippy::only_used_in_recursion)]
    fn analyze_plan_node(
        &mut self,
        node: &PlanNodeEnum,
        loop_layers: usize,
    ) -> Result<(), QueryError> {
        let node_id = node.id();
        let node_name = node.name();

        // Verify whether the execution of the executor will lead to recursion.
        self.recursion_detector
            .validate_executor(node_id, node_name)
            .map_err(|e| QueryError::ExecutionError(e.to_string()))?;

        // Verify the security of the plan nodes.
        self.validate_plan_node(node)?;

        // 使用 dependencies() 方法获取所有依赖，统一处理
        for dep in node.dependencies() {
            self.analyze_plan_node(&dep, loop_layers + 1)?;
        }

        // Leave the current node
        self.recursion_detector.leave_executor();

        Ok(())
    }

    /// Verify the security of the plan nodes.
    fn validate_plan_node(&self, plan_node: &PlanNodeEnum) -> Result<(), QueryError> {
        match plan_node {
            PlanNodeEnum::Expand(node) => {
                let step_limit = node
                    .step_limit()
                    .and_then(|s| usize::try_from(s).ok())
                    .unwrap_or(10);
                if step_limit > 1000 {
                    return Err(QueryError::ExecutionError(format!(
                        "The number of steps limit for the Expand executor {} exceeds the safety threshold of 1000.",
                        step_limit
                    )));
                }
            }
            PlanNodeEnum::Loop(_) => {
                return Err(QueryError::ExecutionError(
                    "循环执行器需要手动构建，不支持通过工厂自动创建".to_string(),
                ));
            }
            _ => {}
        }
        Ok(())
    }

    /// Create an executor based on the planned node.
    pub fn create_executor(
        &mut self,
        plan_node: &PlanNodeEnum,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        self.validate_plan_node(plan_node)?;

        if self.config.enable_recursion_detection {
            self.recursion_detector
                .validate_executor(plan_node.id(), plan_node.name())
                .map_err(|e| QueryError::ExecutionError(e.to_string()))?;
        }

        match plan_node {
            PlanNodeEnum::Start(node) => {
                use crate::query::executor::base::StartExecutor;
                Ok(ExecutorEnum::Start(StartExecutor::new(
                    node.id(),
                    context.expression_context().clone(),
                )))
            }

            // Data Access Executor
            PlanNodeEnum::ScanVertices(node) => {
                DataAccessBuilder::build_scan_vertices(node, storage, context)
            }
            PlanNodeEnum::ScanEdges(node) => {
                DataAccessBuilder::build_scan_edges(node, storage, context)
            }
            PlanNodeEnum::GetVertices(node) => {
                DataAccessBuilder::build_get_vertices(node, storage, context)
            }
            PlanNodeEnum::GetNeighbors(node) => {
                DataAccessBuilder::build_get_neighbors(node, storage, context)
            }
            PlanNodeEnum::EdgeIndexScan(node) => {
                DataAccessBuilder::build_edge_index_scan(node, storage, context)
            }
            PlanNodeEnum::GetEdges(node) => {
                DataAccessBuilder::build_get_edges(node, storage, context)
            }
            PlanNodeEnum::IndexScan(node) => {
                DataAccessBuilder::build_index_scan(node, storage, context)
            }

            // Data Modification Executor
            PlanNodeEnum::InsertVertices(node) => {
                DataModificationBuilder::build_insert_vertices(node, storage, context)
            }
            PlanNodeEnum::InsertEdges(node) => {
                DataModificationBuilder::build_insert_edges(node, storage, context)
            }
            PlanNodeEnum::DeleteVertices(node) => {
                DataModificationBuilder::build_delete_vertices(node, storage, context)
            }
            PlanNodeEnum::DeleteEdges(node) => {
                DataModificationBuilder::build_delete_edges(node, storage, context)
            }
            PlanNodeEnum::Update(node) => {
                DataModificationBuilder::build_update(node, storage, context)
            }
            PlanNodeEnum::UpdateVertices(node) => {
                DataModificationBuilder::build_update_vertices(node, storage, context)
            }
            PlanNodeEnum::UpdateEdges(node) => {
                DataModificationBuilder::build_update_edges(node, storage, context)
            }
            PlanNodeEnum::Remove(node) => {
                DataModificationBuilder::build_remove(node, storage, context)
            }

            // Data Processing Executor
            PlanNodeEnum::Filter(node) => {
                DataProcessingBuilder::build_filter(node, storage, context)
            }
            PlanNodeEnum::Project(node) => {
                DataProcessingBuilder::build_project(node, storage, context)
            }
            PlanNodeEnum::Limit(node) => DataProcessingBuilder::build_limit(node, storage, context),
            PlanNodeEnum::Sort(node) => DataProcessingBuilder::build_sort(node, storage, context),
            PlanNodeEnum::TopN(node) => DataProcessingBuilder::build_topn(node, storage, context),
            PlanNodeEnum::Sample(node) => {
                DataProcessingBuilder::build_sample(node, storage, context)
            }
            PlanNodeEnum::Aggregate(node) => {
                DataProcessingBuilder::build_aggregate(node, storage, context)
            }
            PlanNodeEnum::Dedup(node) => DataProcessingBuilder::build_dedup(node, storage, context),

            // Connect the actuator.
            PlanNodeEnum::InnerJoin(node) => JoinBuilder::build_inner_join(node, storage, context),
            PlanNodeEnum::HashInnerJoin(node) => {
                JoinBuilder::build_hash_inner_join(node, storage, context)
            }
            PlanNodeEnum::LeftJoin(node) => JoinBuilder::build_left_join(node, storage, context),
            PlanNodeEnum::HashLeftJoin(node) => {
                JoinBuilder::build_hash_left_join(node, storage, context)
            }
            PlanNodeEnum::FullOuterJoin(node) => {
                JoinBuilder::build_full_outer_join(node, storage, context)
            }
            PlanNodeEnum::CrossJoin(node) => JoinBuilder::build_cross_join(node, storage, context),

            // Set Operation Executor
            PlanNodeEnum::Union(node) => SetOperationBuilder::build_union(node, storage, context),
            PlanNodeEnum::Minus(node) => SetOperationBuilder::build_minus(node, storage, context),
            PlanNodeEnum::Intersect(node) => {
                SetOperationBuilder::build_intersect(node, storage, context)
            }

            // Graph Traversal Executor
            PlanNodeEnum::Expand(node) => TraversalBuilder::build_expand(node, storage, context),
            PlanNodeEnum::ExpandAll(node) => {
                TraversalBuilder::build_expand_all(node, storage, context)
            }
            PlanNodeEnum::Traverse(node) => {
                TraversalBuilder::build_traverse(node, storage, context)
            }
            PlanNodeEnum::AllPaths(node) => {
                TraversalBuilder::build_all_paths(node, storage, context)
            }
            PlanNodeEnum::ShortestPath(node) => {
                TraversalBuilder::build_shortest_path(node, storage, context)
            }
            PlanNodeEnum::BFSShortest(node) => {
                TraversalBuilder::build_bfs_shortest(node, storage, context)
            }
            PlanNodeEnum::MultiShortestPath(node) => {
                TraversalBuilder::build_multi_shortest_path(node, storage, context)
            }

            // Data Conversion Executor
            PlanNodeEnum::Unwind(node) => {
                TransformationBuilder::build_unwind(node, storage, context)
            }
            PlanNodeEnum::Assign(node) => {
                TransformationBuilder::build_assign(node, storage, context)
            }
            PlanNodeEnum::Materialize(node) => {
                TransformationBuilder::build_materialize(node, storage, context)
            }
            PlanNodeEnum::AppendVertices(node) => {
                TransformationBuilder::build_append_vertices(node, storage, context)
            }
            PlanNodeEnum::RollUpApply(node) => {
                TransformationBuilder::build_rollup_apply(node, storage, context)
            }
            PlanNodeEnum::PatternApply(node) => {
                TransformationBuilder::build_pattern_apply(node, storage, context)
            }

            // Control Flow Executor
            PlanNodeEnum::Loop(node) => self.build_loop_executor(node, storage, context),
            PlanNodeEnum::Select(node) => self.build_select_executor(node, storage, context),
            PlanNodeEnum::Argument(node) => {
                ControlFlowBuilder::build_argument(node, storage, context)
            }
            PlanNodeEnum::PassThrough(node) => {
                ControlFlowBuilder::build_pass_through(node, storage, context)
            }
            PlanNodeEnum::DataCollect(node) => {
                ControlFlowBuilder::build_data_collect(node, storage, context)
            }

            // Manage Executor – Space Management
            PlanNodeEnum::CreateSpace(node) => {
                AdminBuilder::build_create_space(node, storage, context)
            }
            PlanNodeEnum::DropSpace(node) => AdminBuilder::build_drop_space(node, storage, context),
            PlanNodeEnum::DescSpace(node) => AdminBuilder::build_desc_space(node, storage, context),
            PlanNodeEnum::ShowSpaces(node) => {
                AdminBuilder::build_show_spaces(node, storage, context)
            }

            // Manage Executor – Tag Management
            PlanNodeEnum::CreateTag(node) => AdminBuilder::build_create_tag(node, storage, context),
            PlanNodeEnum::AlterTag(node) => AdminBuilder::build_alter_tag(node, storage, context),
            PlanNodeEnum::DescTag(node) => AdminBuilder::build_desc_tag(node, storage, context),
            PlanNodeEnum::DropTag(node) => AdminBuilder::build_drop_tag(node, storage, context),
            PlanNodeEnum::ShowTags(node) => AdminBuilder::build_show_tags(node, storage, context),
            PlanNodeEnum::ShowCreateTag(node) => {
                AdminBuilder::build_show_create_tag(node, storage, context)
            }

            // Manage Executor – Edge Management
            PlanNodeEnum::CreateEdge(node) => {
                AdminBuilder::build_create_edge(node, storage, context)
            }
            PlanNodeEnum::AlterEdge(node) => AdminBuilder::build_alter_edge(node, storage, context),
            PlanNodeEnum::DescEdge(node) => AdminBuilder::build_desc_edge(node, storage, context),
            PlanNodeEnum::DropEdge(node) => AdminBuilder::build_drop_edge(node, storage, context),
            PlanNodeEnum::ShowEdges(node) => AdminBuilder::build_show_edges(node, storage, context),

            // Manage Executor – Tag Index Management
            PlanNodeEnum::CreateTagIndex(node) => {
                AdminBuilder::build_create_tag_index(node, storage, context)
            }
            PlanNodeEnum::DropTagIndex(node) => {
                AdminBuilder::build_drop_tag_index(node, storage, context)
            }
            PlanNodeEnum::DescTagIndex(node) => {
                AdminBuilder::build_desc_tag_index(node, storage, context)
            }
            PlanNodeEnum::ShowTagIndexes(node) => {
                AdminBuilder::build_show_tag_indexes(node, storage, context)
            }
            PlanNodeEnum::RebuildTagIndex(node) => {
                AdminBuilder::build_rebuild_tag_index(node, storage, context)
            }

            // Manage Executor – Edge Index Management
            PlanNodeEnum::CreateEdgeIndex(node) => {
                AdminBuilder::build_create_edge_index(node, storage, context)
            }
            PlanNodeEnum::DropEdgeIndex(node) => {
                AdminBuilder::build_drop_edge_index(node, storage, context)
            }
            PlanNodeEnum::DescEdgeIndex(node) => {
                AdminBuilder::build_desc_edge_index(node, storage, context)
            }
            PlanNodeEnum::ShowEdgeIndexes(node) => {
                AdminBuilder::build_show_edge_indexes(node, storage, context)
            }
            PlanNodeEnum::RebuildEdgeIndex(node) => {
                AdminBuilder::build_rebuild_edge_index(node, storage, context)
            }

            // Manage Executor – User Management
            PlanNodeEnum::CreateUser(node) => {
                AdminBuilder::build_create_user(node, storage, context)
            }
            PlanNodeEnum::DropUser(node) => AdminBuilder::build_drop_user(node, storage, context),
            PlanNodeEnum::AlterUser(node) => AdminBuilder::build_alter_user(node, storage, context),
            PlanNodeEnum::ChangePassword(node) => {
                AdminBuilder::build_change_password(node, storage, context)
            }
            PlanNodeEnum::GrantRole(node) => AdminBuilder::build_grant_role(node, storage, context),
            PlanNodeEnum::RevokeRole(node) => {
                AdminBuilder::build_revoke_role(node, storage, context)
            }

            // Manage Executor – Space Management (Supplementary)
            PlanNodeEnum::SwitchSpace(node) => {
                AdminBuilder::build_switch_space(node, storage, context)
            }
            PlanNodeEnum::AlterSpace(node) => {
                AdminBuilder::build_alter_space(node, storage, context)
            }
            PlanNodeEnum::ClearSpace(node) => {
                AdminBuilder::build_clear_space(node, storage, context)
            }

            // Management Executor – Query Management
            PlanNodeEnum::ShowStats(node) => AdminBuilder::build_show_stats(node, storage, context),

            // Full-text Search Executors
            PlanNodeEnum::CreateFulltextIndex(node) => {
                self.build_create_fulltext_index(node, storage, context)
            }
            PlanNodeEnum::DropFulltextIndex(node) => {
                self.build_drop_fulltext_index(node, storage, context)
            }
            PlanNodeEnum::AlterFulltextIndex(node) => {
                self.build_alter_fulltext_index(node, storage, context)
            }
            PlanNodeEnum::ShowFulltextIndex(node) => {
                self.build_show_fulltext_index(node, storage, context)
            }
            PlanNodeEnum::DescribeFulltextIndex(node) => {
                self.build_describe_fulltext_index(node, storage, context)
            }
            PlanNodeEnum::FulltextSearch(node) => {
                self.build_fulltext_search(node, storage, context)
            }
            PlanNodeEnum::FulltextLookup(node) => {
                self.build_fulltext_lookup(node, storage, context)
            }
            PlanNodeEnum::MatchFulltext(node) => self.build_match_fulltext(node, storage, context),
        }
    }

    /// Building the Loop Executor (auxiliary method to address the borrowing-check issue)
    fn build_loop_executor(
        &mut self,
        node: &crate::query::planning::plan::core::nodes::LoopNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        // First, verify and check the recursion.
        if self.config.enable_recursion_detection {
            self.recursion_detector
                .validate_executor(node.id(), "LoopExecutor")
                .map_err(|e| QueryError::ExecutionError(e.to_string()))?;
        }

        let body = node
            .body()
            .as_ref()
            .ok_or_else(|| QueryError::ExecutionError("Loop节点缺少body".to_string()))?;

        // Temporarily release the borrowing of the `self` object to construct the `bodyExecutor`.
        let body_executor = {
            // Re-obtain the variable reference
            let config = self.config.clone();
            let max_recursion_depth = config.max_recursion_depth;
            let mut temp_factory = ExecutorFactory {
                storage: self.storage.clone(),
                config,
                recursion_detector: RecursionDetector::new(max_recursion_depth),
                fulltext_coordinator: self.fulltext_coordinator.clone(),
            };

            temp_factory.create_executor(body, storage.clone(), context)?
        };

        let condition = node
            .condition()
            .expression()
            .map(|meta| meta.inner().clone());

        use crate::query::executor::logic::LoopExecutor;
        let executor = LoopExecutor::new(
            node.id(),
            storage,
            condition,
            body_executor,
            None,
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::Loop(executor))
    }

    /// Constructing the Select Executor (an auxiliary method to resolve borrowing check issues)
    fn build_select_executor(
        &mut self,
        node: &crate::query::planning::plan::core::nodes::SelectNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        // First, verify and check the recursion.
        if self.config.enable_recursion_detection {
            self.recursion_detector
                .validate_executor(node.id(), "SelectExecutor")
                .map_err(|e| QueryError::ExecutionError(e.to_string()))?;
        }

        let condition = node
            .condition()
            .expression()
            .map(|meta| meta.inner().clone())
            .unwrap_or_else(|| crate::core::Expression::Literal(crate::core::Value::Bool(true)));

        // Construct the `if_branch`.
        let if_branch = {
            let if_node = node
                .if_branch()
                .as_ref()
                .ok_or_else(|| QueryError::ExecutionError("Select节点缺少if_branch".to_string()))?;

            let config = self.config.clone();
            let max_recursion_depth = config.max_recursion_depth;
            let mut temp_factory = ExecutorFactory {
                storage: self.storage.clone(),
                config,
                recursion_detector: RecursionDetector::new(max_recursion_depth),
                fulltext_coordinator: self.fulltext_coordinator.clone(),
            };

            temp_factory.create_executor(if_node, storage.clone(), context)?
        };

        // Construct the `else_branch`.
        let else_branch = {
            if let Some(else_node) = node.else_branch().as_ref() {
                let config = self.config.clone();
                let max_recursion_depth = config.max_recursion_depth;
                let mut temp_factory = ExecutorFactory {
                    storage: self.storage.clone(),
                    config,
                    recursion_detector: RecursionDetector::new(max_recursion_depth),
                    fulltext_coordinator: self.fulltext_coordinator.clone(),
                };

                Some(temp_factory.create_executor(else_node, storage.clone(), context)?)
            } else {
                None
            }
        };

        use crate::query::executor::logic::SelectExecutor;
        let executor = SelectExecutor::new(
            node.id(),
            storage,
            condition,
            if_branch,
            else_branch,
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::Select(executor))
    }

    // Full-text search executor building methods

    fn build_create_fulltext_index(
        &mut self,
        node: &crate::query::planning::plan::core::nodes::CreateFulltextIndexNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        use crate::query::executor::admin::{
            CreateFulltextIndexConfig, CreateFulltextIndexExecutor,
        };

        let coordinator = self
            .fulltext_coordinator
            .as_ref()
            .or_else(|| context.fulltext_coordinator())
            .ok_or_else(|| {
                QueryError::ExecutionError("Fulltext coordinator not available".to_string())
            })?
            .clone();

        let space_id = context.current_space_id().unwrap_or(0);

        let executor = CreateFulltextIndexExecutor::new(
            node.id(),
            storage,
            CreateFulltextIndexConfig {
                index_name: node.index_name.clone(),
                schema_name: node.schema_name.clone(),
                fields: node.fields.clone(),
                engine_type: node.engine_type,
                options: node.options.clone(),
                if_not_exists: node.if_not_exists,
                space_id,
            },
            context.expression_context().clone(),
            coordinator,
        );
        Ok(ExecutorEnum::CreateFulltextIndex(executor))
    }

    fn build_drop_fulltext_index(
        &mut self,
        node: &crate::query::planning::plan::core::nodes::DropFulltextIndexNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        use crate::query::executor::admin::DropFulltextIndexExecutor;

        let coordinator = self
            .fulltext_coordinator
            .as_ref()
            .or_else(|| context.fulltext_coordinator())
            .ok_or_else(|| {
                QueryError::ExecutionError("Fulltext coordinator not available".to_string())
            })?
            .clone();

        let space_id = context.current_space_id().unwrap_or(0);

        let executor = DropFulltextIndexExecutor::new(
            node.id(),
            storage,
            node.index_name.clone(),
            node.if_exists,
            space_id,
            context.expression_context().clone(),
            coordinator,
        );
        Ok(ExecutorEnum::DropFulltextIndex(executor))
    }

    fn build_alter_fulltext_index(
        &mut self,
        node: &crate::query::planning::plan::core::nodes::AlterFulltextIndexNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        use crate::query::executor::admin::AlterFulltextIndexExecutor;

        let executor = AlterFulltextIndexExecutor::new(
            node.id(),
            storage,
            node.index_name.clone(),
            node.actions.clone(),
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::AlterFulltextIndex(executor))
    }

    fn build_show_fulltext_index(
        &mut self,
        node: &crate::query::planning::plan::core::nodes::ShowFulltextIndexNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        use crate::query::executor::admin::ShowFulltextIndexExecutor;

        let coordinator = self
            .fulltext_coordinator
            .as_ref()
            .or_else(|| context.fulltext_coordinator())
            .ok_or_else(|| {
                QueryError::ExecutionError("Fulltext coordinator not available".to_string())
            })?
            .clone();

        let executor = ShowFulltextIndexExecutor::new(
            node.id(),
            storage,
            context.expression_context().clone(),
            coordinator,
        );
        Ok(ExecutorEnum::ShowFulltextIndex(executor))
    }

    fn build_describe_fulltext_index(
        &mut self,
        node: &crate::query::planning::plan::core::nodes::DescribeFulltextIndexNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        use crate::query::executor::admin::DescribeFulltextIndexExecutor;

        let coordinator = self
            .fulltext_coordinator
            .as_ref()
            .or_else(|| context.fulltext_coordinator())
            .ok_or_else(|| {
                QueryError::ExecutionError("Fulltext coordinator not available".to_string())
            })?
            .clone();

        let space_id = context.current_space_id().unwrap_or(0);

        let executor = DescribeFulltextIndexExecutor::new(
            node.id(),
            storage,
            node.index_name.clone(),
            space_id,
            context.expression_context().clone(),
            coordinator,
        );
        Ok(ExecutorEnum::DescribeFulltextIndex(executor))
    }

    fn build_fulltext_search(
        &mut self,
        node: &crate::query::planning::plan::core::nodes::FulltextSearchNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        use crate::query::executor::data_access::FulltextSearchExecutor;
        use crate::query::parser::ast::SearchStatement;

        let statement = SearchStatement {
            span: Span::default(),
            index_name: node.index_name.clone(),
            query: node.query.clone(),
            yield_clause: node.yield_clause.clone(),
            where_clause: node.where_clause.clone(),
            order_clause: node.order_clause.clone(),
            limit: node.limit,
            offset: node.offset,
        };

        let search_engine = context
            .search_engine()
            .ok_or_else(|| QueryError::ExecutionError("Search engine not available".to_string()))?
            .clone();

        let coordinator = context
            .fulltext_coordinator()
            .ok_or_else(|| {
                QueryError::ExecutionError("Fulltext coordinator not available".to_string())
            })?
            .clone();

        let executor = FulltextSearchExecutor::new(
            node.id(),
            statement,
            search_engine,
            context.clone(),
            storage,
            context.expression_context().clone(),
            coordinator,
        );
        Ok(ExecutorEnum::FulltextSearch(executor))
    }

    fn build_fulltext_lookup(
        &mut self,
        node: &crate::query::planning::plan::core::nodes::FulltextLookupNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        use crate::query::executor::data_access::{FulltextScanConfig, FulltextScanExecutor};

        let search_engine = context
            .search_engine()
            .ok_or_else(|| QueryError::ExecutionError("Search engine not available".to_string()))?
            .clone();

        let coordinator = context
            .fulltext_coordinator()
            .ok_or_else(|| {
                QueryError::ExecutionError("Fulltext coordinator not available".to_string())
            })?
            .clone();

        let executor = FulltextScanExecutor::new(
            node.id(),
            FulltextScanConfig {
                index_name: node.index_name.clone(),
                query: node.query.clone(),
                limit: node.limit,
            },
            search_engine,
            context.clone(),
            storage,
            context.expression_context().clone(),
            coordinator,
        );
        Ok(ExecutorEnum::FulltextLookup(executor))
    }

    fn build_match_fulltext(
        &mut self,
        node: &crate::query::planning::plan::core::nodes::MatchFulltextNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        use crate::query::executor::data_access::MatchFulltextExecutor;

        let coordinator = context
            .fulltext_coordinator()
            .ok_or_else(|| {
                QueryError::ExecutionError("Fulltext coordinator not available".to_string())
            })?
            .clone();

        let executor = MatchFulltextExecutor::new(
            node.id(),
            storage,
            node.fulltext_condition.clone(),
            node.yield_clause.clone(),
            context.expression_context().clone(),
            coordinator,
        );
        Ok(ExecutorEnum::MatchFulltext(executor))
    }
}

impl<S: StorageClient + 'static> Clone for ExecutorFactory<S> {
    fn clone(&self) -> Self {
        Self {
            storage: self.storage.clone(),
            config: self.config.clone(),
            recursion_detector: RecursionDetector::new(self.config.max_recursion_depth),
            fulltext_coordinator: self.fulltext_coordinator.clone(),
        }
    }
}

impl<S: StorageClient + 'static> Default for ExecutorFactory<S> {
    fn default() -> Self {
        Self::new()
    }
}
