//! 搜索协调器
//!
//! 协调多字段搜索，管理字段权重和结果合并
//!
//! # 使用示例
//!
//! ```rust
//! use inversearch_service::document::{Document, DocumentConfig};
//! use inversearch_service::search::SearchCoordinator;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = DocumentConfig::new();
//! let doc = Document::new(config)?;
//! let mut coordinator = SearchCoordinator::new(&doc);
//! coordinator.add_field("title", 2.0);  // title 权重 2.0
//! coordinator.add_field("content", 1.0); // content 权重 1.0
//!
//! let result = coordinator.search()?;
//! # Ok(())
//! # }
//! ```

use crate::error::Result;
use crate::DocId;
use crate::{Document, SearchOptions, SearchResult};
use std::collections::{HashMap, HashSet};

// Type alias for complex type
type FieldBoostCondition = Box<dyn Fn(&serde_json::Value) -> bool + Send + Sync>;

/// 字段搜索配置
#[derive(Debug, Clone)]
pub struct FieldSearch {
    name: String,
    weight: f32,
    query: Option<String>,
}

impl FieldSearch {
    pub fn new(name: &str, weight: f32) -> Self {
        FieldSearch {
            name: name.to_string(),
            weight,
            query: None,
        }
    }

    pub fn with_query(mut self, query: &str) -> Self {
        self.query = Some(query.to_string());
        self
    }
}

/// 权重应用策略
#[derive(Debug, Clone, PartialEq, Default)]
pub enum BoostStrategy {
    /// 乘法策略：将字段原始得分乘以权重值
    #[default]
    Multiply,
    /// 加法策略：将权重值作为得分偏移量
    Add,
    /// 指数策略：将权重作为指数
    Exponential,
    /// 对数策略：使用对数函数调整得分
    Logarithmic,
}

/// 字段权重配置
pub struct FieldBoostConfig {
    pub field_name: String,
    pub weight: f32,
    pub strategy: BoostStrategy,
    pub condition: Option<FieldBoostCondition>,
}

impl Clone for FieldBoostConfig {
    fn clone(&self) -> Self {
        FieldBoostConfig {
            field_name: self.field_name.clone(),
            weight: self.weight,
            strategy: self.strategy.clone(),
            condition: None,
        }
    }
}

impl std::fmt::Debug for FieldBoostConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FieldBoostConfig")
            .field("field_name", &self.field_name)
            .field("weight", &self.weight)
            .field("strategy", &self.strategy)
            .field("has_condition", &self.condition.is_some())
            .finish()
    }
}

impl FieldBoostConfig {
    pub fn new(field_name: &str, weight: f32) -> Self {
        FieldBoostConfig {
            field_name: field_name.to_string(),
            weight,
            strategy: BoostStrategy::default(),
            condition: None,
        }
    }

    pub fn with_strategy(mut self, strategy: BoostStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    pub fn with_condition<F>(mut self, condition: F) -> Self
    where
        F: Fn(&serde_json::Value) -> bool + 'static + Send + Sync,
    {
        self.condition = Some(Box::new(condition));
        self
    }
}

/// 多字段搜索选项
#[derive(Debug, Clone, Default)]
pub struct MultiFieldSearchOptions {
    pub query: String,
    pub fields: Vec<FieldSearch>,
    pub boost: HashMap<String, f32>,
    pub boost_configs: Vec<FieldBoostConfig>,
    pub default_weight: f32,
    pub boost_strategy: BoostStrategy,
    pub limit: usize,
    pub offset: usize,
    pub combine: CombineStrategy,
    pub resolve: bool,
}

impl MultiFieldSearchOptions {
    pub fn new() -> Self {
        MultiFieldSearchOptions {
            query: String::new(),
            fields: Vec::new(),
            boost: HashMap::new(),
            boost_configs: Vec::new(),
            default_weight: 1.0,
            boost_strategy: BoostStrategy::default(),
            limit: 100,
            offset: 0,
            combine: CombineStrategy::Or,
            resolve: true,
        }
    }

    pub fn with_query(mut self, query: &str) -> Self {
        self.query = query.to_string();
        self
    }

    pub fn add_field(mut self, name: &str, weight: f32) -> Self {
        self.fields.push(FieldSearch::new(name, weight));
        self
    }

