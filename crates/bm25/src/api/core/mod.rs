pub mod index;
pub mod search;
pub mod document;
pub mod delete;
pub mod batch;
pub mod schema;
pub mod stats;
pub mod persistence;
pub mod stats_extractor;

pub use index::{
    IndexManager, IndexManagerConfig, LogMergePolicyConfig, MergePolicyType, ReloadPolicyConfig,
};
pub use search::{search, SearchOptions, SearchResult};
pub use document::{
    add_document, add_document_with_writer, update_document, update_document_with_writer,
    get_document, add_document_with_storage, update_document_with_storage,
};
pub use delete::{
    delete_document, delete_document_with_writer, batch_delete_documents,
    batch_delete_documents_with_writer, delete_document_with_storage,
};
pub use batch::{
    batch_add_documents, batch_add_documents_with_writer, batch_update_documents,
    batch_update_documents_with_writer, batch_add_documents_optimized,
    batch_add_documents_with_storage,
};
pub use schema::IndexSchema;
pub use stats::{get_stats, IndexStats};
pub use persistence::{
    PersistenceManager, IndexMetadata, BackupInfo,
};
