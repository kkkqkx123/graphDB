//! HTTP 服务器
//!
//! 提供基于 HTTP 的 GraphDB 服务接口

use crate::api::core::{QueryApi, SchemaApi};
use crate::api::server::auth::PasswordAuthenticator;
use crate::api::server::batch::BatchManager;
use crate::api::server::graph_service::GraphService;
use crate::api::server::session::GraphSessionManager;
use crate::config::Config;
use crate::query::executor::expression::functions::FunctionRegistry;
use crate::storage::StorageClient;
use crate::transaction::TransactionManager;
use parking_lot::Mutex;
use std::sync::Arc;

/// HTTP 服务器
///
/// 注意：HttpServer 依赖 GraphService 获取权限管理器和统计管理器
/// 会话管理器通过 GraphService 访问
pub struct HttpServer<S: StorageClient + Clone + 'static> {
    graph_service: Arc<GraphService<S>>,
    query_api: QueryApi<S>,
    txn_manager: Arc<TransactionManager>,
    schema_api: SchemaApi<S>,
    auth_service: PasswordAuthenticator,
    batch_manager: Arc<BatchManager<S>>,
    storage: Arc<Mutex<S>>,
    config: Config,
    function_registry: Arc<Mutex<FunctionRegistry>>,
}

impl<S: StorageClient + Clone + 'static> HttpServer<S> {
    /// 创建新的 HTTP 服务器
    pub fn new(
        graph_service: Arc<GraphService<S>>,
        storage: Arc<Mutex<S>>,
        txn_manager: Arc<TransactionManager>,
        config: &Config,
    ) -> Self {
        Self {
            graph_service: graph_service.clone(),
            query_api: QueryApi::new(storage.clone()),
            txn_manager,
            schema_api: SchemaApi::new(storage.clone()),
            auth_service: PasswordAuthenticator::new_default(config.auth.clone()),
            batch_manager: Arc::new(BatchManager::new(storage.clone())),
            storage: storage.clone(),
            config: config.clone(),
            function_registry: Arc::new(Mutex::new(FunctionRegistry::new())),
        }
    }

    /// 获取 GraphService
    pub fn get_graph_service(&self) -> Arc<GraphService<S>> {
        self.graph_service.clone()
    }

    /// 获取会话管理器（通过 GraphService）
    pub fn get_session_manager(&self) -> &GraphSessionManager {
        self.graph_service.get_session_manager()
    }

    /// 获取查询 API
    pub fn get_query_api(&self) -> &QueryApi<S> {
        &self.query_api
    }

    /// 获取事务管理器
    pub fn get_txn_manager(&self) -> Arc<TransactionManager> {
        self.txn_manager.clone()
    }

    /// 获取 Schema API
    pub fn get_schema_api(&self) -> &SchemaApi<S> {
        &self.schema_api
    }

    /// 获取认证服务
    pub fn get_auth_service(&self) -> &PasswordAuthenticator {
        &self.auth_service
    }

    /// 获取批量任务管理器
    pub fn get_batch_manager(&self) -> Arc<BatchManager<S>> {
        self.batch_manager.clone()
    }

    /// 获取统计管理器（通过 GraphService）
    pub fn get_stats_manager(&self) -> &Arc<crate::core::StatsManager> {
        self.graph_service.get_stats_manager()
    }

    /// 获取存储客户端
    pub fn get_storage(&self) -> Arc<Mutex<S>> {
        self.storage.clone()
    }

    /// 获取配置
    pub fn get_config(&self) -> &Config {
        &self.config
    }

    /// 获取配置的可变引用
    pub fn get_config_mut(&mut self) -> &mut Config {
        &mut self.config
    }

    /// 获取函数注册表
    pub fn get_function_registry(&self) -> Arc<Mutex<FunctionRegistry>> {
        self.function_registry.clone()
    }
}
