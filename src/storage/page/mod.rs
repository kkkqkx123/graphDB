//! Page Storage Module
//!
//! Provides page-based storage with fixed-size records for improved
//! cache locality and memory management.

mod flat_csr;
mod migration;
mod page;
mod page_header;
mod page_manager;
mod record;

pub use flat_csr::{FlatCsr, FlatCsrEdgeIterator, FlatCsrIterator};
pub use migration::{MigrationConfig, MigrationStats, StorageMigrator, verify_migration};
pub use page::Page;
pub use page_header::{PageHeader, PageType, PAGE_SIZE, PAGE_DATA_SIZE, PAGE_HEADER_SIZE};
pub use page_manager::{PageManager, PageManagerStats, StoragePageId};
pub use record::{
    EdgeRecord, VertexRecord, DELETED_TIMESTAMP, EDGE_RECORD_SIZE, INVALID_TIMESTAMP,
    VERTEX_RECORD_SIZE,
};
