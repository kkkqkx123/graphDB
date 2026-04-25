use crate::api::core::IndexManager;
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

    if total_documents == 0 {
        return Ok(IndexStats {
            total_documents: 0,
            total_terms: 0,
            avg_document_length: 0.0,
        });
    }

    let mut total_terms: u64 = 0;
    for segment_reader in searcher.segment_readers() {
        let inv_index = segment_reader.inverted_index(
            manager
                .schema()
                .get_field("content")
                .expect("content field must exist"),
        )?;
        total_terms += inv_index.total_num_tokens() as u64;
    }
    let avg_document_length = total_terms as f64 / total_documents as f64;

    Ok(IndexStats {
        total_documents,
        total_terms,
        avg_document_length,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_empty_stats() -> Result<()> {
        let dir = tempdir()?;
        let path = dir.path().join("test_stats");
        let manager = IndexManager::create(&path)?;

        let stats = get_stats(&manager)?;
        assert_eq!(stats.total_documents, 0);
        assert_eq!(stats.avg_document_length, 0.0);

        Ok(())
    }
}
