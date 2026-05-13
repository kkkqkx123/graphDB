//! Query context
//!
//! Manage the context information throughout the entire lifecycle of queries, from parsing and validation to planning and execution.
//!
//! ## Arena Allocation
//!
//! For high-performance query execution with many temporary allocations,
//! enable arena allocation via `with_arena()`. This is beneficial for:
//!
//! - Complex queries with many intermediate results
//! - Expression evaluation with temporary values
//! - Graph traversal with path accumulation

use std::sync::Arc;

use crate::core::types::{CharsetInfo, SpaceInfo};
use crate::utils::{Arena, IdGenerator};

use super::{QueryExecutionManager, QueryRequestContext};

/// Query context
///
/// The context for each query request is created whenever the query request is received by the query engine.
/// This context object is visible to the parser, planner, optimizer, and executor.
///
/// # Responsibilities
///
/// The context of the query request is available (session information, request parameters).
/// Possession of the Query Execution Manager (execution plan, termination flags)
/// ID generation for query execution
/// Spatial information management (space info, character set)
/// Optional arena allocator for high-performance temporary allocations
pub struct QueryContext {
    /// Query request context
    rctx: Arc<QueryRequestContext>,
    /// Query Execution Manager
    execution_manager: QueryExecutionManager,
    /// ID Generator for query execution
    id_gen: IdGenerator,
    /// Current space information
    space_info: Option<SpaceInfo>,
    /// Character set information
    charset_info: Option<Box<CharsetInfo>>,
    /// Optional arena allocator for temporary allocations during query execution
    arena: Option<Arena>,
}

impl QueryContext {
    /// Create a new query context.
    pub fn new(rctx: Arc<QueryRequestContext>) -> Self {
        Self {
            rctx,
            execution_manager: QueryExecutionManager::new(),
            id_gen: IdGenerator::new(0),
            space_info: None,
            charset_info: None,
            arena: None,
        }
    }

    /// Create a new query context with arena allocation enabled.
    ///
    /// Arena allocation is beneficial for queries that create many
    /// temporary data structures during execution.
    pub fn with_arena(rctx: Arc<QueryRequestContext>) -> Self {
        Self {
            rctx,
            execution_manager: QueryExecutionManager::new(),
            id_gen: IdGenerator::new(0),
            space_info: None,
            charset_info: None,
            arena: Some(Arena::new()),
        }
    }

    /// Create a new query context with a custom arena capacity.
    pub fn with_arena_capacity(rctx: Arc<QueryRequestContext>, capacity: usize) -> Self {
        Self {
            rctx,
            execution_manager: QueryExecutionManager::new(),
            id_gen: IdGenerator::new(0),
            space_info: None,
            charset_info: None,
            arena: Some(Arena::with_capacity(capacity)),
        }
    }

    /// Create a temporary context for verification.
    ///
    /// This is a convenient method for creating a temporary QueryContext during the validation phase.
    ///
    /// # Parameters
    /// `query_text`: The text of the query.
    ///
    /// # Example
    ///
    /// ```rust
    /// use crate::query::context::QueryContext;
    ///
    /// let qctx = QueryContext::new_for_validation("MATCH (n) RETURN n".to_string());
    /// ```
    pub fn new_for_validation(query_text: String) -> Self {
        let rctx = Arc::new(QueryRequestContext::new(query_text));
        Self::new(rctx)
    }

    /// Create a temporary context for planning purposes.
    ///
    /// This is a convenient method for creating a temporary QueryContext during the planning phase.
    ///
    /// # Parameters
    /// - `query_text`: The query text
    ///
    /// # Example
    ///
    /// ```rust
    /// use crate::query::context::QueryContext;
    ///
    /// let qctx = QueryContext::new_for_planning("MATCH (n) RETURN n".to_string());
    /// ```
    pub fn new_for_planning(query_text: String) -> Self {
        let rctx = Arc::new(QueryRequestContext::new(query_text));
        Self::new(rctx)
    }

    /// Create query contexts from various components (for use by the Builder).
    pub(crate) fn from_components(
        rctx: Arc<QueryRequestContext>,
        execution_manager: QueryExecutionManager,
        id_gen: IdGenerator,
        space_info: Option<SpaceInfo>,
        charset_info: Option<Box<CharsetInfo>>,
    ) -> Self {
        Self {
            rctx,
            execution_manager,
            id_gen,
            space_info,
            charset_info,
            arena: None,
        }
    }

    /// Create query contexts from various components with arena (for use by the Builder).
    #[allow(dead_code)]
    pub(crate) fn from_components_with_arena(
        rctx: Arc<QueryRequestContext>,
        execution_manager: QueryExecutionManager,
        id_gen: IdGenerator,
        space_info: Option<SpaceInfo>,
        charset_info: Option<Box<CharsetInfo>>,
        arena: Arena,
    ) -> Self {
        Self {
            rctx,
            execution_manager,
            id_gen,
            space_info,
            charset_info,
            arena: Some(arena),
        }
    }

