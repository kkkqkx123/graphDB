//! 评分模块
//! 
//! 实现高级评分算法，包括TF-IDF、BM25等

use std::collections::HashMap;
use crate::r#type::IntermediateSearchResults;

/// 带评分的ID
#[derive(Debug, Clone, PartialEq)]
pub struct ScoredId {
    pub id: u64,
    pub score: f32,
    pub count: usize,
    pub positions: Vec<usize>,
}

/// 评分算法trait
pub trait ScoringAlgorithm {
    fn calculate_score(&self, doc: &ScoredId, query_terms: &[String], config: &ScoreConfig) -> f32;
    fn name(&self) -> &str;
}

/// 评分配置
#[derive(Debug, Clone)]
pub struct ScoreConfig {
    pub boost_factor: f32,
    pub min_score: f32,
    pub max_score: f32,
    pub field_weights: HashMap<String, f32>,
}

impl Default for ScoreConfig {
    fn default() -> Self {
        ScoreConfig {
            boost_factor: 1.0,
            min_score: 0.0,
            max_score: 1.0,
            field_weights: HashMap::new(),
        }
    }
}

/// TF-IDF评分算法
pub struct TfIdfScorer {
    document_frequency: HashMap<String, usize>,
    total_documents: usize,
}

impl TfIdfScorer {
    pub fn new(document_frequency: HashMap<String, usize>, total_documents: usize) -> Self {
        TfIdfScorer {
            document_frequency,
            total_documents,
        }
    }
    
    /// 计算词频
    fn calculate_tf(&self, _term: &str, doc: &ScoredId) -> f32 {
        // 简化实现：使用计数作为词频
        doc.count as f32
    }
    
    /// 计算逆文档频率
    fn calculate_idf(&self, term: &str) -> f32 {
        let df = self.document_frequency.get(term).unwrap_or(&1);
        ((self.total_documents as f32) / (*df as f32)).ln().max(0.0)
    }
}

impl ScoringAlgorithm for TfIdfScorer {
    fn calculate_score(&self, doc: &ScoredId, query_terms: &[String], config: &ScoreConfig) -> f32 {
        let mut score = 0.0;
        
        for term in query_terms {
            let tf = self.calculate_tf(term, doc);
            let idf = self.calculate_idf(term);
            score += tf * idf;
        }
        
        // 应用配置
        score * config.boost_factor
            .max(config.min_score)
            .min(config.max_score)
    }
    
    fn name(&self) -> &str {
        "TF-IDF"
    }
}

/// BM25评分算法
pub struct Bm25Scorer {
    document_length: HashMap<u64, usize>,
    average_document_length: f32,
    k1: f32,
    b: f32,
}

impl Bm25Scorer {
    pub fn new(
        document_length: HashMap<u64, usize>,
        average_document_length: f32,
        k1: f32,
        b: f32,
    ) -> Self {
        Bm25Scorer {
            document_length,
            average_document_length,
            k1,
            b,
        }
    }
    
    /// 计算BM25分数
    fn calculate_bm25(&self, term_freq: usize, doc_id: u64, _doc_count: usize) -> f32 {
        let doc_len = *self.document_length.get(&doc_id).unwrap_or(&0) as f32;
        let normalized_length = doc_len / self.average_document_length;
        
        let numerator = term_freq as f32 * (self.k1 + 1.0);
        let denominator = term_freq as f32 + self.k1 * (1.0 - self.b + self.b * normalized_length);
        
        if denominator > 0.0 {
            numerator / denominator
        } else {
            0.0
        }
    }
}

impl ScoringAlgorithm for Bm25Scorer {
    fn calculate_score(&self, doc: &ScoredId, query_terms: &[String], config: &ScoreConfig) -> f32 {
        let mut score = 0.0;
        
        for _term in query_terms {
            // 简化实现：假设每个查询词出现一次
            let term_freq = 1;
            let bm25_score = self.calculate_bm25(term_freq, doc.id, 0);
            score += bm25_score;
        }
        
        // 应用配置
        score * config.boost_factor
            .max(config.min_score)
            .min(config.max_score)
    }
    
    fn name(&self) -> &str {
        "BM25"
    }
}

/// 评分管理器
pub struct ScoreManager {
    algorithms: HashMap<String, Box<dyn ScoringAlgorithm>>,
    default_algorithm: String,
}

impl ScoreManager {
    pub fn new() -> Self {
        let mut manager = ScoreManager {
            algorithms: HashMap::new(),
            default_algorithm: "tfidf".to_string(),
        };
        
        // 添加默认算法
        manager.add_algorithm("tfidf", Box::new(TfIdfScorer::new(HashMap::new(), 1000)));
        manager.add_algorithm("bm25", Box::new(Bm25Scorer::new(HashMap::new(), 100.0, 1.2, 0.75)));
        
        manager
    }
    
    pub fn add_algorithm(&mut self, name: &str, algorithm: Box<dyn ScoringAlgorithm>) {
        self.algorithms.insert(name.to_string(), algorithm);
    }
    
    pub fn score_documents(
        &self,
        documents: Vec<ScoredId>,
        query_terms: &[String],
        algorithm: Option<&str>,
        config: &ScoreConfig,
    ) -> Vec<ScoredId> {
        let algorithm_name = algorithm.unwrap_or(&self.default_algorithm);
        
        if let Some(scorer) = self.algorithms.get(algorithm_name) {
            documents.into_iter().map(|mut doc| {
                doc.score = scorer.calculate_score(&doc, query_terms, config);
                doc
            }).collect()
        } else {
            documents
        }
    }
    
    pub fn get_available_algorithms(&self) -> Vec<&str> {
        self.algorithms.keys().map(|s| s.as_str()).collect()
    }
}

/// 对搜索结果进行评分
pub fn score_search_results(
    results: &IntermediateSearchResults,
    query_terms: &[String],
    algorithm: Option<&str>,
    config: &ScoreConfig,
) -> Vec<ScoredId> {
    let mut scored_results = Vec::new();
    
    for (_idx, result_array) in results.iter().enumerate() {
        for (pos, &id) in result_array.iter().enumerate() {
            let scored_id = ScoredId {
                id,
                score: 0.5, // 默认分数
                count: result_array.len(),
                positions: vec![pos],
            };
            scored_results.push(scored_id);
        }
    }
    
    // 使用评分管理器进行评分
    let manager = ScoreManager::new();
    manager.score_documents(scored_results, query_terms, algorithm, config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_score_manager() {
        let manager = ScoreManager::new();
        let algorithms = manager.get_available_algorithms();
        assert!(algorithms.contains(&"tfidf"));
        assert!(algorithms.contains(&"bm25"));
    }
    
    #[test]
    fn test_tfidf_scorer() {
        let mut df = HashMap::new();
        df.insert("test".to_string(), 10);
        
        let scorer = TfIdfScorer::new(df, 100);
        let doc = ScoredId {
            id: 1,
            score: 0.0,
            count: 5,
            positions: vec![0, 1, 2],
        };
        
        let config = ScoreConfig::default();
        let score = scorer.calculate_score(&doc, &["test".to_string()], &config);
        assert!(score >= 0.0);
    }
}