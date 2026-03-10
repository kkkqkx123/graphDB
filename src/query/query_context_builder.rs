//! QueryContext 构建器
//!
//! 提供流式 API 来构建 QueryContext，简化复杂对象的创建过程。

use crate::core::types::CharsetInfo;
use crate::core::types::SpaceInfo;
use crate::query::context::{QueryExecutionManager, QueryResourceContext, QuerySpaceContext};
use crate::query::query_request_context::QueryRequestContext;
use crate::query::QueryContext;
use std::sync::Arc;

/// QueryContext 构建器
///
/// 提供流式 API 来构建 QueryContext，简化复杂对象的创建过程。
///
/// # 示例
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
pub struct QueryContextBuilder {
    rctx: Option<Arc<QueryRequestContext>>,
    execution_manager: Option<QueryExecutionManager>,
    resource_context: Option<QueryResourceContext>,
    space_context: Option<QuerySpaceContext>,
}

impl QueryContextBuilder {
    /// 创建新的构建器
    pub fn new(rctx: Arc<QueryRequestContext>) -> Self {
        Self {
            rctx: Some(rctx),
            execution_manager: None,
            resource_context: None,
            space_context: None,
        }
    }

    /// 设置执行管理器
    pub fn with_execution_manager(mut self, execution_manager: QueryExecutionManager) -> Self {
        self.execution_manager = Some(execution_manager);
        self
    }

    /// 设置资源上下文
    pub fn with_resource_context(mut self, resource_context: QueryResourceContext) -> Self {
        self.resource_context = Some(resource_context);
        self
    }

    /// 设置空间上下文
    pub fn with_space_context(mut self, space_context: QuerySpaceContext) -> Self {
        self.space_context = Some(space_context);
        self
    }

    /// 设置空间信息
    pub fn with_space_info(mut self, space_info: SpaceInfo) -> Self {
        let mut space_context = self.space_context.unwrap_or_default();
        space_context.set_space_info(space_info);
        self.space_context = Some(space_context);
        self
    }

    /// 设置字符集信息
    pub fn with_charset_info(mut self, charset_info: CharsetInfo) -> Self {
        let mut space_context = self.space_context.unwrap_or_default();
        space_context.set_charset_info(charset_info);
        self.space_context = Some(space_context);
        self
    }

    /// 设置 ID 起始值
    pub fn with_start_id(mut self, start_id: i64) -> Self {
        self.resource_context = Some(QueryResourceContext::with_config(start_id));
        self
    }

    /// 构建 QueryContext
    pub fn build(self) -> QueryContext {
        let rctx = self.rctx.expect("QueryRequestContext is required");
        let execution_manager = self.execution_manager.unwrap_or_default();
        let resource_context = self.resource_context.unwrap_or_default();
        let space_context = self.space_context.unwrap_or_default();

        QueryContext::from_components(rctx, execution_manager, resource_context, space_context)
    }
}

impl Default for QueryContextBuilder {
    fn default() -> Self {
        Self {
            rctx: None,
            execution_manager: None,
            resource_context: None,
            space_context: None,
        }
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
