//! Response parsing strategies for embedder
//!
//! Provides different response parsing strategies to support various API formats
//! including standard OpenAI format and BGE-M3 multi-modal format.

use serde::de::{self, Visitor};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

use crate::llm::LlmError;

/// Token usage information
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct TokenUsage {
    /// Prompt tokens used
    pub prompt_tokens: u64,
    /// Total tokens used
    pub total_tokens: u64,
}

/// Standard embedding response data
#[derive(Debug, Clone, Deserialize)]
pub struct StandardEmbeddingData {
    /// Embedding vector
    #[serde(deserialize_with = "deserialize_embedding_value")]
    pub embedding: Vec<f32>,
    /// Index in the input batch (used for ordering to ensure correct correspondence)
    pub index: usize,
}

/// Deserialize embedding value that can be either a float array or base64 string
fn deserialize_embedding_value<'de, D>(deserializer: D) -> Result<Vec<f32>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    use std::fmt;

    struct EmbeddingVisitor;

    impl<'de> Visitor<'de> for EmbeddingVisitor {
        type Value = Vec<f32>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a float array or base64 string")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            // Decode base64
            decode_base64_embedding(value)
                .map_err(|_| E::custom("failed to decode base64 embedding"))
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: de::SeqAccess<'de>,
        {
            let mut vec = Vec::new();
            while let Some(value) = seq.next_element()? {
                vec.push(value);
            }
            Ok(vec)
        }
    }

    deserializer.deserialize_any(EmbeddingVisitor)
}

/// Standard OpenAI-compatible embedding response
#[derive(Debug, Clone, Deserialize)]
pub struct StandardEmbeddingResponse {
    pub data: Vec<StandardEmbeddingData>,
    #[serde(default)]
    pub usage: Option<TokenUsage>,
}

/// BGE-M3 return mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BGEM3Mode {
    /// Only dense vectors (default)
    #[default]
    Dense,
    /// Only sparse vectors (lexical weights)
    Sparse,
    /// Only ColBERT vectors
    Colbert,
    /// All formats
    All,
}

impl BGEM3Mode {
    /// Check if dense vectors are enabled
    pub fn has_dense(&self) -> bool {
        matches!(self, Self::Dense | Self::All)
    }

    /// Check if sparse vectors are enabled
    pub fn has_sparse(&self) -> bool {
        matches!(self, Self::Sparse | Self::All)
    }

    /// Check if ColBERT vectors are enabled
    pub fn has_colbert(&self) -> bool {
        matches!(self, Self::Colbert | Self::All)
    }
}

/// BGE-M3 embedding response data
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct BGEM3EmbeddingData {
    /// Dense embedding vector
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_optional_embedding")]
    pub dense_vecs: Option<Vec<f32>>,

    /// Sparse embedding (token -> weight)
    #[serde(default)]
    pub lexical_weights: Option<HashMap<String, f32>>,

    /// ColBERT vectors (multi-vector representation)
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_optional_colbert")]
    pub colbert_vecs: Option<Vec<Vec<f32>>>,

    /// Index in the input batch
    pub index: usize,
}

/// Deserialize optional embedding value
fn deserialize_optional_embedding<'de, D>(deserializer: D) -> Result<Option<Vec<f32>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    use std::fmt;

    struct OptionalEmbeddingVisitor;

    impl<'de> Visitor<'de> for OptionalEmbeddingVisitor {
        type Value = Option<Vec<f32>>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a float array, base64 string, or null")
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }

        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserialize_embedding_value(deserializer).map(Some)
        }

        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }
    }

    deserializer.deserialize_option(OptionalEmbeddingVisitor)
}

/// Deserialize optional ColBERT vectors
fn deserialize_optional_colbert<'de, D>(deserializer: D) -> Result<Option<Vec<Vec<f32>>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    use std::fmt;

    struct OptionalColbertVisitor;

    impl<'de> Visitor<'de> for OptionalColbertVisitor {
        type Value = Option<Vec<Vec<f32>>>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a 2D float array or null")
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }

        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_seq(ColbertVisitor).map(Some)
        }

        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }
    }

    deserializer.deserialize_option(OptionalColbertVisitor)
}

/// BGE-M3 embedding response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BGEM3EmbeddingResponse {
    pub data: Vec<BGEM3EmbeddingData>,
    #[serde(default)]
    pub usage: Option<TokenUsage>,
}

/// Response parser strategy
///
/// Uses enum-based dispatch for different response formats.
/// This provides compile-time type safety while keeping the code centralized.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ResponseParser {
    /// Standard OpenAI-compatible format
    #[default]
    Standard,
    /// BGE-M3 multi-modal format
    BGEM3(BGEM3Mode),
}

