use crate::api::core::{IndexManager, IndexSchema};
use crate::error::Result;
use crate::storage::MutableStorageManager;
use tantivy::IndexWriter;
use tantivy::Term;

pub fn delete_document(
    manager: &IndexManager,
    schema: &IndexSchema,
    document_id: &str,
) -> Result<()> {
    let mut writer = manager.writer()?;
    let term = Term::from_field_text(schema.document_id, document_id);
    writer.delete_term(term);
    writer.commit()?;
    manager.clear_reader_cache();
    Ok(())
}

/// Delete documents and synchronize them to the storage tier
pub async fn delete_document_with_storage(
    manager: &IndexManager,
    storage: &MutableStorageManager,
    schema: &IndexSchema,
    document_id: &str,
) -> Result<()> {
    // 1. Deletion of statistical information from the storage layer
    storage.delete_doc_stats(document_id).await?;

    // 2. Delete documents from the index
    let mut writer = manager.writer()?;
    let term = Term::from_field_text(schema.document_id, document_id);
    writer.delete_term(term);
    writer.commit()?;
    manager.clear_reader_cache();

    Ok(())
}

pub fn delete_document_with_writer(
    writer: &mut IndexWriter,
    schema: &IndexSchema,
    document_id: &str,
) -> Result<()> {
    let term = Term::from_field_text(schema.document_id, document_id);
    writer.delete_term(term);
    Ok(())
}

pub fn batch_delete_documents(
    manager: &IndexManager,
    schema: &IndexSchema,
    document_ids: &[String],
) -> Result<usize> {
    let mut writer = manager.writer()?;

    for doc_id in document_ids {
        let term = Term::from_field_text(schema.document_id, doc_id);
        writer.delete_term(term);
    }

    writer.commit()?;
    manager.clear_reader_cache();
    Ok(document_ids.len())
}

pub fn batch_delete_documents_with_writer(
    writer: &mut IndexWriter,
    schema: &IndexSchema,
    document_ids: &[String],
) -> Result<usize> {
    for doc_id in document_ids {
        let term = Term::from_field_text(schema.document_id, doc_id);
        writer.delete_term(term);
    }

    Ok(document_ids.len())
}
