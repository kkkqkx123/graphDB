use crate::api::core::stats_extractor::extract_tf_df_stats;
use crate::api::core::{IndexManager, IndexSchema};
use crate::error::Result;
use crate::storage::MutableStorageManager;
use std::collections::HashMap;
use tantivy::IndexWriter;

pub fn add_document(
    manager: &IndexManager,
    schema: &IndexSchema,
    document_id: &str,
    fields: &HashMap<String, String>,
) -> Result<()> {
    let mut writer = manager.writer()?;
    let doc = schema.to_document(document_id, fields);
    writer.add_document(doc)?;
    writer.commit()?;
    manager.clear_reader_cache();
    Ok(())
}

/// Add documents and synchronize them to the storage tier
pub async fn add_document_with_storage(
    manager: &IndexManager,
    storage: &MutableStorageManager,
    schema: &IndexSchema,
    document_id: &str,
    fields: &HashMap<String, String>,
    avg_doc_length: f32,
) -> Result<()> {
    // 1. Extraction of TF/DF statistics
    let stats = extract_tf_df_stats(
        fields,
        0, // total_docs is retrieved from the storage tier and is set to 0 for now.
        avg_doc_length,
    );

    // 2. Submission to the storage layer
    storage.commit_batch(&stats).await?;

    // 3. Write index
    let mut writer = manager.writer()?;
    let doc = schema.to_document(document_id, fields);
    writer.add_document(doc)?;
    writer.commit()?;
    manager.clear_reader_cache();

    Ok(())
}

pub fn add_document_with_writer(
    writer: &mut IndexWriter,
    schema: &IndexSchema,
    document_id: &str,
    fields: &HashMap<String, String>,
) -> Result<()> {
    let doc = schema.to_document(document_id, fields);
    writer.add_document(doc)?;
    Ok(())
}

pub fn update_document(
    manager: &IndexManager,
    schema: &IndexSchema,
    document_id: &str,
    fields: &HashMap<String, String>,
) -> Result<()> {
    let mut writer = manager.writer()?;

    let term = tantivy::Term::from_field_text(schema.document_id, document_id);
    writer.delete_term(term);

    let doc = schema.to_document(document_id, fields);
    writer.add_document(doc)?;
    writer.commit()?;
    manager.clear_reader_cache();
    Ok(())
}

/// Update documents and synchronize them to the storage tier
pub async fn update_document_with_storage(
    manager: &IndexManager,
    storage: &MutableStorageManager,
    schema: &IndexSchema,
    document_id: &str,
    fields: &HashMap<String, String>,
    avg_doc_length: f32,
) -> Result<()> {
    // 1. Delete statistics of old documents from the storage layer
    storage.delete_doc_stats(document_id).await?;

    // 2. Extract TF/DF statistics for new documents
    let new_stats = extract_tf_df_stats(
        fields,
        0, // total_docs is retrieved from the storage tier and is set to 0 for now.
        avg_doc_length,
    );

    // 3. Submission of new statistics to the storage layer
    storage.commit_batch(&new_stats).await?;

    // 4. Updating of indexes
    let mut writer = manager.writer()?;
    let term = tantivy::Term::from_field_text(schema.document_id, document_id);
    writer.delete_term(term);

    let doc = schema.to_document(document_id, fields);
    writer.add_document(doc)?;
    writer.commit()?;
    manager.clear_reader_cache();

    Ok(())
}

pub fn update_document_with_writer(
    writer: &mut IndexWriter,
    schema: &IndexSchema,
    document_id: &str,
    fields: &HashMap<String, String>,
) -> Result<()> {
    let term = tantivy::Term::from_field_text(schema.document_id, document_id);
    writer.delete_term(term);

    let doc = schema.to_document(document_id, fields);
    writer.add_document(doc)?;
    Ok(())
}

pub fn get_document(
    manager: &IndexManager,
    schema: &IndexSchema,
    document_id: &str,
) -> Result<Option<tantivy::schema::TantivyDocument>> {
    let reader = manager.reader()?;
    let searcher = reader.searcher();

    let term = tantivy::Term::from_field_text(schema.document_id, document_id);
    let query = tantivy::query::TermQuery::new(term, tantivy::schema::IndexRecordOption::Basic);
    let top_docs = tantivy::collector::TopDocs::with_limit(1);
    let results = searcher.search(&query, &top_docs)?;

    if results.is_empty() {
        Ok(None)
    } else {
        let (_, doc_address) = &results[0];
        Ok(Some(searcher.doc(*doc_address)?))
    }
}
