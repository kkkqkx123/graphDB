use super::{StorageClient, TransactionId, MemorySchemaManager};
use crate::storage::operations::{VertexReader, EdgeReader, VertexWriter, EdgeWriter};
use crate::core::{Edge, StorageError, Value, Vertex, EdgeDirection};
use crate::core::types::{
    SpaceInfo, TagInfo, EdgeTypeInfo, PropertyDef,
    InsertVertexInfo, InsertEdgeInfo, UpdateInfo, UpdateOp,
    PasswordInfo, SchemaChange, SchemaChangeType,
};
use crate::core::types::metadata::{UserInfo, UserAlterInfo};
pub use crate::core::types::EdgeTypeInfo as EdgeTypeSchema;
use crate::index::Index;
use crate::storage::Schema;
use crate::storage::serializer::{vertex_to_bytes, edge_to_bytes};
use crate::storage::metadata::{RedbExtendedSchemaManager, ExtendedSchemaManager, RedbMetadataManager};
use crate::storage::operations::{RedbReader, RedbWriter};
use crate::storage::index::RedbIndexManager;
use crate::api::service::permission_manager::RoleType;
use redb::Database;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct RedbStorage {
    reader: RedbReader,
    writer: Arc<Mutex<RedbWriter>>,
    index_manager: RedbIndexManager,
    metadata_manager: RedbMetadataManager,
    spaces: Arc<Mutex<HashMap<String, SpaceInfo>>>,
    tags: Arc<Mutex<HashMap<String, HashMap<String, TagInfo>>>>,
    edge_type_infos: Arc<Mutex<HashMap<String, HashMap<String, EdgeTypeSchema>>>>,
    tag_indexes: Arc<Mutex<HashMap<String, HashMap<String, Index>>>>,
    edge_indexes: Arc<Mutex<HashMap<String, HashMap<String, Index>>>>,
    users: Arc<Mutex<HashMap<String, UserInfo>>>,
    pub schema_manager: Arc<MemorySchemaManager>,
    pub extended_schema_manager: Arc<RedbExtendedSchemaManager>,
    db: Arc<Database>,
    db_path: PathBuf,
}

impl std::fmt::Debug for RedbStorage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedbStorage")
            .field("db_path", &self.db_path)
            .finish()
    }
}

impl RedbStorage {
    pub fn new() -> Result<Self, StorageError> {
        Self::new_with_path(PathBuf::from("data/redb"))
    }

    pub fn new_with_path(path: PathBuf) -> Result<Self, StorageError> {
        let schema_manager = Arc::new(MemorySchemaManager::new());

        // 创建或打开 redb 数据库
        let db = Arc::new(Database::create(&path)
            .map_err(|e| StorageError::DbError(format!("创建数据库失败: {}", e)))?);
        let extended_schema_manager = Arc::new(RedbExtendedSchemaManager::new(db.clone()));

        // 创建 reader 和 writer
        let reader = RedbReader::new(db.clone())?;
        let writer = Arc::new(Mutex::new(RedbWriter::new(db.clone())?));

        // 创建索引管理器
        let tag_indexes = Arc::new(Mutex::new(HashMap::new()));
        let edge_indexes = Arc::new(Mutex::new(HashMap::new()));
        let index_manager = RedbIndexManager::new(
            db.clone(),
            tag_indexes.clone(),
            edge_indexes.clone(),
        );

        // 创建元数据管理器
        let spaces = Arc::new(Mutex::new(HashMap::new()));
        let tags = Arc::new(Mutex::new(HashMap::new()));
        let edge_type_infos = Arc::new(Mutex::new(HashMap::new()));
        let users = Arc::new(Mutex::new(HashMap::new()));
        let metadata_manager = RedbMetadataManager::new(
            db.clone(),
            spaces.clone(),
            tags.clone(),
            edge_type_infos.clone(),
            users.clone(),
        );

        Ok(Self {
            reader,
            writer,
            index_manager,
            metadata_manager,
            spaces,
            tags,
            edge_type_infos,
            tag_indexes,
            edge_indexes,
            users,
            schema_manager,
            extended_schema_manager,
            db,
            db_path: path,
        })
    }
}

impl RedbStorage {
    pub fn get_db(&self) -> &Arc<Database> {
        &self.db
    }
    
    pub fn get_reader(&self) -> &RedbReader {
        &self.reader
    }
    
    pub fn get_writer(&self) -> Arc<Mutex<RedbWriter>> {
        self.writer.clone()
    }

    // 解析顶点ID
    fn parse_vertex_id(&self, vertex_id: &str) -> Result<Value, StorageError> {
        // 尝试解析为整数
        if let Ok(i) = vertex_id.parse::<i64>() {
            return Ok(Value::Int(i));
        }
        // 默认识别为字符串
        Ok(Value::String(vertex_id.to_string()))
    }
    
    // 删除顶点相关边
    fn delete_vertex_edges(&mut self, space: &str, vertex_id: &Value) -> Result<(), StorageError> {
        let edges = self.reader.scan_all_edges(space)?;
        for edge in edges {
            if *edge.src == *vertex_id || *edge.dst == *vertex_id {
                {
                    let mut writer = self.writer.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
                    writer.delete_edge(space, &edge.src, &edge.dst, &edge.edge_type)?;
                }
                self.index_manager.delete_edge_indexes(space, &edge.src, &edge.dst, &edge.edge_type)?;
            }
        }
        Ok(())
    }
    
