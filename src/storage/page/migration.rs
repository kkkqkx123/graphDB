//! Storage Format Migration Tool
//!
//! Provides utilities to migrate from the old storage format
//! to the new page-based storage format.

use std::path::Path;

use super::{EdgeRecord, FlatCsr, Page, PageManager, PageType, StoragePageId, VertexRecord};
use crate::core::{StorageError, StorageResult};
use crate::storage::edge::{EdgeTable, MutableCsr};
use crate::storage::vertex::VertexTable;

#[derive(Debug, Clone)]
pub struct MigrationConfig {
    pub batch_size: usize,
    pub verify_checksums: bool,
    pub create_backup: bool,
}

impl Default for MigrationConfig {
    fn default() -> Self {
        Self {
            batch_size: 1000,
            verify_checksums: true,
            create_backup: true,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct MigrationStats {
    pub vertices_migrated: u64,
    pub edges_migrated: u64,
    pub pages_created: u64,
    pub bytes_written: u64,
    pub errors: Vec<String>,
}

pub struct StorageMigrator {
    config: MigrationConfig,
    stats: MigrationStats,
}

impl StorageMigrator {
    pub fn new() -> Self {
        Self::with_config(MigrationConfig::default())
    }

    pub fn with_config(config: MigrationConfig) -> Self {
        Self {
            config,
            stats: MigrationStats::default(),
        }
    }

    pub fn migrate_vertex_table<P: AsRef<Path>>(
        &mut self,
        source: &VertexTable,
        target_path: P,
    ) -> StorageResult<MigrationStats> {
        let page_manager = PageManager::new(target_path);

        let ts = u32::MAX - 1;
        let vertex_count = source.total_count();

        let mut current_page_id: Option<StoragePageId> = None;
        let mut current_page: Option<Page> = None;
        let mut current_offset = 0usize;

        for internal_id in 0..vertex_count as u32 {
            if let Some(vertex) = source.get_by_internal_id(internal_id, ts) {
                let record = VertexRecord::new(internal_id as u64, ts);

                if current_page.is_none() || !self.can_fit_vertex_record(current_page.as_ref().unwrap()) {
                    if let Some(page) = current_page.take() {
                        page_manager.put_page(page)?;
                        self.stats.pages_created += 1;
                    }

                    let page_id = page_manager.allocate_page(PageType::VertexData)?;
                    current_page_id = Some(page_id);
                    current_page = page_manager.get_page(&page_id)?;
                    current_offset = 0;
                }

                if let Some(ref mut page) = current_page {
                    let record_bytes = record.to_bytes();
                    page.write_record(current_offset, &record_bytes)?;
                    current_offset += record_bytes.len();
                    self.stats.vertices_migrated += 1;
                    self.stats.bytes_written += record_bytes.len() as u64;
                }
            }
        }

        if let Some(page) = current_page {
            page_manager.put_page(page)?;
            self.stats.pages_created += 1;
        }

        page_manager.flush_all()?;

        Ok(self.stats.clone())
    }

    pub fn migrate_edge_table<P: AsRef<Path>>(
        &mut self,
        source: &EdgeTable,
        target_path: P,
    ) -> StorageResult<MigrationStats> {
        let page_manager = PageManager::new(target_path);

        let ts = u32::MAX - 1;
        let vertex_capacity = source.vertex_capacity();

        let mut flat_csr = FlatCsr::with_capacity(vertex_capacity, vertex_capacity * 4);

        for src in 0..vertex_capacity as u64 {
            let edges = source.out_edges(src, ts);

            for edge in edges {
                let record = EdgeRecord::new(
                    edge.src_vid,
                    edge.dst_vid,
                    edge.edge_id,
                    0,
                    ts,
                );
                flat_csr.insert(src, record);
                self.stats.edges_migrated += 1;
            }
        }

        let csr_data = flat_csr.dump();
        self.stats.bytes_written += csr_data.len() as u64;

        let mut current_offset = 0usize;
        let chunk_size = 4000usize;

        while current_offset < csr_data.len() {
            let page_id = page_manager.allocate_page(PageType::EdgeData)?;
            let mut page = page_manager.get_page(&page_id)?.unwrap();

            let end = (current_offset + chunk_size).min(csr_data.len());
            let chunk = &csr_data[current_offset..end];

            page.write_record(0, chunk)?;
            page_manager.put_page(page)?;

            self.stats.pages_created += 1;
            self.stats.bytes_written += chunk.len() as u64;
            current_offset = end;
        }

        page_manager.flush_all()?;

        Ok(self.stats.clone())
    }

    pub fn migrate_csr_to_flat(&self, csr: &MutableCsr) -> FlatCsr {
        let ts = u32::MAX - 1;
        let vertex_capacity = csr.vertex_capacity();

        let mut flat_csr = FlatCsr::with_capacity(vertex_capacity, 1024);

        for src in 0..vertex_capacity as u64 {
            let edges = csr.edges_of(src, ts);

            for nbr in edges {
                let record = EdgeRecord::new(
                    src,
                    nbr.neighbor,
                    nbr.edge_id,
                    nbr.prop_offset,
                    nbr.timestamp,
                );
                flat_csr.insert(src, record);
            }
        }

        flat_csr
    }

    fn can_fit_vertex_record(&self, page: &Page) -> bool {
        page.free_space() as usize >= super::VERTEX_RECORD_SIZE
    }

    pub fn stats(&self) -> &MigrationStats {
        &self.stats
    }

    pub fn reset_stats(&mut self) {
        self.stats = MigrationStats::default();
    }
}

impl Default for StorageMigrator {
    fn default() -> Self {
        Self::new()
    }
}

pub fn verify_migration(
    original_vertices: usize,
    original_edges: usize,
    migrated_stats: &MigrationStats,
) -> StorageResult<bool> {
    if migrated_stats.vertices_migrated as usize != original_vertices {
        return Err(StorageError::InvalidOperation(format!(
            "Vertex count mismatch: expected {}, got {}",
            original_vertices, migrated_stats.vertices_migrated
        )));
    }

    if migrated_stats.edges_migrated as usize != original_edges {
        return Err(StorageError::InvalidOperation(format!(
            "Edge count mismatch: expected {}, got {}",
            original_edges, migrated_stats.edges_migrated
        )));
    }

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::vertex::{VertexSchema, PropertyDef};
    use crate::core::{DataType, Value};
    use tempfile::tempdir;

    fn create_test_vertex_schema() -> VertexSchema {
        VertexSchema {
            label_id: 0,
            label_name: "person".to_string(),
            properties: vec![
                PropertyDef::new("name".to_string(), DataType::String),
            ],
            primary_key_index: 0,
        }
    }

    #[test]
    fn test_migrator_creation() {
        let migrator = StorageMigrator::new();
        assert_eq!(migrator.stats().vertices_migrated, 0);
        assert_eq!(migrator.stats().edges_migrated, 0);
    }

    #[test]
    fn test_migration_config_default() {
        let config = MigrationConfig::default();
        assert_eq!(config.batch_size, 1000);
        assert!(config.verify_checksums);
        assert!(config.create_backup);
    }

    #[test]
    fn test_verify_migration_success() {
        let stats = MigrationStats {
            vertices_migrated: 100,
            edges_migrated: 200,
            pages_created: 10,
            bytes_written: 5000,
            errors: vec![],
        };

        let result = verify_migration(100, 200, &stats);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_migration_failure() {
        let stats = MigrationStats {
            vertices_migrated: 50,
            edges_migrated: 200,
            pages_created: 10,
            bytes_written: 5000,
            errors: vec![],
        };

        let result = verify_migration(100, 200, &stats);
        assert!(result.is_err());
    }
}
