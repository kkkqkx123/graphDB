use crate::index::{IndexManager, IndexSchema};
use crate::error::Result;
use std::collections::HashMap;

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
