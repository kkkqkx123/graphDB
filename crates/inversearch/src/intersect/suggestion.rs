//! Recommended System Modules
//!
//! Provide search suggestion and fuzzy matching function

use crate::r#type::IntermediateSearchResults;

/// Recommended configuration
#[derive(Debug, Clone)]
pub struct SuggestionConfig {
    pub max_suggestions: usize,
    pub fuzzy_threshold: f32,
    pub max_alternatives: usize,
    pub min_similarity: f32,
}

impl Default for SuggestionConfig {
    fn default() -> Self {
        SuggestionConfig {
            max_suggestions: 5,
            fuzzy_threshold: 0.7,
            max_alternatives: 3,
            min_similarity: 0.3,
        }
    }
}

/// Recommended results
#[derive(Debug, Clone, Default)]
pub struct SuggestionResult {
    pub suggestions: Vec<String>,
    pub fuzzy_matches: Vec<(u64, f32)>,
    pub alternative_queries: Vec<String>,
}

/// Suggested Engines
pub struct SuggestionEngine {
    config: SuggestionConfig,
}

impl SuggestionEngine {
    /// Creating a new suggestion engine
    pub fn new(config: SuggestionConfig) -> Self {
        SuggestionEngine { config }
    }

    /// Generating recommendations
    pub fn generate_suggestions(
        &self,
        query: &str,
        search_results: &IntermediateSearchResults,
    ) -> SuggestionResult {
        SuggestionResult {
            fuzzy_matches: self.generate_fuzzy_matches(query, search_results),
            alternative_queries: self.generate_alternative_queries(query),
            suggestions: self.generate_query_suggestions(query),
        }
    }

    /// Generating query recommendations
    fn generate_query_suggestions(&self, query: &str) -> Vec<String> {
        let mut suggestions = Vec::new();

        // Simple Spell Checking Suggestions
        if query.len() > 3 {
            // Generate some common spelling variants
            let variants = self.generate_spelling_variants(query);
            suggestions.extend(variants);
        }

        // Limit the number of results
        suggestions.truncate(self.config.max_suggestions);
        suggestions
    }

    /// Generating Spelling Variants
    fn generate_spelling_variants(&self, query: &str) -> Vec<String> {
        let mut variants = Vec::new();

        // Simple Character Exchange Recommendations
        let chars: Vec<char> = query.chars().collect();
        for i in 0..chars.len() - 1 {
            let mut new_chars = chars.clone();
            new_chars.swap(i, i + 1);
            variants.push(new_chars.iter().collect());
        }

        variants
    }

    /// Generating Alternative Queries
    fn generate_alternative_queries(&self, query: &str) -> Vec<String> {
        let mut alternatives = Vec::new();

        // Simple Query Extension
        if query.contains(' ') {
            let parts: Vec<&str> = query.split_whitespace().collect();
            if parts.len() > 1 {
                // Generating partial queries
                alternatives.push(parts[0].to_string());
                if parts.len() > 2 {
                    alternatives.push(format!("{} {}", parts[0], parts[1]));
                }
            }
        }

        // Limit the number of results
        alternatives.truncate(self.config.max_alternatives);
        alternatives
    }

    /// Generate fuzzy matches
    fn generate_fuzzy_matches(
        &self,
        query: &str,
        search_results: &IntermediateSearchResults,
    ) -> Vec<(u64, f32)> {
        let mut matches = Vec::new();

        for result_array in search_results.iter() {
            for &id in result_array {
                // Simplified fuzzy matching algorithm
                let similarity = self.calculate_similarity(query, &id.to_string());
                if similarity >= self.config.fuzzy_threshold {
                    matches.push((id, similarity));
                }
            }
        }

        // Sort by similarity
        matches.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Limit the number of results
        matches.truncate(self.config.max_suggestions);
        matches
    }

    /// Calculating similarity (simplified version)
    fn calculate_similarity(&self, s1: &str, s2: &str) -> f32 {
        let longer = if s1.len() > s2.len() { s1 } else { s2 };
        let shorter = if s1.len() > s2.len() { s2 } else { s1 };

        if longer.is_empty() {
            return 1.0;
        }

        let edit_distance = self.levenshtein_distance(longer, shorter);
        let max_len = longer.len() as f32;
        1.0 - (edit_distance as f32 / max_len)
    }

    /// Calculation of the distance Levenshtein
    fn levenshtein_distance(&self, s1: &str, s2: &str) -> usize {
        let len1 = s1.chars().count();
        let len2 = s2.chars().count();

        if len1 == 0 {
            return len2;
        }
        if len2 == 0 {
            return len1;
        }

        let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

        for (i, row) in matrix.iter_mut().enumerate().take(len1 + 1) {
            row[0] = i;
        }

        for (j, cell) in matrix[0].iter_mut().enumerate().take(len2 + 1) {
            *cell = j;
        }

        for (i, c1) in s1.chars().enumerate() {
            for (j, c2) in s2.chars().enumerate() {
                let cost = if c1 == c2 { 0 } else { 1 };
                matrix[i + 1][j + 1] = std::cmp::min(
                    std::cmp::min(
                        matrix[i][j + 1] + 1, // deletion
                        matrix[i + 1][j] + 1, // insertion
                    ),
                    matrix[i][j] + cost, // substitution
                );
            }
        }

        matrix[len1][len2]
    }
}

/// Suggestion Scorer
pub struct SuggestionScorer {
    config: SuggestionConfig,
}

impl SuggestionScorer {
    /// Create a new suggestion rater
    pub fn new(config: SuggestionConfig) -> Self {
        SuggestionScorer { config }
    }

    /// Recommendations for scoring
    pub fn score_suggestion(&self, original: &str, suggestion: &str) -> f32 {
        let engine = SuggestionEngine::new(self.config.clone());
        engine.calculate_similarity(original, suggestion)
    }
}

/// Convenience Functions for Generating Suggestions
pub fn generate_suggestions(
    query: &str,
    search_results: &IntermediateSearchResults,
    config: &SuggestionConfig,
) -> SuggestionResult {
    let engine = SuggestionEngine::new(config.clone());
    engine.generate_suggestions(query, search_results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suggestion_config_default() {
        let config = SuggestionConfig::default();
        assert_eq!(config.max_suggestions, 5);
        assert_eq!(config.fuzzy_threshold, 0.7);
        assert_eq!(config.max_alternatives, 3);
        assert_eq!(config.min_similarity, 0.3);
    }

    #[test]
    fn test_suggestion_engine_new() {
        let config = SuggestionConfig::default();
        let engine = SuggestionEngine::new(config);
        assert_eq!(engine.config.max_suggestions, 5);
    }

    #[test]
    fn test_generate_suggestions() {
        let config = SuggestionConfig::default();
        let engine = SuggestionEngine::new(config);

        let search_results = vec![vec![1, 2, 3], vec![4, 5, 6]];
        let suggestions = engine.generate_suggestions("test", &search_results);

        assert!(!suggestions.suggestions.is_empty());
        assert!(
            !suggestions.alternative_queries.is_empty() || suggestions.fuzzy_matches.is_empty()
        );
    }

    #[test]
    fn test_suggestion_scorer() {
        let config = SuggestionConfig::default();
        let scorer = SuggestionScorer::new(config);

        let score = scorer.score_suggestion("test", "tset");
        assert!(score > 0.3); // The similarity should be higher

        let score2 = scorer.score_suggestion("test", "completely_different");
        assert!(score2 < 0.3); // The similarity should be very low
    }
}