impl ResponseParser {
    /// Parse response string according to the selected strategy
    pub fn parse(&self, response_body: &str) -> Result<ParsedResponse, LlmError> {
        match self {
            Self::Standard => Self::parse_standard(response_body),
            Self::BGEM3(mode) => Self::parse_bge_m3(response_body, *mode),
        }
    }

    /// Parse standard OpenAI response
    fn parse_standard(response_body: &str) -> Result<ParsedResponse, LlmError> {
        let response: StandardEmbeddingResponse =
            serde_json::from_str(response_body).map_err(|e| {
                LlmError::invalid_response(format!("Failed to parse standard response: {}", e))
            })?;

        // Sort by index to ensure correct ordering
        // This handles cases where API returns data in non-sequential order
        let mut data = response.data;
        data.sort_by_key(|d| d.index);

        let embeddings: Vec<Vec<f32>> = data.into_iter().map(|d| d.embedding).collect();

        Ok(ParsedResponse {
            embeddings,
            sparse_embeddings: Vec::new(),
            colbert_embeddings: Vec::new(),
            usage: response.usage.unwrap_or_default(),
        })
    }

    /// Parse BGE-M3 response
    fn parse_bge_m3(response_body: &str, mode: BGEM3Mode) -> Result<ParsedResponse, LlmError> {
        let response: BGEM3EmbeddingResponse =
            serde_json::from_str(response_body).map_err(|e| {
                LlmError::invalid_response(format!("Failed to parse BGE-M3 response: {}", e))
            })?;

        let mut embeddings = Vec::new();
        let mut sparse_embeddings = Vec::new();
        let mut colbert_embeddings = Vec::new();

        for data in response.data {
            if mode.has_dense() {
                if let Some(dense) = data.dense_vecs {
                    embeddings.push(dense);
                }
            }

            if mode.has_sparse() {
                if let Some(sparse) = data.lexical_weights {
                    sparse_embeddings.push(sparse);
                }
            }

            if mode.has_colbert() {
                if let Some(colbert) = data.colbert_vecs {
                    colbert_embeddings.push(colbert);
                }
            }
        }

        Ok(ParsedResponse {
            embeddings,
            sparse_embeddings,
            colbert_embeddings,
            usage: response.usage.unwrap_or_default(),
        })
    }
}

/// Parsed embedding response
#[derive(Debug, Clone, Default)]
pub struct ParsedResponse {
    /// Dense embedding vectors (standard format)
    pub embeddings: Vec<Vec<f32>>,
    /// Sparse embedding vectors (token -> weight)
    pub sparse_embeddings: Vec<HashMap<String, f32>>,
    /// ColBERT multi-vectors
    pub colbert_embeddings: Vec<Vec<Vec<f32>>>,
    /// Token usage information
    pub usage: TokenUsage,
}

impl ParsedResponse {
    /// Check if this response contains any embeddings
    pub fn is_empty(&self) -> bool {
        self.embeddings.is_empty()
            && self.sparse_embeddings.is_empty()
            && self.colbert_embeddings.is_empty()
    }

    /// Get the number of items in the response
    pub fn len(&self) -> usize {
        // All three vectors should have the same length
        self.embeddings
            .len()
            .max(self.sparse_embeddings.len())
            .max(self.colbert_embeddings.len())
    }
}

struct ColbertVisitor;

impl<'de> Visitor<'de> for ColbertVisitor {
    type Value = Vec<Vec<f32>>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a 2D float array")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Vec<Vec<f32>>, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        let mut result = Vec::new();
        while let Some(inner) = seq.next_element::<Vec<f32>>()? {
            result.push(inner);
        }
        Ok(result)
    }
}

/// Decode base64 embedding to float vector
fn decode_base64_embedding(input: &str) -> Result<Vec<f32>, LlmError> {
    // Simple base64 decode (no external dependency)
    let bytes = decode_base64(input)?;

    // Convert bytes to f32 array (little-endian)
    let count = bytes.len() / 4;
    let mut result = Vec::with_capacity(count);

    for chunk in bytes.chunks_exact(4) {
        let arr: [u8; 4] = chunk
            .try_into()
            .map_err(|_| LlmError::invalid_response("Invalid base64 chunk".to_string()))?;
        result.push(f32::from_le_bytes(arr));
    }

    Ok(result)
}

