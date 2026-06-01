use std::path::Path;

use crate::core::types::CompactTarget;
use crate::core::{StorageError, StorageResult};
use crate::storage::engine::persistence_coordinator::{
    CheckpointData, CheckpointInfo, CheckpointStats,
};
use crate::transaction::compact_transaction::CompactTransaction;
use crate::transaction::wal::recovery::{RecoveryConfig, RecoveryManager, RecoveryStats};

use super::context::GraphStorageContext;

pub(crate) fn save_data(ctx: &GraphStorageContext) -> StorageResult<()> {
    let work_dir = ctx
        .work_dir
        .as_ref()
        .ok_or_else(|| StorageError::db_error("No work directory configured".to_string()))?;

    save_data_to_dir(ctx, work_dir)
}

pub(crate) fn save_data_to_dir(ctx: &GraphStorageContext, dir: &Path) -> StorageResult<()> {
    use std::fs::{self, File};
    use std::io::Write;

    let data_dir = dir.join("data");
    fs::create_dir_all(&data_dir)?;

    let version_file = data_dir.join("version");
    let mut file = File::create(&version_file)?;
    writeln!(file, "1")?;

    ctx.graph.flush_tables_to_dir(&data_dir)?;

    log::info!("Data saved to {:?}", data_dir);
    Ok(())
}

pub(crate) fn flush(ctx: &GraphStorageContext) -> StorageResult<()> {
    save_data(ctx)
}

pub(crate) fn create_checkpoint(
    ctx: &GraphStorageContext,
) -> StorageResult<Option<CheckpointStats>> {
    let persistence = match &ctx.persistence {
        Some(p) => p,
        None => return Ok(None),
    };

    let ts = ctx.get_write_timestamp();
    let graph = ctx.graph.clone();

    let stats = persistence.write().create_checkpoint(
        |checkpoint_dir, _timestamp| {
            let data_dir = checkpoint_dir.join("data");
            std::fs::create_dir_all(&data_dir)?;

            graph.flush_tables_to_dir(&data_dir)?;

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

pub(crate) fn load_latest_checkpoint(
    ctx: &GraphStorageContext,
) -> StorageResult<Option<CheckpointInfo>> {
    let persistence = match &ctx.persistence {
        Some(p) => p,
        None => return Ok(None),
    };

    let graph = ctx.graph.clone();

    persistence
        .write()
        .load_latest_checkpoint(|checkpoint_dir| graph.restore_from_checkpoint(checkpoint_dir))
}

pub(crate) fn should_flush(ctx: &GraphStorageContext) -> bool {
    if let Some(ref persistence) = ctx.persistence {
        persistence.read().should_flush()
    } else {
        false
    }
}

pub(crate) fn should_checkpoint(ctx: &GraphStorageContext) -> bool {
    if let Some(ref persistence) = ctx.persistence {
        persistence.read().should_checkpoint()
    } else {
        false
    }
}

pub(crate) fn auto_flush_if_needed(ctx: &GraphStorageContext) -> StorageResult<bool> {
    if should_flush(ctx) {
        flush(ctx)?;
        if let Some(ref persistence) = ctx.persistence {
            persistence.write().reset_flush_timer();
        }
        return Ok(true);
    }
    Ok(false)
}

pub(crate) fn auto_checkpoint_if_needed(
    ctx: &GraphStorageContext,
) -> StorageResult<Option<CheckpointStats>> {
    if should_checkpoint(ctx) {
        let stats = create_checkpoint(ctx)?;
        if let Some(ref persistence) = ctx.persistence {
            persistence.write().reset_checkpoint_timer();
        }
        return Ok(stats);
    }
    Ok(None)
}

pub(crate) fn compact_transactional(
    ctx: &GraphStorageContext,
    compact_csr: bool,
    reserve_ratio: f32,
) -> StorageResult<()> {
    let persistence = ctx.persistence.as_ref().ok_or_else(|| {
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
    let version_manager = &ctx.version_manager;

    let txn = CompactTransaction::new(
        &*ctx.graph,
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

    let after_stats = ctx.graph.get_compact_stats();
    log::info!(
        "Compaction completed: size={}/{} (freed {} bytes)",
        after_stats.used_size,
        after_stats.total_size,
        before_stats.used_size.saturating_sub(after_stats.used_size)
    );

    Ok(())
}

pub(crate) fn load_from_disk(ctx: &GraphStorageContext) -> StorageResult<()> {
    if let Some(ref path) = ctx.work_dir {
        let schema_path = path.join("schema");
        if schema_path.exists() {
            ctx.schema_manager.load_schema(&schema_path)?;
        }

        let index_meta_path = path.join("index_meta");
        if index_meta_path.exists() {
            ctx.index_metadata_manager.load_indexes(&index_meta_path)?;
        }

        super::schema_adapter::ensure_graph_types_from_schema(ctx)?;
        ctx.graph.restore_from_checkpoint(path)?;

        let index_path = path.join("indexes");
        if index_path.exists() {
            ctx.graph.index_data_manager().write().load(&index_path)?;
        }
    }
    Ok(())
}

pub(crate) fn save_to_disk(ctx: &GraphStorageContext) -> StorageResult<()> {
    if let Some(ref path) = ctx.work_dir {
        std::fs::create_dir_all(path).map_err(|e| StorageError::io_error(e.to_string()))?;

        let schema_path = path.join("schema");
        std::fs::create_dir_all(&schema_path).map_err(|e| StorageError::io_error(e.to_string()))?;
        ctx.schema_manager.save_schema(&schema_path)?;

        let index_meta_path = path.join("index_meta");
        std::fs::create_dir_all(&index_meta_path)
            .map_err(|e| StorageError::io_error(e.to_string()))?;
        ctx.index_metadata_manager.save_indexes(&index_meta_path)?;

        save_data_to_dir(ctx, path)?;

        let index_path = path.join("indexes");
        std::fs::create_dir_all(&index_path).map_err(|e| StorageError::io_error(e.to_string()))?;
        ctx.graph.index_data_manager().read().flush(&index_path)?;
    }
    Ok(())
}

pub(crate) fn recover_from_wal(ctx: &GraphStorageContext) -> StorageResult<RecoveryStats> {
    let work_dir = ctx
        .work_dir
        .as_ref()
        .ok_or_else(|| StorageError::db_error("No work directory configured".to_string()))?;

    let config = RecoveryConfig {
        wal_dir: work_dir.join("wal"),
        data_dir: work_dir.join("data"),
        ..Default::default()
    };

    let mut manager = RecoveryManager::new(config);

    manager.recover_with_applier(&*ctx.graph)
}

pub(crate) fn recover_from_wal_with_config(
    ctx: &GraphStorageContext,
    config: RecoveryConfig,
) -> StorageResult<RecoveryStats> {
    let mut manager = RecoveryManager::new(config);

    manager.recover_with_applier(&*ctx.graph)
}

pub(crate) fn needs_recovery(ctx: &GraphStorageContext) -> bool {
    if let Some(ref work_dir) = ctx.work_dir {
        let wal_dir = work_dir.join("wal");
        if wal_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&wal_dir) {
                return entries.count() > 0;
            }
        }
    }
    false
}
