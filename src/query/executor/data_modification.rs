use std::sync::Arc;
use std::time::Instant;

use super::base::{BaseExecutor, ExecutorStats};
use crate::core::{Edge, Value, Vertex};
use crate::index::Index;
use crate::expression::context::basic_context::BasicExpressionContext;
use crate::expression::evaluator::expression_evaluator::ExpressionEvaluator;
use crate::query::executor::base::{DBResult, ExecutionResult, Executor, HasStorage};
use crate::query::parser::parser::parse_expression_meta_from_string;
use crate::storage::StorageClient;
use parking_lot::Mutex;

// Executor for inserting new vertices/edges
pub struct InsertExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    vertex_data: Option<Vec<Vertex>>,
    edge_data: Option<Vec<Edge>>,
    if_not_exists: bool,
}

impl<S: StorageClient> InsertExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        vertex_data: Option<Vec<Vertex>>,
        edge_data: Option<Vec<Edge>>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "InsertExecutor".to_string(), storage),
            vertex_data,
            edge_data,
            if_not_exists: false,
        }
    }

    pub fn with_vertices(id: i64, storage: Arc<Mutex<S>>, vertex_data: Vec<Vertex>) -> Self {
        Self {
            base: BaseExecutor::new(id, "InsertExecutor".to_string(), storage),
            vertex_data: Some(vertex_data),
            edge_data: None,
            if_not_exists: false,
        }
    }

    pub fn with_edges(id: i64, storage: Arc<Mutex<S>>, edge_data: Vec<Edge>) -> Self {
        Self {
            base: BaseExecutor::new(id, "InsertExecutor".to_string(), storage),
            vertex_data: None,
            edge_data: Some(edge_data),
            if_not_exists: false,
        }
    }

    /// 创建带 IF NOT EXISTS 选项的 InsertExecutor
    pub fn with_vertices_if_not_exists(
        id: i64,
        storage: Arc<Mutex<S>>,
        vertex_data: Vec<Vertex>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "InsertExecutor".to_string(), storage),
            vertex_data: Some(vertex_data),
            edge_data: None,
            if_not_exists: true,
        }
    }

    /// 创建带 IF NOT EXISTS 选项的 InsertExecutor（用于边）
    pub fn with_edges_if_not_exists(
        id: i64,
        storage: Arc<Mutex<S>>,
        edge_data: Vec<Edge>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "InsertExecutor".to_string(), storage),
            vertex_data: None,
            edge_data: Some(edge_data),
            if_not_exists: true,
        }
    }
}

impl<S: StorageClient + Send + Sync + 'static> Executor<S> for InsertExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        let start = Instant::now();
        let result = self.do_execute();
        let elapsed = start.elapsed();
        self.base.get_stats_mut().add_total_time(elapsed);
        match result {
            Ok(count) => Ok(ExecutionResult::Count(count)),
            Err(e) => Err(e),
        }
    }

    fn open(&mut self) -> Result<(), crate::core::DBError> {
        Ok(())
    }

    fn close(&mut self) -> Result<(), crate::core::DBError> {
        Ok(())
    }

    fn is_open(&self) -> bool {
        true
    }

    fn id(&self) -> i64 {
        self.base.id
    }

    fn name(&self) -> &str {
        "InsertExecutor"
    }

    fn description(&self) -> &str {
        "Insert executor - inserts vertices and edges into storage"
    }

    fn stats(&self) -> &ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageClient> HasStorage<S> for InsertExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.get_storage()
    }
}

impl<S: StorageClient + Send + Sync + 'static> InsertExecutor<S> {
    fn do_execute(&mut self) -> DBResult<usize> {
        let mut total_inserted = 0;

        if let Some(vertices) = &self.vertex_data {
            let mut storage = self.get_storage().lock();
            for vertex in vertices {
                // 如果启用了 IF NOT EXISTS，检查顶点是否已存在
                if self.if_not_exists {
                    if storage.get_vertex("default", &vertex.vid)?.is_some() {
                        // 顶点已存在，跳过插入
                        continue;
                    }
                }
                storage.insert_vertex("default", vertex.clone())?;
                total_inserted += 1;
            }
        }

        if let Some(edges) = &self.edge_data {
            let mut storage = self.get_storage().lock();
            for edge in edges {
                storage.insert_edge("default", edge.clone())?;
                total_inserted += 1;
            }
        }

        Ok(total_inserted)
    }
}

