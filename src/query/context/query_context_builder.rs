//! QueryContext Builder
//!
//! Provide a streaming API for building QueryContext, which simplifies the process of creating complex objects.

use crate::core::types::{CharsetInfo, EngineType, SpaceInfo, SpaceStatus, SpaceSummary};
use crate::utils::IdGenerator;
use std::sync::Arc;

use super::{QueryContext, QueryExecutionManager, QueryRequestContext};

/// QueryContext Builder
///
/// A streaming API is provided to build the QueryContext, simplifying the process of creating complex objects.
///
/// # Example
///
/// ```rust
/// use crate::query::context::{QueryContextBuilder, QueryRequestContext};
/// use std::sync::Arc;
///
/// let rctx = Arc::new(QueryRequestContext::new("MATCH (n) RETURN n".to_string()));
///
/// let query_context = QueryContextBuilder::new(rctx)
///     .with_space_info(space_info)
///     .with_charset_info(charset_info)
///     .build();
/// ```
#[derive(Default)]
pub struct QueryContextBuilder {
    rctx: Option<Arc<QueryRequestContext>>,
    execution_manager: Option<QueryExecutionManager>,
    id_gen: Option<IdGenerator>,
    space_info: Option<SpaceInfo>,
    charset_info: Option<Box<CharsetInfo>>,
}

impl QueryContextBuilder {
    /// Create a new builder.
    pub fn new(rctx: Arc<QueryRequestContext>) -> Self {
        Self {
            rctx: Some(rctx),
            execution_manager: None,
            id_gen: None,
            space_info: None,
            charset_info: None,
        }
    }

    /// Create a builder from session context.
    ///
    /// This simplifies the common pattern of creating a QueryContext
    /// from a ClientSession, automatically extracting space information.
    pub fn from_session(rctx: Arc<QueryRequestContext>, space: Option<SpaceSummary>) -> Self {
        Self {
            rctx: Some(rctx),
            execution_manager: None,
            id_gen: None,
            space_info: space.map(SpaceInfo::from),
            charset_info: None,
        }
    }

    /// Setting up the Execution Manager
    pub fn with_execution_manager(mut self, execution_manager: QueryExecutionManager) -> Self {
        self.execution_manager = Some(execution_manager);
        self
    }

    /// Setting spatial information
    pub fn with_space_info(mut self, space_info: SpaceInfo) -> Self {
        self.space_info = Some(space_info);
        self
    }

    /// Setting character set information
    pub fn with_charset_info(mut self, charset_info: CharsetInfo) -> Self {
        self.charset_info = Some(Box::new(charset_info));
        self
    }

    /// Set the initial value for the ID
    pub fn with_start_id(mut self, start_id: i64) -> Self {
        self.id_gen = Some(IdGenerator::new(start_id));
        self
    }

    /// Constructing the QueryContext
    pub fn build(self) -> QueryContext {
        let rctx = self.rctx.expect("QueryRequestContext is required");
        let execution_manager = self.execution_manager.unwrap_or_default();
        let id_gen = self.id_gen.unwrap_or_else(|| IdGenerator::new(0));
        let space_info = self.space_info;
        let charset_info = self.charset_info;

        QueryContext::from_components(rctx, execution_manager, id_gen, space_info, charset_info)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::{DataType, MetadataVersion, IsolationLevel};
    use std::collections::HashMap;

    #[test]
    fn test_builder_basic() {
        let rctx = Arc::new(QueryRequestContext {
            session_id: None,
            user_name: None,
            space_name: None,
            query: "MATCH (n) RETURN n".to_string(),
            parameters: HashMap::new(),
        });

        let query_context = QueryContextBuilder::new(rctx).build();

        assert_eq!(query_context.query(), "MATCH (n) RETURN n");
    }

    #[test]
    fn test_builder_with_space_info() {
        let rctx = Arc::new(QueryRequestContext {
            session_id: None,
            user_name: None,
            space_name: None,
            query: "MATCH (n) RETURN n".to_string(),
            parameters: HashMap::new(),
        });

        let space_info = SpaceInfo {
            space_id: 1,
            space_name: "test_space".to_string(),
            vid_type: DataType::BigInt,
            tags: Vec::new(),
            edge_types: Vec::new(),
            version: MetadataVersion::default(),
            comment: None,
            storage_path: None,
            isolation_level: IsolationLevel::default(),
            partition_num: 100,
            replica_factor: 1,
            engine_type: EngineType::default(),
            status: SpaceStatus::Online,
        };

        let query_context = QueryContextBuilder::new(rctx)
            .with_space_info(space_info)
            .build();

        assert_eq!(query_context.space_id(), Some(1));
        assert_eq!(query_context.space_name(), Some("test_space".to_string()));
    }

    #[test]
    fn test_builder_with_start_id() {
        let rctx = Arc::new(QueryRequestContext {
            session_id: None,
            user_name: None,
            space_name: None,
            query: "MATCH (n) RETURN n".to_string(),
            parameters: HashMap::new(),
        });

        let query_context = QueryContextBuilder::new(rctx).with_start_id(100).build();

        assert_eq!(query_context.current_id(), 100);
    }

    #[test]
    fn test_builder_chaining() {
        let rctx = Arc::new(QueryRequestContext {
            session_id: None,
            user_name: None,
            space_name: None,
            query: "MATCH (n) RETURN n".to_string(),
            parameters: HashMap::new(),
        });

        let space_info = SpaceInfo {
            space_id: 1,
            space_name: "test_space".to_string(),
            vid_type: DataType::BigInt,
            tags: Vec::new(),
            edge_types: Vec::new(),
            version: MetadataVersion::default(),
            comment: None,
            storage_path: None,
            isolation_level: IsolationLevel::default(),
            partition_num: 100,
            replica_factor: 1,
            engine_type: EngineType::default(),
            status: SpaceStatus::Online,
        };

        let query_context = QueryContextBuilder::new(rctx)
            .with_space_info(space_info)
            .with_start_id(100)
            .build();

        assert_eq!(query_context.space_id(), Some(1));
        assert_eq!(query_context.current_id(), 100);
    }
}
