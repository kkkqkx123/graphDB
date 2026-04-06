use crate::error::Result;
use crate::api::core::{IndexManager, IndexSchema};
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

/// 删除文档并同步到存储层
pub async fn delete_document_with_storage(
    manager: &IndexManager,
    storage: &MutableStorageManager,
    schema: &IndexSchema,
    document_id: &str,
) -> Result<()> {
    // 1. 从存储层删除统计信息
    storage.delete_doc_stats(document_id).await?;

    // 2. 从索引中删除文档
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
