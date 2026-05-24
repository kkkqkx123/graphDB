use crate::core::types::{CompactConfig, CompactResult, CompactStats, CompactTarget};
use crate::core::types::{LabelId, Timestamp};
use crate::storage::engine::data_store::EdgeTableKey;
use crate::storage::engine::property_graph::PropertyGraph;

impl CompactTarget for PropertyGraph {
    fn compact(&self, config: &CompactConfig, ts: Timestamp) -> CompactResult<()> {
        log::info!(
            "Starting compaction: enable_structure_compaction={}, reserve_ratio={}, ts={}",
            config.enable_structure_compaction,
            config.reserve_ratio,
            ts
        );

        let mut total_vertices_removed = 0usize;
        let mut total_edges_removed = 0usize;

        *self.last_compacted_vertices.lock() = Vec::new();

        let vertex_labels: Vec<LabelId>;
        {
            let mut vertex_tables = self.data_store.vertex_tables().write();
            vertex_labels = vertex_tables.keys().copied().collect();

            for &label_id in &vertex_labels {
                let table = vertex_tables
                    .get_mut(&label_id)
                    .expect("label must exist");
                let removed = table.compact_with_ts_collect(ts);
                total_vertices_removed += removed.len();
                if !removed.is_empty() {
                    self.last_compacted_vertices
                        .lock()
                        .push((label_id, removed));
                }
            }
        }

        for &label_id in &vertex_labels {
            self.mark_vertex_modified(label_id);
        }

        log::info!(
            "Compacted vertex tables: {} vertices removed",
            total_vertices_removed
        );

        let edge_keys: Vec<EdgeTableKey>;
        {
            let mut edge_tables = self.data_store.edge_tables().write();
            edge_keys = edge_tables.keys().copied().collect();

            if config.enable_structure_compaction {
                for &key in &edge_keys {
                    let table = edge_tables.get_mut(&key).expect("edge key must exist");
                    let removed = table.compact_csr(ts, config.reserve_ratio);
                    total_edges_removed += removed;
                }

                log::info!(
                    "Compacted CSR structures: {} edges removed",
                    total_edges_removed
                );
            }

            for &key in &edge_keys {
                let table = edge_tables.get_mut(&key).expect("edge key must exist");
                table.compact_properties(ts);
            }
        }

        for &key in &edge_keys {
            self.mark_edge_modified(key.edge_label);
        }

        let index_gc_stats = self.gc_index_tombstones(ts).unwrap_or_default();
        if index_gc_stats.total_removed() > 0 {
            log::info!(
                "Index GC during compaction: removed {} vertex entries, {} edge entries",
                index_gc_stats.vertex_entries_removed,
                index_gc_stats.edge_entries_removed
            );
        }

        self.cache_manager.clear_cache();

        log::info!(
            "Compaction completed: {} vertices, {} edges removed",
            total_vertices_removed,
            total_edges_removed
        );

        Ok(())
    }

    fn get_compact_stats(&self) -> CompactStats {
        let total = self.storage_size();
        let used = self.used_storage_size();
        CompactStats::new(total, used)
    }
}
