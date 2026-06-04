use std::path::Path;

use crate::core::types::CompactTarget;
use crate::core::{StorageError, StorageResult};
use crate::storage::engine::paths::StoragePaths;
use crate::storage::engine::persistence_coordinator::{
    CheckpointData, CheckpointInfo, CheckpointStats,
};
use crate::transaction::compact_transaction::CompactTransaction;
use crate::transaction::wal::recovery::{RecoveryConfig, RecoveryManager, RecoveryStats};

use super::context::GraphStorageContext;

fn load_schema_and_index_metadata(ctx: &GraphStorageContext) -> StorageResult<()> {
    if let Some(path) = ctx.work_dir().as_ref() {
        let paths = StoragePaths::new(path.clone());

        let schema_path = paths.schema_file();
        if schema_path.exists() {
            ctx.schema_manager().load_schema(&schema_path)?;
        }

        let index_meta_path = paths.index_meta_file();
        if index_meta_path.exists() {
            ctx.index_metadata_manager()
                .load_indexes(&index_meta_path)?;
        }
    }

    Ok(())
}

fn restore_full_state_from_disk(ctx: &GraphStorageContext) -> StorageResult<()> {
    if let Some(path) = ctx.work_dir().as_ref() {
        let paths = StoragePaths::new(path.clone());
        ctx.restore_from_checkpoint(path)?;
        ctx.user_storage().load_from_dir(paths.data_dir())?;

        let index_path = paths.indexes_dir();
        if index_path.exists() {
            ctx.index_data_manager().write().load(&index_path)?;
        }
    }

    Ok(())
}

pub(crate) fn bootstrap_from_disk(ctx: &GraphStorageContext) -> StorageResult<()> {
    load_schema_and_index_metadata(ctx)?;
    super::schema_adapter::ensure_graph_types_from_schema(ctx)?;

    if load_latest_checkpoint(ctx)?.is_none() {
        restore_full_state_from_disk(ctx)?;
    }

    Ok(())
}

pub(crate) fn initialize_with_recovery(
    ctx: &GraphStorageContext,
) -> StorageResult<Option<RecoveryStats>> {
    bootstrap_from_disk(ctx)?;

    if !needs_recovery(ctx) {
        return Ok(None);
    }

    log::info!("WAL recovery needed, starting recovery...");
    let stats = recover_from_wal(ctx)?;

    log::info!(
        "WAL recovery completed: {} entries replayed in {}ms",
        stats.wal_entries_replayed,
        stats.recovery_time_ms
    );

    Ok(Some(stats))
}

pub(crate) fn save_data(ctx: &GraphStorageContext) -> StorageResult<()> {
    let paths = ctx
        .storage_paths()
        .ok_or_else(|| StorageError::db_error("No work directory configured".to_string()))?;

    save_data_to_dir(ctx, paths.root())
}

pub(crate) fn save_data_to_dir(ctx: &GraphStorageContext, dir: &Path) -> StorageResult<()> {
    use std::fs::{self, File};
    use std::io::Write;

    let paths = StoragePaths::new(dir);
    let data_dir = paths.data_dir();
    fs::create_dir_all(&data_dir)?;

    let version_file = paths.version_file();
    let mut file = File::create(&version_file)?;
    writeln!(file, "1")?;

    ctx.flush_tables_to_dir(&data_dir)?;
    ctx.user_storage().save_to_dir(&data_dir)?;

    if let Some(persistence) = ctx.persistence().as_ref() {
        let wal_lsn = {
            let coordinator = persistence.read();
            coordinator.wal_manager().read().current_lsn()
        };
        persistence.write().mark_flushed(wal_lsn);
    }

    log::info!("Data saved to {:?}", data_dir);
    Ok(())
}

pub(crate) fn flush(ctx: &GraphStorageContext) -> StorageResult<()> {
    save_data(ctx)
}