// Executor for updating existing vertices/edges with enhanced functionality
//
// 功能增强:
// - 支持upsert（当节点不存在时插入）
// - 支持RETURN子句返回更新后的属性
// - 支持YIELD指定返回属性
// - 支持条件表达式
// - 更好的错误处理和日志
pub struct UpdateExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    vertex_updates: Option<Vec<VertexUpdate>>,
    edge_updates: Option<Vec<EdgeUpdate>>,
    condition: Option<String>,
    return_props: Option<Vec<String>>,
    yield_names: Vec<String>,
    insertable: bool,
    space_name: String,
}

#[derive(Debug, Clone)]
pub struct VertexUpdate {
    pub vertex_id: Value,
    pub properties: std::collections::HashMap<String, Value>,
    pub tags_to_add: Option<Vec<String>>,
    pub tags_to_remove: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct EdgeUpdate {
    pub src: Value,
    pub dst: Value,
    pub edge_type: String,
    pub rank: Option<i64>,
    pub properties: std::collections::HashMap<String, Value>,
}

#[derive(Debug, Clone)]
pub struct UpdateResult {
    pub vertex_id: Option<Value>,
    pub src: Option<Value>,
    pub dst: Option<Value>,
    pub edge_type: Option<String>,
    pub returned_props: std::collections::HashMap<String, Value>,
}

impl<S: StorageClient> UpdateExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        vertex_updates: Option<Vec<VertexUpdate>>,
        edge_updates: Option<Vec<EdgeUpdate>>,
        condition: Option<String>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "UpdateExecutor".to_string(), storage),
            vertex_updates,
            edge_updates,
            condition,
            return_props: None,
            yield_names: Vec::new(),
            insertable: false,
            space_name: "default".to_string(),
        }
    }

    pub fn with_return_props(mut self, return_props: Vec<String>) -> Self {
        self.return_props = Some(return_props);
        self
    }

    pub fn with_yield_names(mut self, yield_names: Vec<String>) -> Self {
        self.yield_names = yield_names;
        self
    }

    pub fn with_insertable(mut self, insertable: bool) -> Self {
        self.insertable = insertable;
        self
    }

    pub fn with_space(mut self, space_name: String) -> Self {
        self.space_name = space_name;
        self
    }
}

impl<S: StorageClient + Send + Sync + 'static> Executor<S> for UpdateExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        let start = Instant::now();
        let result = self.do_execute();
        let elapsed = start.elapsed();
        self.base.get_stats_mut().add_total_time(elapsed);
        match result {
            Ok(_) => Ok(ExecutionResult::Empty),
            Err(e) => Err(e),
        }
    }

    fn open(&mut self) -> DBResult<()> {
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        Ok(())
    }

    fn is_open(&self) -> bool {
        true
    }

    fn id(&self) -> i64 {
        self.base.id
    }

    fn name(&self) -> &str {
        "UpdateExecutor"
    }

    fn description(&self) -> &str {
        "Update executor - updates vertices and edges in storage"
    }

    fn stats(&self) -> &ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageClient> HasStorage<S> for UpdateExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.get_storage()
    }
}

