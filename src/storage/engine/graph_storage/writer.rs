//! Graph Storage Writer
//!
//! Provides write operations for the graph storage engine.

use crate::core::types::{InsertEdgeInfo, InsertVertexInfo, LabelId, UpdateInfo, UpdateOp, UpdateTarget, VertexId};
use crate::core::{Edge, EdgeDirection, StorageError, StorageResult, Value, Vertex};
use crate::storage::engine::property_graph::InsertEdgeParams;
use crate::storage::metadata::index_manager::IndexMetadataManager;

use super::context::GraphStorageContext;

pub struct GraphStorageWriter<'a> {
    ctx: &'a GraphStorageContext,
}

impl<'a> GraphStorageWriter<'a> {
    pub fn new(ctx: &'a GraphStorageContext) -> Self {
        Self { ctx }
    }

    pub fn insert_vertex(&self, space: &str, vertex: Vertex) -> StorageResult<VertexId> {
        let space_info = self.ctx.schema_manager.get_space(space)?.ok_or_else(|| {
            StorageError::not_found(format!("Space {} not found", space))
        })?;

        let ts = self.ctx.get_write_timestamp();

        let mut inserted_tags: Vec<(LabelId, String)> = Vec::new();

        for tag in &vertex.tags {
            if let Some(label_id) = self.ctx.graph.get_vertex_label_id(&tag.name) {
                let id_str = vertex.vid.to_string();
                let props: Vec<(String, Value)> = tag.properties.iter().map(|(k, v)| (k.clone(), v.clone())).collect();

                if self.ctx.graph.insert_vertex(label_id, &id_str, &props, ts).is_err() {
                    for (rollback_label, rollback_id) in inserted_tags.iter().rev() {
                        let _ = self.ctx.graph.delete_vertex(*rollback_label, rollback_id, ts);
                    }
                    return Err(StorageError::vertex_already_exists(id_str));
                }

                let vid_value = Value::from(vertex.vid);
                if let Err(e) = Self::update_vertex_indexes(
                    &self.ctx.graph,
                    &self.ctx.index_metadata_manager,
                    space_info.space_id,
                    &vid_value,
                    &tag.name,
                    &props,
                    ts,
                ) {
                    for (rollback_label, rollback_id) in inserted_tags.iter().rev() {
                        let _ = self.ctx.graph.delete_vertex(*rollback_label, rollback_id, ts);
                    }
                    let _ = self.ctx.graph.delete_vertex(label_id, &id_str, ts);
                    return Err(e);
                }

                inserted_tags.push((label_id, id_str));
            }
        }

        Ok(vertex.vid)
    }

    pub fn update_vertex(&self, space: &str, vertex: Vertex) -> StorageResult<()> {
        let space_info = self.ctx.schema_manager.get_space(space)?.ok_or_else(|| {
            StorageError::not_found(format!("Space {} not found", space))
        })?;

        let ts = self.ctx.get_write_timestamp();
        let id_str = vertex.vid.to_string();

        for tag in &vertex.tags {
            if let Some(label_id) = self.ctx.graph.get_vertex_label_id(&tag.name) {
                for (prop_name, value) in &tag.properties {
                    self.ctx.graph.update_vertex_property(label_id, &id_str, prop_name, value, ts)?;
                }

                let props: Vec<(String, Value)> =
                    tag.properties.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
                let vid_value = Value::from(vertex.vid);
                Self::update_vertex_indexes(
                    &self.ctx.graph,
                    &self.ctx.index_metadata_manager,
                    space_info.space_id,
                    &vid_value,
                    &tag.name,
                    &props,
                    ts,
                )?;
            }
        }

        Ok(())
    }

    pub fn delete_vertex(&self, space: &str, id: &VertexId) -> StorageResult<()> {
        let space_info = self.ctx.schema_manager.get_space(space)?.ok_or_else(|| {
            StorageError::not_found(format!("Space {} not found", space))
        })?;

        let tags = self.ctx.schema_manager.list_tags(space)?;
        let ts = self.ctx.get_write_timestamp();
        let id_str = id.to_string();

        for tag in &tags {
            if let Some(label_id) = self.ctx.graph.get_vertex_label_id(&tag.tag_name) {
                let _ = self.ctx.graph.delete_vertex(label_id, &id_str, ts);

                let id_value = Value::from(*id);
                Self::delete_vertex_indexes(
                    &self.ctx.graph,
                    &self.ctx.index_metadata_manager,
                    space_info.space_id,
                    &id_value,
                    &tag.tag_name,
                    ts,
                )?;
            }
        }

        Ok(())
    }

