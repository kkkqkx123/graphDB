use crate::types::*;

pub struct ExtractedSearchParams {
    pub limit: usize,
    pub hnsw_ef: Option<usize>,
    pub score_threshold: Option<f32>,
}

pub fn extract_search_params(query: &SearchQuery) -> ExtractedSearchParams {
    ExtractedSearchParams {
        limit: query.effective_limit(),
        hnsw_ef: query.nprobe.or(query.hnsw_ef()),
        score_threshold: query.score_threshold.or(query.score_threshold()),
    }
}
