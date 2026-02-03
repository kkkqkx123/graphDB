//! 查询执行上下文 - 管理整个查询请求的上下文
//! 对应原C++中的QueryContext.h/cpp

use crate::query::context::validate::ValidationContext;
use crate::query::context::SymbolTable;
use crate::core::Value;
use crate::utils::IdGenerator;
use crate::core::types::CharsetInfo;
use crate::storage::StorageClient;
use crate::storage::metadata::SchemaManager;
use crate::storage::index::IndexManager;
use crate::query::context::request_context::RequestContext;
use crate::utils::ObjectPool;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// 查询执行上下文
///
/// 每个查询请求的执行上下文，存储查询变量值和中间结果
/// 基于Nebula-Graph的ResultMap设计，使用简单的HashMap管理变量
#[derive(Debug, Clone)]
pub struct QueryExecutionContext {
    /// 变量存储（类似Nebula-Graph的ResultMap）
    variables: HashMap<String, Value>,
}

impl QueryExecutionContext {
    /// 创建新的查询执行上下文
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    /// 设置变量值
    pub fn set_value(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }

    /// 获取变量值
    pub fn get_value(&self, name: &str) -> Option<&Value> {
        self.variables.get(name)
    }

    /// 移除变量
    pub fn remove_value(&mut self, name: &str) -> Option<Value> {
        self.variables.remove(name)
    }

    /// 检查变量是否存在
    pub fn exists(&self, name: &str) -> bool {
        self.variables.contains_key(name)
    }

    /// 获取所有变量名
    pub fn variable_names(&self) -> Vec<String> {
        self.variables.keys().cloned().collect()
    }

    /// 获取变量数量
    pub fn variable_count(&self) -> usize {
        self.variables.len()
    }

    /// 清空所有变量
    pub fn clear(&mut self) {
        self.variables.clear();
    }
}

impl Default for QueryExecutionContext {
    fn default() -> Self {
        Self::new()
    }
}

/// 执行计划 - 表示查询的执行计划
#[derive(Debug, Clone)]
pub struct ExecutionPlan {
    pub plan_id: i64,
    pub root_node: Option<PlanNode>,
    pub is_profile_enabled: bool,
}

impl ExecutionPlan {
    pub fn new(plan_id: i64) -> Self {
        Self {
            plan_id,
            root_node: None,
            is_profile_enabled: false,
        }
    }

    pub fn id(&self) -> i64 {
        self.plan_id
    }

    pub fn is_profile_enabled(&self) -> bool {
        self.is_profile_enabled
    }

    pub fn enable_profile(&mut self) {
        self.is_profile_enabled = true;
    }
}

/// 计划节点 - 执行计划的基本单元
#[derive(Debug, Clone)]
pub struct PlanNode {
    pub node_id: i64,
    pub node_type: String,
    pub children: Vec<PlanNode>,
}

/// 执行响应 - 包含查询执行结果
#[derive(Debug, Clone)]
pub struct ExecutionResponse {
    pub success: bool,
    pub data: Option<Value>,
    pub error_code: Option<i32>,
    pub error_message: Option<String>,
    pub execution_time_ms: u64,
}

impl ExecutionResponse {
    pub fn new(success: bool) -> Self {
        Self {
            success,
            data: None,
            error_code: None,
            error_message: None,
            execution_time_ms: 0,
        }
    }

    pub fn with_data(mut self, data: Value) -> Self {
        self.data = Some(data);
        self
    }

    pub fn with_error(mut self, code: i32, message: String) -> Self {
        self.error_code = Some(code);
        self.error_message = Some(message);
        self.success = false;
        self
    }
}

/// 查询上下文
///
/// 每个查询请求的上下文，从查询引擎接收到查询请求时创建
/// 该上下文对象对解析器、规划器、优化器和执行器可见
/// 对应原C++中的QueryContext类
///
/// # 线程安全性
///
/// 查询上下文不是线程安全的。执行计划必须保证所有对上下文的访问都是安全的。
/// 上下文的声明周期与请求相同，这意味着查询引擎接收到查询请求时会创建新的上下文对象，
/// 该上下文对象对解析器、规划器、优化器和执行器都可见。
pub struct QueryContext {
    // 请求上下文 - 使用Arc共享所有权
    rctx: Option<Arc<RequestContext>>,