    pub fn delete_vertex_with_edges(
        &self,
        space: &str,
        id: &VertexId,
        reader: &super::reader::GraphStorageReader<'_>,
    ) -> StorageResult<()> {
        let edges = reader.get_node_edges(space, id, EdgeDirection::Both)?;

        for edge in edges {
            let _ = self.delete_edge(space, &edge.src, &edge.dst, &edge.edge_type, edge.ranking);
        }

        self.delete_vertex(space, id)
    }

    pub fn batch_insert_vertices(
        &self,
        space: &str,
        vertices: Vec<Vertex>,
    ) -> StorageResult<Vec<VertexId>> {
        let mut ids = Vec::with_capacity(vertices.len());
        for vertex in vertices {
            let id = self.insert_vertex(space, vertex)?;
            ids.push(id);
        }
        Ok(ids)
    }

    pub fn delete_tags(
        &self,
        space: &str,
        vertex_id: &VertexId,
        tag_names: &[String],
    ) -> StorageResult<usize> {
        let space_info = self.ctx.schema_manager.get_space(space)?.ok_or_else(|| {
            StorageError::not_found(format!("Space {} not found", space))
        })?;

        let ts = self.ctx.get_write_timestamp();
        let mut deleted_count = 0;

        let id_str = vertex_id.to_string();

        for tag_name in tag_names {
            if let Some(label_id) = self.ctx.graph.get_vertex_label_id(tag_name) {
                if self.ctx.graph.delete_vertex(label_id, &id_str, ts).is_ok() {
                    let vertex_id_value = Value::from(*vertex_id);
                    Self::delete_vertex_indexes(
                        &self.ctx.graph,
                        &self.ctx.index_metadata_manager,
                        space_info.space_id,
                        &vertex_id_value,
                        tag_name,
                        ts,
                    )?;
                    deleted_count += 1;
                }
            }
        }

        Ok(deleted_count)
    }

    pub fn insert_edge(&self, space: &str, edge: Edge) -> StorageResult<()> {
        let space_info = self.ctx.schema_manager.get_space(space)?.ok_or_else(|| {
            StorageError::not_found(format!("Space {} not found", space))
        })?;

        let ts = self.ctx.get_write_timestamp();

        if let Some(edge_label_id) = self.ctx.graph.get_edge_label_id(&edge.edge_type) {
            let edge_types = self.ctx.schema_manager.list_edge_types(space)?;
            for et in edge_types {
                if et.edge_type_name == edge.edge_type {
                    if let Some(src_label_id) = self.ctx.graph.get_vertex_label_id(&et.src_tag_name) {
                        if let Some(dst_label_id) = self.ctx.graph.get_vertex_label_id(&et.dst_tag_name) {
                            let src_str = edge.src.to_string();
                            let dst_str = edge.dst.to_string();
                            let props: Vec<(String, Value)> =
                                edge.props.iter().map(|(k, v)| (k.clone(), v.clone())).collect();

                            self.ctx.graph.insert_edge(InsertEdgeParams {
                                edge_label: edge_label_id,
                                src_label: src_label_id,
                                src_id: &src_str,
                                dst_label: dst_label_id,
                                dst_id: &dst_str,
                                properties: &props,
                                ts,
                            })?;

                            let src_value = Value::from(edge.src);
                            let dst_value = Value::from(edge.dst);
                            Self::update_edge_indexes(EdgeIndexUpdateParams {
                                graph: &self.ctx.graph,
                                index_metadata_manager: &self.ctx.index_metadata_manager,
                                space_id: space_info.space_id,
                                src: &src_value,
                                dst: &dst_value,
                                edge_type: &edge.edge_type,
                                props: &props,
                                ts,
                            })?;
                        }
                    }
                    break;
                }
            }
        }

        Ok(())
    }

