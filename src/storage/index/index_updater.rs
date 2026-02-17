//! 索引更新器
//!
//! 提供 DML 操作时的索引联动更新功能
//! 包括顶点索引更新、边索引更新、索引删除等
//! 所有操作都通过 space_id 来标识空间，实现多空间数据隔离

use crate::core::{StorageError, Value, Edge};
use crate::core::vertex_edge_path::Tag;
use crate::index::Index;
use crate::storage::index::IndexDataManager;
use crate::storage::metadata::IndexMetadataManager;

/// 索引更新器
///
/// 负责在 DML 操作时自动维护索引的一致性
/// 所有操作都通过 space_id 来标识空间，实现多空间数据隔离
pub struct IndexUpdater<'a, I: IndexDataManager, M: IndexMetadataManager> {
    index_data_manager: &'a I,
    index_metadata_manager: &'a M,
    space_name: String,
    /// 空间ID，用于多空间数据隔离
    space_id: i32,
}

impl<'a, I: IndexDataManager, M: IndexMetadataManager> IndexUpdater<'a, I, M> {
    /// 创建新的索引更新器
    pub fn new(
        index_data_manager: &'a I,
        index_metadata_manager: &'a M,
        space_name: String,
        space_id: i32,
    ) -> Self {
        Self {
            index_data_manager,
            index_metadata_manager,
            space_name,
            space_id,
        }
    }

    /// 获取空间名称
    pub fn space_name(&self) -> &str {
        &self.space_name
    }

    /// 获取空间ID
    pub fn space_id(&self) -> i32 {
        self.space_id
    }

    /// 更新顶点的所有索引
    ///
    /// 在顶点插入或更新时调用，自动更新所有相关索引
    ///
    /// # Arguments
    /// * `vertex_id` - 顶点ID
    /// * `tags` - 顶点的所有标签
    pub fn update_vertex_indexes(
        &self,
        vertex_id: &Value,
        tags: &[Tag],
    ) -> Result<(), StorageError> {
        // 获取该空间的所有标签索引
        let indexes = self.index_metadata_manager.list_tag_indexes(self.space_id)?;

        for index in indexes {
            // 检查索引是否关联到当前顶点的某个标签
            for tag in tags {
                if index.schema_name == tag.name {
                    self.update_vertex_index_for_tag(vertex_id, tag, &index)?;
                }
            }
        }

        Ok(())
    }

    /// 更新指定标签的索引
    fn update_vertex_index_for_tag(
        &self,
        vertex_id: &Value,
        tag: &Tag,
        index: &Index,
    ) -> Result<(), StorageError> {
        // 收集索引字段值
        let mut index_props: Vec<(String, Value)> = Vec::new();

        for field in &index.fields {
            if let Some(value) = tag.properties.get(&field.name) {
                index_props.push((field.name.clone(), value.clone()));
            }
        }

        // 如果所有索引字段都有值，则更新索引
        if !index_props.is_empty() {
            self.index_data_manager.update_vertex_indexes(
                self.space_id,
                vertex_id,
                &index.name,
                &index_props,
            )?;
        }

        Ok(())
    }

    /// 删除顶点的所有索引
    ///
    /// 在顶点删除时调用，自动删除所有相关索引
    ///
    /// # Arguments
    /// * `vertex_id` - 顶点ID
    pub fn delete_vertex_indexes(&self, vertex_id: &Value) -> Result<(), StorageError> {
        self.index_data_manager.delete_vertex_indexes(self.space_id, vertex_id)
    }

    /// 删除指定标签的索引
    ///
    /// 在删除顶点的某个标签时调用
    ///
    /// # Arguments
    /// * `vertex_id` - 顶点ID
    /// * `tag_name` - 标签名称
    pub fn delete_tag_indexes(
        &self,
        vertex_id: &Value,
        tag_name: &str,
    ) -> Result<(), StorageError> {
        self.index_data_manager.delete_tag_indexes(self.space_id, vertex_id, tag_name)
    }

    /// 更新边的所有索引
    ///
    /// 在边插入或更新时调用，自动更新所有相关索引
    ///
    /// # Arguments
    /// * `edge` - 边对象
    pub fn update_edge_indexes(&self, edge: &Edge) -> Result<(), StorageError> {
        // 获取该空间的所有边索引
        let indexes = self.index_metadata_manager.list_edge_indexes(self.space_id)?;

        for index in indexes {
            // 检查索引是否关联到当前边的类型
            if index.schema_name == edge.edge_type {
                self.update_edge_index(edge, &index)?;
            }
        }

        Ok(())
    }