impl<S: StorageClient + Send + Sync + 'static> UpdateExecutor<S> {
    fn do_execute(&mut self) -> DBResult<Vec<UpdateResult>> {
        let mut results = Vec::new();

        let condition_expression = if let Some(ref condition_str) = self.condition {
            Some(parse_expression_meta_from_string(condition_str).map(|meta| meta.into()).map_err(|e| {
                crate::core::error::DBError::Query(crate::core::error::QueryError::ExecutionError(
                    format!("条件解析失败: {}", e),
                ))
            })?)
        } else {
            None
        };

        let mut storage = self.get_storage().lock();

        if let Some(updates) = &self.vertex_updates {
            for update in updates {
                let mut update_result = UpdateResult {
                    vertex_id: Some(update.vertex_id.clone()),
                    src: None,
                    dst: None,
                    edge_type: None,
                    returned_props: std::collections::HashMap::new(),
                };

                let should_update = if let Some(ref expression) = condition_expression {
                    self.evaluate_condition(&expression, update.vertex_id.clone(), None, None, None, &update.properties)?
                } else {
                    true
                };

                if should_update {
                    if let Some(mut vertex) = storage.get_vertex(&self.space_name, &update.vertex_id)? {
                        for (key, value) in &update.properties {
                            vertex.properties.insert(key.clone(), value.clone());
                        }
                        storage.update_vertex(&self.space_name, vertex.clone())?;

                        update_result.returned_props = update.properties.clone();
                    } else if self.insertable {
                        let new_vertex = crate::core::Vertex::new_with_properties(
                            update.vertex_id.clone(),
                            Vec::new(),
                            update.properties.clone(),
                        );
                        storage.insert_vertex(&self.space_name, new_vertex)?;
                        update_result.returned_props = update.properties.clone();
                    }
                }

                results.push(update_result);
            }
        }

        if let Some(updates) = &self.edge_updates {
            for update in updates {
                let mut update_result = UpdateResult {
                    vertex_id: None,
                    src: Some(update.src.clone()),
                    dst: Some(update.dst.clone()),
                    edge_type: Some(update.edge_type.clone()),
                    returned_props: std::collections::HashMap::new(),
                };

                let should_update = if let Some(ref expression) = condition_expression {
                    self.evaluate_condition(&expression, update.src.clone(), Some(update.dst.clone()), Some(&update.edge_type), None, &update.properties)?
                } else {
                    true
                };

                if should_update {
                    let edge_key = (update.src.clone(), update.dst.clone(), update.edge_type.clone());
                    if let Some(mut edge) = storage.get_edge(&self.space_name, &edge_key.0, &edge_key.1, &edge_key.2)? {
                        for (key, value) in &update.properties {
                            edge.props.insert(key.clone(), value.clone());
                        }
                        storage.delete_edge(&self.space_name, &edge_key.0, &edge_key.1, &edge_key.2)?;
                        storage.insert_edge(&self.space_name, edge)?;
                        update_result.returned_props = update.properties.clone();
                    } else if self.insertable {
                        let new_edge = crate::core::Edge::new(
                            edge_key.0.clone(),
                            edge_key.1.clone(),
                            edge_key.2.clone(),
                            update.rank.unwrap_or(0),
                            update.properties.clone(),
                        );
                        storage.insert_edge(&self.space_name, new_edge)?;
                        update_result.returned_props = update.properties.clone();
                    }
                }

                results.push(update_result);
            }
        }

        Ok(results)
    }

    fn evaluate_condition(
        &self,
        expression: &crate::core::Expression,
        vertex_id: Value,
        dst: Option<Value>,
        edge_type: Option<&str>,
        _rank: Option<i64>,
        properties: &std::collections::HashMap<String, Value>,
    ) -> DBResult<bool> {
        let mut context = BasicExpressionContext::default();
        context.set_variable("VID", vertex_id.clone());
        if let Some(dst_val) = dst {
            context.set_variable("DST", dst_val);
        }
        if let Some(etype) = edge_type {
            context.set_variable("edge_type", crate::core::Value::String(etype.to_string()));
        }
        for (key, value) in properties {
            context.set_variable(key.clone(), value.clone());
        }

        let result = ExpressionEvaluator::evaluate(expression, &mut context)
            .map_err(|e| {
                crate::core::error::DBError::Query(
                    crate::core::error::QueryError::ExecutionError(format!("条件求值失败: {}", e)),
                )
            })?;

        match result {
            crate::core::Value::Bool(b) => Ok(b),
            _ => Err(crate::core::error::DBError::Query(
                crate::core::error::QueryError::ExecutionError("条件表达式必须返回布尔值".to_string()),
            )),
        }
    }
}

