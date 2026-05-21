use std::sync::Arc;

use jieba_rs::{Jieba, TokenizeMode};
use tokenizer_api::{Token, TokenStream, Tokenizer};

/// Tokenizer for Chinese text that uses jieba-rs for word segmentation.
///
/// Uses `TokenizeMode::Search` which provides finer-grained segmentation
/// suitable for search indexing.
#[derive(Clone)]
pub struct JiebaTokenizer {
    jieba: Arc<Jieba>,
}

impl Default for JiebaTokenizer {
    fn default() -> Self {
        Self {
            jieba: Arc::new(Jieba::new()),
        }
    }
}

impl Tokenizer for JiebaTokenizer {
    type TokenStream<'a> = JiebaTokenStream<'a>;

    fn token_stream<'a>(&'a mut self, text: &'a str) -> Self::TokenStream<'a> {
        let tokens = self.jieba.tokenize(text, TokenizeMode::Search, false);
        JiebaTokenStream {
            tokens,
            text,
            index: 0,
            token: Token::default(),
        }
    }
}

pub struct JiebaTokenStream<'a> {
    tokens: Vec<jieba_rs::Token<'a>>,
    text: &'a str,
    index: usize,
    token: Token,
}

impl<'a> TokenStream for JiebaTokenStream<'a> {
    fn advance(&mut self) -> bool {
        if self.index >= self.tokens.len() {
            return false;
        }
        let tok = &self.tokens[self.index];
        self.token.offset_from = tok.start;
        self.token.offset_to = tok.end;
        self.token.position = self.index + 1;
        self.token.text = self.text[tok.start..tok.end].to_string();
        self.token.position_length = 1;
        self.index += 1;
        true
    }

    fn token(&self) -> &Token {
        &self.token
    }

    fn token_mut(&mut self) -> &mut Token {
        &mut self.token
    }
}