    /// 更新指定边的索引
    fn update_edge_index(&self, edge: &Edge, index: &Index) -> Result<(), StorageError> {
        // 收集索引字段值
        let mut index_props: Vec<(String, Value)> = Vec::new();

        for field in &index.fields {
            if let Some(value) = edge.props.get(&field.name) {
                index_props.push((field.name.clone(), value.clone()));
            }
        }

        // 如果所有索引字段都有值，则更新索引
        if !index_props.is_empty() {
            self.index_data_manager.update_edge_indexes(
                self.space_id,
                &edge.src,
                &edge.dst,
                &index.name,
                &index_props,
            )?;
        }

        Ok(())
    }

    /// 删除边的所有索引
    ///
    /// 在边删除时调用，自动删除所有相关索引
    ///
    /// # Arguments
    /// * `edge` - 边对象
    pub fn delete_edge_indexes(&self, edge: &Edge) -> Result<(), StorageError> {
        self.index_data_manager.delete_edge_indexes(
            self.space_id,
            &edge.src,
            &edge.dst,
            &edge.edge_type,
        )
    }

    /// 批量更新顶点索引
    ///
    /// 用于批量插入顶点时的高效索引更新
    ///
    /// # Arguments
    /// * `vertices` - 顶点列表
    pub fn batch_update_vertex_indexes(
        &self,
        vertices: &[(Value, Vec<Tag>)],
    ) -> Result<(), StorageError> {
        for (vertex_id, tags) in vertices {
            self.update_vertex_indexes(vertex_id, tags)?;
        }
        Ok(())
    }

    /// 批量更新边索引
    ///
    /// 用于批量插入边时的高效索引更新
    ///
    /// # Arguments
    /// * `edges` - 边列表
    pub fn batch_update_edge_indexes(&self, edges: &[Edge]) -> Result<(), StorageError> {
        for edge in edges {
            self.update_edge_indexes(edge)?;
        }
        Ok(())
    }

    /// 批量删除顶点索引
    ///
    /// 用于批量删除顶点时的高效索引删除
    ///
    /// # Arguments
    /// * `vertex_ids` - 顶点ID列表
    pub fn batch_delete_vertex_indexes(&self, vertex_ids: &[Value]) -> Result<(), StorageError> {
        for vertex_id in vertex_ids {
            self.delete_vertex_indexes(vertex_id)?;
        }
        Ok(())
    }

    /// 批量删除边索引
    ///
    /// 用于批量删除边时的高效索引删除
    ///
    /// # Arguments
    /// * `edges` - 边列表
    pub fn batch_delete_edge_indexes(&self, edges: &[Edge]) -> Result<(), StorageError> {
        for edge in edges {
            self.delete_edge_indexes(edge)?;
        }
        Ok(())
    }

    /// 重建指定标签的所有索引
    ///
    /// 用于索引重建操作
    ///
    /// # Arguments
    /// * `tag_name` - 标签名称
    /// * `vertices` - 该标签的所有顶点
    pub fn rebuild_tag_indexes(
        &self,
        tag_name: &str,
        vertices: &[(Value, Tag)],
    ) -> Result<(), StorageError> {
        // 获取该标签的所有索引
        let indexes: Vec<Index> = self
            .index_metadata_manager
            .list_tag_indexes(self.space_id)?
            .into_iter()
            .filter(|idx| idx.schema_name == tag_name)
            .collect();

        // 为每个顶点重建索引
        for (vertex_id, tag) in vertices {
            for index in &indexes {
                self.update_vertex_index_for_tag(vertex_id, tag, index)?;
            }
        }

        Ok(())
    }

    /// 重建指定边类型的所有索引
    ///
    /// 用于索引重建操作
    ///
    /// # Arguments
    /// * `edge_type` - 边类型
    /// * `edges` - 该类型的所有边
    pub fn rebuild_edge_indexes(
        &self,
        edge_type: &str,
        edges: &[Edge],
    ) -> Result<(), StorageError> {
        // 获取该边类型的所有索引
        let indexes: Vec<Index> = self
            .index_metadata_manager
            .list_edge_indexes(self.space_id)?
            .into_iter()
            .filter(|idx| idx.schema_name == edge_type)
            .collect();

        // 为每条边重建索引
        for edge in edges {
            for index in &indexes {
                self.update_edge_index(edge, index)?;
            }
        }

        Ok(())
    }
}