    pub fn set_field_boost(mut self, field: &str, boost: f32) -> Self {
        self.boost.insert(field.to_string(), boost);
        self
    }

    pub fn add_boost_config(mut self, config: FieldBoostConfig) -> Self {
        self.boost_configs.push(config);
        self
    }

    pub fn set_default_weight(mut self, weight: f32) -> Self {
        self.default_weight = weight;
        self
    }

    pub fn set_boost_strategy(mut self, strategy: BoostStrategy) -> Self {
        self.boost_strategy = strategy;
        self
    }

    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    pub fn with_offset(mut self, offset: usize) -> Self {
        self.offset = offset;
        self
    }

    pub fn with_combine(mut self, strategy: CombineStrategy) -> Self {
        self.combine = strategy;
        self
    }

    pub fn with_resolve(mut self, resolve: bool) -> Self {
        self.resolve = resolve;
        self
    }
}

/// 结果合并策略
#[derive(Debug, Clone, PartialEq, Default)]
pub enum CombineStrategy {
    /// 任一字段匹配即可（并集）
    #[default]
    Or,
    /// 所有字段都必须匹配（交集）
    And,
    /// 按权重组合评分
    Weight,
    /// 最佳字段匹配
    BestField,
}

/// 搜索协调器
#[derive(Clone)]
pub struct SearchCoordinator<'a> {
    document: &'a Document,
    options: MultiFieldSearchOptions,
}

impl<'a> SearchCoordinator<'a> {
    /// 创建新的搜索协调器
    pub fn new(document: &'a Document) -> Self {
        SearchCoordinator {
            document,
            options: MultiFieldSearchOptions::new(),
        }
    }

    /// 配置搜索选项
    pub fn options(&mut self) -> &mut MultiFieldSearchOptions {
        &mut self.options
    }

    /// 添加搜索字段
    pub fn add_field(&mut self, name: &str, weight: f32) {
        self.options.fields.push(FieldSearch::new(name, weight));
    }

    /// 设置字段的搜索查询（用于不同字段不同查询）
    pub fn set_field_query(&mut self, name: &str, query: &str) {
        if let Some(field) = self.options.fields.iter_mut().find(|f| f.name == name) {
            field.query = Some(query.to_string());
        }
    }

    /// 设置字段权重
    pub fn set_boost(&mut self, name: &str, boost: f32) {
        self.options.boost.insert(name.to_string(), boost);
    }

    /// 执行多字段搜索
    pub fn search(&self) -> Result<SearchResult> {
        if self.options.query.is_empty() {
            return Ok(SearchResult {
                results: Vec::new(),
                total: 0,
                query: String::new(),
            });
        }

        let mut field_results: Vec<(String, Vec<DocId>, f32)> = Vec::new();

        for field_search in &self.options.fields {
            let field = match self.document.field(&field_search.name) {
                Some(f) => f,
                None => continue,
            };

            let field_query = field_search.query.as_ref().unwrap_or(&self.options.query);

            let base_weight = field_search.weight;
            let field_boost = self
                .options
                .boost
                .get(&field_search.name)
                .copied()
                .unwrap_or(self.options.default_weight);
            let final_weight = self.apply_weight_strategy(base_weight, field_boost);

            let search_opts = SearchOptions {
                query: Some(field_query.clone()),
                limit: Some(self.options.limit * 2),
                offset: Some(0),
                resolve: Some(false),
                ..Default::default()
            };

            let result = crate::search::search(field.index(), &search_opts)?;
            field_results.push((field_search.name.clone(), result.results, final_weight));
        }

        let merged = self.merge_results(&field_results);

        let total = merged.len();
        let final_results: Vec<DocId> = merged
            .into_iter()
            .skip(self.options.offset)
            .take(self.options.limit)
            .collect();

        Ok(SearchResult {
            results: final_results,
            total,
            query: self.options.query.clone(),
        })
    }

    /// 应用权重策略
    fn apply_weight_strategy(&self, base_weight: f32, boost: f32) -> f32 {
        match self.options.boost_strategy {
            BoostStrategy::Multiply => base_weight * boost,
            BoostStrategy::Add => base_weight + boost,
            BoostStrategy::Exponential => {
                if boost.abs() < 1.0 {
                    base_weight * boost.ln()
                } else {
                    base_weight * boost.log2()
                }
            }
            BoostStrategy::Logarithmic => {
                if boost > 0.0 {
                    base_weight * (1.0 + boost.ln())
                } else {
                    base_weight
                }
            }
        }
    }