    /// Create a builder.
    pub fn builder(rctx: Arc<QueryRequestContext>) -> super::QueryContextBuilder {
        super::QueryContextBuilder::new(rctx)
    }

    /// Obtain the context of the query request.
    pub fn request_context(&self) -> &QueryRequestContext {
        &self.rctx
    }

    /// The Arc reference that provides the context for the query request.
    pub fn request_context_arc(&self) -> Arc<QueryRequestContext> {
        self.rctx.clone()
    }

    /// Obtain the context of the query request (compatible with old code)
    pub fn rctx(&self) -> &QueryRequestContext {
        &self.rctx
    }

    /// Obtain the execution plan
    pub fn plan(&self) -> Option<crate::query::planning::plan::ExecutionPlan> {
        self.execution_manager.plan()
    }

    /// Setting the execution plan
    pub fn set_plan(&mut self, plan: crate::query::planning::plan::ExecutionPlan) {
        self.execution_manager.set_plan(plan);
    }

    /// Obtain the execution plan ID
    pub fn plan_id(&self) -> Option<i64> {
        self.execution_manager.plan_id()
    }

    /// Obtaining character set information
    pub fn charset_info(&self) -> Option<&CharsetInfo> {
        self.charset_info.as_ref().map(|ci| ci.as_ref())
    }

    /// Setting character set information
    pub fn set_charset_info(&mut self, charset_info: CharsetInfo) {
        self.charset_info = Some(Box::new(charset_info));
    }

    /// Generate an ID.
    pub fn gen_id(&self) -> i64 {
        self.id_gen.id()
    }

    /// Retrieve the current ID value (without incrementing it).
    pub fn current_id(&self) -> i64 {
        self.id_gen.current_value()
    }

    /// Obtain the current spatial information
    pub fn space_info(&self) -> Option<&SpaceInfo> {
        self.space_info.as_ref()
    }

    /// Set the current space information
    pub fn set_space_info(&mut self, space_info: SpaceInfo) {
        self.space_info = Some(space_info);
    }

    /// Obtain the ID of the current space.
    pub fn space_id(&self) -> Option<u64> {
        self.space_info.as_ref().map(|s| s.space_id)
    }

    /// Get the name of the current space.
    pub fn space_name(&self) -> Option<String> {
        self.space_info.as_ref().map(|s| s.space_name.clone())
    }

    /// Marked as terminated
    pub fn mark_killed(&self) {
        self.execution_manager.mark_killed();
    }

    /// Check whether it was terminated.
    pub fn is_killed(&self) -> bool {
        self.execution_manager.is_killed()
    }

    /// Check whether the parameters exist.
    pub fn exist_parameter(&self, param: &str) -> bool {
        self.rctx.get_parameter(param).is_some()
    }

    /// Obtain the query string
    pub fn query(&self) -> &str {
        &self.rctx.query
    }

    /// Obtain parameters
    pub fn parameters(&self) -> &std::collections::HashMap<String, crate::core::Value> {
        &self.rctx.parameters
    }

    /// Reset the query context
    pub fn reset(&mut self) {
        self.execution_manager.reset();
        self.id_gen.reset(0);
        self.space_info = None;
        self.charset_info = None;
        if let Some(ref mut arena) = self.arena {
            arena.reset();
        }
        log::info!("Query context has been reset");
    }

    /// Check if arena allocation is enabled
    pub fn has_arena(&self) -> bool {
        self.arena.is_some()
    }

    /// Get a reference to the arena allocator
    pub fn arena(&self) -> Option<&Arena> {
        self.arena.as_ref()
    }

    /// Get arena memory statistics (allocated_bytes)
    pub fn arena_stats(&self) -> Option<usize> {
        self.arena.as_ref().map(|a| a.allocated_bytes())
    }

    /// Obtain a reference to the query execution manager.
    pub fn execution_manager(&self) -> &QueryExecutionManager {
        &self.execution_manager
    }

    /// Obtain a variable reference to the query execution manager.
    pub fn execution_manager_mut(&mut self) -> &mut QueryExecutionManager {
        &mut self.execution_manager
    }

    // Note: resource_context() and space_context() methods have been removed
    // as part of the optimization to inline these contexts into QueryContext.
    // Use the direct accessor methods instead:
    // - gen_id(), current_id() for resource operations
    // - space_info(), space_id(), space_name(), charset_info() for space operations
}

impl std::fmt::Debug for QueryContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QueryContext")
            .field("rctx", &self.rctx)
            .field("plan_id", &self.plan_id())
            .field("space_id", &self.space_id())
            .field("killed", &self.is_killed())
            .field("has_arena", &self.arena.is_some())
            .finish()
    }
}

impl Default for QueryContext {
    fn default() -> Self {
        Self::new(Arc::new(QueryRequestContext::default()))
    }
}
