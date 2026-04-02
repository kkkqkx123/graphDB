use crate::index::{IndexManager, IndexSchema};
use crate::error::Result;
use std::collections::HashMap;

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
