//! Graph Storage Reader
//!
//! Provides read-only operations for the graph storage engine.

use crate::core::{Edge, EdgeDirection, StorageError, StorageResult, Value, Vertex};
use crate::storage::metadata::Schema;
use crate::storage::metadata::SchemaManager;

use super::context::GraphStorageContext;
use super::converters::{
    edge_record_to_edge, serialize_properties, value_to_string, vertex_record_to_vertex,
};

pub struct GraphStorageReader<'a> {
    ctx: &'a GraphStorageContext,
}

impl<'a> GraphStorageReader<'a> {
    pub fn new(ctx: &'a GraphStorageContext) -> Self {
        Self { ctx }
    }

    pub fn get_vertex(&self, space: &str, id: &Value) -> StorageResult<Option<Vertex>> {
        let _space_info = self.ctx.schema_manager.get_space(space)?.ok_or_else(|| {
            StorageError::not_found(format!("Space {} not found", space))
        })?;

        let tags = self.ctx.schema_manager.list_tags(space)?;
        if tags.is_empty() {
            return Ok(None);
        }

        let ts = self.ctx.get_read_timestamp();
        let graph = self.ctx.graph.read();
        let id_str = value_to_string(id);

        for tag in &tags {
            if let Some(label_id) = graph.get_vertex_label_id(&tag.tag_name) {
                if let Some(record) = graph.get_vertex(label_id, &id_str, ts) {
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
        let graph = self.ctx.graph.read();
        let mut vertices = Vec::new();

        for tag in &tags {
            if let Some(label_id) = graph.get_vertex_label_id(&tag.tag_name) {
                if let Some(iterator) = graph.scan_vertices(label_id, ts) {
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
        let tag_info = self.ctx.schema_manager.get_tag(space, tag)?.ok_or_else(|| {
            StorageError::not_found(format!("Tag {} not found in space {}", tag, space))
        })?;

        let ts = self.ctx.get_read_timestamp();
        let graph = self.ctx.graph.read();
        let mut vertices = Vec::new();

        let label_id = tag_info.tag_id;
        if let Some(iterator) = graph.scan_vertices(label_id, ts) {
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
        let tag_info = self.ctx.schema_manager.get_tag(space, tag)?.ok_or_else(|| {
            StorageError::not_found(format!("Tag {} not found in space {}", tag, space))
        })?;

        let ts = self.ctx.get_read_timestamp();
        let graph = self.ctx.graph.read();
        let mut vertices = Vec::new();

        let label_id = tag_info.tag_id;
        if let Some(iterator) = graph.scan_vertices(label_id, ts) {
            for record in iterator {
                if record.properties.iter().any(|(k, v)| k == prop && v == value) {
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
        src: &Value,
        dst: &Value,
        edge_type: &str,
        _rank: i64,
    ) -> StorageResult<Option<Edge>> {
        let edge_info = self.ctx.schema_manager.get_edge_type(space, edge_type)?.ok_or_else(|| {
            StorageError::not_found(format!("Edge type {} not found in space {}", edge_type, space))
        })?;

        let ts = self.ctx.get_read_timestamp();
        let graph = self.ctx.graph.read();

        let src_str = value_to_string(src);
        let dst_str = value_to_string(dst);

        let edge_label_id = edge_info.edge_type_id;
        if let Some(src_label_id) = graph.get_vertex_label_id(&edge_info.src_tag_name) {
            if let Some(dst_label_id) = graph.get_vertex_label_id(&edge_info.dst_tag_name) {
                if let Some(record) =
                    graph.get_edge(edge_label_id, src_label_id, &src_str, dst_label_id, &dst_str, ts)
                {
                    let edge = edge_record_to_edge(&record, edge_type, &src_str, &dst_str);
                    return Ok(Some(edge));
                }
            }
        }

        Ok(None)
    }

    pub fn get_node_edges(
        &self,
        space: &str,
        node_id: &Value,
        direction: EdgeDirection,
    ) -> StorageResult<Vec<Edge>> {
        let edge_types = self.ctx.schema_manager.list_edge_types(space)?;
        if edge_types.is_empty() {
            return Ok(Vec::new());
        }

        let ts = self.ctx.get_read_timestamp();
        let graph = self.ctx.graph.read();
        let node_str = value_to_string(node_id);
        let mut edges = Vec::new();

        for edge_info in &edge_types {
            let edge_label_id = edge_info.edge_type_id;
            let edge_type_name = &edge_info.edge_type_name;

            if let Some(src_label_id) = graph.get_vertex_label_id(&edge_info.src_tag_name) {
                if let Some(dst_label_id) = graph.get_vertex_label_id(&edge_info.dst_tag_name) {
                    match direction {
                        EdgeDirection::Out => {
                            if let Some(out_edges) =
                                graph.out_edges(edge_label_id, src_label_id, dst_label_id, &node_str, ts)
                            {
                                for record in out_edges {
                                    let edge = edge_record_to_edge(
                                        &record,
                                        edge_type_name,
                                        &node_str,
                                        &format!("{}", record.dst_vid),
                                    );
                                    edges.push(edge);
                                }
                            }
                        }
                        EdgeDirection::In => {
                            if let Some(in_edges) =
                                graph.in_edges(edge_label_id, src_label_id, dst_label_id, &node_str, ts)
                            {
                                for record in in_edges {
                                    let edge = edge_record_to_edge(
                                        &record,
                                        edge_type_name,
                                        &format!("{}", record.src_vid),
                                        &node_str,
                                    );
                                    edges.push(edge);
                                }
                            }
                        }
                        EdgeDirection::Both => {
                            if let Some(out_edges) =
                                graph.out_edges(edge_label_id, src_label_id, dst_label_id, &node_str, ts)
                            {
                                for record in out_edges {
                                    let edge = edge_record_to_edge(
                                        &record,
                                        edge_type_name,
                                        &node_str,
                                        &format!("{}", record.dst_vid),
                                    );
                                    edges.push(edge);
                                }
                            }
                            if let Some(in_edges) =
                                graph.in_edges(edge_label_id, src_label_id, dst_label_id, &node_str, ts)
                            {
                                for record in in_edges {
                                    let edge = edge_record_to_edge(
                                        &record,
                                        edge_type_name,
                                        &format!("{}", record.src_vid),
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

    pub fn get_node_edges_filtered<F>(
        &self,
        space: &str,
        node_id: &Value,
        direction: EdgeDirection,
        filter: Option<F>,
    ) -> StorageResult<Vec<Edge>>
    where
        F: Fn(&Edge) -> bool,
    {
        let edges = self.get_node_edges(space, node_id, direction)?;
        match filter {
            Some(f) => Ok(edges.into_iter().filter(f).collect()),
            None => Ok(edges),
        }
    }

    pub fn scan_edges_by_type(&self, space: &str, edge_type: &str) -> StorageResult<Vec<Edge>> {
        let edge_info = self.ctx.schema_manager.get_edge_type(space, edge_type)?.ok_or_else(|| {
            StorageError::not_found(format!("Edge type {} not found in space {}", edge_type, space))
        })?;

        let ts = self.ctx.get_read_timestamp();
        let graph = self.ctx.graph.read();
        let mut edges = Vec::new();

        let edge_label_id = edge_info.edge_type_id;
        if let Some(src_label_id) = graph.get_vertex_label_id(&edge_info.src_tag_name) {
            if let Some(dst_label_id) = graph.get_vertex_label_id(&edge_info.dst_tag_name) {
                if let Some(table) = graph.get_edge_table(src_label_id, dst_label_id, edge_label_id) {
                    for record in table.scan(ts) {
                        let edge = edge_record_to_edge(
                            &record,
                            edge_type,
                            &format!("{}", record.src_vid),
                            &format!("{}", record.dst_vid),
                        );
                        edges.push(edge);
                    }
                }
            }
        }

        Ok(edges)
    }

    pub fn scan_all_edges(&self, space: &str) -> StorageResult<Vec<Edge>> {
        let _space_info = self.ctx.schema_manager.get_space(space)?.ok_or_else(|| {
            StorageError::not_found(format!("Space {} not found", space))
        })?;

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
    ) -> StorageResult<Option<(Schema, Vec<u8>)>> {
        let tag_info = self.ctx.schema_manager.get_tag(space, tag)?.ok_or_else(|| {
            StorageError::not_found(format!("Tag {} not found in space {}", tag, space))
        })?;

        let ts = self.ctx.get_read_timestamp();
        let graph = self.ctx.graph.read();
        let id_str = value_to_string(id);

        let label_id = tag_info.tag_id;
        if let Some(record) = graph.get_vertex(label_id, &id_str, ts) {
            let schema = self.ctx.schema_manager.get_tag_schema(space, tag)?;
            let data = serialize_properties(&record.properties);
            return Ok(Some((schema, data)));
        }

        Ok(None)
    }

    pub fn get_edge_with_schema(
        &self,
        space: &str,
        edge_type: &str,
        src: &Value,
        dst: &Value,
    ) -> StorageResult<Option<(Schema, Vec<u8>)>> {
        let edge_info = self.ctx.schema_manager.get_edge_type(space, edge_type)?.ok_or_else(|| {
            StorageError::not_found(format!("Edge type {} not found in space {}", edge_type, space))
        })?;

        let ts = self.ctx.get_read_timestamp();
        let graph = self.ctx.graph.read();
        let src_str = value_to_string(src);
        let dst_str = value_to_string(dst);

        let edge_label_id = edge_info.edge_type_id;
        if let Some(src_label_id) = graph.get_vertex_label_id(&edge_info.src_tag_name) {
            if let Some(dst_label_id) = graph.get_vertex_label_id(&edge_info.dst_tag_name) {
                if let Some(record) =
                    graph.get_edge(edge_label_id, src_label_id, &src_str, dst_label_id, &dst_str, ts)
                {
                    let schema = self.ctx.schema_manager.get_edge_type_schema(space, edge_type)?;
                    let data = serialize_properties(&record.properties);
                    return Ok(Some((schema, data)));
                }
            }
        }

        Ok(None)
    }

    pub fn scan_vertices_with_schema(
        &self,
        space: &str,
        tag: &str,
    ) -> StorageResult<Vec<(Schema, Vec<u8>)>> {
        let tag_info = self.ctx.schema_manager.get_tag(space, tag)?.ok_or_else(|| {
            StorageError::not_found(format!("Tag {} not found in space {}", tag, space))
        })?;

        let ts = self.ctx.get_read_timestamp();
        let graph = self.ctx.graph.read();
        let mut results = Vec::new();

        let label_id = tag_info.tag_id;
        if let Some(iterator) = graph.scan_vertices(label_id, ts) {
            let schema = self.ctx.schema_manager.get_tag_schema(space, tag)?;
            for record in iterator {
                let data = serialize_properties(&record.properties);
                results.push((schema.clone(), data));
            }
        }

        Ok(results)
    }

    pub fn scan_edges_with_schema(
        &self,
        space: &str,
        edge_type: &str,
    ) -> StorageResult<Vec<(Schema, Vec<u8>)>> {
        let edge_info = self.ctx.schema_manager.get_edge_type(space, edge_type)?.ok_or_else(|| {
            StorageError::not_found(format!("Edge type {} not found in space {}", edge_type, space))
        })?;

        let ts = self.ctx.get_read_timestamp();
        let graph = self.ctx.graph.read();
        let mut results = Vec::new();

        let edge_label_id = edge_info.edge_type_id;
        if let Some(src_label_id) = graph.get_vertex_label_id(&edge_info.src_tag_name) {
            if let Some(dst_label_id) = graph.get_vertex_label_id(&edge_info.dst_tag_name) {
                if let Some(table) = graph.get_edge_table(src_label_id, dst_label_id, edge_label_id) {
                    let schema = self.ctx.schema_manager.get_edge_type_schema(space, edge_type)?;

                    for record in table.scan(ts) {
                        let data = serialize_properties(&record.properties);
                        results.push((schema.clone(), data));
                    }
                }
            }
        }

        Ok(results)
    }
}
