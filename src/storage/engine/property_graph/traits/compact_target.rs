//! CompactTarget Implementation
//!
//! Implements the CompactTarget trait for PropertyGraph.

use crate::storage::vertex::LabelId;
use crate::transaction::compact_transaction::{CompactTarget, CompactTransactionResult};
use crate::transaction::wal::types::Timestamp;

use super::super::PropertyGraph;

impl CompactTarget for PropertyGraph {
    fn compact(
        &mut self,
        compact_csr: bool,
        reserve_ratio: f32,
        ts: Timestamp,
    ) -> CompactTransactionResult<()> {
        log::info!(
            "Starting compaction: compact_csr={}, reserve_ratio={}, ts={}",
            compact_csr,
            reserve_ratio,
            ts
        );

        let mut total_vertices_removed = 0usize;
        let mut total_edges_removed = 0usize;

        self.last_compacted_vertices.clear();

        let vertex_labels: Vec<LabelId> =
            self.schema_ops.vertex_tables.keys().copied().collect();

        for &label_id in &vertex_labels {
            let table = self
                .schema_ops
                .vertex_tables
                .get_mut(&label_id)
                .expect("label must exist");
            let removed = table.compact_with_ts_collect(ts);
            total_vertices_removed += removed.len();
            if !removed.is_empty() {
                self.last_compacted_vertices.push((label_id, removed));
            }
        }

        for &label_id in &vertex_labels {
            self.mark_vertex_modified(label_id);
        }

        log::info!(
            "Compacted vertex tables: {} vertices removed",
            total_vertices_removed
        );

        let edge_keys: Vec<(LabelId, LabelId, LabelId)> =
            self.edge_ops.edge_tables.keys().copied().collect();

        if compact_csr {
            for &key in &edge_keys {
                let table = self
                    .edge_ops
                    .edge_tables
                    .get_mut(&key)
                    .expect("edge key must exist");
                let removed = table.compact_csr(ts, reserve_ratio);
                total_edges_removed += removed;
            }

            for &(_, _, edge_label) in &edge_keys {
                self.mark_edge_modified(edge_label);
            }

            log::info!(
                "Compacted CSR structures: {} edges removed",
                total_edges_removed
            );
        }

        for &key in &edge_keys {
            let table = self
                .edge_ops
                .edge_tables
                .get_mut(&key)
                .expect("edge key must exist");
            table.compact_properties(ts);
        }

        for &(_, _, edge_label) in &edge_keys {
            self.mark_edge_modified(edge_label);
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

    fn storage_size(&self) -> usize {
        let mut total = 0usize;

        for table in self.schema_ops.vertex_tables.values() {
            total += table.memory_size();
        }

        for table in self.edge_ops.edge_tables.values() {
            total += table.memory_size();
        }

        total
    }

    fn used_storage_size(&self) -> usize {
        let mut total = 0usize;

        for table in self.schema_ops.vertex_tables.values() {
            total += table.used_memory_size();
        }

        for table in self.edge_ops.edge_tables.values() {
            total += table.used_memory_size();
        }

        total
    }
}