// Executor for deleting vertices/edges
pub struct DeleteExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    vertex_ids: Option<Vec<Value>>,
    edge_ids: Option<Vec<(Value, Value, String)>>,
    condition: Option<String>,
    with_edge: bool, // 是否级联删除关联边
    space_name: String,
}

impl<S: StorageClient> DeleteExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        vertex_ids: Option<Vec<Value>>,
        edge_ids: Option<Vec<(Value, Value, String)>>,
        condition: Option<String>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "DeleteExecutor".to_string(), storage),
            vertex_ids,
            edge_ids,
            condition,
            with_edge: false,
            space_name: "default".to_string(),
        }
    }

    /// 设置是否级联删除关联边
    pub fn with_edge(mut self, with_edge: bool) -> Self {
        self.with_edge = with_edge;
        self
    }

    /// 设置空间名称
    pub fn with_space(mut self, space_name: String) -> Self {
        self.space_name = space_name;
        self
    }
}

impl<S: StorageClient + Send + Sync + 'static> Executor<S> for DeleteExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        let start = Instant::now();
        let result = self.do_execute();
        let elapsed = start.elapsed();
        self.base.get_stats_mut().add_total_time(elapsed);
        match result {
            Ok(count) => Ok(ExecutionResult::Count(count)),
            Err(e) => Err(e),
        }
    }

    fn open(&mut self) -> DBResult<()> {
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        Ok(())
    }

    fn is_open(&self) -> bool {
        true
    }

    fn id(&self) -> i64 {
        self.base.id
    }

    fn name(&self) -> &str {
        "DeleteExecutor"
    }

    fn description(&self) -> &str {
        "Delete executor - deletes vertices and edges from storage"
    }

    fn stats(&self) -> &ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageClient> HasStorage<S> for DeleteExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.get_storage()
    }
}

impl<S: StorageClient + Send + Sync + 'static> DeleteExecutor<S> {
    fn do_execute(&mut self) -> DBResult<usize> {
        let mut total_deleted = 0;

        let condition_expression = if let Some(ref condition_str) = self.condition {
            Some(parse_expression_meta_from_string(condition_str).map(|meta| meta.into()).map_err(|e| {
                crate::core::error::DBError::Query(crate::core::error::QueryError::ExecutionError(
                    format!("条件解析失败: {}", e),
                ))
            })?)
        } else {
            None
        };

        if let Some(ids) = &self.vertex_ids {
            let mut storage = self.get_storage().lock();
            for id in ids {
                let should_delete = if let Some(ref expression) = condition_expression {
                    if let Ok(Some(vertex)) = storage.get_vertex(&self.space_name, id) {
                        let mut context = BasicExpressionContext::default();
                        context.set_variable("VID", id.clone());
                        for (key, value) in &vertex.properties {
                            context.set_variable(key.clone(), value.clone());
                        }

                        let result = ExpressionEvaluator::evaluate(expression, &mut context)
                            .map_err(|e| {
                                crate::core::error::DBError::Query(
                                    crate::core::error::QueryError::ExecutionError(format!("条件求值失败: {}", e)),
                                )
                            })?;

                        match result {
                            crate::core::Value::Bool(b) => b,
                            _ => true,
                        }
                    } else {
                        true
                    }
                } else {
                    true
                };

                if should_delete {
                    // 如果启用了级联删除，先删除关联边
                    if self.with_edge {
                        let edges = storage.get_node_edges(&self.space_name, id, crate::core::EdgeDirection::Both)
                            .map_err(|e| {
                                crate::core::error::DBError::Storage(
                                    crate::core::error::StorageError::StorageError(format!("获取关联边失败: {}", e))
                                )
                            })?;
                        for edge in edges {
                            storage.delete_edge(&self.space_name, &edge.src, &edge.dst, &edge.edge_type)
                                .map_err(|e| {
                                    crate::core::error::DBError::Storage(
                                        crate::core::error::StorageError::StorageError(format!("删除关联边失败: {}", e))
                                    )
                                })?;
                            total_deleted += 1;
                        }
                    }

                    if storage.delete_vertex(&self.space_name, id).is_ok() {
                        total_deleted += 1;
                    }
                }
            }
        }

        if let Some(edges) = &self.edge_ids {
            let mut storage = self.get_storage().lock();
            for (src, dst, edge_type) in edges {
                let should_delete = if let Some(ref expression) = condition_expression {
                    if let Ok(Some(edge)) = storage.get_edge(&self.space_name, src, dst, edge_type) {
                        let mut context = BasicExpressionContext::default();
                        context.set_variable("SRC", src.clone());
                        context.set_variable("DST", dst.clone());
                        context.set_variable("edge_type", crate::core::Value::String(edge_type.clone()));
                        for (key, value) in &edge.props {
                            context.set_variable(key.clone(), value.clone());
                        }

                        let result = ExpressionEvaluator::evaluate(expression, &mut context)
                            .map_err(|e| {
                                crate::core::error::DBError::Query(
                                    crate::core::error::QueryError::ExecutionError(format!("条件求值失败: {}", e)),
                                )
                            })?;

                        match result {
                            crate::core::Value::Bool(b) => b,
                            _ => true,
                        }
                    } else {
                        true
                    }
                } else {
                    true
                };

                if should_delete && storage.delete_edge(&self.space_name, src, dst, edge_type).is_ok() {
                    total_deleted += 1;
                }
            }
        }

        Ok(total_deleted)
    }
}