pub(crate) fn create_checkpoint(
    ctx: &GraphStorageContext,
) -> StorageResult<Option<CheckpointStats>> {
    let persistence = match ctx.persistence().as_ref() {
        Some(p) => p,
        None => return Ok(None),
    };

    let ts = ctx.get_write_timestamp();
    let graph = ctx.clone();
    let user_storage = ctx.user_storage().clone();

    let stats = persistence.write().create_checkpoint(
        |checkpoint_dir, _timestamp| {
            let data_dir = StoragePaths::new(checkpoint_dir).data_dir();
            std::fs::create_dir_all(&data_dir)?;

            graph.flush_tables_to_dir(&data_dir)?;
            user_storage.save_to_dir(&data_dir)?;

            let vertex_count = graph.total_vertex_count() as u64;
            let edge_count = graph.total_edge_count() as u64;

            let data_size = std::fs::metadata(&data_dir).map(|m| m.len()).unwrap_or(0);

            Ok(CheckpointData {
                vertex_count,
                edge_count,
                data_size,
            })
        },
        ts,
    )?;

    Ok(Some(stats))
}

pub(crate) fn verify_snapshot(ctx: &GraphStorageContext, snapshot_id: u64) -> StorageResult<bool> {
    let persistence = ctx
        .persistence()
        .as_ref()
        .ok_or_else(|| StorageError::not_supported("Snapshots are not available"))?;

    persistence.read().verify_snapshot(snapshot_id)
}

pub(crate) fn cleanup_snapshots(ctx: &GraphStorageContext) -> StorageResult<usize> {
    let persistence = ctx
        .persistence()
        .as_ref()
        .ok_or_else(|| StorageError::not_supported("Snapshots are not available"))?;

    persistence.read().cleanup_old_snapshots()
}

pub(crate) fn snapshot_stats(ctx: &GraphStorageContext) -> crate::storage::SnapshotStats {
    ctx.persistence()
        .as_ref()
        .map(|persistence| persistence.read().snapshot_stats())
        .unwrap_or_default()
}

pub(crate) fn load_latest_checkpoint(
    ctx: &GraphStorageContext,
) -> StorageResult<Option<CheckpointInfo>> {
    let persistence = match &ctx.persistence() {
        Some(p) => p,
        None => return Ok(None),
    };

    let graph = ctx.clone();
    let user_storage = ctx.user_storage().clone();

    persistence
        .write()
        .load_latest_checkpoint(|checkpoint_dir| {
            graph.restore_from_checkpoint(checkpoint_dir)?;
            user_storage.load_from_dir(StoragePaths::new(checkpoint_dir).data_dir())
        })
        .map(|result| {
            if let Some(ref info) = result {
                persistence.write().mark_checkpointed(info.lsn);
            }
            result
        })
}

pub(crate) fn should_flush(ctx: &GraphStorageContext) -> bool {
    if let Some(persistence) = ctx.persistence().as_ref() {
        persistence.read().should_flush()
    } else {
        false
    }
}

pub(crate) fn should_checkpoint(ctx: &GraphStorageContext) -> bool {
    if let Some(persistence) = ctx.persistence().as_ref() {
        persistence.read().should_checkpoint()
    } else {
        false
    }
}

pub(crate) fn auto_flush_if_needed(ctx: &GraphStorageContext) -> StorageResult<bool> {
    if should_flush(ctx) {
        flush(ctx)?;
        return Ok(true);
    }
    Ok(false)
}

pub(crate) fn auto_checkpoint_if_needed(
    ctx: &GraphStorageContext,
) -> StorageResult<Option<CheckpointStats>> {
    if should_checkpoint(ctx) {
        let stats = create_checkpoint(ctx)?;
        return Ok(stats);
    }
    Ok(None)
}