    /// 执行搜索并返回评分
    pub fn search_with_scores(&self) -> Result<Vec<(DocId, f32)>> {
        if self.options.query.is_empty() {
            return Ok(Vec::new());
        }

        let mut field_results: Vec<(String, Vec<DocId>, f32)> = Vec::new();

        for field_search in &self.options.fields {
            let field = match self.document.field(&field_search.name) {
                Some(f) => f,
                None => continue,
            };

            let field_query = field_search.query.as_ref().unwrap_or(&self.options.query);
            let field_boost = self
                .options
                .boost
                .get(&field_search.name)
                .copied()
                .unwrap_or(1.0);

            let search_opts = SearchOptions {
                query: Some(field_query.clone()),
                limit: Some(self.options.limit * 2),
                offset: Some(0),
                resolve: Some(false),
                ..Default::default()
            };

            let result = crate::search::search(field.index(), &search_opts)?;
            field_results.push((
                field_search.name.clone(),
                result.results,
                field_boost * field_search.weight,
            ));
        }

        let merged = self.merge_results(&field_results);
        let scores = self.calculate_scores(&field_results, &merged);

        let scored: Vec<(DocId, f32)> = merged.into_iter().zip(scores).collect();

        Ok(scored)
    }

    /// 合并多字段结果
    fn merge_results(&self, results: &[(String, Vec<DocId>, f32)]) -> Vec<DocId> {
        match self.options.combine {
            CombineStrategy::Or => self.merge_or(results),
            CombineStrategy::And => self.merge_and(results),
            CombineStrategy::Weight => self.merge_weight(results),
            CombineStrategy::BestField => self.merge_best(results),
        }
    }

    /// 并集合并
    fn merge_or(&self, results: &[(String, Vec<DocId>, f32)]) -> Vec<DocId> {
        let mut seen = HashSet::new();
        let mut result = Vec::new();

        for (_, docs, _) in results {
            for &doc_id in docs {
                if seen.insert(doc_id) {
                    result.push(doc_id);
                }
            }
        }

        result
    }

    /// 交集合并
    fn merge_and(&self, results: &[(String, Vec<DocId>, f32)]) -> Vec<DocId> {
        if results.is_empty() {
            return Vec::new();
        }

        let mut sets: Vec<HashSet<DocId>> = results
            .iter()
            .map(|(_, docs, _)| docs.iter().cloned().collect())
            .collect();

        let mut result = sets.remove(0);
        for set in sets {
            result = result.intersection(&set).cloned().collect();
        }

        result.into_iter().collect()
    }

    /// 加权合并
    fn merge_weight(&self, results: &[(String, Vec<DocId>, f32)]) -> Vec<DocId> {
        let mut scored: Vec<(DocId, f32)> = Vec::new();
        let mut seen: HashMap<DocId, usize> = HashMap::new();

        for (field_name, docs, weight) in results {
            for &doc_id in docs {
                let boost = self.options.boost.get(field_name).copied().unwrap_or(1.0);
                let score = weight * boost;

                if let Some(&pos) = seen.get(&doc_id) {
                    scored[pos].1 += score;
                } else {
                    seen.insert(doc_id, scored.len());
                    scored.push((doc_id, score));
                }
            }
        }

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.into_iter().map(|(id, _)| id).collect()
    }

    /// 最佳字段合并
    fn merge_best(&self, results: &[(String, Vec<DocId>, f32)]) -> Vec<DocId> {
        if results.is_empty() {
            return Vec::new();
        }

        // 找到结果最多的字段作为主字段
        let mut best_field = &results[0];
        for result in results {
            if result.1.len() > best_field.1.len() {
                best_field = result;
            }
        }

        best_field.1.clone()
    }