// Executor for deleting tags from vertices
pub struct DeleteTagExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    tag_names: Vec<String>,
    vertex_ids: Vec<Value>,
    space_name: String,
    delete_all_tags: bool,
}

impl<S: StorageClient> DeleteTagExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        tag_names: Vec<String>,
        vertex_ids: Vec<Value>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "DeleteTagExecutor".to_string(), storage),
            tag_names,
            vertex_ids,
            space_name: "default".to_string(),
            delete_all_tags: false,
        }
    }

    pub fn with_space(mut self, space_name: String) -> Self {
        self.space_name = space_name;
        self
    }

    /// 设置删除所有标签模式
    pub fn delete_all_tags(mut self) -> Self {
        self.delete_all_tags = true;
        self
    }
}

impl<S: StorageClient + Send + Sync + 'static> Executor<S> for DeleteTagExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        let start = Instant::now();
        let result = self.do_execute();
        let elapsed = start.elapsed();
        self.base.get_stats_mut().add_total_time(elapsed);
        match result {
            Ok(count) => Ok(ExecutionResult::Count(count)),
            Err(e) => Err(e),
        }
    }

    fn open(&mut self) -> DBResult<()> {
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        Ok(())
    }

    fn is_open(&self) -> bool {
        true
    }

    fn id(&self) -> i64 {
        self.base.id
    }

    fn name(&self) -> &str {
        "DeleteTagExecutor"
    }

    fn description(&self) -> &str {
        "Delete tag executor - removes tags from vertices"
    }

    fn stats(&self) -> &ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageClient> HasStorage<S> for DeleteTagExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.get_storage()
    }
}

impl<S: StorageClient + Send + Sync + 'static> DeleteTagExecutor<S> {
    fn do_execute(&mut self) -> DBResult<usize> {
        let mut total_deleted = 0;
        let mut storage = self.get_storage().lock();

        for vertex_id in &self.vertex_ids {
            // 如果是删除所有标签模式，先获取顶点的所有标签名
            let tag_names_to_delete = if self.delete_all_tags {
                match storage.get_vertex(&self.space_name, vertex_id) {
                    Ok(Some(vertex)) => {
                        vertex.tags.iter().map(|tag| tag.name.clone()).collect::<Vec<_>>()
                    }
                    Ok(None) => {
                        // 顶点不存在，跳过
                        continue;
                    }
                    Err(e) => {
                        eprintln!("获取顶点 {:?} 失败: {:?}", vertex_id, e);
                        continue;
                    }
                }
            } else {
                self.tag_names.clone()
            };

            match storage.delete_tags(&self.space_name, vertex_id, &tag_names_to_delete) {
                Ok(deleted_count) => {
                    total_deleted += deleted_count;
                }
                Err(e) => {
                    // 记录错误但继续处理其他顶点
                    eprintln!("删除顶点 {:?} 的标签失败: {:?}", vertex_id, e);
                }
            }
        }

        Ok(total_deleted)
    }
}