    // 更新顶点属性
    fn update_vertex_property(&self, space: &str, vertex_id: &Value, tag: &str, prop: &str, op: &UpdateOp, value: &Value) -> Result<(), StorageError> {
        if let Some(mut vertex) = self.reader.get_vertex(space, vertex_id)? {
            for tag_data in &mut vertex.tags {
                if tag_data.name == tag {
                    match op {
                        UpdateOp::Set => {
                            tag_data.properties.insert(prop.to_string(), value.clone());
                        }
                        UpdateOp::Add => {
                            if let Some(existing) = tag_data.properties.get(prop) {
                                if let (Value::Int(a), Value::Int(b)) = (existing, value) {
                                    tag_data.properties.insert(prop.to_string(), Value::Int(a + b));
                                }
                            }
                        }
                        UpdateOp::Subtract => {
                            if let Some(existing) = tag_data.properties.get(prop) {
                                if let (Value::Int(a), Value::Int(b)) = (existing, value) {
                                    tag_data.properties.insert(prop.to_string(), Value::Int(a - b));
                                }
                            }
                        }
                        _ => {} // 其他操作暂不支持
                    }
                    break;
                }
            }
            // 使用 writer 更新顶点
            let mut writer = self.writer.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
            writer.update_vertex(space, vertex)?;
        }
        Ok(())
    }

    // 构建顶点schema
    fn build_vertex_schema(&self, tag_info: &crate::core::types::TagInfo) -> Result<Schema, StorageError> {
        let mut schema = Schema::new(tag_info.tag_name.clone(), 1);
        for prop in &tag_info.properties {
            let field_def = super::types::FieldDef {
                name: prop.name.clone(),
                field_type: prop.data_type.clone(),
                nullable: prop.nullable,
                default_value: prop.default.clone(),
                fixed_length: None,
                offset: 0,
                null_flag_pos: None,
                geo_shape: None,
            };
            schema = schema.add_field(field_def);
        }
        Ok(schema)
    }
    
    // 构建边schema
    fn build_edge_schema(&self, edge_type_info: &crate::core::types::EdgeTypeInfo) -> Result<Schema, StorageError> {
        let mut schema = Schema::new(edge_type_info.edge_type_name.clone(), 1);
        for prop in &edge_type_info.properties {
            let field_def = super::types::FieldDef {
                name: prop.name.clone(),
                field_type: prop.data_type.clone(),
                nullable: prop.nullable,
                default_value: prop.default.clone(),
                fixed_length: None,
                offset: 0,
                null_flag_pos: None,
                geo_shape: None,
            };
            schema = schema.add_field(field_def);
        }
        Ok(schema)
    }

}

impl StorageClient for RedbStorage {
    fn get_vertex(&self, space: &str, id: &Value) -> Result<Option<Vertex>, StorageError> {
        self.reader.get_vertex(space, id)
    }

    fn scan_vertices(&self, space: &str) -> Result<Vec<Vertex>, StorageError> {
        self.reader.scan_vertices(space).map(|r| r.into_vec())
    }

    fn scan_vertices_by_tag(&self, space: &str, tag: &str) -> Result<Vec<Vertex>, StorageError> {
        self.reader.scan_vertices_by_tag(space, tag).map(|r| r.into_vec())
    }

    fn scan_vertices_by_prop(
        &self,
        space: &str,
        tag: &str,
        prop: &str,
        value: &Value,
    ) -> Result<Vec<Vertex>, StorageError> {
        self.reader.scan_vertices_by_prop(space, tag, prop, value).map(|r| r.into_vec())
    }

    fn get_edge(
        &self,
        space: &str,
        src: &Value,
        dst: &Value,
        edge_type: &str,
    ) -> Result<Option<Edge>, StorageError> {
        self.reader.get_edge(space, src, dst, edge_type)
    }

    fn get_node_edges(
        &self,
        space: &str,
        node_id: &Value,
        direction: EdgeDirection,
    ) -> Result<Vec<Edge>, StorageError> {
        self.reader.get_node_edges(space, node_id, direction).map(|r| r.into_vec())
    }

    fn get_node_edges_filtered(
        &self,
        space: &str,
        node_id: &Value,
        direction: EdgeDirection,
        filter: Option<Box<dyn Fn(&Edge) -> bool + Send + Sync + 'static>>,
    ) -> Result<Vec<Edge>, StorageError> {
        self.reader.get_node_edges_filtered(space, node_id, direction, filter).map(|r| r.into_vec())
    }

    fn scan_edges_by_type(&self, space: &str, edge_type: &str) -> Result<Vec<Edge>, StorageError> {
        self.reader.scan_edges_by_type(space, edge_type).map(|r| r.into_vec())
    }

    fn scan_all_edges(&self, space: &str) -> Result<Vec<Edge>, StorageError> {
        self.reader.scan_all_edges(space).map(|r| r.into_vec())
    }

    fn insert_vertex(&mut self, space: &str, vertex: Vertex) -> Result<Value, StorageError> {
        let mut writer = self.writer.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        writer.insert_vertex(space, vertex)
    }

    fn update_vertex(&mut self, space: &str, vertex: Vertex) -> Result<(), StorageError> {
        let mut writer = self.writer.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        writer.update_vertex(space, vertex)
    }

    fn delete_vertex(&mut self, space: &str, id: &Value) -> Result<(), StorageError> {
        let mut writer = self.writer.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        writer.delete_vertex(space, id)
    }

    fn batch_insert_vertices(&mut self, space: &str, vertices: Vec<Vertex>) -> Result<Vec<Value>, StorageError> {
        let mut writer = self.writer.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        writer.batch_insert_vertices(space, vertices)
    }

    fn insert_edge(&mut self, space: &str, edge: Edge) -> Result<(), StorageError> {
        let mut writer = self.writer.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        writer.insert_edge(space, edge)
    }

