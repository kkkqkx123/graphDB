use jieba_rs::Jieba;
use tantivy::tokenizer::{Token, TokenStream, Tokenizer};

pub struct MixedTokenizer {
    jieba: Jieba,
}

impl MixedTokenizer {
    pub fn new() -> Self {
        Self {
            jieba: Jieba::new(),
        }
    }

    fn is_cjk(c: char) -> bool {
        matches!(
            c,
            '\u{4E00}'..='\u{9FFF}'
            | '\u{3400}'..='\u{4DBF}'
            | '\u{20000}'..='\u{2A6DF}'
            | '\u{2A700}'..='\u{2B73F}'
            | '\u{2B740}'..='\u{2B81F}'
            | '\u{2B820}'..='\u{2CEAF}'
            | '\u{F900}'..='\u{FAFF}'
            | '\u{2F800}'..='\u{2FA1F}'
        )
    }

    pub fn tokenize_text(&self, text: &str) -> Vec<(String, usize, usize)> {
        let mut tokens = Vec::new();
        let mut ascii_start: Option<usize> = None;
        let mut ascii_buffer = String::new();
        let char_indices: Vec<(usize, char)> = text.char_indices().collect();

        let mut i = 0;
        while i < char_indices.len() {
            let (byte_idx, c) = char_indices[i];

            if Self::is_cjk(c) {
                if !ascii_buffer.is_empty() {
                    Self::tokenize_english_segment(
                        &ascii_buffer,
                        ascii_start.expect("ascii_start must be set"),
                        &mut tokens,
                    );
                    ascii_buffer.clear();
                    ascii_start = None;
                }

                let cjk_start = byte_idx;
                let mut cjk_text = String::new();
                cjk_text.push(c);

                i += 1;
                while i < char_indices.len() {
                    let (_, next_c) = char_indices[i];
                    if Self::is_cjk(next_c) {
                        cjk_text.push(next_c);
                        i += 1;
                    } else {
                        break;
                    }
                }

                let cjk_tokens = self.jieba.cut(&cjk_text, true);
                let mut offset = cjk_start;
                for token_str in cjk_tokens {
                    let token_len = token_str.len();
                    if !token_str.trim().is_empty() {
                        tokens.push((token_str.to_string(), offset, offset + token_len));
                    }
                    offset += token_len;
                }
            } else {
                if ascii_start.is_none() {
                    ascii_start = Some(byte_idx);
                }
                ascii_buffer.push(c);
                i += 1;
            }
        }

        if !ascii_buffer.is_empty() {
            Self::tokenize_english_segment(
                &ascii_buffer,
                ascii_start.expect("ascii_start must be set"),
                &mut tokens,
            );
        }

        tokens
    }

    fn tokenize_english_segment(
        text: &str,
        start_offset: usize,
        tokens: &mut Vec<(String, usize, usize)>,
    ) {
        for word in text.split(|c: char| !c.is_alphanumeric()) {
            let word_stripped = word.trim();
            if word_stripped.is_empty() {
                continue;
            }

            let word_start_in_segment = text.find(word_stripped).unwrap_or(0);
            let actual_start = start_offset + word_start_in_segment;

            let lower = word_stripped.to_lowercase();
            if lower.len() >= 2 {
                tokens.push((lower, actual_start, actual_start + word_stripped.len()));
            }
        }
    }
}

impl Default for MixedTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for MixedTokenizer {
    fn clone(&self) -> Self {
        Self {
            jieba: Jieba::new(),
        }
    }
}

impl Tokenizer for MixedTokenizer {
    type TokenStream<'a> = MixedTokenStream<'a>;

    fn token_stream<'a>(&'a mut self, text: &'a str) -> Self::TokenStream<'a> {
        let raw_tokens = self.tokenize_text(text);
        MixedTokenStream::new(raw_tokens)
    }
}

