pub mod batch;
pub mod delete;
pub mod document;
pub mod index;
pub mod persistence;
pub mod schema;
pub mod search;
pub mod stats;
pub mod stats_extractor;

pub use batch::{
    batch_add_documents, batch_add_documents_optimized, batch_add_documents_with_storage,
    batch_add_documents_with_writer, batch_update_documents, batch_update_documents_with_writer,
};
pub use delete::{
    batch_delete_documents, batch_delete_documents_with_writer, delete_document,
    delete_document_with_storage, delete_document_with_writer,
};
pub use document::{
    add_document, add_document_with_storage, add_document_with_writer, get_document,
    update_document, update_document_with_storage, update_document_with_writer,
};
pub use index::{
    IndexManager, IndexManagerConfig, LogMergePolicyConfig, MergePolicyType, ReloadPolicyConfig,
};
pub use persistence::{BackupInfo, IndexMetadata, PersistenceManager};
pub use schema::IndexSchema;
pub use search::{search, SearchOptions, SearchResult};
pub use stats::{get_stats, IndexStats};
