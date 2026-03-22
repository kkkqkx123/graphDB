use crate::encoder::Encoder;
use crate::error::Result;

pub struct MatchResult {
    pub found: bool,
    pub match_str: String,
    pub match_length: usize,
}

pub fn find_best_match(
    doc_org_cur: &str,
    doc_enc_cur: &str,
    query_enc: &[String],
    markup_open: &str,
    markup_close: &str,
) -> MatchResult {
    if doc_enc_cur.is_empty() || doc_org_cur.is_empty() {
        return MatchResult {
            found: false,
            match_str: String::new(),
            match_length: 0,
        };
    }

    let doc_org_cur_len = doc_org_cur.chars().count();
    let doc_org_diff = if doc_enc_cur.len() < doc_org_cur.len() {
        doc_org_cur.chars().count() - doc_enc_cur.chars().count()
    } else {
        0
    };

    let mut best_match_str = String::new();
    let mut best_match_length = 0;
    let mut found = false;

    for query_term in query_enc {
        if query_term.is_empty() {
            continue;
        }

        let mut query_term_len = query_term.chars().count();
        // Add length from shrinking phonetic transformations
        query_term_len += doc_org_diff;

        // Skip query token when match length can't exceed previously highest found match
        if best_match_length > 0 && query_term_len <= best_match_length {
            continue;
        }

        if let Some(position) = doc_enc_cur.find(query_term) {
            // Convert byte position to char position for Unicode safety
            let char_position = doc_enc_cur[..position].chars().count();
            let query_char_len = query_term.chars().count();

            // Extract prefix, match content, and suffix using char indices
            let mut chars = doc_org_cur.chars();
            let prefix: String = chars.by_ref().take(char_position).collect();
            let match_content: String = chars.by_ref().take(query_char_len).collect();
            let suffix: String = chars.collect();

            best_match_str = format!(
                "{}{}{}{}{}",
                prefix,
                markup_open,
                match_content,
                markup_close,
                suffix
            );
            best_match_length = query_char_len;
            found = true;
        }
    }

    MatchResult {
        found,
        match_str: best_match_str,
        match_length: best_match_length,
    }
}

pub fn encode_and_join(text: &str, encoder: &Encoder) -> Result<String> {
    let encoded = encoder.encode(text)?;
    if encoded.len() > 1 {
        Ok(encoded.join(" "))
    } else if !encoded.is_empty() {
        Ok(encoded[0].clone())
    } else {
        Ok(String::new())
    }
}