use crate::api::core::stats_extractor::extract_batch_tf_df_stats;
use crate::api::core::{IndexManager, IndexSchema};
use crate::error::Result;
use crate::storage::MutableStorageManager;
use std::collections::HashMap;
use tantivy::IndexWriter;

pub fn batch_add_documents(
    manager: &IndexManager,
    schema: &IndexSchema,
    documents: Vec<(String, HashMap<String, String>)>,
) -> Result<usize> {
    let count = documents.len();
    let mut writer = manager.writer()?;

    for (doc_id, fields) in documents {
        let doc = schema.to_document(&doc_id, &fields);
        writer.add_document(doc)?;
    }

    writer.commit()?;
    Ok(count)
}

/// Batch add documents and synchronize to storage tier
pub async fn batch_add_documents_with_storage(
    manager: &IndexManager,
    storage: &MutableStorageManager,
    schema: &IndexSchema,
    documents: Vec<(String, HashMap<String, String>)>,
    avg_doc_length: f32,
) -> Result<usize> {
    let count = documents.len();

    // 1. Batch extraction of TF/DF statistics
    let stats = extract_batch_tf_df_stats(
        &documents,
        0, // total_docs is retrieved from the storage tier and is set to 0 for now.
        avg_doc_length,
    );

    // 2. Submission to the storage layer
    storage.commit_batch(&stats).await?;

    // 3. Write index
    let mut writer = manager.writer()?;
    for (doc_id, fields) in documents {
        let doc = schema.to_document(&doc_id, &fields);
        writer.add_document(doc)?;
    }
    writer.commit()?;

    Ok(count)
}

pub fn batch_add_documents_with_writer(
    writer: &mut IndexWriter,
    schema: &IndexSchema,
    documents: Vec<(String, HashMap<String, String>)>,
) -> Result<usize> {
    let count = documents.len();

    for (doc_id, fields) in documents {
        let doc = schema.to_document(&doc_id, &fields);
        writer.add_document(doc)?;
    }

    Ok(count)
}

pub fn batch_update_documents(
    manager: &IndexManager,
    schema: &IndexSchema,
    documents: Vec<(String, HashMap<String, String>)>,
) -> Result<usize> {
    let count = documents.len();
    let mut writer = manager.writer()?;

    for (doc_id, fields) in documents {
        let doc = schema.to_document(&doc_id, &fields);
        let term = tantivy::Term::from_field_text(schema.document_id, &doc_id);
        writer.delete_term(term);
        writer.add_document(doc)?;
    }

    writer.commit()?;
    Ok(count)
}

pub fn batch_update_documents_with_writer(
    writer: &mut IndexWriter,
    schema: &IndexSchema,
    documents: Vec<(String, HashMap<String, String>)>,
) -> Result<usize> {
    let count = documents.len();

    for (doc_id, fields) in documents {
        let doc = schema.to_document(&doc_id, &fields);
        let term = tantivy::Term::from_field_text(schema.document_id, &doc_id);
        writer.delete_term(term);
        writer.add_document(doc)?;
    }

    Ok(count)
}

pub fn batch_add_documents_optimized(
    manager: &IndexManager,
    schema: &IndexSchema,
    documents: Vec<(String, HashMap<String, String>)>,
    batch_size: usize,
) -> Result<usize> {
    let mut indexed_count = 0;

    for batch in documents.chunks(batch_size) {
        let mut writer = manager.writer()?;

        for (doc_id, fields) in batch {
            let doc = schema.to_document(doc_id, fields);
            writer.add_document(doc)?;
            indexed_count += 1;
        }

        writer.commit()?;
    }

    Ok(indexed_count)
}
