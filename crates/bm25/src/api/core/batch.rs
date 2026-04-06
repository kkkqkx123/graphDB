use crate::error::Result;
use crate::api::core::{IndexManager, IndexSchema};
use crate::storage::MutableStorageManager;
use crate::api::core::stats_extractor::extract_batch_tf_df_stats;
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

/// 批量添加文档并同步到存储层
pub async fn batch_add_documents_with_storage(
    manager: &IndexManager,
    storage: &MutableStorageManager,
    schema: &IndexSchema,
    documents: Vec<(String, HashMap<String, String>)>,
    avg_doc_length: f32,
) -> Result<usize> {
    let count = documents.len();

    // 1. 批量提取 TF/DF 统计
    let stats = extract_batch_tf_df_stats(
        &documents,
        0, // total_docs 从存储层获取，暂时设为 0
        avg_doc_length,
    );

    // 2. 提交到存储层
    storage.commit_batch(&stats).await?;

    // 3. 写入索引
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