    fn delete_edge(&mut self, space: &str, src: &Value, dst: &Value, edge_type: &str) -> Result<(), StorageError> {
        let mut writer = self.writer.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        writer.delete_edge(space, src, dst, edge_type)
    }

    fn batch_insert_edges(&mut self, space: &str, edges: Vec<Edge>) -> Result<(), StorageError> {
        let mut writer = self.writer.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        writer.batch_insert_edges(space, edges)
    }

    fn begin_transaction(&mut self, _space: &str) -> Result<TransactionId, StorageError> {
        // 事务管理由 RedbWriter 内部处理
        // 返回一个模拟的事务ID
        Ok(TransactionId::new(1))
    }

    fn commit_transaction(&mut self, _space: &str, _tx_id: TransactionId) -> Result<(), StorageError> {
        // 事务提交由 RedbWriter 内部处理
        Ok(())
    }

    fn rollback_transaction(&mut self, _space: &str, _tx_id: TransactionId) -> Result<(), StorageError> {
        // 事务回滚由 RedbWriter 内部处理
        Ok(())
    }

    fn create_space(&mut self, space: &SpaceInfo) -> Result<bool, StorageError> {
        let mut spaces = self.spaces.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if spaces.contains_key(&space.space_name) {
            return Ok(false);
        }
        spaces.insert(space.space_name.clone(), space.clone());
        
        let mut tags = self.tags.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        tags.insert(space.space_name.clone(), HashMap::new());
        
        let mut edge_type_infos = self.edge_type_infos.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        edge_type_infos.insert(space.space_name.clone(), HashMap::new());
        
        let mut tag_indexes = self.tag_indexes.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        tag_indexes.insert(space.space_name.clone(), HashMap::new());
        
        let mut edge_indexes = self.edge_indexes.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        edge_indexes.insert(space.space_name.clone(), HashMap::new());
        
        Ok(true)
    }

    fn drop_space(&mut self, space_name: &str) -> Result<bool, StorageError> {
        let mut spaces = self.spaces.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if !spaces.contains_key(space_name) {
            return Ok(false);
        }
        spaces.remove(space_name);
        
        let mut tags = self.tags.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        tags.remove(space_name);
        
        let mut edge_type_infos = self.edge_type_infos.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        edge_type_infos.remove(space_name);
        
        let mut tag_indexes = self.tag_indexes.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        tag_indexes.remove(space_name);
        
        let mut edge_indexes = self.edge_indexes.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        edge_indexes.remove(space_name);
        
        Ok(true)
    }

    fn get_space(&self, space_name: &str) -> Result<Option<SpaceInfo>, StorageError> {
        let spaces = self.spaces.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        Ok(spaces.get(space_name).cloned())
    }

    fn get_space_by_id(&self, space_id: i32) -> Result<Option<SpaceInfo>, StorageError> {
        let spaces = self.spaces.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        Ok(spaces.values().find(|s| s.space_id == space_id).cloned())
    }

    fn list_spaces(&self) -> Result<Vec<SpaceInfo>, StorageError> {
        let spaces = self.spaces.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        Ok(spaces.values().cloned().collect())
    }

    fn get_space_id(&self, space_name: &str) -> Result<i32, StorageError> {
        let spaces = self.spaces.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(space) = spaces.get(space_name) {
            Ok(space.space_id)
        } else {
            Err(StorageError::DbError(format!("Space '{}' not found", space_name)))
        }
    }

    fn space_exists(&self, space_name: &str) -> bool {
        let spaces = self.spaces.lock().unwrap();
        spaces.contains_key(space_name)
    }

    fn clear_space(&mut self, space_name: &str) -> Result<bool, StorageError> {
        let mut tags = self.tags.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        let mut edge_types = self.edge_type_infos.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        let mut tag_indexes = self.tag_indexes.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        let mut edge_indexes = self.edge_indexes.lock().map_err(|e| StorageError::DbError(e.to_string()))?;

        if !tags.contains_key(space_name) {
            return Err(StorageError::DbError(format!("Space '{}' not found", space_name)));
        }

        tags.insert(space_name.to_string(), HashMap::new());
        edge_types.insert(space_name.to_string(), HashMap::new());
        tag_indexes.insert(space_name.to_string(), HashMap::new());
        edge_indexes.insert(space_name.to_string(), HashMap::new());

        Ok(true)
    }

    fn alter_space_partition_num(&mut self, space_id: i32, partition_num: usize) -> Result<bool, StorageError> {
        let mut spaces = self.spaces.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        for space in spaces.values_mut() {
            if space.space_id == space_id {
                space.partition_num = partition_num as i32;
                return Ok(true);
            }
        }
        Err(StorageError::DbError(format!("Space with ID {} not found", space_id)))
    }

    fn alter_space_replica_factor(&mut self, space_id: i32, replica_factor: usize) -> Result<bool, StorageError> {
        let mut spaces = self.spaces.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        for space in spaces.values_mut() {
            if space.space_id == space_id {
                space.replica_factor = replica_factor as i32;
                return Ok(true);
            }
        }
        Err(StorageError::DbError(format!("Space with ID {} not found", space_id)))
    }

    fn alter_space_comment(&mut self, space_id: i32, comment: String) -> Result<bool, StorageError> {
        let mut spaces = self.spaces.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        for space in spaces.values_mut() {
            if space.space_id == space_id {
                space.comment = Some(comment);
                return Ok(true);
            }
        }
        Err(StorageError::DbError(format!("Space with ID {} not found", space_id)))
    }