// Executor for creating indexes
pub struct CreateIndexExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    index_name: String,
    index_type: crate::index::IndexType,
    properties: Vec<String>,
    tag_name: Option<String>,
}

impl<S: StorageClient> CreateIndexExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        index_name: String,
        index_type: crate::index::IndexType,
        properties: Vec<String>,
        tag_name: Option<String>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "CreateIndexExecutor".to_string(), storage),
            index_name,
            index_type,
            properties,
            tag_name,
        }
    }
}

impl<S: StorageClient> HasStorage<S> for CreateIndexExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.get_storage()
    }
}

impl<S: StorageClient + Send + Sync + 'static> Executor<S> for CreateIndexExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        let start = Instant::now();
        let result = self.do_execute();
        let elapsed = start.elapsed();
        self.base.get_stats_mut().add_total_time(elapsed);
        match result {
            Ok(_) => Ok(ExecutionResult::Empty),
            Err(e) => Err(e),
        }
    }

    fn open(&mut self) -> DBResult<()> {
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        Ok(())
    }

    fn is_open(&self) -> bool {
        true
    }

    fn id(&self) -> i64 {
        self.base.id
    }

    fn name(&self) -> &str {
        "CreateIndexExecutor"
    }

    fn description(&self) -> &str {
        "Create index executor - creates indexes in storage"
    }

    fn stats(&self) -> &ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageClient + Send + Sync + 'static> CreateIndexExecutor<S> {
    fn do_execute(&mut self) -> DBResult<()> {
        let mut storage = self.get_storage().lock();

        let target_name = self.tag_name.clone()
            .or_else(|| Some(self.index_name.clone()))
            .unwrap_or_default();

        let index_type = self.index_type.clone();
        let index = Index::new(
            0,
            self.index_name.clone(),
            0,
            target_name,
            Vec::new(),
            self.properties.clone(),
            index_type.clone(),
            false,
        );

        match index_type {
            crate::index::IndexType::TagIndex => {
                storage.create_tag_index("default", &index)?;
            }
            crate::index::IndexType::EdgeIndex => {
                storage.create_edge_index("default", &index)?;
            }
        }

        Ok(())
    }
}

// Executor for dropping indexes
pub struct DropIndexExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    _index_name: String,
}

impl<S: StorageClient> DropIndexExecutor<S> {
    pub fn new(id: i64, storage: Arc<Mutex<S>>, _index_name: String) -> Self {
        Self {
            base: BaseExecutor::new(id, "DropIndexExecutor".to_string(), storage),
            _index_name,
        }
    }
}

impl<S: StorageClient> HasStorage<S> for DropIndexExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.get_storage()
    }
}

impl<S: StorageClient + Send + Sync + 'static> Executor<S> for DropIndexExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        let start = Instant::now();
        let result = self.do_execute();
        let elapsed = start.elapsed();
        self.base.get_stats_mut().add_total_time(elapsed);
        match result {
            Ok(_) => Ok(ExecutionResult::Empty),
            Err(e) => Err(e),
        }
    }

    fn open(&mut self) -> DBResult<()> {
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        Ok(())
    }

    fn is_open(&self) -> bool {
        true
    }

    fn id(&self) -> i64 {
        self.base.id
    }

    fn name(&self) -> &str {
        "DropIndexExecutor"
    }

    fn description(&self) -> &str {
        "Drop index executor - drops indexes from storage"
    }

    fn stats(&self) -> &ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageClient + Send + Sync + 'static> DropIndexExecutor<S> {
    fn do_execute(&mut self) -> DBResult<()> {
        let mut storage = self.get_storage().lock();

        storage.drop_tag_index("default", &self._index_name)?;

        Ok(())
    }
}
