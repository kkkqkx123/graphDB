use crate::encoder::Encoder;
use crate::error::Result;
use crate::highlight::types::*;
use crate::highlight::core::*;
use crate::highlight::boundary::*;
use crate::highlight::matcher::*;
use serde_json::Value;
use std::collections::HashMap;

pub struct HighlightProcessor {
    encoder_cache: HashMap<String, Vec<String>>,
}

impl HighlightProcessor {
    pub fn new() -> Self {
        Self {
            encoder_cache: HashMap::new(),
        }
    }

    pub fn highlight_fields(
        &mut self,
        query: &str,
        results: &mut FieldSearchResults,
        index_encoders: &HashMap<String, Encoder>,
        pluck: Option<&str>,
        options: &HighlightOptions,
    ) -> Result<()> {
        let config = HighlightConfig::from_options(options)?;

        if let Some(pluck_field) = pluck {
            // Single field mode (pluck)
            if let Some(field_result) = results.iter_mut().find(|r| r.field == pluck_field) {
                if let Some(encoder) = index_encoders.get(pluck_field) {
                    self.highlight_single_field(query, field_result, encoder, &config)?;
                }
            }
        } else {
            // Multi-field mode
            for field_result in results.iter_mut() {
                if let Some(encoder) = index_encoders.get(&field_result.field) {
                    self.highlight_single_field(query, field_result, encoder, &config)?;
                }
            }
        }

        Ok(())
    }

    fn highlight_single_field(
        &mut self,
        query: &str,
        field_result: &mut FieldSearchResult,
        encoder: &Encoder,
        config: &HighlightConfig,
    ) -> Result<()> {
        let query_enc = self.get_or_encode_query(query, encoder)?;

        for result in field_result.result.iter_mut() {
            if let Some(doc) = &result.doc {
                if let Some(highlighted) = self.highlight_document_content(
                    doc,
                    &field_result.field,
                    encoder,
                    config,
                    &query_enc,
                )? {
                    result.highlight = Some(highlighted);
                }
            }
        }

        Ok(())
    }

    fn highlight_document_content(
        &self,
        document: &Value,
        field_path: &str,
        encoder: &Encoder,
        config: &HighlightConfig,
        query_enc: &[String],
    ) -> Result<Option<String>> {
        let content = crate::common::parse_simple(document, field_path)?;
        if content.is_empty() {
            return Ok(None);
        }

        let highlighted = self.process_content_with_boundary(&content, query_enc, encoder, config)?;
        
        // Apply merge if configured
        let final_result = if let Some(merge_pattern) = &config.merge {
            let regex = regex::Regex::new(&regex::escape(merge_pattern))
                .map_err(|e| crate::error::InversearchError::Highlight(format!("Invalid merge pattern: {}", e)))?;
            regex.replace_all(&highlighted, " ").to_string()
        } else {
            highlighted
        };

        Ok(Some(final_result))
    }

    fn process_content_with_boundary(
        &self,
        content: &str,
        query_enc: &[String],
        encoder: &Encoder,
        config: &HighlightConfig,
    ) -> Result<String> {
        let doc_terms: Vec<&str> = content.split_whitespace().collect();
        let mut boundary_terms = Vec::new();
        let mut match_positions = Vec::new();
        let mut first_match_pos = -1i32;
        let mut total_match_length = 0usize;

        for (term_idx, doc_term) in doc_terms.iter().enumerate() {
            let doc_term_trimmed = doc_term.trim();
            if doc_term_trimmed.is_empty() {
                continue;
            }

            let doc_enc = encode_and_join(doc_term_trimmed, encoder)?;
            let match_result = find_best_match(
                doc_term_trimmed,
                &doc_enc,
                query_enc,
                &config.markup_open,
                &config.markup_close,
            );

            if match_result.found {
                let current_pos = self.calculate_current_position(&boundary_terms);
                
                if first_match_pos < 0 {
                    first_match_pos = current_pos;
                }
                total_match_length += doc_term_trimmed.len();
                match_positions.push(term_idx);

                boundary_terms.push(BoundaryTerm {
                    is_match: true,
                    content: match_result.match_str,
                    original_pos: term_idx,
                });
            } else {
                boundary_terms.push(BoundaryTerm {
                    is_match: false,
                    content: doc_term_trimmed.to_string(),
                    original_pos: term_idx,
                });
            }

            // Early termination check
            if let Some(boundary) = &config.boundary {
                let boundary_total = boundary.total.unwrap_or(900000);
                if total_match_length >= boundary_total {
                    break;
                }
            }
        }

        // Apply boundary processing if needed
        if config.boundary.is_some() && (first_match_pos >= 0 || config.boundary.as_ref().unwrap().before.is_some() || config.boundary.as_ref().unwrap().after.is_some()) {
            apply_advanced_boundary(boundary_terms, config)
        } else {
            self.join_boundary_terms(&boundary_terms)
        }
    }

    fn calculate_current_position(&self, terms: &[BoundaryTerm]) -> i32 {
        terms
            .iter()
            .map(|term| term.content.len() as i32 + 1) // +1 for space
            .sum::<i32>()
            .saturating_sub(1) // Remove last space
    }

    fn join_boundary_terms(&self, terms: &[BoundaryTerm]) -> Result<String> {
        let mut result = String::new();
        for (i, term) in terms.iter().enumerate() {
            if i > 0 {
                result.push(' ');
            }
            result.push_str(&term.content);
        }
        Ok(result)
    }

    fn get_or_encode_query(&mut self, query: &str, encoder: &Encoder) -> Result<Vec<String>> {
        // Use a simple key for caching - in production, this should be more sophisticated
        let key = format!("encoder_{}", std::ptr::addr_of!(encoder) as usize);
        
        if let Some(cached) = self.encoder_cache.get(&key) {
            return Ok(cached.clone());
        }

        let encoded = encoder.encode(query)?;
        self.encoder_cache.insert(key, encoded.clone());
        Ok(encoded)
    }
}

impl Default for HighlightProcessor {
    fn default() -> Self {
        Self::new()
    }
}

// Public API functions for backward compatibility and simple usage
pub fn highlight_fields(
    query: &str,
    results: &mut FieldSearchResults,
    index_encoders: &HashMap<String, Encoder>,
    pluck: Option<&str>,
    options: &HighlightOptions,
) -> Result<()> {
    let mut processor = HighlightProcessor::new();
    processor.highlight_fields(query, results, index_encoders, pluck, options)
}