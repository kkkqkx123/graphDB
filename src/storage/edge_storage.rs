use crate::core::types::{EdgeTypeInfo, InsertEdgeInfo};
use crate::core::{Edge, EdgeDirection, StorageError, Value};
use crate::storage::index::{IndexDataManager, RedbIndexDataManager};
use crate::storage::metadata::{
    IndexMetadataManager, RedbIndexMetadataManager, RedbSchemaManager, SchemaManager,
};
use crate::storage::operations::{EdgeReader, EdgeWriter, RedbReader, RedbWriter, VertexReader};
use crate::storage::Schema;
use parking_lot::Mutex;
use redb::Database;
use std::sync::Arc;

/// 边存储管理器
///
/// 负责边的增删改查以及悬挂边检测修复
#[derive(Clone)]
pub struct EdgeStorage {
    reader: Arc<Mutex<RedbReader>>,
    writer: Arc<Mutex<RedbWriter>>,
    index_data_manager: RedbIndexDataManager,
    schema_manager: Arc<RedbSchemaManager>,
    index_metadata_manager: Arc<RedbIndexMetadataManager>,
}

impl std::fmt::Debug for EdgeStorage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EdgeStorage").finish()
    }
}

impl EdgeStorage {
    /// 创建新的边存储实例
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

    /// 获取单条边
    pub fn get_edge(
        &self,
        space: &str,
        src: &Value,
        dst: &Value,
        edge_type: &str,
    ) -> Result<Option<Edge>, StorageError> {
        self.reader.lock().get_edge(space, src, dst, edge_type)
    }

    /// 获取节点的边
    pub fn get_node_edges(
        &self,
        space: &str,
        node_id: &Value,
        direction: EdgeDirection,
    ) -> Result<Vec<Edge>, StorageError> {
        self.reader
            .lock()
            .get_node_edges(space, node_id, direction)
            .map(|r| r.into_vec())
    }

    /// 获取节点的边（带过滤）
    pub fn get_node_edges_filtered<F>(
        &self,
        space: &str,
        node_id: &Value,
        direction: EdgeDirection,
        filter: Option<F>,
    ) -> Result<Vec<Edge>, StorageError>
    where
        F: Fn(&Edge) -> bool,
    {
        self.reader
            .lock()
            .get_node_edges_filtered(space, node_id, direction, filter)
            .map(|r| r.into_vec())
    }

    /// 按类型扫描边
    pub fn scan_edges_by_type(
        &self,
        space: &str,
        edge_type: &str,
    ) -> Result<Vec<Edge>, StorageError> {
        self.reader
            .lock()
            .scan_edges_by_type(space, edge_type)
            .map(|r| r.into_vec())
    }

    /// 扫描所有边
    pub fn scan_all_edges(&self, space: &str) -> Result<Vec<Edge>, StorageError> {
        self.reader
            .lock()
            .scan_all_edges(space)
            .map(|r| r.into_vec())
    }

    /// 插入边
    pub fn insert_edge(&self, space: &str, space_id: u64, edge: Edge) -> Result<(), StorageError> {
        {
            let mut writer = self.writer.lock();
            writer.insert_edge(space, edge.clone())?;
        }

        // 更新索引
        let indexes = self.index_metadata_manager.list_edge_indexes(space_id)?;

        for index in indexes {
            if index.schema_name == edge.edge_type {
                let mut index_props = Vec::new();
                for field in &index.fields {
                    if let Some(value) = edge.props.get(&field.name) {
                        index_props.push((field.name.clone(), value.clone()));
                    }
                }

                if !index_props.is_empty() {
                    self.index_data_manager.update_edge_indexes(
                        space_id,
                        &edge.src,
                        &edge.dst,
                        &index.name,
                        &index_props,
                    )?;
                }
            }
        }

        Ok(())
    }

    /// 删除边
    pub fn delete_edge(
        &self,
        space: &str,
        space_id: u64,
        src: &Value,
        dst: &Value,
        edge_type: &str,
    ) -> Result<(), StorageError> {
        {
            let mut writer = self.writer.lock();
            writer.delete_edge(space, src, dst, edge_type)?;
        }

        // 删除索引
        let indexes = self.index_metadata_manager.list_edge_indexes(space_id)?;
        let index_names: Vec<String> = indexes
            .into_iter()
            .filter(|idx| idx.schema_name == edge_type)
            .map(|idx| idx.name)
            .collect();
        self.index_data_manager
            .delete_edge_indexes(space_id, src, dst, &index_names)?;

        Ok(())
    }

    /// 批量插入边
    pub fn batch_insert_edges(&self, space: &str, edges: Vec<Edge>) -> Result<(), StorageError> {
        let mut writer = self.writer.lock();
        for edge in edges {
            writer.insert_edge(space, edge)?;
        }
        Ok(())
    }

    /// 删除与顶点相关的所有边
    pub fn delete_vertex_edges(
        &self,
        space: &str,
        space_id: u64,
        vertex_id: &Value,
    ) -> Result<(), StorageError> {
        let edges = self.reader.lock().scan_all_edges(space)?;

        for edge in edges {
            if *edge.src == *vertex_id || *edge.dst == *vertex_id {
                {
                    let mut writer = self.writer.lock();
                    writer.delete_edge(space, &edge.src, &edge.dst, &edge.edge_type)?;
                }
                let indexes = self.index_metadata_manager.list_edge_indexes(space_id)?;
                let index_names: Vec<String> = indexes
                    .into_iter()
                    .filter(|idx| idx.schema_name == edge.edge_type)
                    .map(|idx| idx.name)
                    .collect();
                self.index_data_manager.delete_edge_indexes(
                    space_id,
                    &edge.src,
                    &edge.dst,
                    &index_names,
                )?;
            }
        }
        Ok(())
    }