    fn create_tag(&mut self, space: &str, info: &TagInfo) -> Result<bool, StorageError> {
        let mut tags = self.tags.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(space_tags) = tags.get_mut(space) {
            if space_tags.contains_key(&info.tag_name) {
                return Ok(false);
            }
            space_tags.insert(info.tag_name.clone(), info.clone());
            drop(tags);

            // 保存 schema 快照
            if let Ok(space_info) = self.get_space(space) {
                if let Some(space_id) = space_info.map(|s| s.space_id) {
                    let tags_list = self.list_tags(space).unwrap_or_default();
                    let edge_types = self.list_edge_types(space).unwrap_or_default();
                    let _ = self.extended_schema_manager.save_schema_snapshot(
                        space_id,
                        tags_list,
                        edge_types,
                        Some(format!("创建标签: {}", info.tag_name)),
                    );

                    // 记录 schema 变更
                    let change = SchemaChange {
                        change_type: SchemaChangeType::AddProperty,
                        target: info.tag_name.clone(),
                        property: None,
                        timestamp: chrono::Utc::now().timestamp_millis(),
                    };
                    let _ = self.extended_schema_manager.record_schema_change(space_id, change);
                }
            }

            Ok(true)
        } else {
            Err(StorageError::DbError(format!("Space '{}' not found", space)))
        }
    }

