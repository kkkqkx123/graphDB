use crate::error::Result;
use crate::api::core::{IndexManager, IndexSchema};
use crate::storage::MutableStorageManager;
use crate::api::core::stats_extractor::extract_tf_df_stats;
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
    Ok(())
}

/// 添加文档并同步到存储层
pub async fn add_document_with_storage(
    manager: &IndexManager,
    storage: &MutableStorageManager,
    schema: &IndexSchema,
    document_id: &str,
    fields: &HashMap<String, String>,
    avg_doc_length: f32,
) -> Result<()> {
    // 1. 提取 TF/DF 统计
    let stats = extract_tf_df_stats(
        fields,
        0, // total_docs 从存储层获取，暂时设为 0
        avg_doc_length,
    );

    // 2. 提交到存储层
    storage.commit_batch(&stats).await?;

    // 3. 写入索引
    let mut writer = manager.writer()?;
    let doc = schema.to_document(document_id, fields);
    writer.add_document(doc)?;
    writer.commit()?;

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
    Ok(())
}

/// 更新文档并同步到存储层
pub async fn update_document_with_storage(
    manager: &IndexManager,
    storage: &MutableStorageManager,
    schema: &IndexSchema,
    document_id: &str,
    fields: &HashMap<String, String>,
    avg_doc_length: f32,
) -> Result<()> {
    // 1. 提取新文档的 TF/DF 统计
    let new_stats = extract_tf_df_stats(
        fields,
        0, // total_docs 从存储层获取，暂时设为 0
        avg_doc_length,
    );

    // 2. 获取旧文档的统计（用于删除）
    // TODO: 实现从存储层获取旧文档统计并删除

    // 3. 提交新统计到存储层
    storage.commit_batch(&new_stats).await?;

    // 4. 更新索引
    let mut writer = manager.writer()?;
    let term = tantivy::Term::from_field_text(schema.document_id, document_id);
    writer.delete_term(term);

    let doc = schema.to_document(document_id, fields);
    writer.add_document(doc)?;
    writer.commit()?;

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