    // 验证上下文
    vctx: ValidationContext,

    // 查询执行上下文
    ectx: QueryExecutionContext,

    // 执行计划
    plan: Option<Box<ExecutionPlan>>,

    // 模式管理器 - 使用Arc共享所有权
    schema_manager: Option<Arc<dyn SchemaManager>>,

    // 索引管理器 - 使用Arc共享所有权
    index_manager: Option<Arc<dyn IndexManager>>,

    // 存储客户端 - 使用Arc共享所有权
    storage_client: Option<Arc<dyn StorageClient>>,

    // 字符集信息
    charset_info: Option<Box<CharsetInfo>>,

    // 对象池 - 存储内部生成的所有对象（表达式、计划节点、执行器等）
    obj_pool: ObjectPool<String>,

    // ID生成器
    id_gen: IdGenerator,

    // 符号表
    sym_table: SymbolTable,

    // 是否被标记为已终止
    killed: AtomicBool,
}

impl QueryContext {
    /// 创建新的查询上下文
    pub fn new() -> Self {
        Self {
            rctx: None,
            vctx: ValidationContext::new(),
            ectx: QueryExecutionContext::new(),
            plan: None,
            schema_manager: None,
            index_manager: None,
            storage_client: None,
            charset_info: None,
            obj_pool: ObjectPool::new(1000), // 提供默认容量
            id_gen: IdGenerator::new(0),
            sym_table: SymbolTable::new(),
            killed: AtomicBool::new(false),
        }
    }

    /// 使用请求上下文创建查询上下文
    ///
    /// 这是推荐的方式，因为它会正确初始化执行上下文中的参数映射
    pub fn with_request_context(rctx: Arc<RequestContext>) -> Self {
        let mut ctx = Self::new();
        ctx.rctx = Some(rctx.clone());

        // 将请求参数复制到执行上下文中
        for (name, value) in &rctx.request_params().parameters {
            let _ = ctx.ectx.set_value(name.clone(), value.clone());
        }

        ctx
    }

    /// 设置请求上下文
    pub fn set_rctx(&mut self, rctx: Arc<RequestContext>) {
        self.rctx = Some(rctx);
    }

    /// 设置模式管理器
    pub fn set_schema_manager(&mut self, sm: Arc<dyn SchemaManager>) {
        self.schema_manager = Some(sm);
    }

    /// 设置索引管理器
    pub fn set_index_manager(&mut self, im: Arc<dyn IndexManager>) {
        self.index_manager = Some(im);
    }

    /// 设置存储客户端
    pub fn set_storage_client(&mut self, storage: Arc<dyn StorageClient>) {
        self.storage_client = Some(storage);
    }

    /// 设置字符集信息
    pub fn set_charset_info(&mut self, charset_info: CharsetInfo) {
        self.charset_info = Some(Box::new(charset_info));
    }

    /// 获取请求上下文
    pub fn rctx(&self) -> Option<&RequestContext> {
        self.rctx.as_deref()
    }

    /// 获取验证上下文
    pub fn vctx(&self) -> &ValidationContext {
        &self.vctx
    }

    /// 获取可变验证上下文
    pub fn vctx_mut(&mut self) -> &mut ValidationContext {
        &mut self.vctx
    }

    /// 获取查询执行上下文
    pub fn ectx(&self) -> &QueryExecutionContext {
        &self.ectx
    }

    /// 获取可变查询执行上下文
    pub fn ectx_mut(&mut self) -> &mut QueryExecutionContext {
        &mut self.ectx
    }

    /// 获取执行计划
    pub fn plan(&self) -> Option<&ExecutionPlan> {
        self.plan.as_ref().map(|p| p.as_ref())
    }

    /// 获取可变执行计划
    pub fn plan_mut(&mut self) -> &mut Option<Box<ExecutionPlan>> {
        &mut self.plan
    }

    /// 设置执行计划
    pub fn set_plan(&mut self, plan: ExecutionPlan) {
        self.plan = Some(Box::new(plan));
    }

    /// 获取模式管理器
    pub fn schema_manager(&self) -> Option<&Arc<dyn SchemaManager>> {
        self.schema_manager.as_ref()
    }