    pub fn delete_edge(
        &self,
        space: &str,
        src: &VertexId,
        dst: &VertexId,
        edge_type: &str,
        _rank: i64,
    ) -> StorageResult<()> {
        let space_info = self.ctx.schema_manager.get_space(space)?.ok_or_else(|| {
            StorageError::not_found(format!("Space {} not found", space))
        })?;

        let ts = self.ctx.get_write_timestamp();

        if let Some(edge_label_id) = self.ctx.graph.get_edge_label_id(edge_type) {
            let edge_types = self.ctx.schema_manager.list_edge_types(space)?;
            for et in edge_types {
                if et.edge_type_name == edge_type {
                    if let Some(src_label_id) = self.ctx.graph.get_vertex_label_id(&et.src_tag_name) {
                        if let Some(dst_label_id) = self.ctx.graph.get_vertex_label_id(&et.dst_tag_name) {
                            let src_str = src.to_string();
                            let dst_str = dst.to_string();

                            self.ctx.graph.delete_edge(edge_label_id, src_label_id, &src_str, dst_label_id, &dst_str, ts)?;

                            let src_value = Value::from(*src);
                            let dst_value = Value::from(*dst);
                            Self::delete_edge_indexes(
                                &self.ctx.graph,
                                &self.ctx.index_metadata_manager,
                                space_info.space_id,
                                &src_value,
                                &dst_value,
                                edge_type,
                                ts,
                            )?;
                        }
                    }
                    break;
                }
            }
        }

        Ok(())
    }

    pub fn batch_insert_edges(&self, space: &str, edges: Vec<Edge>) -> StorageResult<()> {
        for edge in edges {
            self.insert_edge(space, edge)?;
        }
        Ok(())
    }

    pub fn insert_vertex_data(
        &self,
        space: &str,
        info: &InsertVertexInfo,
    ) -> StorageResult<bool> {
        let space_info = self.ctx.schema_manager.get_space(space)?.ok_or_else(|| {
            StorageError::not_found(format!("Space {} not found", space))
        })?;

        let _tag = self.ctx.schema_manager.get_tag(space, &info.tag_name)?.ok_or_else(|| {
            StorageError::not_found(format!("Tag {} not found", info.tag_name))
        })?;

        if info.space_id != space_info.space_id {
            return Err(StorageError::db_error("Space ID mismatch".to_string()));
        }

        let ts = self.ctx.get_write_timestamp();

        if let Some(label_id) = self.ctx.graph.get_vertex_label_id(&info.tag_name) {
            let vid = VertexId::try_from(&info.vertex_id)
                .map_err(|e| StorageError::invalid_input(e.to_string()))?;
            let id_str = vid.to_string();

            let result = self.ctx.graph.insert_vertex(label_id, &id_str, &info.props, ts);
            match result {
                Ok(_) => {
                    Self::update_vertex_indexes(
                        &self.ctx.graph,
                        &self.ctx.index_metadata_manager,
                        space_info.space_id,
                        &info.vertex_id,
                        &info.tag_name,
                        &info.props,
                        ts,
                    )?;
                    Ok(true)
                }
                Err(ref e)
                    if e.kind()
                        == crate::core::error::storage::StorageErrorKind::VertexAlreadyExists =>
                {
                    Ok(false)
                }
                Err(e) => Err(e),
            }
        } else {
            Err(StorageError::not_found(format!(
                "Tag {} not found in graph",
                info.tag_name
            )))
        }
    }

    pub fn insert_edge_data(&self, space: &str, info: &InsertEdgeInfo) -> StorageResult<bool> {
        let space_info = self.ctx.schema_manager.get_space(space)?.ok_or_else(|| {
            StorageError::not_found(format!("Space {} not found", space))
        })?;

        let _edge_type = self.ctx.schema_manager.get_edge_type(space, &info.edge_name)?.ok_or_else(|| {
            StorageError::not_found(format!("Edge type {} not found", info.edge_name))
        })?;

        if info.space_id != space_info.space_id {
            return Err(StorageError::db_error("Space ID mismatch".to_string()));
        }

        let ts = self.ctx.get_write_timestamp();

        if let Some(edge_label_id) = self.ctx.graph.get_edge_label_id(&info.edge_name) {
            let src_vid = VertexId::try_from(&info.src_vertex_id)
                .map_err(|e| StorageError::invalid_input(e.to_string()))?;
            let dst_vid = VertexId::try_from(&info.dst_vertex_id)
                .map_err(|e| StorageError::invalid_input(e.to_string()))?;
            let src_id = src_vid.to_string();
            let dst_id = dst_vid.to_string();

            let edge_types = self.ctx.schema_manager.list_edge_types(space)?;
            for et in edge_types {
                if et.edge_type_name == info.edge_name {
                    if let Some(src_label_id) = self.ctx.graph.get_vertex_label_id(&et.src_tag_name) {
                        if let Some(dst_label_id) = self.ctx.graph.get_vertex_label_id(&et.dst_tag_name) {
                            let result = self.ctx.graph.insert_edge(InsertEdgeParams {
                                edge_label: edge_label_id,
                                src_label: src_label_id,
                                src_id: &src_id,
                                dst_label: dst_label_id,
                                dst_id: &dst_id,
                                properties: &info.props,
                                ts,
                            });
                            match result {
                                Ok(_) => {
                                    Self::update_edge_indexes(EdgeIndexUpdateParams {
                                        graph: &self.ctx.graph,
                                        index_metadata_manager: &self.ctx.index_metadata_manager,
                                        space_id: space_info.space_id,
                                        src: &info.src_vertex_id,
                                        dst: &info.dst_vertex_id,
                                        edge_type: &info.edge_name,
                                        props: &info.props,
                                        ts,
                                    })?;
                                    return Ok(true);
                                }
                                Err(ref e)
                                    if e.kind()
                                        == crate::core::error::storage::StorageErrorKind::EdgeAlreadyExists =>
                                {
                                    return Ok(false);
                                }
                                Err(e) => return Err(e),
                            }
                        }
                    }
                }
            }
        }

        Err(StorageError::not_found(format!(
            "Edge type {} not found in graph",
            info.edge_name
        )))
    }

