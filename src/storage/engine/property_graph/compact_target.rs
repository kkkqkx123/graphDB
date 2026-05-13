//! CompactTarget Implementation
//!
//! Implements the CompactTarget trait for PropertyGraph.

use crate::storage::metadata::TableId;
use crate::transaction::compact_transaction::{CompactTarget, CompactTransactionResult};
use crate::transaction::wal::types::Timestamp;

use super::PropertyGraph;

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

        for (label_id, table) in &mut self.schema_ops.vertex_tables {
            let removed = table.compact_with_ts_collect(ts);
            total_vertices_removed += removed.len();
            if !removed.is_empty() {
                self.last_compacted_vertices.push((*label_id, removed));
            }
            self.table_tracker.mark_modified(TableId::vertex(*label_id));
        }

        log::info!(
            "Compacted vertex tables: {} vertices removed",
            total_vertices_removed
        );

        if compact_csr {
            for ((src_label, dst_label, edge_label), table) in &mut self.edge_ops.edge_tables {
                let removed = table.compact_csr(ts, reserve_ratio);
                total_edges_removed += removed;
                self.table_tracker.mark_modified(TableId::edge(*edge_label));
            }

            log::info!(
                "Compacted CSR structures: {} edges removed",
                total_edges_removed
            );
        }

        for ((_, _, edge_label), table) in &mut self.edge_ops.edge_tables {
            table.compact_properties(ts);
            self.table_tracker.mark_modified(TableId::edge(*edge_label));
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
