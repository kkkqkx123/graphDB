use crate::core::{StorageError, StorageResult};
use crate::storage::edge::EdgeTable;
use crate::storage::vertex::LabelId;
use crate::storage::vertex::VertexTable;
use std::collections::HashMap;

use super::flush_manager::FlushManagerWrapper;

const DATA_FORMAT_VERSION: u32 = 1;

pub struct PersistenceOps;

impl PersistenceOps {
    pub fn flush(
        vertex_tables: &HashMap<LabelId, VertexTable>,
        edge_tables: &HashMap<(LabelId, LabelId, LabelId), EdgeTable>,
        work_dir: &std::path::Path,
        flush_manager: &FlushManagerWrapper,
    ) -> StorageResult<()> {
        use std::fs;
        use std::io::Write;

        let data_dir = work_dir.join("data");
        fs::create_dir_all(&data_dir)?;

        let version_file = data_dir.join("version");
        let mut file = fs::File::create(&version_file)
            .map_err(|e| StorageError::IOError(format!("Failed to create version file: {}", e)))?;
        writeln!(file, "{}", DATA_FORMAT_VERSION)
            .map_err(|e| StorageError::IOError(format!("Failed to write version file: {}", e)))?;

        let vertex_dir = data_dir.join("vertices");
        fs::create_dir_all(&vertex_dir)?;

        for (label_id, table) in vertex_tables {
            let table_dir = vertex_dir.join(format!("label_{}", label_id));
            table.flush(&table_dir)?;
        }

        let edge_dir = data_dir.join("edges");
        fs::create_dir_all(&edge_dir)?;

        for ((src_label, dst_label, edge_label), table) in edge_tables {
            let table_dir = edge_dir.join(format!("{}_{}_{}", src_label, dst_label, edge_label));
            table.flush(&table_dir)?;
        }

        flush_manager.sync_wal()?;

        Ok(())
    }

    pub fn load(
        vertex_tables: &mut HashMap<LabelId, VertexTable>,
        edge_tables: &mut HashMap<(LabelId, LabelId, LabelId), EdgeTable>,
        work_dir: &std::path::Path,
    ) -> StorageResult<()> {
        use std::fs;
        use std::io::Read;

        let data_dir = work_dir.join("data");

        let version_file = data_dir.join("version");
        if version_file.exists() {
            let mut file = fs::File::open(&version_file).map_err(|e| {
                StorageError::IOError(format!("Failed to open version file: {}", e))
            })?;
            let mut content = String::new();
            file.read_to_string(&mut content).map_err(|e| {
                StorageError::IOError(format!("Failed to read version file: {}", e))
            })?;
            let version: u32 = content.trim().parse().map_err(|e| {
                StorageError::DeserializeError(format!("Invalid version format: {}", e))
            })?;
            if version > DATA_FORMAT_VERSION {
                return Err(StorageError::DeserializeError(format!(
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
                                    if let Some(table) = vertex_tables.get_mut(&label_id) {
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
                                    if let Some(table) = edge_tables.get_mut(&key) {
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

    pub fn flush_tables_to_dir(
        vertex_tables: &HashMap<LabelId, VertexTable>,
        edge_tables: &HashMap<(LabelId, LabelId, LabelId), EdgeTable>,
        data_dir: &std::path::Path,
        flush_manager: &FlushManagerWrapper,
    ) -> StorageResult<()> {
        use std::fs;

        let vertex_dir = data_dir.join("vertices");
        fs::create_dir_all(&vertex_dir)?;

        for (label_id, table) in vertex_tables {
            let table_dir = vertex_dir.join(format!("label_{}", label_id));
            table.flush(&table_dir)?;
        }

        let edge_dir = data_dir.join("edges");
        fs::create_dir_all(&edge_dir)?;

        for ((src_label, dst_label, edge_label), table) in edge_tables {
            let table_dir = edge_dir.join(format!("{}_{}_{}", src_label, dst_label, edge_label));
            table.flush(&table_dir)?;
        }

        flush_manager.sync_wal()?;

        Ok(())
    }

    pub fn restore_from_checkpoint(
        vertex_tables: &mut HashMap<LabelId, VertexTable>,
        edge_tables: &mut HashMap<(LabelId, LabelId, LabelId), EdgeTable>,
        checkpoint_dir: &std::path::Path,
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
                                    if let Some(table) = vertex_tables.get_mut(&label_id) {
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
                                    if let Some(table) = edge_tables.get_mut(&key) {
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
}
