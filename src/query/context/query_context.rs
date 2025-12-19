//! 查询上下文 - 核心查询上下文
//!
//! 统一的查询上下文，包含查询的所有核心信息
//! 对应原C++中的QueryContext.h/cpp

use crate::core::Value;
use std::result::Result;
use crate::query::context::managers::{
    IndexManager, MetaClient, SchemaManager, StorageClient, SpaceInfo,
};
use std::collections::HashMap;
use std::sync::Arc;

/// 查询统计信息
#[derive(Debug, Clone, Default)]
pub struct QueryStatistics {
    /// 执行时间（毫秒）
    pub execution_time_ms: u64,
    /// 扫描的节点数
    pub scanned_nodes: u64,
    /// 扫描的边数
    pub scanned_edges: u64,
    /// 返回的行数
    pub returned_rows: u64,
    /// 影响的行数
    pub affected_rows: u64,
}

/// 函数接口
pub trait Function: Send + Sync + std::fmt::Debug {
    /// 函数名称
    fn name(&self) -> &str;
    /// 调用函数
    fn call(&self, args: Vec<Value>) -> Result<Value, String>;
}

/// 统一的查询上下文，包含查询的所有核心信息
#[derive(Debug)]
pub struct QueryContext {
    // 会话信息
    pub session_id: String,
    pub user_id: String,
    pub space_id: Option<i32>,

    // Schema管理器
    pub schema_manager: Arc<dyn SchemaManager>,
    pub index_manager: Arc<dyn IndexManager>,
    pub meta_client: Arc<dyn MetaClient>,
    pub storage_client: Arc<dyn StorageClient>,

    // 查询状态
    variables: HashMap<String, Value>,
    parameters: HashMap<String, Value>,
    functions: HashMap<String, Box<dyn Function>>,

    // 统计信息
    pub statistics: QueryStatistics,
}

impl QueryContext {
    /// 创建新的查询上下文
    pub fn new(
        session_id: String,
        user_id: String,
        schema_manager: Arc<dyn SchemaManager>,
        index_manager: Arc<dyn IndexManager>,
        meta_client: Arc<dyn MetaClient>,
        storage_client: Arc<dyn StorageClient>,
    ) -> Self {
        Self {
            session_id,
            user_id,
            space_id: None,
            schema_manager,
            index_manager,
            meta_client,
            storage_client,
            variables: HashMap::new(),
            parameters: HashMap::new(),
            functions: HashMap::new(),
            statistics: QueryStatistics::default(),
        }
    }

    /// 获取变量值
    pub fn get_variable(&self, name: &str) -> Option<&Value> {
        self.variables.get(name)
    }

    /// 设置变量值
    pub fn set_variable(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }

    /// 删除变量
    pub fn remove_variable(&mut self, name: &str) -> Option<Value> {
        self.variables.remove(name)
    }

    /// 检查变量是否存在
    pub fn has_variable(&self, name: &str) -> bool {
        self.variables.contains_key(name)
    }

    /// 获取所有变量名
    pub fn variable_names(&self) -> Vec<String> {
        self.variables.keys().cloned().collect()
    }

    /// 获取参数值
    pub fn get_parameter(&self, name: &str) -> Option<&Value> {
        self.parameters.get(name)
    }

    /// 设置参数值
    pub fn set_parameter(&mut self, name: String, value: Value) {
        self.parameters.insert(name, value);
    }

    /// 删除参数
    pub fn remove_parameter(&mut self, name: &str) -> Option<Value> {
        self.parameters.remove(name)
    }

    /// 检查参数是否存在
    pub fn has_parameter(&self, name: &str) -> bool {
        self.parameters.contains_key(name)
    }

    /// 获取所有参数名
    pub fn parameter_names(&self) -> Vec<String> {
        self.parameters.keys().cloned().collect()
    }

    /// 注册函数
    pub fn register_function(&mut self, name: String, function: Box<dyn Function>) {
        self.functions.insert(name, function);
    }

    /// 获取函数
    pub fn get_function(&self, name: &str) -> Option<&dyn Function> {
        self.functions.get(name).map(|f| f.as_ref())
    }

    /// 获取空间信息
    pub fn space(&self) -> Option<SpaceInfo> {
        self.space_id.and_then(|id| self.meta_client.get_space_info(id).ok())
    }

    /// 设置当前空间
    pub fn set_space(&mut self, space_id: i32) {
        self.space_id = Some(space_id);
    }

    /// 清除所有变量
    pub fn clear_variables(&mut self) {
        self.variables.clear();
    }

    /// 清除所有参数
    pub fn clear_parameters(&mut self) {
        self.parameters.clear();
    }

    /// 重置统计信息
    pub fn reset_statistics(&mut self) {
        self.statistics = QueryStatistics::default();
    }

    /// 更新执行时间
    pub fn update_execution_time(&mut self, time_ms: u64) {
        self.statistics.execution_time_ms = time_ms;
    }

    /// 增加扫描节点数
    pub fn add_scanned_nodes(&mut self, count: u64) {
        self.statistics.scanned_nodes += count;
    }

    /// 增加扫描边数
    pub fn add_scanned_edges(&mut self, count: u64) {
        self.statistics.scanned_edges += count;
    }

    /// 设置返回行数
    pub fn set_returned_rows(&mut self, count: u64) {
        self.statistics.returned_rows = count;
    }

    /// 设置影响行数
    pub fn set_affected_rows(&mut self, count: u64) {
        self.statistics.affected_rows = count;
    }
}

