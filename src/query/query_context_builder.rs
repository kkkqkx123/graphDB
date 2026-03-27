//! QueryContext Builder
//!
//! Provide a streaming API for building QueryContext, which simplifies the process of creating complex objects.

use crate::core::types::{CharsetInfo, SpaceInfo};
use crate::query::context::QueryExecutionManager;
use crate::query::query_request_context::QueryRequestContext;
use crate::query::QueryContext;
use crate::utils::IdGenerator;
use std::sync::Arc;

/// QueryContext Builder
///
/// A streaming API is provided to build the QueryContext, simplifying the process of creating complex objects.
///
/// # Example
///
/// ```rust
/// use crate::query::query_context_builder::QueryContextBuilder;
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
    use crate::query::query_request_context::QueryRequestContext;
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
            vid_type: crate::core::types::DataType::Int64,
            tags: Vec::new(),
            edge_types: Vec::new(),
            version: crate::core::types::MetadataVersion::default(),
            comment: None,
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
            vid_type: crate::core::types::DataType::Int64,
            tags: Vec::new(),
            edge_types: Vec::new(),
            version: crate::core::types::MetadataVersion::default(),
            comment: None,
        };

        let query_context = QueryContextBuilder::new(rctx)
            .with_space_info(space_info)
            .with_start_id(100)
            .build();

        assert_eq!(query_context.space_id(), Some(1));
        assert_eq!(query_context.current_id(), 100);
    }
}