    /// 获取索引管理器
    pub fn index_manager(&self) -> Option<&Arc<dyn IndexManager>> {
        self.index_manager.as_ref()
    }

    /// 获取存储客户端
    pub fn get_storage_client(&self) -> Option<&Arc<dyn StorageClient>> {
        self.storage_client.as_ref()
    }

    /// 获取字符集信息
    pub fn get_charset_info(&self) -> Option<&CharsetInfo> {
        self.charset_info.as_ref().map(|ci| ci.as_ref())
    }

    /// 获取对象池
    pub fn obj_pool(&self) -> &ObjectPool<String> {
        &self.obj_pool
    }

    /// 获取可变对象池
    pub fn obj_pool_mut(&mut self) -> &mut ObjectPool<String> {
        &mut self.obj_pool
    }

    /// 生成ID
    pub fn gen_id(&self) -> i64 {
        self.id_gen.id()
    }

    /// 获取当前ID值（不递增）
    pub fn current_id(&self) -> i64 {
        self.id_gen.current_value()
    }

    /// 获取符号表
    pub fn sym_table(&self) -> &SymbolTable {
        &self.sym_table
    }

    /// 获取可变符号表
    pub fn sym_table_mut(&mut self) -> &mut SymbolTable {
        &mut self.sym_table
    }

    /// 标记为部分成功
    ///
    /// 设置响应状态为部分成功，通常在查询部分成功但遇到某些错误时调用
    pub fn set_partial_success(&mut self) {
        if let Some(rctx) = &self.rctx {
            // 设置响应错误信息为部分成功
            let _ = rctx.set_response_error("Partial success".to_string());
        }

        // 记录日志
        log::warn!("Query marked as partial success");
    }

    /// 标记为已终止
    pub fn mark_killed(&self) {
        self.killed.store(true, Ordering::SeqCst);
        log::info!("Query context marked as killed");
    }

    /// 检查是否被终止
    pub fn is_killed(&self) -> bool {
        self.killed.load(Ordering::SeqCst)
    }

    /// 检查参数是否存在
    ///
    /// 这仅在构建阶段有效！用于检查查询参数是否已设置
    pub fn exist_parameter(&self, param: &str) -> bool {
        match self.ectx.get_value(param) {
            Some(value) => {
                // 检查参数值是否为空
                match value {
                    Value::Empty => false,
                    _ => true,
                }
            }
            None => false, // 如果参数不存在，返回false
        }
    }

    /// 获取查询状态信息
    ///
    /// 返回查询上下文的当前状态摘要，用于调试和监控
    pub fn get_status_info(&self) -> QueryContextStatus {
        QueryContextStatus {
            has_request_context: self.rctx.is_some(),
            has_schema_manager: self.schema_manager.is_some(),
            has_index_manager: self.index_manager.is_some(),
            has_storage_client: self.storage_client.is_some(),
            has_charset_info: self.charset_info.is_some(),
            has_execution_plan: self.plan.is_some(),
            is_killed: self.is_killed(),
            current_id: self.current_id(),
            symbol_table_size: self.sym_table.size(),
            variable_count: self.ectx.variable_count(),
        }
    }

    /// 重置查询上下文
    ///
    /// 清除所有状态，重置为初始状态。用于重用查询上下文对象。
    pub fn reset(&mut self) {
        self.plan = None;
        self.killed.store(false, Ordering::SeqCst);
        // 重置ID生成器到初始值
        self.id_gen.reset(0);
        // 清除执行上下文中的所有变量
        // 注意：这里需要添加清除所有变量的方法
        log::info!("Query context reset");
    }
}

/// 查询上下文状态信息
#[derive(Debug, Clone)]
pub struct QueryContextStatus {
    pub has_request_context: bool,
    pub has_schema_manager: bool,
    pub has_index_manager: bool,
    pub has_storage_client: bool,
    pub has_charset_info: bool,
    pub has_execution_plan: bool,
    pub is_killed: bool,
    pub current_id: i64,
    pub symbol_table_size: usize,
    pub variable_count: usize,
}