    fn alter_tag(&mut self, space_name: &str, tag_name: &str, additions: Vec<PropertyDef>, deletions: Vec<String>) -> Result<bool, StorageError> {
        let mut tags = self.tags.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(space_tags) = tags.get_mut(space_name) {
            if let Some(tag_info) = space_tags.get_mut(tag_name) {
                for prop in &additions {
                    tag_info.properties.retain(|p| p.name != prop.name);
                    tag_info.properties.push(prop.clone());
                }
                for prop_name in &deletions {
                    tag_info.properties.retain(|p| p.name != *prop_name);
                }
                drop(tags);

                // 保存 schema 快照
                if let Ok(space_info) = self.get_space(space_name) {
                    if let Some(space_id) = space_info.map(|s| s.space_id) {
                        let tags_list = self.list_tags(space_name).unwrap_or_default();
                        let edge_types = self.list_edge_types(space_name).unwrap_or_default();
                        let _ = self.extended_schema_manager.save_schema_snapshot(
                            space_id,
                            tags_list,
                            edge_types,
                            Some(format!("修改标签: {}", tag_name)),
                        );

                        // 记录属性添加变更
                        for prop in additions {
                            let change = SchemaChange {
                                change_type: SchemaChangeType::AddProperty,
                                target: format!("{}.{}", tag_name, prop.name),
                                property: Some(prop),
                                timestamp: chrono::Utc::now().timestamp_millis(),
                            };
                            let _ = self.extended_schema_manager.record_schema_change(space_id, change);
                        }

                        // 记录属性删除变更
                        for prop_name in deletions {
                            let change = SchemaChange {
                                change_type: SchemaChangeType::DropProperty,
                                target: format!("{}.{}", tag_name, prop_name),
                                property: None,
                                timestamp: chrono::Utc::now().timestamp_millis(),
                            };
                            let _ = self.extended_schema_manager.record_schema_change(space_id, change);
                        }
                    }
                }

                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            Err(StorageError::DbError(format!("Space '{}' not found", space_name)))
        }
    }

    fn get_tag(&self, space_name: &str, tag_name: &str) -> Result<Option<TagInfo>, StorageError> {
        let tags = self.tags.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(space_tags) = tags.get(space_name) {
            Ok(space_tags.get(tag_name).cloned())
        } else {
            Err(StorageError::DbError(format!("Space '{}' not found", space_name)))
        }
    }

    fn drop_tag(&mut self, space_name: &str, tag_name: &str) -> Result<bool, StorageError> {
        let mut tags = self.tags.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(space_tags) = tags.get_mut(space_name) {
            let removed = space_tags.remove(tag_name).is_some();
            drop(tags);

            // 保存 schema 快照
            if removed {
                if let Ok(space_info) = self.get_space(space_name) {
                    if let Some(space_id) = space_info.map(|s| s.space_id) {
                        let tags_list = self.list_tags(space_name).unwrap_or_default();
                        let edge_types = self.list_edge_types(space_name).unwrap_or_default();
                        let _ = self.extended_schema_manager.save_schema_snapshot(
                            space_id,
                            tags_list,
                            edge_types,
                            Some(format!("删除标签: {}", tag_name)),
                        );

                        // 记录 schema 变更
                        let change = SchemaChange {
                            change_type: SchemaChangeType::DropProperty,
                            target: tag_name.to_string(),
                            property: None,
                            timestamp: chrono::Utc::now().timestamp_millis(),
                        };
                        let _ = self.extended_schema_manager.record_schema_change(space_id, change);
                    }
                }
            }

            Ok(removed)
        } else {
            Err(StorageError::DbError(format!("Space '{}' not found", space_name)))
        }
    }

    fn list_tags(&self, space: &str) -> Result<Vec<TagInfo>, StorageError> {
        let tags = self.tags.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(space_tags) = tags.get(space) {
            Ok(space_tags.values().cloned().collect())
        } else {
            Err(StorageError::DbError(format!("Space '{}' not found", space)))
        }
    }

    fn create_edge_type(&mut self, space: &str, edge: &EdgeTypeInfo) -> Result<bool, StorageError> {
        let mut edge_type_infos = self.edge_type_infos.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(space_edge_types) = edge_type_infos.get_mut(space) {
            if space_edge_types.contains_key(&edge.edge_type_name) {
                return Ok(false);
            }
            space_edge_types.insert(edge.edge_type_name.clone(), edge.clone());
            drop(edge_type_infos);

            // 保存 schema 快照
            if let Ok(space_info) = self.get_space(space) {
                if let Some(space_id) = space_info.map(|s| s.space_id) {
                    let tags_list = self.list_tags(space).unwrap_or_default();
                    let edge_types = self.list_edge_types(space).unwrap_or_default();
                    let _ = self.extended_schema_manager.save_schema_snapshot(
                        space_id,
                        tags_list,
                        edge_types,
                        Some(format!("创建边类型: {}", edge.edge_type_name)),
                    );

                    // 记录 schema 变更
                    let change = SchemaChange {
                        change_type: SchemaChangeType::AddProperty,
                        target: edge.edge_type_name.clone(),
                        property: None,
                        timestamp: chrono::Utc::now().timestamp_millis(),
                    };
                    let _ = self.extended_schema_manager.record_schema_change(space_id, change);
                }
            }

            Ok(true)
        } else {
            Err(StorageError::DbError(format!("Space '{}' not found", space)))
        }
    }

    fn alter_edge_type(&mut self, space_name: &str, edge_type_name: &str, additions: Vec<PropertyDef>, deletions: Vec<String>) -> Result<bool, StorageError> {
        let mut edge_type_infos = self.edge_type_infos.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(space_edge_types) = edge_type_infos.get_mut(space_name) {
            if let Some(edge_type_info) = space_edge_types.get_mut(edge_type_name) {
                for prop in &additions {
                    edge_type_info.properties.retain(|p| p.name != prop.name);
                    edge_type_info.properties.push(prop.clone());
                }
                for prop_name in &deletions {
                    edge_type_info.properties.retain(|p| p.name != *prop_name);
                }
                drop(edge_type_infos);

                // 保存 schema 快照
                if let Ok(space_info) = self.get_space(space_name) {
                    if let Some(space_id) = space_info.map(|s| s.space_id) {
                        let tags_list = self.list_tags(space_name).unwrap_or_default();
                        let edge_types = self.list_edge_types(space_name).unwrap_or_default();
                        let _ = self.extended_schema_manager.save_schema_snapshot(
                            space_id,
                            tags_list,
                            edge_types,
                            Some(format!("修改边类型: {}", edge_type_name)),
                        );

                        // 记录属性添加变更
                        for prop in additions {
                            let change = SchemaChange {
                                change_type: SchemaChangeType::AddProperty,
                                target: format!("{}.{}", edge_type_name, prop.name),
                                property: Some(prop),
                                timestamp: chrono::Utc::now().timestamp_millis(),
                            };
                            let _ = self.extended_schema_manager.record_schema_change(space_id, change);
                        }

                        // 记录属性删除变更
                        for prop_name in deletions {
                            let change = SchemaChange {
                                change_type: SchemaChangeType::DropProperty,
                                target: format!("{}.{}", edge_type_name, prop_name),
                                property: None,
                                timestamp: chrono::Utc::now().timestamp_millis(),
                            };
                            let _ = self.extended_schema_manager.record_schema_change(space_id, change);
                        }
                    }
                }

                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            Err(StorageError::DbError(format!("Space '{}' not found", space_name)))
        }
    }

    fn get_edge_type(&self, space_name: &str, edge_type_name: &str) -> Result<Option<EdgeTypeInfo>, StorageError> {
        let edge_type_infos = self.edge_type_infos.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(space_edge_types) = edge_type_infos.get(space_name) {
            Ok(space_edge_types.get(edge_type_name).cloned())
        } else {
            Err(StorageError::DbError(format!("Space '{}' not found", space_name)))
        }
    }

    fn drop_edge_type(&mut self, space_name: &str, edge_type_name: &str) -> Result<bool, StorageError> {
        let mut edge_type_infos = self.edge_type_infos.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(space_edge_types) = edge_type_infos.get_mut(space_name) {
            let removed = space_edge_types.remove(edge_type_name).is_some();
            drop(edge_type_infos);

            // 保存 schema 快照
            if removed {
                if let Ok(space_info) = self.get_space(space_name) {
                    if let Some(space_id) = space_info.map(|s| s.space_id) {
                        let tags_list = self.list_tags(space_name).unwrap_or_default();
                        let edge_types = self.list_edge_types(space_name).unwrap_or_default();
                        let _ = self.extended_schema_manager.save_schema_snapshot(
                            space_id,
                            tags_list,
                            edge_types,
                            Some(format!("删除边类型: {}", edge_type_name)),
                        );

                        // 记录 schema 变更
                        let change = SchemaChange {
                            change_type: SchemaChangeType::DropProperty,
                            target: edge_type_name.to_string(),
                            property: None,
                            timestamp: chrono::Utc::now().timestamp_millis(),
                        };
                        let _ = self.extended_schema_manager.record_schema_change(space_id, change);
                    }
                }
            }

            Ok(removed)
        } else {
            Err(StorageError::DbError(format!("Space '{}' not found", space_name)))
        }
    }

    fn list_edge_types(&self, space: &str) -> Result<Vec<EdgeTypeInfo>, StorageError> {
        let edge_type_infos = self.edge_type_infos.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(space_edge_types) = edge_type_infos.get(space) {
            Ok(space_edge_types.values().cloned().collect())
        } else {
            Err(StorageError::DbError(format!("Space '{}' not found", space)))
        }
    }

    fn create_tag_index(&mut self, space: &str, info: &Index) -> Result<bool, StorageError> {
        let mut tag_indexes = self.tag_indexes.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(space_indexes) = tag_indexes.get_mut(space) {
            if space_indexes.contains_key(&info.name) {
                return Ok(false);
            }
            space_indexes.insert(info.name.clone(), info.clone());
            Ok(true)
        } else {
            Err(StorageError::DbError(format!("Space '{}' not found", space)))
        }
    }

    fn drop_tag_index(&mut self, space: &str, index: &str) -> Result<bool, StorageError> {
        let mut tag_indexes = self.tag_indexes.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(space_indexes) = tag_indexes.get_mut(space) {
            Ok(space_indexes.remove(index).is_some())
        } else {
            Err(StorageError::DbError(format!("Space '{}' not found", space)))
        }
    }

    fn get_tag_index(&self, space: &str, index: &str) -> Result<Option<Index>, StorageError> {
        let tag_indexes = self.tag_indexes.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(space_indexes) = tag_indexes.get(space) {
            Ok(space_indexes.get(index).cloned())
        } else {
            Err(StorageError::DbError(format!("Space '{}' not found", space)))
        }
    }

    fn list_tag_indexes(&self, space: &str) -> Result<Vec<Index>, StorageError> {
        let tag_indexes = self.tag_indexes.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(space_indexes) = tag_indexes.get(space) {
            Ok(space_indexes.values().cloned().collect())
        } else {
            Err(StorageError::DbError(format!("Space '{}' not found", space)))
        }
    }

    fn rebuild_tag_index(&mut self, _space: &str, _index: &str) -> Result<bool, StorageError> {
        Ok(true)
    }

    fn create_edge_index(&mut self, space: &str, info: &Index) -> Result<bool, StorageError> {
        let mut edge_indexes = self.edge_indexes.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(space_indexes) = edge_indexes.get_mut(space) {
            if space_indexes.contains_key(&info.name) {
                return Ok(false);
            }
            space_indexes.insert(info.name.clone(), info.clone());
            Ok(true)
        } else {
            Err(StorageError::DbError(format!("Space '{}' not found", space)))
        }
    }

    fn drop_edge_index(&mut self, space: &str, index: &str) -> Result<bool, StorageError> {
        let mut edge_indexes = self.edge_indexes.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(space_indexes) = edge_indexes.get_mut(space) {
            Ok(space_indexes.remove(index).is_some())
        } else {
            Err(StorageError::DbError(format!("Space '{}' not found", space)))
        }
    }

    fn get_edge_index(&self, space: &str, index: &str) -> Result<Option<Index>, StorageError> {
        let edge_indexes = self.edge_indexes.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(space_indexes) = edge_indexes.get(space) {
            Ok(space_indexes.get(index).cloned())
        } else {
            Err(StorageError::DbError(format!("Space '{}' not found", space)))
        }
    }

    fn list_edge_indexes(&self, space: &str) -> Result<Vec<Index>, StorageError> {
        let edge_indexes = self.edge_indexes.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(space_indexes) = edge_indexes.get(space) {
            Ok(space_indexes.values().cloned().collect())
        } else {
            Err(StorageError::DbError(format!("Space '{}' not found", space)))
        }
    }

    fn rebuild_edge_index(&mut self, space: &str, index_name: &str) -> Result<bool, StorageError> {
        // 获取索引信息
        let edge_indexes = self.edge_indexes.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        let index = edge_indexes.get(space)
            .and_then(|space_indexes| space_indexes.get(index_name))
            .cloned()
            .ok_or_else(|| StorageError::DbError(format!("Edge index '{}' not found in space '{}'", index_name, space)))?;
        
        // 清除现有索引数据
        self.index_manager.clear_edge_index(space, index_name)?;
        
        // 重新扫描所有边并重建索引
        let edges = self.reader.scan_all_edges(space)?;
        for edge in edges {
            self.index_manager.build_edge_index_entry(space, &index, &edge)?;
        }
        
        Ok(true)
    }

    fn insert_vertex_data(&mut self, space: &str, info: &InsertVertexInfo) -> Result<bool, StorageError> {
        // 获取标签信息
        let tag_name = info.tag_name.clone();
        {
            let tags = self.tags.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
            let space_tags = tags.get(space)
                .ok_or_else(|| StorageError::DbError(format!("Space '{}' not found", space)))?;
            let _tag_info = space_tags.get(&tag_name)
                .ok_or_else(|| StorageError::DbError(format!("Tag '{}' not found in space '{}'", tag_name, space)))?;
        }
        
        // 构建顶点属性映射
        let mut properties = std::collections::HashMap::new();
        for (prop_name, prop_value) in &info.props {
            properties.insert(prop_name.clone(), prop_value.clone());
        }
        
        // 创建标签
        let tag = crate::core::vertex_edge_path::Tag {
            name: tag_name.clone(),
            properties,
        };
        
        // 获取或创建顶点
        let vertex = match self.reader.get_vertex(space, &info.vertex_id)? {
            Some(mut existing_vertex) => {
                // 更新现有顶点
                existing_vertex.tags.retain(|t| t.name != tag_name);
                existing_vertex.tags.push(tag);
                existing_vertex
            }
            None => {
                // 创建新顶点
                crate::core::Vertex {
                    vid: Box::new(info.vertex_id.clone()),
                    id: 0,
                    tags: vec![tag],
                    properties: std::collections::HashMap::new(),
                }
            }
        };
        
        // 使用 VertexWriter 插入顶点
        {
            let mut writer = self.writer.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
            writer.update_vertex(space, vertex)?;
        }
        
        // 更新索引
        self.index_manager.update_vertex_indexes(space, &info.vertex_id, &tag_name, &info.props)?;
        
        Ok(true)
    }

    fn insert_edge_data(&mut self, space: &str, info: &InsertEdgeInfo) -> Result<bool, StorageError> {
        // 获取边类型信息
        let edge_name = info.edge_name.clone();
        let src_vertex_id = info.src_vertex_id.clone();
        let dst_vertex_id = info.dst_vertex_id.clone();
        let rank = info.rank;
        let props = info.props.clone();
        
        {
            let edge_type_infos = self.edge_type_infos.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
            let space_edge_types = edge_type_infos.get(space)
                .ok_or_else(|| StorageError::DbError(format!("Space '{}' not found", space)))?;
            let _edge_type_info = space_edge_types.get(&edge_name)
                .ok_or_else(|| StorageError::DbError(format!("Edge type '{}' not found in space '{}'", edge_name, space)))?;
        }
        
        // 构建边属性映射
        let mut properties = std::collections::HashMap::new();
        for (prop_name, prop_value) in &props {
            properties.insert(prop_name.clone(), prop_value.clone());
        }
        
        // 创建边
        let edge = crate::core::Edge {
            src: Box::new(src_vertex_id.clone()),
            dst: Box::new(dst_vertex_id.clone()),
            edge_type: edge_name.clone(),
            ranking: rank,
            id: 0,
            props: properties,
        };
        
        // 使用 EdgeWriter 插入边
        {
            let mut writer = self.writer.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
            writer.insert_edge(space, edge)?;
        }
        
        // 更新边索引
        self.index_manager.update_edge_indexes(
            space, 
            &src_vertex_id, 
            &dst_vertex_id, 
            &edge_name, 
            &props
        )?;
        
        Ok(true)
    }

    fn delete_vertex_data(&mut self, space: &str, vertex_id: &str) -> Result<bool, StorageError> {
        // 解析顶点ID
        let vid = self.parse_vertex_id(vertex_id)?;
        
        // 首先删除所有相关的边
        self.delete_vertex_edges(space, &vid)?;
        
        // 删除顶点索引
        self.index_manager.delete_vertex_indexes(space, &vid)?;
        
        // 删除顶点本身
        {
            let mut writer = self.writer.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
            writer.delete_vertex(space, &vid)?;
        }
        
        Ok(true)
    }

    fn delete_edge_data(&mut self, space: &str, src: &str, dst: &str, rank: i64) -> Result<bool, StorageError> {
        // 解析顶点ID
        let src_id = self.parse_vertex_id(src)?;
        let dst_id = self.parse_vertex_id(dst)?;
        
        // 扫描找到匹配的边
        let edges = self.reader.scan_all_edges(space)?;
        let mut deleted = false;
        
        for edge in edges {
            if *edge.src == src_id && *edge.dst == dst_id && edge.ranking == rank {
                {
                    let mut writer = self.writer.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
                    writer.delete_edge(space, &edge.src, &edge.dst, &edge.edge_type)?;
                }
                self.index_manager.delete_edge_indexes(space, &edge.src, &edge.dst, &edge.edge_type)?;
                deleted = true;
                break;
            }
        }
        
        Ok(deleted)
    }

    fn update_data(&mut self, space: &str, info: &UpdateInfo) -> Result<bool, StorageError> {
        self.update_vertex_property(space, &info.update_target.id, &info.update_target.label, &info.update_target.prop, &info.update_op, &info.value)?;
        Ok(true)
    }

    fn change_password(&mut self, info: &PasswordInfo) -> Result<bool, StorageError> {
        let mut users = self.users.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        let username = info.username.clone().ok_or_else(|| StorageError::DbError("用户名不能为空".to_string()))?;
        if let Some(user) = users.get_mut(&username) {
            user.password = info.new_password.clone();
            Ok(true)
        } else {
            Err(StorageError::DbError(format!("用户 {} 不存在", username)))
        }
    }

    fn create_user(&mut self, info: &UserInfo) -> Result<bool, StorageError> {
        let mut users = self.users.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if users.contains_key(&info.username) {
            return Err(StorageError::DbError(format!("用户 {} 已存在", info.username)));
        }
        users.insert(info.username.clone(), info.clone());
        Ok(true)
    }

    fn alter_user(&mut self, info: &UserAlterInfo) -> Result<bool, StorageError> {
        let mut users = self.users.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(user) = users.get_mut(&info.username) {
            if let Some(new_role) = &info.new_role {
                user.role = new_role.clone();
            }
            if let Some(is_locked) = info.is_locked {
                user.is_locked = is_locked;
            }
            Ok(true)
        } else {
            Err(StorageError::DbError(format!("用户 {} 不存在", info.username)))
        }
    }

    fn drop_user(&mut self, username: &str) -> Result<bool, StorageError> {
        let mut users = self.users.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        users.remove(username);
        Ok(true)
    }

    fn grant_role(&mut self, username: &str, space_id: i32, role: RoleType) -> Result<bool, StorageError> {
        let mut users = self.users.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(user) = users.get_mut(username) {
            if !user.roles.contains_key(&space_id) {
                user.roles.insert(space_id, format!("{:?}", role));
                Ok(true)
            } else {
                Err(StorageError::DbError(format!("User {} already has a role in space {}", username, space_id)))
            }
        } else {
            Err(StorageError::DbError(format!("User {} not found", username)))
        }
    }

    fn revoke_role(&mut self, username: &str, space_id: i32) -> Result<bool, StorageError> {
        let mut users = self.users.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(user) = users.get_mut(username) {
            if user.roles.remove(&space_id).is_some() {
                Ok(true)
            } else {
                Err(StorageError::DbError(format!("User {} does not have a role in space {}", username, space_id)))
            }
        } else {
            Err(StorageError::DbError(format!("User {} not found", username)))
        }
    }

    fn lookup_index(&self, space: &str, index_name: &str, value: &Value) -> Result<Vec<Value>, StorageError> {
        let results = self.lookup_index_with_score(space, index_name, value)?;
        Ok(results.into_iter().map(|(v, _)| v).collect())
    }

    fn lookup_index_with_score(&self, space: &str, index_name: &str, value: &Value) -> Result<Vec<(Value, f32)>, StorageError> {
        // 获取索引信息
        let tag_indexes = self.tag_indexes.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        let edge_indexes = self.edge_indexes.lock().map_err(|e| StorageError::DbError(e.to_string()))?;

        let mut results = Vec::new();

        // 检查标签索引
        if let Some(space_tag_indexes) = tag_indexes.get(space) {
            if let Some(index) = space_tag_indexes.get(index_name) {
                let indexed_values = self.index_manager.lookup_tag_index(space, index, value)?;
                results.extend(indexed_values.into_iter().map(|v| (v, 1.0f32)));
            }
        }

        // 检查边索引
        if let Some(space_edge_indexes) = edge_indexes.get(space) {
            if let Some(index) = space_edge_indexes.get(index_name) {
                let indexed_values = self.index_manager.lookup_edge_index(space, index, value)?;
                results.extend(indexed_values.into_iter().map(|v| (v, 1.0f32)));
            }
        }
        
        Ok(results)
    }

    fn get_vertex_with_schema(&self, space: &str, tag: &str, id: &Value) -> Result<Option<(Schema, Vec<u8>)>, StorageError> {
        // 获取顶点
        if let Some(vertex) = self.reader.get_vertex(space, id)? {
            // 获取标签信息
            let tags = self.tags.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
            if let Some(space_tags) = tags.get(space) {
                if let Some(tag_info) = space_tags.get(tag) {
                    // 构建schema
                    let schema = self.build_vertex_schema(tag_info)?;
                    
                    // 序列化顶点数据
                    let vertex_data = vertex_to_bytes(&vertex)?;
                    
                    return Ok(Some((schema, vertex_data)));
                }
            }
        }
        Ok(None)
    }

    fn get_edge_with_schema(&self, space: &str, edge_type: &str, src: &Value, dst: &Value) -> Result<Option<(Schema, Vec<u8>)>, StorageError> {
        // 获取边
        if let Some(edge) = self.reader.get_edge(space, src, dst, edge_type)? {
            // 获取边类型信息
            let edge_type_infos = self.edge_type_infos.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
            if let Some(space_edge_types) = edge_type_infos.get(space) {
                if let Some(edge_type_info) = space_edge_types.get(edge_type) {
                    // 构建schema
                    let schema = self.build_edge_schema(edge_type_info)?;
                    
                    // 序列化边数据
                    let edge_data = edge_to_bytes(&edge)?;
                    
                    return Ok(Some((schema, edge_data)));
                }
            }
        }
        Ok(None)
    }

    fn scan_vertices_with_schema(&self, space: &str, tag: &str) -> Result<Vec<(Schema, Vec<u8>)>, StorageError> {
        let mut results = Vec::new();
        
        // 获取标签信息
        let tags = self.tags.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        let tag_info = tags.get(space)
            .and_then(|space_tags| space_tags.get(tag))
            .ok_or_else(|| StorageError::DbError(format!("Tag '{}' not found in space '{}'", tag, space)))?;
        
        // 构建schema
        let schema = self.build_vertex_schema(tag_info)?;
        
        // 扫描所有顶点并过滤
        let vertices = self.reader.scan_vertices(space)?;
        for vertex in vertices {
            if vertex.tags.iter().any(|t| t.name == tag) {
                let vertex_data = vertex_to_bytes(&vertex)?;
                results.push((schema.clone(), vertex_data));
            }
        }
        
        Ok(results)
    }

    fn scan_edges_with_schema(&self, space: &str, edge_type: &str) -> Result<Vec<(Schema, Vec<u8>)>, StorageError> {
        let mut results = Vec::new();
        
        // 获取边类型信息
        let edge_type_infos = self.edge_type_infos.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        let edge_type_info = edge_type_infos.get(space)
            .and_then(|space_edge_types| space_edge_types.get(edge_type))
            .ok_or_else(|| StorageError::DbError(format!("Edge type '{}' not found in space '{}'", edge_type, space)))?;
        
        // 构建schema
        let schema = self.build_edge_schema(edge_type_info)?;
        
        // 扫描所有边并过滤
        let edges = self.reader.scan_edges_by_type(space, edge_type)?;
        for edge in edges {
            let edge_data = edge_to_bytes(&edge)?;
            results.push((schema.clone(), edge_data));
        }
        
        Ok(results)
    }

    fn load_from_disk(&mut self) -> Result<(), StorageError> {
        // Redb 引擎自动从磁盘加载数据
        // 加载元数据
        self.metadata_manager.load_metadata()?;
        
        Ok(())
    }

    fn save_to_disk(&self) -> Result<(), StorageError> {
        // Redb 引擎自动将数据保存到磁盘
        // 保存元数据
        self.metadata_manager.save_metadata()?;
        
        Ok(())
    }

    fn get_storage_stats(&self) -> crate::storage::storage_client::StorageStats {
        let total_spaces = self.spaces.lock().map(|s| s.len()).unwrap_or(0);
        let mut total_tags = 0;
        let mut total_edge_types = 0;
        
        if let Ok(tags) = self.tags.lock() {
            for space_tags in tags.values() {
                total_tags += space_tags.len();
            }
        }
        
        if let Ok(edge_types) = self.edge_type_infos.lock() {
            for space_edge_types in edge_types.values() {
                total_edge_types += space_edge_types.len();
            }
        }
        
        // 使用 reader 统计顶点数量
        let total_vertices = self.reader.scan_vertices("")
            .map(|r| r.into_vec().len())
            .unwrap_or(0);
        let total_edges = self.reader.scan_all_edges("")
            .map(|r| r.into_vec().len())
            .unwrap_or(0);
        
        crate::storage::storage_client::StorageStats {
            total_vertices,
            total_edges,
            total_spaces,
            total_tags,
            total_edge_types,
        }
    }
}

pub type DefaultStorage = RedbStorage;
