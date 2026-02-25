//! HTTP 服务器
//!
//! 提供基于 HTTP 的 GraphDB 服务接口

use crate::api::core::{QueryApi, TransactionApi, SchemaApi};
use crate::api::server::auth::PasswordAuthenticator;
use crate::api::server::session::GraphSessionManager;
use crate::api::server::permission::PermissionManager;
use crate::api::server::graph_service::GraphService;
use crate::core::StatsManager;
use crate::storage::StorageClient;
use crate::transaction::TransactionManager;
use crate::config::Config;
use std::sync::Arc;
use std::time::Duration;
use parking_lot::Mutex;

/// HTTP 服务器
pub struct HttpServer<S: StorageClient + Clone + 'static> {
    graph_service: Arc<GraphService<S>>,
    query_api: QueryApi<S>,
    txn_api: TransactionApi,
    schema_api: SchemaApi<S>,
    auth_service: PasswordAuthenticator,
    session_manager: Arc<GraphSessionManager>,
    permission_manager: Arc<PermissionManager>,
    stats_manager: Arc<StatsManager>,
}

impl<S: StorageClient + Clone + 'static> HttpServer<S> {
    /// 创建新的 HTTP 服务器
    pub fn new(
        graph_service: Arc<GraphService<S>>,
        storage: Arc<Mutex<S>>,
        txn_manager: Arc<TransactionManager>,
        config: &Config,
    ) -> Self {
        let _session_idle_timeout = Duration::from_secs(config.transaction.default_timeout * 10);
        
        Self {
            graph_service: graph_service.clone(),
            query_api: QueryApi::new(storage.clone()),
            txn_api: TransactionApi::new(txn_manager.clone()),
            schema_api: SchemaApi::new(storage.clone()),
            auth_service: PasswordAuthenticator::new_default(config.auth.clone()),
            session_manager: Arc::clone(&graph_service.get_session_manager()),
            permission_manager: Arc::clone(&graph_service.get_permission_manager()),
            stats_manager: Arc::clone(&graph_service.stats_manager),
        }
    }

    /// 获取 GraphService
    pub fn get_graph_service(&self) -> &GraphService<S> {
        &self.graph_service
    }

    /// 获取会话管理器
    pub fn get_session_manager(&self) -> &GraphSessionManager {
        &self.session_manager
    }

    /// 获取统计管理器
    pub fn get_stats_manager(&self) -> &StatsManager {
        &self.stats_manager
    }

    /// 获取查询 API
    pub fn get_query_api(&self) -> &QueryApi<S> {
        &self.query_api
    }

    /// 获取事务 API
    pub fn get_txn_api(&self) -> &TransactionApi {
        &self.txn_api
    }

    /// 获取 Schema API
    pub fn get_schema_api(&self) -> &SchemaApi<S> {
        &self.schema_api
    }

    /// 获取认证服务
    pub fn get_auth_service(&self) -> &PasswordAuthenticator {
        &self.auth_service
    }

    /// 获取权限管理器
    pub fn get_permission_manager(&self) -> &PermissionManager {
        &self.permission_manager
    }
}
