use crate::encoder::Encoder;
use crate::tokenizer::Tokenizer;

/// Lightweight text tokenizer trait for engine-level pluggability.
///
/// This allows the search engine to accept different tokenization strategies
/// without depending on the full Encoder pipeline.
pub trait TextTokenizer: Send + Sync {
    /// Tokenize text into a list of tokens.
    fn tokenize(&self, text: &str) -> Vec<String>;

    /// Tokenize text with position information.
    fn tokenize_with_positions(&self, text: &str) -> Vec<(String, usize)>;
}

impl TextTokenizer for Encoder {
    fn tokenize(&self, text: &str) -> Vec<String> {
        self.encode(text).unwrap_or_default()
    }

    fn tokenize_with_positions(&self, text: &str) -> Vec<(String, usize)> {
        self.encode(text)
            .unwrap_or_default()
            .into_iter()
            .enumerate()
            .map(|(pos, token)| (token, pos))
            .collect()
    }
}

impl TextTokenizer for Tokenizer {
    fn tokenize(&self, text: &str) -> Vec<String> {
        self.tokenize(text)
    }

    fn tokenize_with_positions(&self, text: &str) -> Vec<(String, usize)> {
        self.tokenize_with_positions(text)
    }
}
