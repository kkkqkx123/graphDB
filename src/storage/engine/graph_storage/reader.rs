//! Graph Storage Reader
//!
//! Provides read-only operations for the graph storage engine.

use crate::core::types::{EdgeTypeInfo, TagInfo, VertexId};
use crate::core::{Edge, EdgeDirection, StorageError, StorageResult, Value, Vertex};

use super::context::GraphStorageContext;
use super::type_utils::{
    edge_record_to_edge, serialize_properties, value_to_string, vertex_record_to_vertex,
};

pub struct GraphStorageReader<'a> {
    ctx: &'a GraphStorageContext,
}

impl<'a> GraphStorageReader<'a> {
    pub fn new(ctx: &'a GraphStorageContext) -> Self {
        Self { ctx }
    }

    pub fn get_vertex(&self, space: &str, id: &VertexId) -> StorageResult<Option<Vertex>> {
        let _space_info = self
            .ctx
            .schema_manager
            .get_space(space)?
            .ok_or_else(|| StorageError::not_found(format!("Space {} not found", space)))?;

        let tags = self.ctx.schema_manager.list_tags(space)?;
        if tags.is_empty() {
            return Ok(None);
        }

        let ts = self.ctx.get_read_timestamp();

        for tag in &tags {
            if let Some(label_id) = self.ctx.graph.get_vertex_label_id(&tag.tag_name) {
                let record = if let Some(id_int) = id.as_int64() {
                    self.ctx.graph.get_vertex_by_i64(label_id, id_int, ts)
                } else {
                    let id_str = id.to_string();
                    self.ctx.graph.get_vertex(label_id, &id_str, ts)
                };

                if let Some(record) = record {
                    let vertex = vertex_record_to_vertex(&record, &tag.tag_name);
                    return Ok(Some(vertex));
                }
            }
        }

        Ok(None)
    }

    pub fn scan_vertices(&self, space: &str) -> StorageResult<Vec<Vertex>> {
        let tags = self.ctx.schema_manager.list_tags(space)?;
        let ts = self.ctx.get_read_timestamp();
        let mut vertices = Vec::new();

        for tag in &tags {
            if let Some(label_id) = self.ctx.graph.get_vertex_label_id(&tag.tag_name) {
                if let Some(iterator) = self.ctx.graph.scan_vertices(label_id, ts) {
                    for record in iterator {
                        let vertex = vertex_record_to_vertex(&record, &tag.tag_name);
                        vertices.push(vertex);
                    }
                }
            }
        }

        Ok(vertices)
    }

    pub fn scan_vertices_by_tag(&self, space: &str, tag: &str) -> StorageResult<Vec<Vertex>> {
        let tag_info = self
            .ctx
            .schema_manager
            .get_tag(space, tag)?
            .ok_or_else(|| {
                StorageError::not_found(format!("Tag {} not found in space {}", tag, space))
            })?;

        let ts = self.ctx.get_read_timestamp();
        let mut vertices = Vec::new();

        let label_id = tag_info.tag_id;
        if let Some(iterator) = self.ctx.graph.scan_vertices(label_id, ts) {
            for record in iterator {
                let vertex = vertex_record_to_vertex(&record, tag);
                vertices.push(vertex);
            }
        }

        Ok(vertices)
    }

    pub fn scan_vertices_by_prop(
        &self,
        space: &str,
        tag: &str,
        prop: &str,
        value: &Value,
    ) -> StorageResult<Vec<Vertex>> {
        let tag_info = self
            .ctx
            .schema_manager
            .get_tag(space, tag)?
            .ok_or_else(|| {
                StorageError::not_found(format!("Tag {} not found in space {}", tag, space))
            })?;

        let ts = self.ctx.get_read_timestamp();
        let mut vertices = Vec::new();

        let label_id = tag_info.tag_id;
        if let Some(iterator) = self.ctx.graph.scan_vertices(label_id, ts) {
            for record in iterator {
                if record
                    .properties
                    .iter()
                    .any(|(k, v)| k == prop && v == value)
                {
                    let vertex = vertex_record_to_vertex(&record, tag);
                    vertices.push(vertex);
                }
            }
        }

        Ok(vertices)
    }

