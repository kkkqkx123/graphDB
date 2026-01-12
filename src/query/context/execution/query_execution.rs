//! 查询执行上下文 - 管理整个查询请求的上下文
//! 对应原C++中的QueryContext.h/cpp

use crate::core::context::request::RequestContext;
use crate::core::context::{QueryExecutionContext, ValidationContext};
use crate::core::{SymbolTable, Value};
use crate::core::error::ManagerResult;
use crate::graph::utils::IdGenerator;
use crate::query::context::managers::{
    CharsetInfo, IndexManager, MetaClient, SchemaManager, StorageClient,
};
use crate::utils::ObjectPool;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

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

    // 元数据客户端 - 使用Arc共享所有权
    meta_client: Option<Arc<dyn MetaClient>>,

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
            vctx: ValidationContext::new("default_query_context".to_string()),
            ectx: QueryExecutionContext::new(),
            plan: None,
            schema_manager: None,
            index_manager: None,
            storage_client: None,
            meta_client: None,
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
            let _ = ctx.ectx.set_value(name, value.clone());
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

    /// 设置元数据客户端
    pub fn set_meta_client(&mut self, meta_client: Arc<dyn MetaClient>) {
        self.meta_client = Some(meta_client);
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

    /// 获取元数据客户端
    pub fn get_meta_client(&self) -> Option<&Arc<dyn MetaClient>> {
        self.meta_client.as_ref()
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
            Ok(value) => {
                // 检查参数值是否为空
                match value {
                    Value::Empty => false,
                    _ => true,
                }
            }
            Err(_) => false, // 如果参数不存在，返回false
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
            has_meta_client: self.meta_client.is_some(),
            has_charset_info: self.charset_info.is_some(),
            has_execution_plan: self.plan.is_some(),
            is_killed: self.is_killed(),
            current_id: self.current_id(),
            symbol_table_size: self.sym_table.size().unwrap_or(0),
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
    pub has_meta_client: bool,
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
            meta_client: self.meta_client.clone(),
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
            .field("meta_client", &self.meta_client.is_some())
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
    use crate::query::context::managers::{Index, Schema};
    use std::collections::HashMap;

    // Mock实现用于测试
    #[derive(Debug)]
    struct MockSchemaManager {
        schemas: HashMap<String, Schema>,
    }

    impl MockSchemaManager {
        fn new() -> Self {
            let mut schemas = HashMap::new();
            schemas.insert(
                "test_schema".to_string(),
                Schema {
                    name: "test_schema".to_string(),
                    fields: HashMap::new(),
                    is_vertex: true,
                },
            );
            Self { schemas }
        }
    }

    impl SchemaManager for MockSchemaManager {
        fn get_schema(&self, name: &str) -> Option<Schema> {
            self.schemas.get(name).cloned()
        }

        fn list_schemas(&self) -> Vec<String> {
            self.schemas.keys().cloned().collect()
        }

        fn has_schema(&self, name: &str) -> bool {
            self.schemas.contains_key(name)
        }

        fn create_tag(
            &self,
            _space_id: i32,
            _tag_name: &str,
            _fields: Vec<crate::query::context::managers::FieldDef>,
        ) -> ManagerResult<i32> {
            Ok(1)
        }

        fn drop_tag(&self, _space_id: i32, _tag_id: i32) -> ManagerResult<()> {
            Ok(())
        }

        fn get_tag(
            &self,
            _space_id: i32,
            _tag_id: i32,
        ) -> Option<crate::query::context::managers::TagDef> {
            None
        }

        fn list_tags(
            &self,
            _space_id: i32,
        ) -> ManagerResult<Vec<crate::query::context::managers::TagDef>> {
            Ok(Vec::new())
        }

        fn has_tag(&self, _space_id: i32, _tag_id: i32) -> bool {
            false
        }

        fn create_edge_type(
            &self,
            _space_id: i32,
            _edge_type_name: &str,
            _fields: Vec<crate::query::context::managers::FieldDef>,
        ) -> ManagerResult<i32> {
            Ok(1)
        }

        fn drop_edge_type(&self, _space_id: i32, _edge_type_id: i32) -> ManagerResult<()> {
            Ok(())
        }

        fn get_edge_type(
            &self,
            _space_id: i32,
            _edge_type_id: i32,
        ) -> Option<crate::query::context::managers::EdgeTypeDef> {
            None
        }

        fn list_edge_types(
            &self,
            _space_id: i32,
        ) -> ManagerResult<Vec<crate::query::context::managers::EdgeTypeDef>> {
            Ok(Vec::new())
        }

        fn has_edge_type(&self, _space_id: i32, _edge_type_id: i32) -> bool {
            false
        }

        fn load_from_disk(&self) -> ManagerResult<()> {
            Ok(())
        }

        fn save_to_disk(&self) -> ManagerResult<()> {
            Ok(())
        }

        fn create_schema_version(
            &self,
            _space_id: i32,
            _comment: Option<String>,
        ) -> ManagerResult<i32> {
            Ok(1)
        }

        fn get_schema_version(
            &self,
            _space_id: i32,
            _version: i32,
        ) -> Option<crate::query::context::managers::SchemaVersion> {
            None
        }

        fn get_latest_schema_version(&self, _space_id: i32) -> Option<i32> {
            Some(1)
        }

        fn get_schema_history(
            &self,
            _space_id: i32,
        ) -> ManagerResult<crate::query::context::managers::SchemaHistory> {
            Ok(crate::query::context::managers::SchemaHistory {
                space_id: _space_id,
                versions: Vec::new(),
                current_version: 1,
            })
        }

        fn rollback_schema(&self, _space_id: i32, _version: i32) -> ManagerResult<()> {
            Ok(())
        }

        fn get_current_version(&self, _space_id: i32) -> Option<i32> {
            Some(1)
        }
    }

    #[derive(Debug)]
    struct MockIndexManager {
        indexes: HashMap<String, Index>,
    }

    impl MockIndexManager {
        fn new() -> Self {
            let mut indexes = HashMap::new();
            indexes.insert(
                "test_index".to_string(),
                Index {
                    id: 1,
                    name: "test_index".to_string(),
                    space_id: 1,
                    schema_name: "test_schema".to_string(),
                    fields: vec!["id".to_string()],
                    index_type: crate::query::context::managers::IndexType::TagIndex,
                    status: crate::query::context::managers::IndexStatus::Active,
                    is_unique: true,
                    comment: None,
                },
            );
            Self { indexes }
        }
    }

    impl IndexManager for MockIndexManager {
        fn get_index(&self, name: &str) -> Option<Index> {
            self.indexes.get(name).cloned()
        }

        fn list_indexes(&self) -> Vec<String> {
            self.indexes.keys().cloned().collect()
        }

        fn has_index(&self, name: &str) -> bool {
            self.indexes.contains_key(name)
        }

        fn create_index(&self, _space_id: i32, _index: Index) -> ManagerResult<i32> {
            Ok(1)
        }

        fn drop_index(&self, _space_id: i32, _index_id: i32) -> ManagerResult<()> {
            Ok(())
        }

        fn get_index_status(
            &self,
            _space_id: i32,
            _index_id: i32,
        ) -> Option<crate::query::context::managers::IndexStatus> {
            Some(crate::query::context::managers::IndexStatus::Active)
        }

        fn list_indexes_by_space(&self, _space_id: i32) -> ManagerResult<Vec<Index>> {
            Ok(self.indexes.values().cloned().collect())
        }

        fn lookup_vertex_by_index(
            &self,
            _space_id: i32,
            _index_name: &str,
            _values: &[crate::core::Value],
        ) -> ManagerResult<Vec<crate::core::Vertex>> {
            Ok(Vec::new())
        }

        fn lookup_edge_by_index(
            &self,
            _space_id: i32,
            _index_name: &str,
            _values: &[crate::core::Value],
        ) -> ManagerResult<Vec<crate::core::Edge>> {
            Ok(Vec::new())
        }

        fn range_lookup_vertex(
            &self,
            _space_id: i32,
            _index_name: &str,
            _start: &crate::core::Value,
            _end: &crate::core::Value,
        ) -> ManagerResult<Vec<crate::core::Vertex>> {
            Ok(Vec::new())
        }

        fn range_lookup_edge(
            &self,
            _space_id: i32,
            _index_name: &str,
            _start: &crate::core::Value,
            _end: &crate::core::Value,
        ) -> ManagerResult<Vec<crate::core::Edge>> {
            Ok(Vec::new())
        }

        fn load_from_disk(&self) -> ManagerResult<()> {
            Ok(())
        }

        fn save_to_disk(&self) -> ManagerResult<()> {
            Ok(())
        }

        fn insert_vertex_to_index(
            &self,
            _space_id: i32,
            _vertex: &crate::core::Vertex,
        ) -> ManagerResult<()> {
            Ok(())
        }

        fn delete_vertex_from_index(
            &self,
            _space_id: i32,
            _vertex: &crate::core::Vertex,
        ) -> ManagerResult<()> {
            Ok(())
        }

        fn update_vertex_in_index(
            &self,
            _space_id: i32,
            _old_vertex: &crate::core::Vertex,
            _new_vertex: &crate::core::Vertex,
        ) -> ManagerResult<()> {
            Ok(())
        }

        fn insert_edge_to_index(
            &self,
            _space_id: i32,
            _edge: &crate::core::Edge,
        ) -> ManagerResult<()> {
            Ok(())
        }

        fn delete_edge_from_index(
            &self,
            _space_id: i32,
            _edge: &crate::core::Edge,
        ) -> ManagerResult<()> {
            Ok(())
        }

        fn update_edge_in_index(
            &self,
            _space_id: i32,
            _old_edge: &crate::core::Edge,
            _new_edge: &crate::core::Edge,
        ) -> ManagerResult<()> {
            Ok(())
        }
    }

    #[derive(Debug)]
    struct MockStorageClient;

    impl StorageClient for MockStorageClient {
        fn execute(
            &self,
            _operation: crate::query::context::managers::StorageOperation,
        ) -> ManagerResult<crate::query::context::managers::StorageResponse> {
            Ok(crate::query::context::managers::StorageResponse {
                success: true,
                data: None,
                error_message: None,
            })
        }

        fn is_connected(&self) -> bool {
            true
        }

        fn add_vertex(
            &self,
            _space_id: i32,
            _vertex: crate::core::Vertex,
        ) -> ManagerResult<crate::query::context::managers::ExecResponse> {
            Ok(crate::query::context::managers::ExecResponse::ok())
        }

        fn add_vertices(
            &self,
            _space_id: i32,
            _vertices: Vec<crate::query::context::managers::NewVertex>,
        ) -> ManagerResult<crate::query::context::managers::ExecResponse> {
            Ok(crate::query::context::managers::ExecResponse::ok())
        }

        fn get_vertex(
            &self,
            _space_id: i32,
            _vid: &crate::core::Value,
        ) -> ManagerResult<Option<crate::core::Vertex>> {
            Ok(None)
        }

        fn get_vertices(
            &self,
            _space_id: i32,
            _vids: &[crate::core::Value],
        ) -> ManagerResult<Vec<Option<crate::core::Vertex>>> {
            Ok(Vec::new())
        }

        fn delete_vertex(
            &self,
            _space_id: i32,
            _vid: &crate::core::Value,
        ) -> ManagerResult<crate::query::context::managers::ExecResponse> {
            Ok(crate::query::context::managers::ExecResponse::ok())
        }

        fn delete_vertices(
            &self,
            _space_id: i32,
            _vids: &[crate::core::Value],
        ) -> ManagerResult<crate::query::context::managers::ExecResponse> {
            Ok(crate::query::context::managers::ExecResponse::ok())
        }

        fn delete_tags(
            &self,
            _space_id: i32,
            _del_tags: Vec<crate::query::context::managers::DelTags>,
        ) -> ManagerResult<crate::query::context::managers::ExecResponse> {
            Ok(crate::query::context::managers::ExecResponse::ok())
        }

        fn update_vertex(
            &self,
            _space_id: i32,
            _vid: &crate::core::Value,
            _tag_id: i32,
            _updated_props: Vec<crate::query::context::managers::UpdatedProp>,
            _insertable: bool,
            _return_props: Vec<String>,
            _condition: Option<String>,
        ) -> ManagerResult<crate::query::context::managers::UpdateResponse> {
            Ok(crate::query::context::managers::UpdateResponse::ok(
                false, None,
            ))
        }

        fn add_edge(
            &self,
            _space_id: i32,
            _edge: crate::core::Edge,
        ) -> ManagerResult<crate::query::context::managers::ExecResponse> {
            Ok(crate::query::context::managers::ExecResponse::ok())
        }

        fn add_edges(
            &self,
            _space_id: i32,
            _edges: Vec<crate::query::context::managers::NewEdge>,
        ) -> ManagerResult<crate::query::context::managers::ExecResponse> {
            Ok(crate::query::context::managers::ExecResponse::ok())
        }

        fn get_edge(
            &self,
            _space_id: i32,
            _edge_key: &crate::query::context::managers::EdgeKey,
        ) -> ManagerResult<Option<crate::core::Edge>> {
            Ok(None)
        }

        fn get_edges(
            &self,
            _space_id: i32,
            _edge_keys: &[crate::query::context::managers::EdgeKey],
        ) -> ManagerResult<Vec<Option<crate::core::Edge>>> {
            Ok(Vec::new())
        }

        fn delete_edge(
            &self,
            _space_id: i32,
            _edge_key: &crate::query::context::managers::EdgeKey,
        ) -> ManagerResult<crate::query::context::managers::ExecResponse> {
            Ok(crate::query::context::managers::ExecResponse::ok())
        }

        fn delete_edges(
            &self,
            _space_id: i32,
            _edge_keys: &[crate::query::context::managers::EdgeKey],
        ) -> ManagerResult<crate::query::context::managers::ExecResponse> {
            Ok(crate::query::context::managers::ExecResponse::ok())
        }

        fn update_edge(
            &self,
            _space_id: i32,
            _edge_key: &crate::query::context::managers::EdgeKey,
            _updated_props: Vec<crate::query::context::managers::UpdatedProp>,
            _insertable: bool,
            _return_props: Vec<String>,
            _condition: Option<String>,
        ) -> ManagerResult<crate::query::context::managers::UpdateResponse> {
            Ok(crate::query::context::managers::UpdateResponse::ok(
                false, None,
            ))
        }

        fn scan_vertices(
            &self,
            _space_id: i32,
            _limit: Option<usize>,
        ) -> ManagerResult<Vec<crate::core::Vertex>> {
            Ok(Vec::new())
        }

        fn scan_vertices_by_tag(
            &self,
            _space_id: i32,
            _tag_id: i32,
            _limit: Option<usize>,
        ) -> ManagerResult<Vec<crate::core::Vertex>> {
            Ok(Vec::new())
        }

        fn scan_edges(
            &self,
            _space_id: i32,
            _limit: Option<usize>,
        ) -> ManagerResult<Vec<crate::core::Edge>> {
            Ok(Vec::new())
        }

        fn scan_edges_by_type(
            &self,
            _space_id: i32,
            _edge_type: &str,
            _limit: Option<usize>,
        ) -> ManagerResult<Vec<crate::core::Edge>> {
            Ok(Vec::new())
        }

        fn scan_edges_by_src(
            &self,
            _space_id: i32,
            _src: &crate::core::Value,
            _limit: Option<usize>,
        ) -> ManagerResult<Vec<crate::core::Edge>> {
            Ok(Vec::new())
        }

        fn scan_edges_by_dst(
            &self,
            _space_id: i32,
            _dst: &crate::core::Value,
            _limit: Option<usize>,
        ) -> ManagerResult<Vec<crate::core::Edge>> {
            Ok(Vec::new())
        }
    }

    #[derive(Debug)]
    struct MockMetaClient;

    impl MetaClient for MockMetaClient {
        fn get_cluster_info(&self) -> ManagerResult<crate::query::context::managers::ClusterInfo> {
            Ok(crate::query::context::managers::ClusterInfo {
                cluster_id: "test_cluster".to_string(),
                meta_servers: vec!["127.0.0.1:9559".to_string()],
                storage_servers: vec!["127.0.0.1:9779".to_string()],
            })
        }

        fn get_space_info(
            &self,
            space_id: i32,
        ) -> ManagerResult<crate::query::context::managers::SpaceInfo> {
            Ok(crate::query::context::managers::SpaceInfo {
                space_id,
                space_name: "test_space".to_string(),
                partition_num: 10,
                replica_factor: 1,
            })
        }

        fn is_connected(&self) -> bool {
            true
        }

        fn create_space(
            &self,
            _space_name: &str,
            _partition_num: i32,
            _replica_factor: i32,
        ) -> ManagerResult<i32> {
            Ok(1)
        }

        fn drop_space(&self, _space_id: i32) -> ManagerResult<()> {
            Ok(())
        }

        fn list_spaces(&self) -> ManagerResult<Vec<crate::query::context::managers::SpaceInfo>> {
            Ok(Vec::new())
        }

        fn has_space(&self, _space_id: i32) -> bool {
            false
        }

        fn load_from_disk(&self) -> ManagerResult<()> {
            Ok(())
        }

        fn save_to_disk(&self) -> ManagerResult<()> {
            Ok(())
        }
    }

    #[test]
    fn test_query_context_creation() {
        let ctx = QueryContext::new();

        // 测试ID生成
        let id1 = ctx.gen_id();
        let id2 = ctx.gen_id();
        assert_eq!(id2, id1 + 1);

        // 测试验证上下文
        // ValidationContext不管理空间信息，已删除 switch_to_space 和 current_space 方法
        // ValidationContext 不再提供 error_count() 方法

        // 测试存在参数检查（参数不存在）
        assert!(!ctx.exist_parameter("non_existent_param"));

        // 测试终止标记
        assert!(!ctx.is_killed());
        ctx.mark_killed();
        assert!(ctx.is_killed());
    }

    #[test]
    fn test_context_access() {
        let mut ctx = QueryContext::new();

        // 测试符号表
        ctx.sym_table_mut()
            .new_variable("test_var")
            .expect("Failed to create new variable");
        assert!(ctx.sym_table().has_variable("test_var"));

        // 测试执行上下文
        let value = crate::core::Value::Int(42);
        ctx.ectx()
            .set_value("test_val", value.clone())
            .expect("Failed to set value");
        let retrieved = ctx
            .ectx()
            .get_value("test_val")
            .expect("Failed to get value");
        assert_eq!(retrieved, value);
    }

    #[test]
    fn test_managers() {
        let mut ctx = QueryContext::new();

        // 设置Schema管理器
        let schema_manager = Arc::new(MockSchemaManager::new());
        ctx.set_schema_manager(schema_manager.clone());
        assert!(ctx.schema_manager().is_some());
        assert!(ctx
            .schema_manager()
            .expect("Schema manager should exist")
            .has_schema("test_schema"));

        // 设置索引管理器
        let index_manager = Arc::new(MockIndexManager::new());
        ctx.set_index_manager(index_manager.clone());
        assert!(ctx.index_manager().is_some());
        assert!(ctx
            .index_manager()
            .expect("Index manager should exist")
            .has_index("test_index"));

        // 设置存储客户端
        let storage_client = Arc::new(MockStorageClient);
        ctx.set_storage_client(storage_client.clone());
        assert!(ctx.get_storage_client().is_some());
        assert!(ctx
            .get_storage_client()
            .expect("Storage client should exist")
            .is_connected());

        // 设置元数据客户端
        let meta_client = Arc::new(MockMetaClient);
        ctx.set_meta_client(meta_client.clone());
        assert!(ctx.get_meta_client().is_some());
        assert!(ctx
            .get_meta_client()
            .expect("Meta client should exist")
            .is_connected());
    }

    #[test]
    fn test_execution_plan() {
        let mut ctx = QueryContext::new();

        // 创建执行计划
        let plan = ExecutionPlan::new(ctx.gen_id());
        ctx.set_plan(plan);

        // 检查执行计划
        assert!(ctx.plan().is_some());
        assert_eq!(ctx.plan().expect("Plan should exist").id(), 0);
        assert!(!ctx.plan().expect("Plan should exist").is_profile_enabled());

        // 启用性能分析
        ctx.plan_mut()
            .as_mut()
            .expect("Plan should exist")
            .enable_profile();
        assert!(ctx.plan().expect("Plan should exist").is_profile_enabled());
    }

    #[test]
    fn test_charset_info() {
        let mut ctx = QueryContext::new();

        // 设置字符集信息
        let charset_info = crate::query::context::managers::CharsetInfo {
            charset: "utf8mb4".to_string(),
            collation: "utf8mb4_general_ci".to_string(),
        };
        ctx.set_charset_info(charset_info.clone());

        // 检查字符集信息
        assert!(ctx.get_charset_info().is_some());
        assert_eq!(
            ctx.get_charset_info()
                .expect("Charset info should exist")
                .charset,
            charset_info.charset
        );
        assert_eq!(
            ctx.get_charset_info()
                .expect("Charset info should exist")
                .collation,
            charset_info.collation
        );
    }

    #[test]
    fn test_status_info() {
        let mut ctx = QueryContext::new();

        // 获取初始状态
        let status = ctx.get_status_info();
        assert!(!status.has_request_context);
        assert!(!status.has_schema_manager);
        assert!(!status.has_index_manager);
        assert!(!status.has_storage_client);
        assert!(!status.has_meta_client);
        assert!(!status.has_charset_info);
        assert!(!status.has_execution_plan);
        assert!(!status.is_killed);
        assert_eq!(status.current_id, 0);
        assert_eq!(status.variable_count, 0); // 初始变量数量应为0

        // 设置一些组件
        ctx.set_schema_manager(Arc::new(MockSchemaManager::new()));
        ctx.set_plan(ExecutionPlan::new(ctx.gen_id()));

        // 添加一些变量到执行上下文
        ctx.ectx()
            .set_value("test_var1", crate::core::Value::Int(42))
            .expect("Failed to set value for test_var1");
        ctx.ectx()
            .set_value("test_var2", crate::core::Value::String("hello".to_string()))
            .expect("Failed to set value for test_var2");

        // 检查更新后的状态
        let status = ctx.get_status_info();
        assert!(status.has_schema_manager);
        assert!(status.has_execution_plan);
        assert_eq!(status.current_id, 1);
        assert_eq!(status.variable_count, 2); // 现在应该有2个变量
    }

    #[test]
    fn test_reset() {
        let mut ctx = QueryContext::new();

        // 设置一些状态
        ctx.set_plan(ExecutionPlan::new(ctx.gen_id()));
        ctx.mark_killed();

        // 检查状态
        assert!(ctx.plan().is_some());
        assert!(ctx.is_killed());

        // 重置
        ctx.reset();

        // 检查重置后的状态
        assert!(ctx.plan().is_none());
        assert!(!ctx.is_killed());
        assert_eq!(ctx.current_id(), 0);
    }

    #[test]
    fn test_with_request_context() {
        // 创建请求上下文
        let request_ctx = Arc::new(RequestContext::with_session(
            "request_ctx_1".to_string(),
            "MATCH (n) RETURN n".to_string(),
            "test_session",
            "test_user",
            "127.0.0.1",
            0,
        ));

        // 使用请求上下文创建查询上下文
        let ctx = QueryContext::with_request_context(request_ctx.clone());

        // 检查请求上下文是否正确设置
        assert!(ctx.rctx().is_some());
        assert_eq!(
            ctx.rctx().expect("Request context should exist").query(),
            "MATCH (n) RETURN n"
        );
    }
}