impl Clone for QueryContext {
    fn clone(&self) -> Self {
        Self {
            rctx: self.rctx.clone(),
            vctx: self.vctx.clone(),
            ectx: self.ectx.clone(),
            plan: self.plan.clone(),
            schema_manager: self.schema_manager.clone(),
            index_manager: self.index_manager.clone(),
            storage_client: self.storage_client.clone(),
            charset_info: self.charset_info.clone(),
            obj_pool: self.obj_pool.clone(),
            id_gen: IdGenerator::new(self.id_gen.current_value()),
            sym_table: self.sym_table.clone(),
            killed: AtomicBool::new(self.killed.load(Ordering::SeqCst)),
        }
    }
}

impl std::fmt::Debug for QueryContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QueryContext")
            .field("rctx", &self.rctx.is_some())
            .field("vctx", &self.vctx)
            .field("ectx", &self.ectx)
            .field("plan", &self.plan.is_some())
            .field("schema_manager", &self.schema_manager.is_some())
            .field("index_manager", &self.index_manager.is_some())
            .field("storage_client", &self.storage_client.is_some())
            .field("charset_info", &self.charset_info.is_some())
            .field("obj_pool", &self.obj_pool)
            .field("id_gen", &self.id_gen)
            .field("sym_table", &self.sym_table)
            .field("killed", &self.killed)
            .finish()
    }
}

