use crate::core::types::{InsertVertexInfo, PropertyDef, TagInfo, UpdateInfo, UpdateOp};
use crate::core::{StorageError, Value, Vertex};
use crate::storage::index::{IndexDataManager, RedbIndexDataManager};
use crate::storage::metadata::{
    IndexMetadataManager, RedbIndexMetadataManager, RedbSchemaManager, SchemaManager,
};
use crate::storage::operations::{RedbReader, RedbWriter, VertexReader, VertexWriter};
use crate::storage::Schema;
use parking_lot::Mutex;
use redb::Database;
use std::sync::Arc;

/// 顶点存储管理器
/// 
/// 负责顶点的增删改查以及标签管理
#[derive(Clone)]
pub struct VertexStorage {
    reader: Arc<Mutex<RedbReader>>,
    writer: Arc<Mutex<RedbWriter>>,
    index_data_manager: RedbIndexDataManager,
    schema_manager: Arc<RedbSchemaManager>,
    index_metadata_manager: Arc<RedbIndexMetadataManager>,
}

impl std::fmt::Debug for VertexStorage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VertexStorage").finish()
    }
}

impl VertexStorage {
    /// 创建新的顶点存储实例
    pub fn new(
        db: Arc<Database>,
        reader: Arc<Mutex<RedbReader>>,
        writer: Arc<Mutex<RedbWriter>>,
        schema_manager: Arc<RedbSchemaManager>,
        index_metadata_manager: Arc<RedbIndexMetadataManager>,
    ) -> Result<Self, StorageError> {
        let index_data_manager = RedbIndexDataManager::new(db);

        Ok(Self {
            reader,
            writer,
            index_data_manager,
            schema_manager,
            index_metadata_manager,
        })
    }

    /// 获取单个顶点
    pub fn get_vertex(&self, space: &str, id: &Value) -> Result<Option<Vertex>, StorageError> {
        self.reader.lock().get_vertex(space, id)
    }

    /// 扫描所有顶点
    pub fn scan_vertices(&self, space: &str) -> Result<Vec<Vertex>, StorageError> {
        self.reader.lock().scan_vertices(space).map(|r| r.into_vec())
    }

    /// 按标签扫描顶点
    pub fn scan_vertices_by_tag(&self, space: &str, tag: &str) -> Result<Vec<Vertex>, StorageError> {
        self.reader
            .lock()
            .scan_vertices_by_tag(space, tag)
            .map(|r| r.into_vec())
    }

    /// 按属性扫描顶点
    pub fn scan_vertices_by_prop(
        &self,
        space: &str,
        tag: &str,
        prop: &str,
        value: &Value,
    ) -> Result<Vec<Vertex>, StorageError> {
        self.reader
            .lock()
            .scan_vertices_by_prop(space, tag, prop, value)
            .map(|r| r.into_vec())
    }

    /// 插入顶点
    pub fn insert_vertex(
        &self,
        space: &str,
        space_id: u64,
        vertex: Vertex,
    ) -> Result<Value, StorageError> {
        let id = {
            let mut writer = self.writer.lock();
            writer.insert_vertex(space, vertex.clone())?
        };

        // 更新索引
        for tag in &vertex.tags {
            let indexes = self.index_metadata_manager.list_tag_indexes(space_id)?;

            for index in indexes {
                if index.schema_name == tag.name {
                    let mut index_props = Vec::new();
                    for field in &index.fields {
                        if let Some(value) = tag.properties.get(&field.name) {
                            index_props.push((field.name.clone(), value.clone()));
                        }
                    }

                    if !index_props.is_empty() {
                        self.index_data_manager.update_vertex_indexes(
                            space_id,
                            &id,
                            &index.name,
                            &index_props,
                        )?;
                    }
                }
            }
        }

        Ok(id)
    }

    /// 更新顶点
    pub fn update_vertex(&self, space: &str, vertex: Vertex) -> Result<(), StorageError> {
        let mut writer = self.writer.lock();
        writer.update_vertex(space, vertex)
    }

    /// 删除顶点
    pub fn delete_vertex(&self, space: &str, space_id: u64, id: &Value) -> Result<(), StorageError> {
        {
            let mut writer = self.writer.lock();
            writer.delete_vertex(space, id)?;
        }

        // 删除索引
        self.index_data_manager
            .delete_vertex_indexes(space_id, id)?;

        Ok(())
    }

    /// 批量插入顶点
    pub fn batch_insert_vertices(
        &self,
        space: &str,
        vertices: Vec<Vertex>,
    ) -> Result<Vec<Value>, StorageError> {
        let mut writer = self.writer.lock();
        writer.batch_insert_vertices(space, vertices)
    }

    /// 删除顶点上的指定标签
    pub fn delete_tags(
        &self,
        space: &str,
        space_id: u64,
        vertex_id: &Value,
        tag_names: &[String],
    ) -> Result<usize, StorageError> {
        let deleted_count = {
            let mut writer = self.writer.lock();
            writer.delete_tags(space, vertex_id, tag_names)?
        };

        // 删除相关索引
        for tag_name in tag_names {
            self.index_data_manager
                .delete_tag_indexes(space_id, vertex_id, tag_name)?;
        }

        Ok(deleted_count)
    }

