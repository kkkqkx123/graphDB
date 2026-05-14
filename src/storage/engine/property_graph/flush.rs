//! Flush Operations
//!
//! Contains flush, load, and checkpoint operations for PropertyGraph.
//! This module handles low-level data persistence at the storage engine level.

use std::path::Path;

use crate::core::types::LabelId;
use crate::core::{StorageError, StorageResult};
use crate::storage::metadata::{TableId, TableType};

use super::{DATA_FORMAT_VERSION, PropertyGraph};

pub fn flush(graph: &PropertyGraph) -> StorageResult<()> {
    use std::fs;
    use std::io::Write;

    let data_dir = graph.config.work_dir.join("data");
    fs::create_dir_all(&data_dir)?;

    let version_file = data_dir.join("version");
    let mut file = fs::File::create(&version_file)
        .map_err(|e| StorageError::io_error(format!("Failed to create version file: {}", e)))?;
    writeln!(file, "{}", DATA_FORMAT_VERSION)
        .map_err(|e| StorageError::io_error(format!("Failed to write version file: {}", e)))?;

    let vertex_dir = data_dir.join("vertices");
    fs::create_dir_all(&vertex_dir)?;

    for (label_id, table) in &graph.schema_ops.vertex_tables {
        let table_dir = vertex_dir.join(format!("label_{}", label_id));
        table.flush(&table_dir)?;
    }

    let edge_dir = data_dir.join("edges");
    fs::create_dir_all(&edge_dir)?;

    for ((src_label, dst_label, edge_label), table) in &graph.edge_ops.edge_tables {
        let table_dir = edge_dir.join(format!("{}_{}_{}", src_label, dst_label, edge_label));
        table.flush(&table_dir)?;
    }

    graph.wal_manager.sync()?;

    graph.table_tracker.clear();

    Ok(())
}

pub fn flush_incremental(graph: &PropertyGraph) -> StorageResult<Vec<TableId>> {
    let modified_tables = graph.table_tracker.flush_and_reset();

    if modified_tables.is_empty() {
        return Ok(modified_tables);
    }

    use std::fs;
    let data_dir = graph.config.work_dir.join("data");
    fs::create_dir_all(&data_dir)?;

    let mut flushed_labels = std::collections::HashSet::new();

    for table_id in &modified_tables {
        match table_id.table_type {
            TableType::Vertex => {
                if flushed_labels.insert(("vertex", table_id.label_id)) {
                    if let Some(table) = graph.schema_ops.vertex_tables.get(&table_id.label_id) {
                        let vertex_dir = data_dir.join("vertices");
                        let table_dir = vertex_dir.join(format!("label_{}", table_id.label_id));
                        table.flush(&table_dir)?;
                    }
                }
            }
            TableType::Edge => {
                if flushed_labels.insert(("edge", table_id.label_id)) {
                    for ((src, dst, label), table) in &graph.edge_ops.edge_tables {
                        if *label == table_id.label_id {
                            let edge_dir = data_dir.join("edges");
                            let table_dir = edge_dir.join(format!("{}_{}_{}", src, dst, label));
                            table.flush(&table_dir)?;
                        }
                    }
                }
            }
            TableType::Schema => {}
            TableType::Property => {}
        }
    }

    graph.wal_manager.sync()?;

    Ok(modified_tables)
}

pub fn flush_tables_to_dir(graph: &PropertyGraph, data_dir: &Path) -> StorageResult<()> {
    use std::fs;

    let vertex_dir = data_dir.join("vertices");
    fs::create_dir_all(&vertex_dir)?;

    for (label_id, table) in &graph.schema_ops.vertex_tables {
        let table_dir = vertex_dir.join(format!("label_{}", label_id));
        table.flush(&table_dir)?;
    }

    let edge_dir = data_dir.join("edges");
    fs::create_dir_all(&edge_dir)?;

    for ((src_label, dst_label, edge_label), table) in &graph.edge_ops.edge_tables {
        let table_dir = edge_dir.join(format!("{}_{}_{}", src_label, dst_label, edge_label));
        table.flush(&table_dir)?;
    }

    let index_dir = data_dir.join("indexes");
    fs::create_dir_all(&index_dir)?;
    graph.index_data_manager.flush(&index_dir)?;

    graph.wal_manager.sync()?;

    Ok(())
}

