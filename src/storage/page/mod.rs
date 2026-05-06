//! Page Storage Module
//!
//! Provides page-based storage with fixed-size records for improved
//! cache locality and memory management.

mod flat_csr;
mod page;
mod page_header;
mod page_manager;
mod record;

pub use flat_csr::{FlatCsr, FlatCsrEdgeIterator, FlatCsrIterator};
pub use page::Page;
pub use page_header::{PageHeader, PageType, PAGE_DATA_SIZE, PAGE_FLAG_DIRTY, PAGE_HEADER_SIZE, PAGE_SIZE};
pub use page_manager::{PageManager, PageManagerConfig, PageManagerStats, StoragePageId};
pub use record::{
    EdgeRecord, VertexRecord, DELETED_TIMESTAMP, EDGE_RECORD_SIZE, INVALID_TIMESTAMP,
    VERTEX_RECORD_SIZE,
};
