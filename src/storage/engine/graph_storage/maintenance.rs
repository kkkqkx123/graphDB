//! Maintenance Operations
//!
//! Provides maintenance operations like stats and dangling edge detection.

use crate::core::{Edge, StorageError, StorageResult};
use crate::storage::interface::StorageStats;

use super::context::GraphStorageContext;
use super::type_utils::edge_record_to_edge;

pub struct MaintenanceOps<'a> {
    ctx: &'a GraphStorageContext,
}

impl<'a> MaintenanceOps<'a> {
    pub fn new(ctx: &'a GraphStorageContext) -> Self {
        Self { ctx }
    }

    pub fn get_storage_stats(&self) -> StorageStats {
        let graph = self.ctx.graph.read();

        let total_vertices: usize = graph
            .vertex_tables()
            .values()
            .map(|table| table.total_count())
            .sum();

        let total_edges: usize = graph
            .edge_tables()
            .map(|(_, table)| table.edge_count() as usize)
            .sum();

        let spaces = self.ctx.schema_manager.list_spaces().unwrap_or_default();
        let tags = spaces
            .iter()
            .filter_map(|s| self.ctx.schema_manager.list_tags(&s.space_name).ok())
            .flatten()
            .count();

        let edge_types = spaces
            .iter()
            .filter_map(|s| self.ctx.schema_manager.list_edge_types(&s.space_name).ok())
            .flatten()
            .count();

        StorageStats {
            total_vertices,
            total_edges,
            total_spaces: spaces.len(),
            total_tags: tags,
            total_edge_types: edge_types,
        }
    }

    pub fn find_dangling_edges(&self, space: &str) -> StorageResult<Vec<Edge>> {
        let _space_info = self.ctx.schema_manager.get_space(space)?.ok_or_else(|| {
            StorageError::not_found(format!("Space {} not found", space))
        })?;

        let ts = self.ctx.get_read_timestamp();
        let graph = self.ctx.graph.read();
        let mut dangling_edges = Vec::new();

        for ((src_label_id, dst_label_id, _edge_label_id), table) in graph.edge_tables() {
            let edge_type_name = table.label_name().to_string();
            for record in table.scan(ts) {
                let src_exists = graph
                    .get_vertex_by_internal_id(*src_label_id, record.src_vid as u32, ts)
                    .is_some();
                let dst_exists = graph
                    .get_vertex_by_internal_id(*dst_label_id, record.dst_vid as u32, ts)
                    .is_some();

                if !src_exists || !dst_exists {
                    let edge = edge_record_to_edge(
                        &record,
                        &edge_type_name,
                        &format!("{}", record.src_vid),
                        &format!("{}", record.dst_vid),
                    );
                    dangling_edges.push(edge);
                }
            }
        }

        Ok(dangling_edges)
    }

    pub fn repair_dangling_edges(
        &self,
        space: &str,
        writer: &super::writer::GraphStorageWriter<'_>,
    ) -> StorageResult<usize> {
        let dangling_edges = self.find_dangling_edges(space)?;
        let mut repaired_count = 0;

        for edge in &dangling_edges {
            if writer
                .delete_edge(space, &edge.src, &edge.dst, &edge.edge_type, edge.ranking)
                .is_ok()
            {
                repaired_count += 1;
            }
        }

        Ok(repaired_count)
    }
}