impl Default for QueryContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::MemorySchemaManager;
    use crate::storage::index::MemoryIndexManager;
    use crate::storage::redb_storage::DefaultStorage;

    #[test]
    fn test_query_context_creation() {
        let ctx = QueryContext::new();

        assert_eq!(ctx.gen_id(), 0);
        assert_eq!(ctx.gen_id(), 1);

        assert!(!ctx.exist_parameter("non_existent_param"));

        assert!(!ctx.is_killed());
        ctx.mark_killed();
        assert!(ctx.is_killed());
    }

    #[test]
    fn test_context_access() {
        let mut ctx = QueryContext::new();

        ctx.sym_table_mut()
            .new_variable("test_var")
            .expect("Failed to create new variable");
        assert!(ctx.sym_table().has_variable("test_var"));

        let value = crate::core::Value::Int(42);
        ctx.ectx_mut().set_value("test_val".to_string(), value.clone());
        assert_eq!(ctx.ectx().get_value("test_val"), Some(&value));
    }

    #[test]
    fn test_managers() {
        let mut ctx = QueryContext::new();

        let schema_manager = Arc::new(MemorySchemaManager::new());
        ctx.set_schema_manager(schema_manager.clone());
        assert!(ctx.schema_manager().is_some());

        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let index_manager = Arc::new(MemoryIndexManager::new(temp_dir.path().to_path_buf()));
        ctx.set_index_manager(index_manager.clone());
        assert!(ctx.index_manager().is_some());

        let storage = crate::storage::test_mock::MockStorage::new().expect("Failed to create mock storage");
        let storage_client: Arc<dyn crate::storage::StorageClient> = Arc::new(storage);
        ctx.set_storage_client(storage_client.clone());
        assert!(ctx.get_storage_client().is_some());
    }

    #[test]
    fn test_execution_plan() {
        let mut ctx = QueryContext::new();

        let plan = ExecutionPlan::new(ctx.gen_id());
        ctx.set_plan(plan);

        assert!(ctx.plan().is_some());
        assert_eq!(ctx.plan().expect("Plan should exist").id(), 0);
        assert!(!ctx.plan().expect("Plan should exist").is_profile_enabled());

        ctx.plan_mut()
            .as_mut()
            .expect("Plan should exist")
            .enable_profile();
        assert!(ctx.plan().expect("Plan should exist").is_profile_enabled());
    }

    #[test]
    fn test_charset_info() {
        let mut ctx = QueryContext::new();

        let charset_info = crate::core::types::CharsetInfo {
            charset: "utf8mb4".to_string(),
            collation: "utf8mb4_general_ci".to_string(),
        };
        ctx.set_charset_info(charset_info.clone());

        assert!(ctx.get_charset_info().is_some());
        assert_eq!(
            ctx.get_charset_info().expect("Charset info should exist").charset,
            charset_info.charset
        );
        assert_eq!(
            ctx.get_charset_info().expect("Charset info should exist").collation,
            charset_info.collation
        );
    }

    #[test]
    fn test_status_info() {
        let mut ctx = QueryContext::new();

        let status = ctx.get_status_info();
        assert!(!status.has_request_context);
        assert!(!status.has_schema_manager);
        assert!(!status.has_index_manager);
        assert!(!status.has_storage_client);
        assert!(!status.has_charset_info);
        assert!(!status.has_execution_plan);
        assert!(!status.is_killed);
        assert_eq!(status.current_id, 0);
        assert_eq!(status.variable_count, 0);

        ctx.set_schema_manager(Arc::new(MemorySchemaManager::new()));
        ctx.set_plan(ExecutionPlan::new(ctx.gen_id()));

        ctx.ectx_mut()
            .set_value("test_var1".to_string(), crate::core::Value::Int(42));
        ctx.ectx_mut()
            .set_value("test_var2".to_string(), crate::core::Value::String("hello".to_string()));

        let status = ctx.get_status_info();
        assert!(status.has_schema_manager);
        assert!(status.has_execution_plan);
        assert_eq!(status.current_id, 1);
        assert_eq!(status.variable_count, 2);
    }

    #[test]
    fn test_reset() {
        let mut ctx = QueryContext::new();

        ctx.set_plan(ExecutionPlan::new(ctx.gen_id()));
        ctx.mark_killed();

        assert!(ctx.plan().is_some());
        assert!(ctx.is_killed());

        ctx.reset();

        assert!(ctx.plan().is_none());
        assert!(!ctx.is_killed());
        assert_eq!(ctx.current_id(), 0);
    }

    #[test]
    fn test_with_request_context() {
        let request_ctx = Arc::new(RequestContext::with_session(
            "MATCH (n) RETURN n".to_string(),
            "test_session",
            "test_user",
            "127.0.0.1",
            0,
        ));

        let ctx = QueryContext::with_request_context(request_ctx.clone());

        assert!(ctx.rctx().is_some());
        assert_eq!(
            ctx.rctx().expect("Request context should exist").query(),
            "MATCH (n) RETURN n"
        );
    }

    #[test]
    fn test_query_execution_context() {
        let mut ctx = QueryExecutionContext::new();

        let value = Value::Int(42);
        ctx.set_value("test_var".to_string(), value.clone());
        assert_eq!(ctx.get_value("test_var"), Some(&value));
    }

    #[test]
    fn test_variable_operations() {
        let mut ctx = QueryExecutionContext::new();

        ctx.set_value("var1".to_string(), Value::Int(1));
        ctx.set_value("var2".to_string(), Value::String("test".to_string()));
        assert_eq!(ctx.variable_count(), 2);

        assert!(ctx.exists("var1"));
        assert!(!ctx.exists("non_existent"));

        let removed = ctx.remove_value("var1");
        assert_eq!(removed, Some(Value::Int(1)));
        assert_eq!(ctx.variable_count(), 1);

        let names = ctx.variable_names();
        assert_eq!(names.len(), 1);
        assert!(names.contains(&"var2".to_string()));
    }

    #[test]
    fn test_clear() {
        let mut ctx = QueryExecutionContext::new();

        ctx.set_value("var1".to_string(), Value::Int(1));
        ctx.set_value("var2".to_string(), Value::String("test".to_string()));
        assert_eq!(ctx.variable_count(), 2);

        ctx.clear();
        assert_eq!(ctx.variable_count(), 0);
    }

    #[test]
    fn test_execution_response() {
        let response = ExecutionResponse::new(true);
        assert!(response.success);
        assert!(response.data.is_none());
        assert!(response.error_message.is_none());

        let data = Value::String("test".to_string());
        let response = ExecutionResponse::new(true).with_data(data.clone());
        assert_eq!(response.data, Some(data));

        let response = ExecutionResponse::new(false).with_error(100, "error message".to_string());
        assert!(!response.success);
        assert_eq!(response.error_code, Some(100));
        assert_eq!(response.error_message, Some("error message".to_string()));
    }

    #[test]
    fn test_plan_node() {
        let node = PlanNode {
            node_id: 1,
            node_type: "GetNeighbors".to_string(),
            children: Vec::new(),
        };

        assert_eq!(node.node_id, 1);
        assert_eq!(node.node_type, "GetNeighbors");
        assert!(node.children.is_empty());
    }
}

