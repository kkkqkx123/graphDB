use crate::core::types::{CompactConfig, CompactResult, CompactStats, CompactTarget};
use crate::core::types::{CompactError, Timestamp};
use crate::storage::engine::graph_storage::GraphStorageContext;

impl CompactTarget for GraphStorageContext {
    fn compact(&self, config: &CompactConfig, ts: Timestamp) -> CompactResult<()> {
        log::info!(
            "Starting compaction: enable_structure_compaction={}, reserve_ratio={}, ts={}",
            config.enable_structure_compaction,
            config.reserve_ratio,
            ts
        );

        self.compact_maintenance(config, ts)
            .map_err(|err| CompactError::StorageError(err.to_string()))
    }

    fn get_compact_stats(&self) -> CompactStats {
        let total = self.storage_size();
        let used = self.used_storage_size();
        CompactStats::new(total, used)
    }
}
