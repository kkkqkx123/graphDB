use crate::encoder::Encoder;
use crate::error::Result;
use crate::highlight::types::*;
use crate::highlight::matcher::*;
use crate::document::tree::{parse_tree, extract_value};
use crate::DocId;
use serde_json::Value;

/// 从文档中提取字段值的内部工具函数
fn extract_field_value(document: &Value, field_path: &str) -> Result<String> {
    let mut marker = vec![];
    let tree_path = parse_tree(field_path, &mut marker);
    match extract_value(document, &tree_path) {
        Some(value) => Ok(value),
        None => Ok(String::new()),
    }
}

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

    if config.boundary.is_some() {
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

/// 高亮单个文档并返回结构化结果（新函数）
pub fn highlight_single_document_structured(
    query: &str,
    content: &str,
    encoder: &Encoder,
    config: &HighlightConfig,
) -> Result<DocumentHighlight> {
    let query_enc = encoder.encode(query)?;
    let doc_terms: Vec<&str> = content.split_whitespace().collect();

    let mut highlighted_terms = Vec::new();
    let mut matches = Vec::new();
    let mut match_positions = Vec::new();
    let mut first_match_pos = -1i32;
    let mut last_match_pos = -1i32;
    let mut total_match_length = 0usize;
    let mut current_char_pos = 0usize; // 跟踪当前字符位置

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
            // 计算匹配项在原文中的位置
            let start_pos = current_char_pos;
            let end_pos = current_char_pos + doc_term_trimmed.len();

            // 记录匹配信息
            matches.push(HighlightMatch {
                text: doc_term_trimmed.to_string(),
                start_pos,
                end_pos,
                matched_query: query.to_string(),
                score: None,
            });

            if config.boundary.is_some() {
                let current_text_length = highlighted_terms
                    .iter()
                    .map(|term: &HighlightedTerm| term.content.len())
                    .sum::<usize>()
                    + highlighted_terms.len().saturating_sub(1);

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

        // 更新字符位置（+1 表示空格）
        current_char_pos += doc_term_trimmed.len() + 1;

        // Early termination if we exceed boundary
        if let Some(boundary) = &config.boundary {
            let boundary_total = boundary.total.unwrap_or(900000);
            if total_match_length >= boundary_total {
                break;
            }
        }
    }

    // 构建高亮后的文本
    let highlighted_text = highlighted_terms
        .iter()
        .map(|term: &HighlightedTerm| term.content.as_str())
        .collect::<Vec<&str>>()
        .join(" ");

    let final_text = if config.boundary.is_some() {
        apply_boundary_simple(
            &highlighted_text,
            &match_positions,
            first_match_pos,
            last_match_pos,
            total_match_length,
            config,
        )?
    } else {
        highlighted_text
    };

    // 应用合并逻辑
    let final_highlighted_text = if let Some(merge_pattern) = &config.merge {
        let regex = regex::Regex::new(&regex::escape(merge_pattern))
            .map_err(|e| crate::error::InversearchError::Highlight(format!("Invalid merge pattern: {}", e)))?;
        regex.replace_all(&final_text, " ").to_string()
    } else {
        final_text
    };

    // 收集匹配的查询词
    let matched_queries: Vec<String> = matches
        .iter()
        .map(|m| m.matched_query.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    let field_highlight = FieldHighlight {
        field: "content".to_string(), // 默认字段名
        matches,
        highlighted_text: Some(final_highlighted_text),
        matched_queries,
    };

    let total_matches = field_highlight.matches.len();

    Ok(DocumentHighlight {
        id: 0, // 调用方需要设置
        fields: vec![field_highlight],
        total_matches,
    })
}

pub fn highlight_document(
    query: &str,
    document: &Value,
    field_path: &str,
    encoder: &Encoder,
    config: &HighlightConfig,
) -> Result<Option<String>> {
    let content = extract_field_value(document, field_path)?;
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

/// 高亮文档并返回结构化结果（新函数）
pub fn highlight_document_structured(
    query: &str,
    document: &Value,
    field_path: &str,
    doc_id: DocId,
    encoder: &Encoder,
    config: &HighlightConfig,
) -> Result<Option<DocumentHighlight>> {
    let content = extract_field_value(document, field_path)?;
    if content.is_empty() {
        return Ok(None);
    }

    let mut highlight = highlight_single_document_structured(query, &content, encoder, config)?;
    
    // 如果没有匹配，返回 None
    if highlight.total_matches == 0 {
        return Ok(None);
    }
    
    highlight.id = doc_id;
    highlight.fields[0].field = field_path.to_string();

    Ok(Some(highlight))
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
            first_match_pos - boundary_before
        } else {
            first_match_pos - ((boundary_length - length) / 2)
        };

        let end = if boundary_after > 0 {
            last_match_pos + boundary_after
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