impl Clone for QueryContext {
    fn clone(&self) -> Self {
        Self {
            session_id: self.session_id.clone(),
            user_id: self.user_id.clone(),
            space_id: self.space_id,
            schema_manager: self.schema_manager.clone(),
            index_manager: self.index_manager.clone(),
            meta_client: self.meta_client.clone(),
            storage_client: self.storage_client.clone(),
            variables: self.variables.clone(),
            parameters: self.parameters.clone(),
            functions: self.functions.clone(),
            statistics: self.statistics.clone(),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::context::managers::r#impl::{
        MockIndexManager, MockMetaClient, MockSchemaManager, MockStorageClient,
    };

    #[test]
    fn test_query_context_creation() {
        let schema_manager = Arc::new(MockSchemaManager::new());
        let index_manager = Arc::new(MockIndexManager::new());
        let meta_client = Arc::new(MockMetaClient::new());
        let storage_client = Arc::new(MockStorageClient::new());

        let ctx = QueryContext::new(
            "session123".to_string(),
            "user456".to_string(),
            schema_manager,
            index_manager,
            meta_client,
            storage_client,
        );

        assert_eq!(ctx.session_id, "session123");
        assert_eq!(ctx.user_id, "user456");
        assert!(ctx.space_id.is_none());
        assert_eq!(ctx.variables.len(), 0);
        assert_eq!(ctx.parameters.len(), 0);
    }

    #[test]
    fn test_variable_management() {
        let schema_manager = Arc::new(MockSchemaManager::new());
        let index_manager = Arc::new(MockIndexManager::new());
        let meta_client = Arc::new(MockMetaClient::new());
        let storage_client = Arc::new(MockStorageClient::new());

        let mut ctx = QueryContext::new(
            "session123".to_string(),
            "user456".to_string(),
            schema_manager,
            index_manager,
            meta_client,
            storage_client,
        );

        // 设置变量
        ctx.set_variable("x".to_string(), Value::Int(42));
        assert_eq!(ctx.get_variable("x"), Some(&Value::Int(42)));
        assert!(ctx.has_variable("x"));
        assert!(ctx.variable_names().contains(&"x".to_string()));

        // 删除变量
        let removed = ctx.remove_variable("x");
        assert_eq!(removed, Some(Value::Int(42)));
        assert!(!ctx.has_variable("x"));
    }

    #[test]
    fn test_parameter_management() {
        let schema_manager = Arc::new(MockSchemaManager::new());
        let index_manager = Arc::new(MockIndexManager::new());
        let meta_client = Arc::new(MockMetaClient::new());
        let storage_client = Arc::new(MockStorageClient::new());

        let mut ctx = QueryContext::new(
            "session123".to_string(),
            "user456".to_string(),
            schema_manager,
            index_manager,
            meta_client,
            storage_client,
        );

        // 设置参数
        ctx.set_parameter("name".to_string(), Value::String("Alice".to_string()));
        assert_eq!(ctx.get_parameter("name"), Some(&Value::String("Alice".to_string())));
        assert!(ctx.has_parameter("name"));
        assert!(ctx.parameter_names().contains(&"name".to_string()));

        // 删除参数
        let removed = ctx.remove_parameter("name");
        assert_eq!(removed, Some(Value::String("Alice".to_string())));
        assert!(!ctx.has_parameter("name"));
    }

    #[test]
    fn test_statistics() {
        let schema_manager = Arc::new(MockSchemaManager::new());
        let index_manager = Arc::new(MockIndexManager::new());
        let meta_client = Arc::new(MockMetaClient::new());
        let storage_client = Arc::new(MockStorageClient::new());

        let mut ctx = QueryContext::new(
            "session123".to_string(),
            "user456".to_string(),
            schema_manager,
            index_manager,
            meta_client,
            storage_client,
        );

        // 更新统计信息
        ctx.update_execution_time(1000);
        ctx.add_scanned_nodes(100);
        ctx.add_scanned_edges(50);
        ctx.set_returned_rows(10);
        ctx.set_affected_rows(5);

        assert_eq!(ctx.statistics.execution_time_ms, 1000);
        assert_eq!(ctx.statistics.scanned_nodes, 100);
        assert_eq!(ctx.statistics.scanned_edges, 50);
        assert_eq!(ctx.statistics.returned_rows, 10);
        assert_eq!(ctx.statistics.affected_rows, 5);

        // 重置统计信息
        ctx.reset_statistics();
        assert_eq!(ctx.statistics.execution_time_ms, 0);
        assert_eq!(ctx.statistics.scanned_nodes, 0);
        assert_eq!(ctx.statistics.scanned_edges, 0);
        assert_eq!(ctx.statistics.returned_rows, 0);
        assert_eq!(ctx.statistics.affected_rows, 0);
    }

    #[derive(Debug)]
    struct TestFunction {
        name: String,
    }

    impl TestFunction {
        fn new(name: String) -> Self {
            Self { name }
        }
    }

    impl Function for TestFunction {
        fn name(&self) -> &str {
            &self.name
        }

        fn call(&self, args: Vec<Value>) -> Result<Value, String> {
            Ok(Value::Int(args.len() as i64))
        }
    }

    #[test]
    fn test_function_management() {
        let schema_manager = Arc::new(MockSchemaManager::new());
        let index_manager = Arc::new(MockIndexManager::new());
        let meta_client = Arc::new(MockMetaClient::new());
        let storage_client = Arc::new(MockStorageClient::new());

        let mut ctx = QueryContext::new(
            "session123".to_string(),
            "user456".to_string(),
            schema_manager,
            index_manager,
            meta_client,
            storage_client,
        );

        // 注册函数
        let test_func = TestFunction::new("count".to_string());
        ctx.register_function("count".to_string(), Box::new(test_func));

        // 获取函数
        let func = ctx.get_function("count").unwrap();
        assert_eq!(func.name(), "count");

        // 调用函数
        let result = func.call(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        assert_eq!(result, Ok(Value::Int(3)));
    }
}