    pub fn delete_vertex_data(&self, space: &str, vertex_id: &str) -> StorageResult<bool> {
        let space_info = self.ctx.schema_manager.get_space(space)?.ok_or_else(|| {
            StorageError::not_found(format!("Space {} not found", space))
        })?;

        let tags = self.ctx.schema_manager.list_tags(space)?;
        let ts = self.ctx.get_write_timestamp();
        let mut deleted = false;

        for tag in tags {
            if let Some(label_id) = self.ctx.graph.get_vertex_label_id(&tag.tag_name) {
                if self.ctx.graph.delete_vertex(label_id, vertex_id, ts).is_ok() {
                    Self::delete_vertex_indexes(
                        &self.ctx.graph,
                        &self.ctx.index_metadata_manager,
                        space_info.space_id,
                        &Value::String(vertex_id.to_string()),
                        &tag.tag_name,
                        ts,
                    )?;
                    deleted = true;
                }
            }
        }

        Ok(deleted)
    }

    pub fn delete_edge_data(
        &self,
        space: &str,
        src: &str,
        dst: &str,
        _rank: i64,
    ) -> StorageResult<bool> {
        let space_info = self.ctx.schema_manager.get_space(space)?.ok_or_else(|| {
            StorageError::not_found(format!("Space {} not found", space))
        })?;

        let edge_types = self.ctx.schema_manager.list_edge_types(space)?;
        let ts = self.ctx.get_write_timestamp();
        let mut deleted = false;

        for et in edge_types {
            if let Some(edge_label_id) = self.ctx.graph.get_edge_label_id(&et.edge_type_name) {
                if let Some(src_label_id) = self.ctx.graph.get_vertex_label_id(&et.src_tag_name) {
                    if let Some(dst_label_id) = self.ctx.graph.get_vertex_label_id(&et.dst_tag_name) {
                        if self.ctx.graph
                            .delete_edge(edge_label_id, src_label_id, src, dst_label_id, dst, ts)
                            .is_ok()
                        {
                            Self::delete_edge_indexes(
                                &self.ctx.graph,
                                &self.ctx.index_metadata_manager,
                                space_info.space_id,
                                &Value::String(src.to_string()),
                                &Value::String(dst.to_string()),
                                &et.edge_type_name,
                                ts,
                            )?;
                            deleted = true;
                        }
                    }
                }
            }
        }

        Ok(deleted)
    }