pub fn load_data(graph: &mut PropertyGraph) -> StorageResult<()> {
    use std::fs;
    use std::io::Read;

    let data_dir = graph.config.work_dir.join("data");

    let version_file = data_dir.join("version");
    if version_file.exists() {
        let mut file = fs::File::open(&version_file)
            .map_err(|e| StorageError::io_error(format!("Failed to open version file: {}", e)))?;
        let mut content = String::new();
        file.read_to_string(&mut content)
            .map_err(|e| StorageError::io_error(format!("Failed to read version file: {}", e)))?;
        let version: u32 = content.trim().parse().map_err(|e| {
            StorageError::deserialize_error(format!("Invalid version format: {}", e))
        })?;
        if version > DATA_FORMAT_VERSION {
            return Err(StorageError::deserialize_error(format!(
                "Data format version {} is newer than supported version {}",
                version, DATA_FORMAT_VERSION
            )));
        }
    }

    let vertex_dir = data_dir.join("vertices");
    if vertex_dir.exists() {
        for entry in fs::read_dir(&vertex_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                if let Some(dir_name) = path.file_name() {
                    if let Some(name_str) = dir_name.to_str() {
                        if let Some(label_str) = name_str.strip_prefix("label_") {
                            if let Ok(label_id) = label_str.parse::<LabelId>() {
                                if let Some(table) = graph.schema_ops.vertex_tables.get_mut(&label_id)
                                {
                                    table.load(&path)?;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let edge_dir = data_dir.join("edges");
    if edge_dir.exists() {
        for entry in fs::read_dir(&edge_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                if let Some(dir_name) = path.file_name() {
                    if let Some(name_str) = dir_name.to_str() {
                        let parts: Vec<&str> = name_str.splitn(3, '_').collect();
                        if parts.len() == 3 {
                            if let (Ok(src_label), Ok(dst_label), Ok(edge_label)) = (
                                parts[0].parse::<LabelId>(),
                                parts[1].parse::<LabelId>(),
                                parts[2].parse::<LabelId>(),
                            ) {
                                let key = (src_label, dst_label, edge_label);
                                if let Some(table) = graph.edge_ops.edge_tables.get_mut(&key) {
                                    table.load(&path)?;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

pub fn restore_from_checkpoint(
    graph: &mut PropertyGraph,
    checkpoint_dir: &Path,
) -> StorageResult<()> {
    use std::fs;

    let data_dir = checkpoint_dir.join("data");

    let vertex_dir = data_dir.join("vertices");
    if vertex_dir.exists() {
        for entry in fs::read_dir(&vertex_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                if let Some(dir_name) = path.file_name() {
                    if let Some(name_str) = dir_name.to_str() {
                        if let Some(label_str) = name_str.strip_prefix("label_") {
                            if let Ok(label_id) = label_str.parse::<LabelId>() {
                                if let Some(table) = graph.schema_ops.vertex_tables.get_mut(&label_id)
                                {
                                    table.load(&path)?;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let edge_dir = data_dir.join("edges");
    if edge_dir.exists() {
        for entry in fs::read_dir(&edge_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                if let Some(dir_name) = path.file_name() {
                    if let Some(name_str) = dir_name.to_str() {
                        let parts: Vec<&str> = name_str.splitn(3, '_').collect();
                        if parts.len() == 3 {
                            if let (Ok(src_label), Ok(dst_label), Ok(edge_label)) = (
                                parts[0].parse::<LabelId>(),
                                parts[1].parse::<LabelId>(),
                                parts[2].parse::<LabelId>(),
                            ) {
                                let key = (src_label, dst_label, edge_label);
                                if let Some(table) = graph.edge_ops.edge_tables.get_mut(&key) {
                                    table.load(&path)?;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let index_dir = data_dir.join("indexes");
    if index_dir.exists() {
        graph.index_data_manager.load(&index_dir)?;
    }

    Ok(())
}