/// 索引更新上下文
///
/// 用于批量 DML 操作时的索引更新管理
pub struct IndexUpdateContext<'a, I: IndexDataManager, M: IndexMetadataManager> {
    updater: IndexUpdater<'a, I, M>,
    pending_vertex_updates: Vec<(Value, Vec<Tag>)>,
    pending_edge_updates: Vec<Edge>,
    pending_vertex_deletes: Vec<Value>,
    pending_edge_deletes: Vec<Edge>,
}

impl<'a, I: IndexDataManager, M: IndexMetadataManager> IndexUpdateContext<'a, I, M> {
    /// 创建新的索引更新上下文
    pub fn new(
        index_data_manager: &'a I,
        index_metadata_manager: &'a M,
        space_name: String,
        space_id: i32,
    ) -> Self {
        Self {
            updater: IndexUpdater::new(index_data_manager, index_metadata_manager, space_name, space_id),
            pending_vertex_updates: Vec::new(),
            pending_edge_updates: Vec::new(),
            pending_vertex_deletes: Vec::new(),
            pending_edge_deletes: Vec::new(),
        }
    }

    /// 添加顶点更新
    pub fn add_vertex_update(&mut self, vertex_id: Value, tags: Vec<Tag>) {
        self.pending_vertex_updates.push((vertex_id, tags));
    }

    /// 添加边更新
    pub fn add_edge_update(&mut self, edge: Edge) {
        self.pending_edge_updates.push(edge);
    }

    /// 添加顶点删除
    pub fn add_vertex_delete(&mut self, vertex_id: Value) {
        self.pending_vertex_deletes.push(vertex_id);
    }

    /// 添加边删除
    pub fn add_edge_delete(&mut self, edge: Edge) {
        self.pending_edge_deletes.push(edge);
    }

    /// 提交所有索引更新
    ///
    /// 在事务提交时调用，批量应用所有索引更新
    pub fn commit(&mut self) -> Result<(), StorageError> {
        // 先处理删除操作，再处理更新操作
        // 这样可以避免删除后重新创建索引的问题

        // 删除顶点索引
        if !self.pending_vertex_deletes.is_empty() {
            self.updater.batch_delete_vertex_indexes(&self.pending_vertex_deletes)?;
            self.pending_vertex_deletes.clear();
        }

        // 删除边索引
        if !self.pending_edge_deletes.is_empty() {
            self.updater.batch_delete_edge_indexes(&self.pending_edge_deletes)?;
            self.pending_edge_deletes.clear();
        }

        // 更新顶点索引
        if !self.pending_vertex_updates.is_empty() {
            self.updater.batch_update_vertex_indexes(&self.pending_vertex_updates)?;
            self.pending_vertex_updates.clear();
        }

        // 更新边索引
        if !self.pending_edge_updates.is_empty() {
            self.updater.batch_update_edge_indexes(&self.pending_edge_updates)?;
            self.pending_edge_updates.clear();
        }

        Ok(())
    }

    /// 回滚所有待处理的索引更新
    ///
    /// 在事务回滚时调用，清除所有待处理的更新
    pub fn rollback(&mut self) {
        self.pending_vertex_updates.clear();
        self.pending_edge_updates.clear();
        self.pending_vertex_deletes.clear();
        self.pending_edge_deletes.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Value;
    use crate::index::{IndexField, IndexType};

    #[test]
    fn test_index_field_creation() {
        let field = IndexField::new(
            "name".to_string(),
            Value::String("test".to_string()),
            false,
        );

        assert_eq!(field.name, "name");
        assert!(!field.is_nullable);
    }

    #[test]
    fn test_index_creation() {
        let index = Index::new(
            1,
            "idx_person_name".to_string(),
            1,
            "person".to_string(),
            vec![IndexField::new(
                "name".to_string(),
                Value::String("".to_string()),
                false,
            )],
            vec![],
            IndexType::TagIndex,
            false,
        );

        assert_eq!(index.name, "idx_person_name");
        assert_eq!(index.schema_name, "person");
        assert_eq!(index.index_type, IndexType::TagIndex);
    }
}
