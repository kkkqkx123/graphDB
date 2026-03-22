use crate::encoder::Encoder;
use crate::error::Result;
use crate::highlight::types::*;
use crate::highlight::matcher::*;
use serde_json::Value;

pub struct HighlightedTerm {
    pub is_match: bool,
    pub content: String,
}

pub fn highlight_single_document(
    query: &str,
    content: &str,
    encoder: &Encoder,
    config: &HighlightConfig,
) -> Result<String> {
    let query_enc = encoder.encode(query)?;
    let doc_terms: Vec<&str> = content.split_whitespace().collect();

    let mut highlighted_terms = Vec::new();
    let mut match_positions = Vec::new();
    let mut first_match_pos = -1i32;
    let mut last_match_pos = -1i32;
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
            &query_enc,
            &config.markup_open,
            &config.markup_close,
        );

        if match_result.found {
            if config.boundary.is_some() {
                let current_text_length = highlighted_terms
                    .iter()
                    .map(|term: &HighlightedTerm| term.content.len())
                    .sum::<usize>()
                    + highlighted_terms.len().saturating_sub(1); // spaces

                if first_match_pos < 0 {
                    first_match_pos = current_text_length as i32;
                }
                last_match_pos = current_text_length as i32 + match_result.match_str.len() as i32;
                total_match_length += doc_term_trimmed.len();
                match_positions.push(term_idx);
            }

            highlighted_terms.push(HighlightedTerm {
                is_match: true,
                content: match_result.match_str,
            });
        } else {
            highlighted_terms.push(HighlightedTerm {
                is_match: false,
                content: doc_term_trimmed.to_string(),
            });
        }

        // Early termination if we exceed boundary
        if let Some(boundary) = &config.boundary {
            let boundary_total = boundary.total.unwrap_or(900000);
            if total_match_length >= boundary_total {
                break;
            }
        }
    }

    let result = highlighted_terms
        .iter()
        .map(|term: &HighlightedTerm| term.content.as_str())
        .collect::<Vec<&str>>()
        .join(" ");

    if let Some(boundary) = &config.boundary {
        apply_boundary_simple(
            &result,
            &match_positions,
            first_match_pos,
            last_match_pos,
            total_match_length,
            config,
        )
    } else {
        Ok(result)
    }
}

pub fn highlight_document(
    query: &str,
    document: &Value,
    field_path: &str,
    encoder: &Encoder,
    config: &HighlightConfig,
) -> Result<Option<String>> {
    let content = crate::common::parse_simple(document, field_path)?;
    if content.is_empty() {
        return Ok(None);
    }

    let highlighted = highlight_single_document(query, &content, encoder, config)?;
    
    // Apply merge if configured
    if let Some(merge_pattern) = &config.merge {
        let regex = regex::Regex::new(&regex::escape(merge_pattern))
            .map_err(|e| crate::error::InversearchError::Highlight(format!("Invalid merge pattern: {}", e)))?;
        Ok(Some(regex.replace_all(&highlighted, " ").to_string()))
    } else {
        Ok(Some(highlighted))
    }
}

fn apply_boundary_simple(
    text: &str,
    match_positions: &[usize],
    first_match_pos: i32,
    last_match_pos: i32,
    _total_match_length: usize,
    config: &HighlightConfig,
) -> Result<String> {
    let boundary = match config.boundary.as_ref() {
        Some(b) => b,
        None => return Ok(text.to_string()),
    };
    let boundary_total = boundary.total.unwrap_or(900000);
    let boundary_before = boundary.before.unwrap_or(0);
    let boundary_after = boundary.after.unwrap_or(0);

    let markup_length = match_positions.len() * (config.template.len() - 2);
    let ellipsis = &config.ellipsis;
    let ellipsis_length = ellipsis.len();

    let boundary_length = (boundary_total + markup_length - ellipsis_length * 2) as i32;
    let length = last_match_pos - first_match_pos;

    if boundary_before > 0 || boundary_after > 0 || (text.len() - markup_length) > boundary_total {
        let start = if boundary_before > 0 {
            first_match_pos - boundary_before as i32
        } else {
            first_match_pos - ((boundary_length - length) / 2)
        };

        let end = if boundary_after > 0 {
            last_match_pos + boundary_after as i32
        } else {
            start + boundary_length
        };

        let start_usize = std::cmp::max(0, start) as usize;
        let end_usize = std::cmp::min(text.len(), end as usize);

        let result = if start_usize > 0 {
            format!("{}{}{}", ellipsis, &text[start_usize..end_usize], if end_usize < text.len() { ellipsis.clone() } else { String::new() })
        } else {
            format!("{}{}", &text[start_usize..end_usize], if end_usize < text.len() { ellipsis.clone() } else { String::new() })
        };

        Ok(result)
    } else {
        Ok(text.to_string())
    }
}