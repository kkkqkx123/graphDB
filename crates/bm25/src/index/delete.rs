use crate::error::Result;
use crate::index::IndexManager;
use crate::index::IndexSchema;
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
