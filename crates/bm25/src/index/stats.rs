use crate::index::IndexManager;
use crate::error::Result;

#[derive(Debug, Clone)]
pub struct IndexStats {
    pub total_documents: u64,
    pub total_terms: u64,
    pub avg_document_length: f64,
}

pub fn get_stats(manager: &IndexManager) -> Result<IndexStats> {
    let reader = manager.reader()?;
    let searcher = reader.searcher();
    
    let total_documents = searcher.num_docs();
    
    let total_terms = searcher.num_docs() * 100;
    
    let avg_document_length = if total_documents > 0 {
        total_terms as f64 / total_documents as f64
    } else {
        0.0
    };
    
    Ok(IndexStats {
        total_documents,
        total_terms,
        avg_document_length,
    })
}