    pub fn get_edge(
        &self,
        space: &str,
        src: &VertexId,
        dst: &VertexId,
        edge_type: &str,
        _rank: i64,
    ) -> StorageResult<Option<Edge>> {
        let edge_info = self
            .ctx
            .schema_manager
            .get_edge_type(space, edge_type)?
            .ok_or_else(|| {
                StorageError::not_found(format!(
                    "Edge type {} not found in space {}",
                    edge_type, space
                ))
            })?;

        let ts = self.ctx.get_read_timestamp();

        let edge_label_id = edge_info.edge_type_id;
        if let Some(src_label_id) = self.ctx.graph.get_vertex_label_id(&edge_info.src_tag_name) {
            if let Some(dst_label_id) = self.ctx.graph.get_vertex_label_id(&edge_info.dst_tag_name)
            {
                let src_str = src.to_string();
                let dst_str = dst.to_string();

                if let Some(record) = self.ctx.graph.get_edge(
                    edge_label_id,
                    src_label_id,
                    &src_str,
                    dst_label_id,
                    &dst_str,
                    ts,
                ) {
                    let edge = edge_record_to_edge(&record, edge_type, &src_str, &dst_str);
                    return Ok(Some(edge));
                }

                if let (Some(src_int), Some(dst_int)) = (src.as_int64(), dst.as_int64()) {
                    if let Some(record) = self.ctx.graph.get_edge_by_i64(
                        edge_label_id,
                        src_label_id,
                        src_int,
                        dst_label_id,
                        dst_int,
                        ts,
                    ) {
                        let edge = edge_record_to_edge(&record, edge_type, &src_str, &dst_str);
                        return Ok(Some(edge));
                    }
                }
            }
        }

        Ok(None)
    }