    /// 插入边数据（高级接口）
    pub fn insert_edge_data(
        &self,
        space: &str,
        space_id: u64,
        info: &InsertEdgeInfo,
    ) -> Result<bool, StorageError> {
        let edge_name = info.edge_name.clone();
        let src_vertex_id = info.src_vertex_id.clone();
        let dst_vertex_id = info.dst_vertex_id.clone();
        let rank = info.rank;
        let props = info.props.clone();

        let _edge_type_info = self
            .schema_manager
            .get_edge_type(space, &edge_name)?
            .ok_or_else(|| {
                StorageError::DbError(format!(
                    "Edge type '{}' not found in space '{}'",
                    edge_name, space
                ))
            })?;

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

        // 插入边
        {
            let mut writer = self.writer.lock();
            writer.insert_edge(space, edge)?;
        }

        // 更新边索引
        self.index_data_manager.update_edge_indexes(
            space_id,
            &src_vertex_id,
            &dst_vertex_id,
            &edge_name,
            &props,
        )?;

        Ok(true)
    }

    /// 删除边数据（高级接口）
    pub fn delete_edge_data(
        &self,
        space: &str,
        space_id: u64,
        src: &Value,
        dst: &Value,
        rank: i64,
    ) -> Result<bool, StorageError> {
        // 扫描找到匹配的边
        let edges = self.reader.lock().scan_all_edges(space)?;
        let mut deleted = false;

        for edge in edges {
            if *edge.src == *src && *edge.dst == *dst && edge.ranking == rank {
                {
                    let mut writer = self.writer.lock();
                    writer.delete_edge(space, &edge.src, &edge.dst, &edge.edge_type)?;
                }
                let indexes = self.index_metadata_manager.list_edge_indexes(space_id)?;
                let index_names: Vec<String> = indexes
                    .into_iter()
                    .filter(|idx| idx.schema_name == edge.edge_type)
                    .map(|idx| idx.name)
                    .collect();
                self.index_data_manager.delete_edge_indexes(
                    space_id,
                    &edge.src,
                    &edge.dst,
                    &index_names,
                )?;
                deleted = true;
                break;
            }
        }

        Ok(deleted)
    }

    /// 查找悬挂边
    pub fn find_dangling_edges(&self, space: &str) -> Result<Vec<Edge>, StorageError> {
        let mut dangling_edges = Vec::new();
        let edges = self.reader.lock().scan_all_edges(space)?;

        for edge in edges {
            let src_exists = self.reader.lock().get_vertex(space, &edge.src)?.is_some();
            let dst_exists = self.reader.lock().get_vertex(space, &edge.dst)?.is_some();

            if !src_exists || !dst_exists {
                dangling_edges.push(edge);
            }
        }

        Ok(dangling_edges)
    }

    /// 修复悬挂边
    pub fn repair_dangling_edges(&self, space: &str, space_id: u64) -> Result<usize, StorageError> {
        let dangling_edges = self.find_dangling_edges(space)?;
        let count = dangling_edges.len();

        for edge in dangling_edges {
            {
                let mut writer = self.writer.lock();
                writer.delete_edge(space, &edge.src, &edge.dst, &edge.edge_type)?;
            }
            let indexes = self.index_metadata_manager.list_edge_indexes(space_id)?;
            let index_names: Vec<String> = indexes
                .into_iter()
                .filter(|idx| idx.schema_name == edge.edge_type)
                .map(|idx| idx.name)
                .collect();
            self.index_data_manager.delete_edge_indexes(
                space_id,
                &edge.src,
                &edge.dst,
                &index_names,
            )?;
        }

        Ok(count)
    }

    /// 构建边 schema
    pub fn build_edge_schema(&self, edge_type_info: &EdgeTypeInfo) -> Result<Schema, StorageError> {
        let mut schema = Schema::new(edge_type_info.edge_type_name.clone(), 1);
        for prop in &edge_type_info.properties {
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

    /// 获取带 schema 的边
    pub fn get_edge_with_schema(
        &self,
        space: &str,
        edge_type: &str,
        src: &Value,
        dst: &Value,
    ) -> Result<Option<(Schema, Vec<u8>)>, StorageError> {
        use bincode::{config::standard, encode_to_vec};

        if let Some(edge) = self.reader.lock().get_edge(space, src, dst, edge_type)? {
            let edge_type_info = self
                .schema_manager
                .get_edge_type(space, edge_type)?
                .ok_or_else(|| {
                    StorageError::DbError(format!(
                        "Edge type '{}' not found in space '{}'",
                        edge_type, space
                    ))
                })?;
            let schema = self.build_edge_schema(&edge_type_info)?;
            let edge_data = encode_to_vec(&edge, standard())?;
            return Ok(Some((schema, edge_data)));
        }
        Ok(None)
    }

    /// 扫描带 schema 的边
    pub fn scan_edges_with_schema(
        &self,
        space: &str,
        edge_type: &str,
    ) -> Result<Vec<(Schema, Vec<u8>)>, StorageError> {
        use bincode::{config::standard, encode_to_vec};

        let mut results = Vec::new();
        let edge_type_info = self
            .schema_manager
            .get_edge_type(space, edge_type)?
            .ok_or_else(|| {
                StorageError::DbError(format!(
                    "Edge type '{}' not found in space '{}'",
                    edge_type, space
                ))
            })?;
        let schema = self.build_edge_schema(&edge_type_info)?;

        let edges = self.reader.lock().scan_edges_by_type(space, edge_type)?;
        for edge in edges {
            let edge_data = encode_to_vec(&edge, standard())?;
            results.push((schema.clone(), edge_data));
        }

        Ok(results)
    }
}