    /// 计算文档评分
    fn calculate_scores(&self, results: &[(String, Vec<DocId>, f32)], docs: &[DocId]) -> Vec<f32> {
        let mut doc_scores: HashMap<DocId, f32> = HashMap::new();
        let mut doc_counts: HashMap<DocId, usize> = HashMap::new();

        for (field_name, field_docs, weight) in results {
            for &doc_id in field_docs {
                let boost = self.options.boost.get(field_name).copied().unwrap_or(1.0);
                let score = weight * boost;
                *doc_scores.entry(doc_id).or_insert(0.0) += score;
                *doc_counts.entry(doc_id).or_insert(0) += 1;
            }
        }

        docs.iter()
            .map(|&id| {
                let base_score = doc_scores.get(&id).copied().unwrap_or(0.0);
                let count = doc_counts.get(&id).copied().unwrap_or(1) as f32;
                match self.options.combine {
                    CombineStrategy::Or => base_score,
                    _ => base_score * (1.0 + (count - 1.0) * 0.1),
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::{Document, DocumentConfig, FieldConfig};
    use serde_json::json;

    fn create_test_document() -> Document {
        let config = DocumentConfig::new()
            .add_field(FieldConfig::new("title"))
            .add_field(FieldConfig::new("content"));

        let mut doc = Document::new(config).expect("Document::new should succeed");

        doc.add(
            1,
            &json!({"title": "Rust Programming", "content": "Learn Rust today"}),
        )
        .expect("add doc 1 should succeed");
        doc.add(
            2,
            &json!({"title": "JavaScript Guide", "content": "JavaScript tutorial"}),
        )
        .expect("add doc 2 should succeed");
        doc.add(
            3,
            &json!({"title": "Rust vs Go", "content": "Comparing Rust and Go"}),
        )
        .expect("add doc 3 should succeed");

        doc
    }

    #[test]
    fn test_search_coordinator_basic() {
        let doc = create_test_document();
        let mut coordinator = SearchCoordinator::new(&doc);

        coordinator.add_field("title", 2.0);
        coordinator.add_field("content", 1.0);
        coordinator.options.query = "Rust".to_string();

        let result = coordinator.search().expect("search should succeed");

        // 应该找到包含 "Rust" 的文档 (1 和 3)
        assert!(result.results.contains(&1));
        assert!(result.results.contains(&3));
    }

    #[test]
    fn test_search_coordinator_weight() {
        let doc = create_test_document();
        let mut coordinator = SearchCoordinator::new(&doc);

        coordinator.add_field("title", 2.0);
        coordinator.add_field("content", 1.0);
        coordinator.options.query = "Rust".to_string();

        let scored = coordinator
            .search_with_scores()
            .expect("search_with_scores should succeed");

        assert!(!scored.is_empty());
        // 文档1和3都包含 Rust
        for s in &scored {
            assert!(s.1 > 0.0);
        }
    }

    #[test]
    fn test_search_coordinator_or_strategy() {
        let doc = create_test_document();
        let mut coordinator = SearchCoordinator::new(&doc);
        coordinator.options.combine = CombineStrategy::Or;

        coordinator.add_field("title", 1.0);
        coordinator.options.query = "Rust".to_string();

        let result = coordinator.search().expect("search should succeed");
        assert!(!result.results.is_empty());
    }

    #[test]
    fn test_search_coordinator_and_strategy() {
        let doc = create_test_document();
        let mut coordinator = SearchCoordinator::new(&doc);
        coordinator.options.combine = CombineStrategy::And;

        coordinator.add_field("title", 1.0);
        coordinator.options.query = "Rust".to_string();

        let result = coordinator.search().expect("search should succeed");
        assert!(!result.results.is_empty());
    }

    #[test]
    fn test_search_coordinator_limit() {
        let doc = create_test_document();
        let mut coordinator = SearchCoordinator::new(&doc);

        coordinator.add_field("title", 1.0);
        coordinator.options.query = "Rust".to_string();
        coordinator.options.limit = 1;

        let result = coordinator.search().expect("search should succeed");
        assert!(result.results.len() <= 1);
    }

    #[test]
    fn test_search_coordinator_offset() {
        let doc = create_test_document();
        let mut coordinator = SearchCoordinator::new(&doc);

        coordinator.add_field("title", 1.0);
        coordinator.options.query = "Rust".to_string();
        coordinator.options.limit = 10;
        coordinator.options.offset = 1;

        let result = coordinator.search().expect("search should succeed");
        assert!(result.total >= 1);
    }
}