    pub fn get_node_edges(
        &self,
        space: &str,
        node_id: &VertexId,
        direction: EdgeDirection,
    ) -> StorageResult<Vec<Edge>> {
        let edge_types = self.ctx.schema_manager.list_edge_types(space)?;
        if edge_types.is_empty() {
            return Ok(Vec::new());
        }

        let ts = self.ctx.get_read_timestamp();
        let node_str = node_id.to_string();
        let mut edges = Vec::new();

        for edge_info in &edge_types {
            let edge_label_id = edge_info.edge_type_id;
            let edge_type_name = &edge_info.edge_type_name;

            if let Some(src_label_id) = self.ctx.graph.get_vertex_label_id(&edge_info.src_tag_name)
            {
                if let Some(dst_label_id) =
                    self.ctx.graph.get_vertex_label_id(&edge_info.dst_tag_name)
                {
                    match direction {
                        EdgeDirection::Out => {
                            if let Some(out_edges) = self.ctx.graph.out_edges(
                                edge_label_id,
                                src_label_id,
                                dst_label_id,
                                &node_str,
                                ts,
                            ) {
                                for record in out_edges {
                                    let dst_internal = record.dst_vid.as_int64().unwrap_or(0) as u32;
                                    let dst_external = self.ctx.graph
                                        .get_external_id(dst_label_id, dst_internal, ts)
                                        .unwrap_or_else(|| format!("{}", record.dst_vid));
                                    
                                    let edge = edge_record_to_edge(
                                        &record,
                                        edge_type_name,
                                        &node_str,
                                        &dst_external,
                                    );
                                    edges.push(edge);
                                }
                            }
                        }
                        EdgeDirection::In => {
                            if let Some(in_edges) = self.ctx.graph.in_edges(
                                edge_label_id,
                                src_label_id,
                                dst_label_id,
                                &node_str,
                                ts,
                            ) {
                                for record in in_edges {
                                    let src_internal = record.src_vid.as_int64().unwrap_or(0) as u32;
                                    let src_external = self.ctx.graph
                                        .get_external_id(src_label_id, src_internal, ts)
                                        .unwrap_or_else(|| format!("{}", record.src_vid));
                                    
                                    let edge = edge_record_to_edge(
                                        &record,
                                        edge_type_name,
                                        &src_external,
                                        &node_str,
                                    );
                                    edges.push(edge);
                                }
                            }
                        }
                        EdgeDirection::Both => {
                            if let Some(out_edges) = self.ctx.graph.out_edges(
                                edge_label_id,
                                src_label_id,
                                dst_label_id,
                                &node_str,
                                ts,
                            ) {
                                for record in out_edges {
                                    let dst_internal = record.dst_vid.as_int64().unwrap_or(0) as u32;
                                    let dst_external = self.ctx.graph
                                        .get_external_id(dst_label_id, dst_internal, ts)
                                        .unwrap_or_else(|| format!("{}", record.dst_vid));
                                    
                                    let edge = edge_record_to_edge(
                                        &record,
                                        edge_type_name,
                                        &node_str,
                                        &dst_external,
                                    );
                                    edges.push(edge);
                                }
                            }
                            if let Some(in_edges) = self.ctx.graph.in_edges(
                                edge_label_id,
                                src_label_id,
                                dst_label_id,
                                &node_str,
                                ts,
                            ) {
                                for record in in_edges {
                                    let src_internal = record.src_vid.as_int64().unwrap_or(0) as u32;
                                    let src_external = self.ctx.graph
                                        .get_external_id(src_label_id, src_internal, ts)
                                        .unwrap_or_else(|| format!("{}", record.src_vid));
                                    
                                    let edge = edge_record_to_edge(
                                        &record,
                                        edge_type_name,
                                        &src_external,
                                        &node_str,
                                    );
                                    edges.push(edge);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(edges)
    }

    pub fn scan_edges_by_type(&self, space: &str, edge_type: &str) -> StorageResult<Vec<Edge>> {
        let edge_info = self
            .ctx
            .schema_manager
            .get_edge_type(space, edge_type)?
            .ok_or_else(|| {
                StorageError::not_found(format!(
                    "Edge type {} not found in space {}",
                    edge_type, space
                ))
            })?;

        let ts = self.ctx.get_read_timestamp();
        let mut edges = Vec::new();

        let edge_label_id = edge_info.edge_type_id;
        if let Some(src_label_id) = self.ctx.graph.get_vertex_label_id(&edge_info.src_tag_name) {
            if let Some(dst_label_id) = self.ctx.graph.get_vertex_label_id(&edge_info.dst_tag_name)
            {
                let records =
                    self.ctx
                        .graph
                        .scan_edges(src_label_id, dst_label_id, edge_label_id, ts);
                for record in records {
                    let src_internal = record.src_vid.as_int64().unwrap_or(0) as u32;
                    let dst_internal = record.dst_vid.as_int64().unwrap_or(0) as u32;
                    
                    let src_external = self.ctx.graph
                        .get_external_id(src_label_id, src_internal, ts)
                        .unwrap_or_else(|| format!("{}", record.src_vid));
                    
                    let dst_external = self.ctx.graph
                        .get_external_id(dst_label_id, dst_internal, ts)
                        .unwrap_or_else(|| format!("{}", record.dst_vid));
                    
                    let edge = edge_record_to_edge(
                        &record,
                        edge_type,
                        &src_external,
                        &dst_external,
                    );
                    edges.push(edge);
                }
            }
        }

        Ok(edges)
    }

    pub fn scan_all_edges(&self, space: &str) -> StorageResult<Vec<Edge>> {
        let _space_info = self
            .ctx
            .schema_manager
            .get_space(space)?
            .ok_or_else(|| StorageError::not_found(format!("Space {} not found", space)))?;

        let mut edges = Vec::new();
        let edge_types = self.ctx.schema_manager.list_edge_types(space)?;

        for et in edge_types {
            let type_edges = self.scan_edges_by_type(space, &et.edge_type_name)?;
            edges.extend(type_edges);
        }

        Ok(edges)
    }

    pub fn get_vertex_with_schema(
        &self,
        space: &str,
        tag: &str,
        id: &Value,
    ) -> StorageResult<Option<(TagInfo, Vec<u8>)>> {
        let tag_info = self
            .ctx
            .schema_manager
            .get_tag(space, tag)?
            .ok_or_else(|| {
                StorageError::not_found(format!("Tag {} not found in space {}", tag, space))
            })?;

        let ts = self.ctx.get_read_timestamp();
        let id_str = value_to_string(id);

        let label_id = tag_info.tag_id;
        if let Some(record) = self.ctx.graph.get_vertex(label_id, &id_str, ts) {
            let data = serialize_properties(&record.properties);
            return Ok(Some((tag_info, data)));
        }

        Ok(None)
    }

    pub fn get_edge_with_schema(
        &self,
        space: &str,
        edge_type: &str,
        src: &Value,
        dst: &Value,
    ) -> StorageResult<Option<(EdgeTypeInfo, Vec<u8>)>> {
        let edge_info = self
            .ctx
            .schema_manager
            .get_edge_type(space, edge_type)?
            .ok_or_else(|| {
                StorageError::not_found(format!(
                    "Edge type {} not found in space {}",
                    edge_type, space
                ))
            })?;

        let ts = self.ctx.get_read_timestamp();
        let src_str = value_to_string(src);
        let dst_str = value_to_string(dst);

        let edge_label_id = edge_info.edge_type_id;
        if let Some(src_label_id) = self.ctx.graph.get_vertex_label_id(&edge_info.src_tag_name) {
            if let Some(dst_label_id) = self.ctx.graph.get_vertex_label_id(&edge_info.dst_tag_name)
            {
                if let Some(record) = self.ctx.graph.get_edge(
                    edge_label_id,
                    src_label_id,
                    &src_str,
                    dst_label_id,
                    &dst_str,
                    ts,
                ) {
                    let data = serialize_properties(&record.properties);
                    return Ok(Some((edge_info, data)));
                }
            }
        }

        Ok(None)
    }

    pub fn scan_vertices_with_schema(
        &self,
        space: &str,
        tag: &str,
    ) -> StorageResult<Vec<(TagInfo, Vec<u8>)>> {
        let tag_info = self
            .ctx
            .schema_manager
            .get_tag(space, tag)?
            .ok_or_else(|| {
                StorageError::not_found(format!("Tag {} not found in space {}", tag, space))
            })?;

        let ts = self.ctx.get_read_timestamp();
        let mut results = Vec::new();

        let label_id = tag_info.tag_id;
        if let Some(iterator) = self.ctx.graph.scan_vertices(label_id, ts) {
            for record in iterator {
                let data = serialize_properties(&record.properties);
                results.push((tag_info.clone(), data));
            }
        }

        Ok(results)
    }

    pub fn scan_edges_with_schema(
        &self,
        space: &str,
        edge_type: &str,
    ) -> StorageResult<Vec<(EdgeTypeInfo, Vec<u8>)>> {
        let edge_info = self
            .ctx
            .schema_manager
            .get_edge_type(space, edge_type)?
            .ok_or_else(|| {
                StorageError::not_found(format!(
                    "Edge type {} not found in space {}",
                    edge_type, space
                ))
            })?;

        let ts = self.ctx.get_read_timestamp();
        let mut results = Vec::new();

        let edge_label_id = edge_info.edge_type_id;
        if let Some(src_label_id) = self.ctx.graph.get_vertex_label_id(&edge_info.src_tag_name) {
            if let Some(dst_label_id) = self.ctx.graph.get_vertex_label_id(&edge_info.dst_tag_name)
            {
                let records =
                    self.ctx
                        .graph
                        .scan_edges(src_label_id, dst_label_id, edge_label_id, ts);
                for record in records {
                    let data = serialize_properties(&record.properties);
                    results.push((edge_info.clone(), data));
                }
            }
        }

        Ok(results)
    }
}