    /// 插入顶点数据（高级接口）
    pub fn insert_vertex_data(
        &self,
        space: &str,
        space_id: u64,
        info: &InsertVertexInfo,
    ) -> Result<bool, StorageError> {
        // 获取标签信息
        let tag_name = info.tag_name.clone();
        let _tag_info = self
            .schema_manager
            .get_tag(space, &tag_name)?
            .ok_or_else(|| {
                StorageError::DbError(format!("Tag '{}' not found in space '{}'", tag_name, space))
            })?;

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
        let vertex = match self.reader.lock().get_vertex(space, &info.vertex_id)? {
            Some(mut existing_vertex) => {
                existing_vertex.tags.retain(|t| t.name != tag_name);
                existing_vertex.tags.push(tag);
                existing_vertex
            }
            None => {
                crate::core::Vertex {
                    vid: Box::new(info.vertex_id.clone()),
                    id: 0,
                    tags: vec![tag],
                    properties: std::collections::HashMap::new(),
                }
            }
        };

        // 插入顶点
        {
            let mut writer = self.writer.lock();
            writer.update_vertex(space, vertex)?;
        }

        // 更新索引
        self.index_data_manager.update_vertex_indexes(
            space_id,
            &info.vertex_id,
            &tag_name,
            &info.props,
        )?;

        Ok(true)
    }

    /// 删除顶点数据（高级接口）
    pub fn delete_vertex_data(
        &self,
        space: &str,
        space_id: u64,
        vertex_id: &Value,
    ) -> Result<bool, StorageError> {
        // 删除顶点索引
        self.index_data_manager
            .delete_vertex_indexes(space_id, vertex_id)?;

        // 删除顶点本身
        let mut writer = self.writer.lock();
        writer.delete_vertex(space, vertex_id)?;

        Ok(true)
    }

    /// 更新顶点属性
    pub fn update_data(&self, space: &str, info: &UpdateInfo) -> Result<bool, StorageError> {
        self.update_vertex_property(
            space,
            &info.update_target.id,
            &info.update_target.label,
            &info.update_target.prop,
            &info.update_op,
            &info.value,
        )?;
        Ok(true)
    }

    /// 更新顶点属性（内部方法）
    fn update_vertex_property(
        &self,
        space: &str,
        vertex_id: &Value,
        tag: &str,
        prop: &str,
        op: &UpdateOp,
        value: &Value,
    ) -> Result<(), StorageError> {
        if let Some(mut vertex) = self.reader.lock().get_vertex(space, vertex_id)? {
            for tag_data in &mut vertex.tags {
                if tag_data.name == tag {
                    match op {
                        UpdateOp::Set => {
                            tag_data.properties.insert(prop.to_string(), value.clone());
                        }
                        UpdateOp::Add => {
                            if let Some(existing) = tag_data.properties.get(prop) {
                                if let (Value::Int(a), Value::Int(b)) = (existing, value) {
                                    tag_data
                                        .properties
                                        .insert(prop.to_string(), Value::Int(a + b));
                                }
                            }
                        }
                        UpdateOp::Subtract => {
                            if let Some(existing) = tag_data.properties.get(prop) {
                                if let (Value::Int(a), Value::Int(b)) = (existing, value) {
                                    tag_data
                                        .properties
                                        .insert(prop.to_string(), Value::Int(a - b));
                                }
                            }
                        }
                        _ => {}
                    }
                    break;
                }
            }
            let mut writer = self.writer.lock();
            writer.update_vertex(space, vertex)?;
        }
        Ok(())
    }

    /// 构建顶点 schema
    pub fn build_vertex_schema(&self, tag_info: &TagInfo) -> Result<Schema, StorageError> {
        let mut schema = Schema::new(tag_info.tag_name.clone(), 1);
        for prop in &tag_info.properties {
            let field_def = crate::storage::types::FieldDef {
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

    /// 获取带 schema 的顶点
    pub fn get_vertex_with_schema(
        &self,
        space: &str,
        tag: &str,
        id: &Value,
    ) -> Result<Option<(Schema, Vec<u8>)>, StorageError> {
        use bincode::{config::standard, encode_to_vec};

        if let Some(vertex) = self.reader.lock().get_vertex(space, id)? {
            let tag_info = self.schema_manager.get_tag(space, tag)?.ok_or_else(|| {
                StorageError::DbError(format!("Tag '{}' not found in space '{}'", tag, space))
            })?;
            let schema = self.build_vertex_schema(&tag_info)?;
            let vertex_data = encode_to_vec(&vertex, standard())?;
            return Ok(Some((schema, vertex_data)));
        }
        Ok(None)
    }

    /// 扫描带 schema 的顶点
    pub fn scan_vertices_with_schema(
        &self,
        space: &str,
        tag: &str,
    ) -> Result<Vec<(Schema, Vec<u8>)>, StorageError> {
        use bincode::{config::standard, encode_to_vec};

        let mut results = Vec::new();
        let tag_info = self.schema_manager.get_tag(space, tag)?.ok_or_else(|| {
            StorageError::DbError(format!("Tag '{}' not found in space '{}'", tag, space))
        })?;
        let schema = self.build_vertex_schema(&tag_info)?;

        let vertices = self.reader.lock().scan_vertices(space)?;
        for vertex in vertices {
            if vertex.tags.iter().any(|t| t.name == tag) {
                let vertex_data = encode_to_vec(&vertex, standard())?;
                results.push((schema.clone(), vertex_data));
            }
        }

        Ok(results)
    }
}
