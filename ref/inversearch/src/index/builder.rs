use super::{Index, DocId, TokenizeMode};
use crate::error::Result;
use std::collections::HashMap;

pub fn add_document(
    index: &mut Index,
    id: DocId,
    content: &str,
    append: bool,
    skip_update: bool,
) -> Result<()> {
    if content.is_empty() || id == 0 {
        return Ok(());
    }

    if !skip_update && !append {
        if index.contains(id) {
            return index.update(id, content);
        }
    }

    let depth = index.depth;
    let encoded = index.encoder.encode(content)?;
    let word_length = encoded.len();

    if word_length == 0 {
        return Ok(());
    }

    let mut dupes_ctx = HashMap::new();
    let mut dupes = HashMap::new();
    let resolution = index.resolution;
    let rtl = index.rtl;

    for i in 0..word_length {
        let term_idx = if rtl { word_length - 1 - i } else { i };
        let term = &encoded[term_idx];
        let term_length = term.len();

        if term_length == 0 {
            continue;
        }

        if depth == 0 && dupes.contains_key(term) {
            continue;
        }

        let score = index.get_score(resolution, word_length, i, Some(term_length), None);

        match index.tokenize {
            TokenizeMode::Full => {
                if term_length > 2 {
                    for x in 0..term_length {
                        for y in (x + 1..=term_length).rev() {
                            let token = &term[x..y];
                            let x_idx = if rtl { term_length - 1 - x } else { x };
                            let partial_score = index.get_score(
                                resolution,
                                word_length,
                                i,
                                Some(term_length),
                                Some(x_idx),
                            );
                            index.push_index(&mut dupes, token, partial_score, id, append, None);
                        }
                    }
                } else {
                    add_strict(index, &mut dupes, term, score, id, append, depth, rtl, i, &encoded, word_length);
                }
            }
            TokenizeMode::Bidirectional => {
                if term_length > 1 {
                    for x in (1..term_length).rev() {
                        let token = &term[if rtl { term_length - 1 - x } else { x }..];
                        let partial_score = index.get_score(
                            resolution,
                            word_length,
                            i,
                            Some(term_length),
                            Some(x),
                        );
                        index.push_index(&mut dupes, token, partial_score, id, append, None);
                    }
                }
                add_forward(index, &mut dupes, term, score, id, append, rtl);
                if depth > 0 {
                    add_context(index, &mut dupes_ctx, term, i, depth, rtl, &encoded, word_length, id, append);
                }
            }
            TokenizeMode::Reverse => {
                if term_length > 1 {
                    for x in (1..term_length).rev() {
                        let token = &term[if rtl { term_length - 1 - x } else { x }..];
                        let partial_score = index.get_score(
                            resolution,
                            word_length,
                            i,
                            Some(term_length),
                            Some(x),
                        );
                        index.push_index(&mut dupes, token, partial_score, id, append, None);
                    }
                }
                if depth > 0 {
                    add_context(index, &mut dupes_ctx, term, i, depth, rtl, &encoded, word_length, id, append);
                }
            }
            TokenizeMode::Forward => {
                if term_length > 1 {
                    add_forward(index, &mut dupes, term, score, id, append, rtl);
                } else {
                    index.push_index(&mut dupes, term, score, id, append, None);
                }
                if depth > 0 {
                    add_context(index, &mut dupes_ctx, term, i, depth, rtl, &encoded, word_length, id, append);
                }
            }
            TokenizeMode::Strict => {
                add_strict(index, &mut dupes, term, score, id, append, depth, rtl, i, &encoded, word_length);
            }
        }
    }

    if !index.fastupdate {
        if let super::Register::Set(reg) = &mut index.reg {
            reg.add(id);
        }
    }

    Ok(())
}

