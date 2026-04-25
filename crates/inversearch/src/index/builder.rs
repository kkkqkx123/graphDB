use super::{DocId, Index, TokenizeMode};
use crate::error::Result;
use std::collections::HashMap;

/// Indexing the build context, encapsulating the parameters needed for add_strict and add_context
struct IndexContext<'a> {
    index: &'a mut Index,
    dupes: &'a mut HashMap<String, bool>,
    term: &'a str,
    score: usize,
    id: DocId,
    append: bool,
}

/// Context extension parameter for the add_context function
struct ContextParams<'a> {
    i: usize,
    depth: usize,
    rtl: bool,
    encoded: &'a [String],
    word_length: usize,
}

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

    if !skip_update && !append && index.contains(id) {
        return index.update(id, content);
    }

    index.documents.insert(id, content.to_string());
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
                    let ctx = IndexContext {
                        index,
                        dupes: &mut dupes,
                        term,
                        score,
                        id,
                        append,
                    };
                    let params = ContextParams {
                        i,
                        depth,
                        rtl,
                        encoded: &encoded,
                        word_length,
                    };
                    add_strict(ctx, &params);
                }
            }
            TokenizeMode::Bidirectional => {
                if term_length > 1 {
                    for x in (1..term_length).rev() {
                        let token = &term[if rtl { term_length - 1 - x } else { x }..];
                        let partial_score =
                            index.get_score(resolution, word_length, i, Some(term_length), Some(x));
                        index.push_index(&mut dupes, token, partial_score, id, append, None);
                    }
                }
                add_forward(index, &mut dupes, term, score, id, append, rtl);
                if depth > 0 {
                    let ctx = IndexContext {
                        index,
                        dupes: &mut dupes_ctx,
                        term,
                        score,
                        id,
                        append,
                    };
                    let params = ContextParams {
                        i,
                        depth,
                        rtl,
                        encoded: &encoded,
                        word_length,
                    };
                    add_context(ctx, &params);
                }
            }
            TokenizeMode::Reverse => {
                if term_length > 1 {
                    for x in (1..term_length).rev() {
                        let token = &term[if rtl { term_length - 1 - x } else { x }..];
                        let partial_score =
                            index.get_score(resolution, word_length, i, Some(term_length), Some(x));
                        index.push_index(&mut dupes, token, partial_score, id, append, None);
                    }
                }
                if depth > 0 {
                    let ctx = IndexContext {
                        index,
                        dupes: &mut dupes_ctx,
                        term,
                        score,
                        id,
                        append,
                    };
                    let params = ContextParams {
                        i,
                        depth,
                        rtl,
                        encoded: &encoded,
                        word_length,
                    };
                    add_context(ctx, &params);
                }
            }
            TokenizeMode::Forward => {
                if term_length > 1 {
                    add_forward(index, &mut dupes, term, score, id, append, rtl);
                } else {
                    index.push_index(&mut dupes, term, score, id, append, None);
                }
                if depth > 0 {
                    let ctx = IndexContext {
                        index,
                        dupes: &mut dupes_ctx,
                        term,
                        score,
                        id,
                        append,
                    };
                    let params = ContextParams {
                        i,
                        depth,
                        rtl,
                        encoded: &encoded,
                        word_length,
                    };
                    add_context(ctx, &params);
                }
            }
            TokenizeMode::Strict => {
                let ctx = IndexContext {
                    index,
                    dupes: &mut dupes,
                    term,
                    score,
                    id,
                    append,
                };
                let params = ContextParams {
                    i,
                    depth,
                    rtl,
                    encoded: &encoded,
                    word_length,
                };
                add_strict(ctx, &params);
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

fn add_strict(ctx: IndexContext<'_>, params: &ContextParams<'_>) {
    ctx.index
        .push_index(ctx.dupes, ctx.term, ctx.score, ctx.id, ctx.append, None);

    if params.depth > 0 && params.word_length > 1 && params.i < params.word_length - 1 {
        add_context(ctx, params);
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
            eprintln!(
                "Index builder: Character index {} out of bounds for term '{}'",
                char_idx, term
            );
            continue;
        }
        index.push_index(dupes, &token, score, id, append, None);
    }
}

fn add_context(ctx: IndexContext<'_>, params: &ContextParams<'_>) {
    let mut dupes_inner = HashMap::new();
    let resolution = ctx.index.resolution_ctx;
    let keyword = ctx.term;
    let size = params.depth.min(if params.rtl {
        params.i + 1
    } else {
        params.word_length - params.i
    });

    dupes_inner.insert(keyword.to_string(), true);

    for x in 1..size {
        let term_idx = if params.rtl {
            params.word_length - 1 - params.i - x
        } else {
            params.i + x
        };

        if term_idx >= params.word_length {
            break;
        }

        let context_term = &params.encoded[term_idx];

        if !context_term.is_empty() && !dupes_inner.contains_key(context_term) {
            dupes_inner.insert(context_term.to_string(), true);

            let context_score = ctx.index.get_score(
                resolution
                    + if params.word_length / 2 > resolution {
                        0
                    } else {
                        1
                    },
                params.word_length,
                params.i,
                Some(size - 1),
                Some(x - 1),
            );

            let swap = ctx.index.bidirectional && **context_term > *keyword;
            let (ctx_term, ctx_keyword) = if swap {
                (keyword, &context_term[..])
            } else {
                (&context_term[..], keyword)
            };

            ctx.index.push_index(
                ctx.dupes,
                ctx_term,
                context_score,
                ctx.id,
                ctx.append,
                Some(ctx_keyword),
            );
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

        ((resolution - 1) as f64 / total_length as f64 * (i + offset) as f64 + 1.0) as usize
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
        add_document(&mut index, 1, "hello world", false, false)
            .expect("add_document should succeed");
        assert!(index.contains(1));
    }

    #[test]
    fn test_add_document_empty() {
        let mut index = Index::default();
        add_document(&mut index, 1, "", false, false)
            .expect("add_document with empty content should succeed");
        assert!(!index.contains(1));
    }

    #[test]
    fn test_add_document_append() {
        let mut index = Index::default();
        add_document(&mut index, 1, "hello", false, false).expect("add_document should succeed");
        add_document(&mut index, 1, "world", true, false)
            .expect("add_document append should succeed");
        assert!(index.contains(1));
    }
}