/// Decode base64 string to bytes
fn decode_base64(input: &str) -> Result<Vec<u8>, LlmError> {
    // Base64 alphabet
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    // Build decode table
    let mut decode_table = [0xFFu8; 256];
    for (i, &c) in ALPHABET.iter().enumerate() {
        decode_table[c as usize] = i as u8;
    }

    let input_bytes = input.as_bytes();
    let padding = input_bytes.iter().rev().take_while(|&&c| c == b'=').count();

    let output_len = (input_bytes.len() * 3) / 4 - padding;
    let mut result = Vec::with_capacity(output_len);

    let mut buffer = 0u32;
    let mut bits = 0;

    for &c in input_bytes {
        if c == b'=' {
            break;
        }

        let val = decode_table[c as usize];
        if val == 0xFF {
            return Err(LlmError::invalid_response(format!(
                "Invalid base64 character: {}",
                c as char
            )));
        }

        buffer = (buffer << 6) | (val as u32);
        bits += 6;

        if bits >= 8 {
            bits -= 8;
            result.push((buffer >> bits) as u8);
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bgem3_mode() {
        assert!(BGEM3Mode::Dense.has_dense());
        assert!(!BGEM3Mode::Dense.has_sparse());

        assert!(BGEM3Mode::Sparse.has_sparse());
        assert!(!BGEM3Mode::Sparse.has_dense());

        assert!(BGEM3Mode::All.has_dense());
        assert!(BGEM3Mode::All.has_sparse());
        assert!(BGEM3Mode::All.has_colbert());
    }

    #[test]
    fn test_parse_standard_response() {
        let json = r#"{
            "data": [
                {"embedding": [0.1, 0.2, 0.3], "index": 0},
                {"embedding": [0.4, 0.5, 0.6], "index": 1}
            ],
            "usage": {"prompt_tokens": 10, "total_tokens": 10}
        }"#;

        let parser = ResponseParser::Standard;
        let result = parser.parse(json).expect("parse failed");

        assert_eq!(result.embeddings.len(), 2);
        assert_eq!(result.embeddings[0], vec![0.1, 0.2, 0.3]);
        assert_eq!(result.usage.prompt_tokens, 10);
    }

    #[test]
    fn test_parse_standard_response_unordered() {
        // Test that response with out-of-order index is correctly sorted
        let json = r#"{
            "data": [
                {"embedding": [0.4, 0.5, 0.6], "index": 1},
                {"embedding": [0.1, 0.2, 0.3], "index": 0},
                {"embedding": [0.7, 0.8, 0.9], "index": 2}
            ],
            "usage": {"prompt_tokens": 15, "total_tokens": 15}
        }"#;

        let parser = ResponseParser::Standard;
        let result = parser.parse(json).expect("parse failed");

        assert_eq!(result.embeddings.len(), 3);
        // Verify embeddings are sorted by index
        assert_eq!(result.embeddings[0], vec![0.1, 0.2, 0.3]); // index 0
        assert_eq!(result.embeddings[1], vec![0.4, 0.5, 0.6]); // index 1
        assert_eq!(result.embeddings[2], vec![0.7, 0.8, 0.9]); // index 2
    }

    #[test]
    fn test_parse_bge_m3_dense_response() {
        let json = r#"{
            "data": [
                {"dense_vecs": [0.1, 0.2, 0.3], "index": 0}
            ],
            "usage": {"prompt_tokens": 5, "total_tokens": 5}
        }"#;

        let parser = ResponseParser::BGEM3(BGEM3Mode::Dense);
        let result = parser.parse(json).expect("parse failed");

        assert_eq!(result.embeddings.len(), 1);
        assert_eq!(result.sparse_embeddings.len(), 0);
    }

    #[test]
    fn test_parse_bge_m3_sparse_response() {
        let json = r#"{
            "data": [
                {
                    "lexical_weights": {"hello": 0.5, "world": 0.3},
                    "index": 0
                }
            ],
            "usage": {"prompt_tokens": 5, "total_tokens": 5}
        }"#;

        let parser = ResponseParser::BGEM3(BGEM3Mode::Sparse);
        let result = parser.parse(json).expect("parse failed");

        assert_eq!(result.sparse_embeddings.len(), 1);
        assert!(result.sparse_embeddings[0].contains_key("hello"));
    }

    #[test]
    fn test_parsed_response_is_empty() {
        let empty = ParsedResponse::default();
        assert!(empty.is_empty());

        let with_data = ParsedResponse {
            embeddings: vec![vec![0.1, 0.2]],
            ..Default::default()
        };
        assert!(!with_data.is_empty());
    }
}