fn add_strict(
    index: &mut Index,
    dupes: &mut HashMap<String, bool>,
    term: &str,
    score: usize,
    id: DocId,
    append: bool,
    depth: usize,
    rtl: bool,
    i: usize,
    encoded: &[String],
    word_length: usize,
) {
    index.push_index(dupes, term, score, id, append, None);

    if depth > 0 && word_length > 1 && i < word_length - 1 {
        add_context(index, dupes, term, i, depth, rtl, encoded, word_length, id, append);
    }
}

fn add_forward(
    index: &mut Index,
    dupes: &mut HashMap<String, bool>,
    term: &str,
    score: usize,
    id: DocId,
    append: bool,
    rtl: bool,
) {
    let mut token = String::new();
    for x in 0..term.len() {
        let char_idx = if rtl { term.len() - 1 - x } else { x };
        if let Some(ch) = term.chars().nth(char_idx) {
            token.push(ch);
        } else {
            eprintln!("Index builder: Character index {} out of bounds for term '{}'", char_idx, term);
            continue;
        }
        index.push_index(dupes, &token, score, id, append, None);
    }
}

fn add_context(
    index: &mut Index,
    dupes: &mut HashMap<String, bool>,
    term: &str,
    i: usize,
    depth: usize,
    rtl: bool,
    encoded: &[String],
    word_length: usize,
    id: DocId,
    append: bool,
) {
    let mut dupes_inner = HashMap::new();
    let resolution = index.resolution_ctx;
    let keyword = term;
    let size = depth.min(if rtl { i + 1 } else { word_length - i });

    dupes_inner.insert(keyword.to_string(), true);

    for x in 1..size {
        let term_idx = if rtl {
            word_length - 1 - i - x
        } else {
            i + x
        };

        if term_idx >= word_length {
            break;
        }

        let context_term = &encoded[term_idx];

        if !context_term.is_empty() && !dupes_inner.contains_key(context_term) {
            dupes_inner.insert(context_term.to_string(), true);

            let context_score = index.get_score(
                resolution + if word_length / 2 > resolution { 0 } else { 1 },
                word_length,
                i,
                Some(size - 1),
                Some(x - 1),
            );

            let swap = index.bidirectional && **context_term > *keyword;
            let (ctx_term, ctx_keyword) = if swap {
                (&keyword[..], &context_term[..])
            } else {
                (&context_term[..], &keyword[..])
            };

            index.push_index(dupes, ctx_term, context_score, id, append, Some(ctx_keyword));
        }
    }
}

pub fn get_score(
    resolution: usize,
    length: usize,
    i: usize,
    term_length: Option<usize>,
    x: Option<usize>,
) -> usize {
    if i == 0 || resolution <= 1 {
        return 0;
    }

    let total_length = length + term_length.unwrap_or(0);
    let offset = x.unwrap_or(0);

    if total_length <= resolution {
        i + offset
    } else {
        // Match JavaScript implementation: ((resolution - 1) / total_length * (i + offset) + 1) | 0
        let calculation = ((resolution - 1) as f64 / total_length as f64 * (i + offset) as f64 + 1.0) as usize;
        calculation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_score() {
        assert_eq!(get_score(9, 10, 0, Some(5), None), 0);
        assert_eq!(get_score(9, 10, 1, Some(5), None), 1);
        assert_eq!(get_score(9, 10, 5, Some(5), None), 3);
        assert_eq!(get_score(9, 100, 10, Some(5), None), 1);
    }

    #[test]
    fn test_add_document() {
        let mut index = Index::default();
        add_document(&mut index, 1, "hello world", false, false).unwrap();
        assert!(index.contains(1));
    }

    #[test]
    fn test_add_document_empty() {
        let mut index = Index::default();
        add_document(&mut index, 1, "", false, false).unwrap();
        assert!(!index.contains(1));
    }

    #[test]
    fn test_add_document_append() {
        let mut index = Index::default();
        add_document(&mut index, 1, "hello", false, false).unwrap();
        add_document(&mut index, 1, "world", true, false).unwrap();
        assert!(index.contains(1));
    }
}