pub struct MixedTokenStream<'a> {
    tokens: Vec<(String, usize, usize)>,
    current: usize,
    token: Token,
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> MixedTokenStream<'a> {
    fn new(tokens: Vec<(String, usize, usize)>) -> Self {
        Self {
            tokens,
            current: 0,
            token: Token::default(),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl TokenStream for MixedTokenStream<'_> {
    fn advance(&mut self) -> bool {
        if self.current < self.tokens.len() {
            let (ref text, offset_from, offset_to) = self.tokens[self.current];
            self.token.offset_from = offset_from;
            self.token.offset_to = offset_to;
            self.token.position = self.current;
            self.token.position_length = 1;
            self.token.text.clear();
            self.token.text.push_str(text);
            self.current += 1;
            true
        } else {
            false
        }
    }

    fn token(&self) -> &Token {
        &self.token
    }

    fn token_mut(&mut self) -> &mut Token {
        &mut self.token
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chinese_tokenization() {
        let tokenizer = MixedTokenizer::new();
        let tokens = tokenizer.tokenize_text("计算总价的方法");
        let texts: Vec<&str> = tokens.iter().map(|(t, _, _)| t.as_str()).collect();
        assert!(texts.contains(&"计算"), "Expected '计算' in {:?}", texts);
        assert!(texts.contains(&"总价"), "Expected '总价' in {:?}", texts);
        assert!(texts.contains(&"方法"), "Expected '方法' in {:?}", texts);
    }

    #[test]
    fn test_english_tokenization() {
        let tokenizer = MixedTokenizer::new();
        let tokens = tokenizer.tokenize_text("Calculate total price");
        let texts: Vec<&str> = tokens.iter().map(|(t, _, _)| t.as_str()).collect();
        assert!(
            texts.contains(&"calculate"),
            "Expected 'calculate' in {:?}",
            texts
        );
        assert!(texts.contains(&"total"), "Expected 'total' in {:?}", texts);
        assert!(texts.contains(&"price"), "Expected 'price' in {:?}", texts);
    }

    #[test]
    fn test_mixed_tokenization() {
        let tokenizer = MixedTokenizer::new();
        let tokens = tokenizer.tokenize_text("计算 total price");
        let texts: Vec<&str> = tokens.iter().map(|(t, _, _)| t.as_str()).collect();
        assert!(texts.contains(&"计算"), "Expected '计算' in {:?}", texts);
        assert!(texts.contains(&"total"), "Expected 'total' in {:?}", texts);
        assert!(texts.contains(&"price"), "Expected 'price' in {:?}", texts);
    }

    #[test]
    fn test_complex_comment() {
        let tokenizer = MixedTokenizer::new();
        let text = "计算总价 Calculate total price";
        let tokens = tokenizer.tokenize_text(text);
        let texts: Vec<&str> = tokens.iter().map(|(t, _, _)| t.as_str()).collect();

        assert!(texts.contains(&"计算"), "Expected '计算' in {:?}", texts);
        assert!(texts.contains(&"总价"), "Expected '总价' in {:?}", texts);
        assert!(
            texts.contains(&"calculate"),
            "Expected 'calculate' in {:?}",
            texts
        );
        assert!(texts.contains(&"total"), "Expected 'total' in {:?}", texts);
        assert!(texts.contains(&"price"), "Expected 'price' in {:?}", texts);
    }

    #[test]
    fn test_empty_text() {
        let tokenizer = MixedTokenizer::new();
        let tokens = tokenizer.tokenize_text("");
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_short_english_skipped() {
        let tokenizer = MixedTokenizer::new();
        let tokens = tokenizer.tokenize_text("a b cd");
        let texts: Vec<&str> = tokens.iter().map(|(t, _, _)| t.as_str()).collect();
        assert!(!texts.contains(&"a"), "Single char 'a' should be skipped");
        assert!(!texts.contains(&"b"), "Single char 'b' should be skipped");
        assert!(texts.contains(&"cd"), "Expected 'cd' in {:?}", texts);
    }

    #[test]
    fn test_tokenizer_trait_integration() {
        use tantivy::tokenizer::Tokenizer;

        let mut tokenizer = MixedTokenizer::new();
        let mut stream = tokenizer.token_stream("Hello, world.");

        let mut results = Vec::new();
        while stream.advance() {
            results.push(stream.token().text.clone());
        }

        assert!(
            results.contains(&"hello".to_string()),
            "Expected 'hello' in {:?}",
            results
        );
        assert!(
            results.contains(&"world".to_string()),
            "Expected 'world' in {:?}",
            results
        );
    }
}