    pub fn update_data(
        &self,
        space: &str,
        space_id: u64,
        info: &UpdateInfo,
    ) -> StorageResult<bool> {
        let space_info = self.ctx.schema_manager.get_space(space)?.ok_or_else(|| {
            StorageError::not_found(format!("Space {} not found", space))
        })?;

        if space_info.space_id != space_id {
            return Err(StorageError::db_error("Space ID mismatch".to_string()));
        }

        let ts = self.ctx.get_write_timestamp();

        let UpdateTarget {
            space_name,
            label,
            id,
            prop,
        } = &info.update_target;

        if space_name != space {
            return Err(StorageError::db_error(
                "Space name mismatch in update target".to_string(),
            ));
        }

        if let Some(label_id) = self.ctx.graph.get_vertex_label_id(label) {
            let vid = VertexId::try_from(id)
                .map_err(|e| StorageError::invalid_input(e.to_string()))?;
            let id_str = vid.to_string();
            let value = match &info.update_op {
                UpdateOp::Set => info.value.clone(),
                UpdateOp::Add => {
                    if let Some(current) = self.ctx.graph.get_vertex(label_id, &id_str, ts) {
                        let current_val = current.properties.iter().find(|(k, _)| k == prop).map(|(_, v)| v);
                        if let (
                            Some(crate::core::Value::Int(cv)),
                            crate::core::Value::Int(add_val),
                        ) = (current_val, &info.value)
                        {
                            crate::core::Value::Int(cv + add_val)
                        } else {
                            info.value.clone()
                        }
                    } else {
                        info.value.clone()
                    }
                }
                UpdateOp::Subtract => {
                    if let Some(current) = self.ctx.graph.get_vertex(label_id, &id_str, ts) {
                        let current_val = current.properties.iter().find(|(k, _)| k == prop).map(|(_, v)| v);
                        if let (
                            Some(crate::core::Value::Int(cv)),
                            crate::core::Value::Int(sub_val),
                        ) = (current_val, &info.value)
                        {
                            crate::core::Value::Int(cv - sub_val)
                        } else {
                            info.value.clone()
                        }
                    } else {
                        info.value.clone()
                    }
                }
                _ => info.value.clone(),
            };

            self.ctx.graph.update_vertex_property(label_id, &id_str, prop, &value, ts)?;

            let props = vec![(prop.clone(), value)];
            Self::update_vertex_indexes(
                &self.ctx.graph,
                &self.ctx.index_metadata_manager,
                space_info.space_id,
                id,
                label,
                &props,
                ts,
            )?;
            Ok(true)
        } else {
            Err(StorageError::not_found(format!("Label {} not found", label)))
        }
    }

    fn update_vertex_indexes(
        graph: &crate::storage::engine::PropertyGraph,
        index_metadata_manager: &crate::storage::metadata::IndexManager,
        space_id: u64,
        vertex_id: &Value,
        tag_name: &str,
        props: &[(String, Value)],
        ts: u32,
    ) -> StorageResult<()> {
        let indexes = index_metadata_manager.list_tag_indexes(space_id)?;
        for index in indexes {
            if index.schema_name == tag_name {
                graph.update_vertex_indexes_mvcc(space_id, vertex_id, &index.name, props, ts)?;
            }
        }
        Ok(())
    }
}

struct EdgeIndexUpdateParams<'a> {
    graph: &'a crate::storage::engine::PropertyGraph,
    index_metadata_manager: &'a crate::storage::metadata::IndexManager,
    space_id: u64,
    src: &'a Value,
    dst: &'a Value,
    edge_type: &'a str,
    props: &'a [(String, Value)],
    ts: u32,
}

impl<'a> GraphStorageWriter<'a> {
    fn update_edge_indexes(
        params: EdgeIndexUpdateParams,
    ) -> StorageResult<()> {
        let indexes = params
            .index_metadata_manager
            .list_edge_indexes(params.space_id)?;
        for index in indexes {
            if index.schema_name == params.edge_type {
                params.graph.update_edge_indexes_mvcc(
                    params.space_id,
                    params.src,
                    params.dst,
                    &index.name,
                    params.props,
                    params.ts,
                )?;
            }
        }
        Ok(())
    }

    fn delete_vertex_indexes(
        graph: &crate::storage::engine::PropertyGraph,
        index_metadata_manager: &crate::storage::metadata::IndexManager,
        space_id: u64,
        vertex_id: &Value,
        tag_name: &str,
        ts: u32,
    ) -> StorageResult<()> {
        let indexes = index_metadata_manager.list_tag_indexes(space_id)?;
        for index in indexes {
            if index.schema_name == tag_name {
                graph.delete_vertex_indexes_mvcc(space_id, vertex_id, ts)?;
            }
        }
        Ok(())
    }

    fn delete_edge_indexes(
        graph: &crate::storage::engine::PropertyGraph,
        index_metadata_manager: &crate::storage::metadata::IndexManager,
        space_id: u64,
        src: &Value,
        dst: &Value,
        edge_type: &str,
        ts: u32,
    ) -> StorageResult<()> {
        let indexes = index_metadata_manager.list_edge_indexes(space_id)?;
        let index_names: Vec<String> = indexes
            .iter()
            .filter(|index| index.schema_name == edge_type)
            .map(|index| index.name.clone())
            .collect();

        if !index_names.is_empty() {
            graph.delete_edge_indexes_mvcc(space_id, src, dst, &index_names, ts)?;
        }
        Ok(())
    }
}
