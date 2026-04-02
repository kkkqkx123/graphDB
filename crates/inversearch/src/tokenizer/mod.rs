use crate::r#type::EncoderOptions;
use crate::encoder::Encoder;
use crate::error::Result;

pub struct Tokenizer {
    pub encoder: Encoder,
    pub mode: TokenizerMode,
}

pub enum TokenizerMode {
    Strict,
    Forward,
    Reverse,
    Full,
    Ngram(usize),
}

impl Tokenizer {
    pub fn new(options: EncoderOptions) -> Result<Self> {
        let encoder = Encoder::new(options)?;
        Ok(Tokenizer {
            encoder,
            mode: TokenizerMode::Strict,
        })
    }

    pub fn set_mode(&mut self, mode: TokenizerMode) {
        self.mode = mode;
    }

    pub fn tokenize(&self, text: &str) -> Vec<String> {
        let tokens = self.encoder.encode(text).unwrap_or_default();

        match &self.mode {
            TokenizerMode::Strict => tokens,
            TokenizerMode::Forward => self.tokenize_forward(&tokens),
            TokenizerMode::Reverse => self.tokenize_reverse(&tokens),
            TokenizerMode::Full => self.tokenize_full(&tokens),
            TokenizerMode::Ngram(n) => self.tokenize_ngram(&tokens, *n),
        }
    }

    fn tokenize_forward(&self, tokens: &[String]) -> Vec<String> {
        let mut result = Vec::new();

        for i in 0..tokens.len() {
            for j in (i + 1)..=tokens.len() {
                let phrase = tokens[i..j].join(" ");
                result.push(phrase);
            }
        }

        result
    }

    fn tokenize_reverse(&self, tokens: &[String]) -> Vec<String> {
        let mut result = Vec::new();

        for i in (0..tokens.len()).rev() {
            for j in (0..=i).rev() {
                let phrase = tokens[j..=i].join(" ");
                result.push(phrase);
            }
        }

        result
    }

    fn tokenize_full(&self, tokens: &[String]) -> Vec<String> {
        let mut result = Vec::new();

        for i in 0..tokens.len() {
            for j in (i + 1)..=tokens.len() {
                let phrase = tokens[i..j].join(" ");
                result.push(phrase);
            }

            for j in (0..=i).rev() {
                let phrase = tokens[j..=i].join(" ");
                result.push(phrase);
            }
        }

        result
    }

    fn tokenize_ngram(&self, tokens: &[String], n: usize) -> Vec<String> {
        let mut result = Vec::new();

        if n == 0 || tokens.is_empty() {
            return result;
        }

        for window in tokens.windows(n) {
            let phrase = window.join(" ");
            result.push(phrase);
        }

        result
    }

    pub fn tokenize_with_positions(&self, text: &str) -> Vec<(String, usize)> {
        let tokens = self.encoder.encode(text).unwrap_or_default();
        let mut result = Vec::new();
        let mut position = 0;

        for token in &tokens {
            result.push((token.clone(), position));
            position += 1;
        }

        result
    }

    pub fn tokenize_with_offsets(&self, text: &str) -> Vec<(String, usize, usize)> {
        let tokens = self.encoder.encode(text).unwrap_or_default();
        let mut result = Vec::new();
        let mut offset = 0;

        for token in &tokens {
            let start = offset;
            let end = offset + token.len();
            result.push((token.clone(), start, end));
            offset = end + 1;
        }

        result
    }
}

impl Default for Tokenizer {
    fn default() -> Self {
        Tokenizer::new(EncoderOptions::default()).expect("Default tokenizer creation should succeed")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenizer_strict() {
        let tokenizer = Tokenizer::default();
        let result = tokenizer.tokenize("hello world");
        assert_eq!(result, vec!["hello", "world"]);
    }

    #[test]
    fn test_tokenizer_forward() {
        let mut tokenizer = Tokenizer::default();
        tokenizer.set_mode(TokenizerMode::Forward);
        let result = tokenizer.tokenize("hello world");
        assert!(result.contains(&"hello".to_string()));
        assert!(result.contains(&"world".to_string()));
        assert!(result.contains(&"hello world".to_string()));
    }

    #[test]
    fn test_tokenizer_reverse() {
        let mut tokenizer = Tokenizer::default();
        tokenizer.set_mode(TokenizerMode::Reverse);
        let result = tokenizer.tokenize("hello world");
        assert!(result.contains(&"hello".to_string()));
        assert!(result.contains(&"world".to_string()));
        assert!(result.contains(&"hello world".to_string()));
    }

    #[test]
    fn test_tokenizer_full() {
        let mut tokenizer = Tokenizer::default();
        tokenizer.set_mode(TokenizerMode::Full);
        let result = tokenizer.tokenize("hello world");
        assert!(result.contains(&"hello".to_string()));
        assert!(result.contains(&"world".to_string()));
        assert!(result.contains(&"hello world".to_string()));
    }

    #[test]
    fn test_tokenizer_ngram() {
        let mut tokenizer = Tokenizer::default();
        tokenizer.set_mode(TokenizerMode::Ngram(2));
        let result = tokenizer.tokenize("hello world test");
        assert!(result.contains(&"hello world".to_string()));
        assert!(result.contains(&"world test".to_string()));
    }

    #[test]
    fn test_tokenize_with_positions() {
        let tokenizer = Tokenizer::default();
        let result = tokenizer.tokenize_with_positions("hello world");
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], ("hello".to_string(), 0));
        assert_eq!(result[1], ("world".to_string(), 1));
    }

    #[test]
    fn test_tokenize_with_offsets() {
        let tokenizer = Tokenizer::default();
        let result = tokenizer.tokenize_with_offsets("hello world");
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, "hello");
        assert_eq!(result[1].0, "world");
    }
}