pub(crate) fn compact_transactional(
    ctx: &GraphStorageContext,
    compact_csr: bool,
    reserve_ratio: f32,
) -> StorageResult<()> {
    let persistence = ctx.persistence().as_ref().ok_or_else(|| {
        StorageError::db_error("Persistence not available for transactional compaction".to_string())
    })?;

    let wal_writer = {
        let coordinator = persistence.read();
        let wal_mgr = coordinator.wal_manager();
        let wal_reader = wal_mgr.read();
        wal_reader
            .writer()
            .ok_or_else(|| StorageError::db_error("WAL writer not initialized".to_string()))?
    };

    let mut wal_writer_guard = wal_writer.write();
    let version_manager = ctx.version_manager().as_ref();

    let txn = CompactTransaction::new(
        ctx,
        version_manager,
        &mut *wal_writer_guard,
        compact_csr,
        reserve_ratio,
    )
    .map_err(|e| StorageError::db_error(format!("Failed to create compact transaction: {}", e)))?;

    let before_stats = txn.storage_stats();
    log::info!(
        "Starting transactional compaction: compact_csr={}, reserve_ratio={:.2}, size={}/{}",
        compact_csr,
        reserve_ratio,
        before_stats.used_size,
        before_stats.total_size
    );

    txn.commit()
        .map_err(|e| StorageError::db_error(format!("Compact transaction failed: {}", e)))?;

    let after_stats = ctx.get_compact_stats();
    log::info!(
        "Compaction completed: size={}/{} (freed {} bytes)",
        after_stats.used_size,
        after_stats.total_size,
        before_stats.used_size.saturating_sub(after_stats.used_size)
    );

    Ok(())
}

pub(crate) fn load_from_disk(ctx: &GraphStorageContext) -> StorageResult<()> {
    load_schema_and_index_metadata(ctx)?;
    super::schema_adapter::ensure_graph_types_from_schema(ctx)?;
    restore_full_state_from_disk(ctx)
}

pub(crate) fn save_to_disk(ctx: &GraphStorageContext) -> StorageResult<()> {
    if let Some(path) = ctx.work_dir().as_ref() {
        let paths = StoragePaths::new(path.clone());
        std::fs::create_dir_all(paths.root()).map_err(|e| StorageError::io_error(e.to_string()))?;

        let schema_dir = paths.schema_dir();
        std::fs::create_dir_all(&schema_dir).map_err(|e| StorageError::io_error(e.to_string()))?;
        let schema_path = paths.schema_file();
        ctx.schema_manager().save_schema(&schema_path)?;

        let index_meta_dir = paths.index_meta_dir();
        std::fs::create_dir_all(&index_meta_dir)
            .map_err(|e| StorageError::io_error(e.to_string()))?;
        let index_meta_path = paths.index_meta_file();
        ctx.index_metadata_manager()
            .save_indexes(&index_meta_path)?;

        save_data_to_dir(ctx, paths.root())?;

        let index_path = paths.indexes_dir();
        std::fs::create_dir_all(&index_path).map_err(|e| StorageError::io_error(e.to_string()))?;
        ctx.index_data_manager().read().flush(&index_path)?;
    }
    Ok(())
}

pub(crate) fn recover_from_wal(ctx: &GraphStorageContext) -> StorageResult<RecoveryStats> {
    let paths = ctx
        .storage_paths()
        .ok_or_else(|| StorageError::db_error("No work directory configured".to_string()))?;

    let config = RecoveryConfig {
        wal_dir: paths.wal_dir(),
        data_dir: paths.data_dir(),
        ..Default::default()
    };

    let mut manager = RecoveryManager::new(config);

    let stats = manager.recover_with_applier(ctx)?;

    if let Some(persistence) = ctx.persistence().as_ref() {
        persistence.write().mark_checkpointed(stats.last_lsn);
    }

    Ok(stats)
}

pub(crate) fn recover_from_wal_with_config(
    ctx: &GraphStorageContext,
    config: RecoveryConfig,
) -> StorageResult<RecoveryStats> {
    let mut manager = RecoveryManager::new(config);

    let stats = manager.recover_with_applier(ctx)?;

    if let Some(persistence) = ctx.persistence().as_ref() {
        persistence.write().mark_checkpointed(stats.last_lsn);
    }

    Ok(stats)
}

pub(crate) fn needs_recovery(ctx: &GraphStorageContext) -> bool {
    if let Some(paths) = ctx.storage_paths() {
        let wal_dir = paths.wal_dir();
        if wal_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&wal_dir) {
                return entries.count() > 0;
            }
        }
    }
    false
}
