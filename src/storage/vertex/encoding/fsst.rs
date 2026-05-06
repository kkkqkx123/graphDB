//! FSST (Fast Static Symbol Table) String Compression
//!
//! A fast string compression technique using a static symbol table.
//! Effective for long strings and high-cardinality scenarios where
//! dictionary encoding is less effective.
//!
//! # Algorithm
//!
//! 1. Analyze input strings to find frequent byte sequences (1-8 bytes)
//! 2. Build a symbol table mapping frequent sequences to single-byte codes
//! 3. Encode strings using the symbol table
//! 4. Decoding is a simple table lookup - very fast

use std::collections::HashMap;

const MAX_SYMBOL_LEN: usize = 8;
const SYMBOL_TABLE_SIZE: usize = 255;

#[derive(Debug, Clone)]
pub struct FsstSymbol {
    bytes: Vec<u8>,
    code: u8,
    frequency: usize,
}

impl FsstSymbol {
    pub fn new(bytes: Vec<u8>, code: u8) -> Self {
        Self {
            bytes,
            code,
            frequency: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct FsstSymbolTable {
    symbols: Vec<Option<FsstSymbol>>,
    code_to_symbol: HashMap<u8, Vec<u8>>,
    byte_to_codes: HashMap<Vec<u8>, u8>,
}

impl FsstSymbolTable {
    pub fn new() -> Self {
        Self {
            symbols: vec![None; SYMBOL_TABLE_SIZE + 1],
            code_to_symbol: HashMap::new(),
            byte_to_codes: HashMap::new(),
        }
    }

    pub fn insert(&mut self, bytes: Vec<u8>, code: u8) {
        let symbol = FsstSymbol::new(bytes.clone(), code);
        self.code_to_symbol.insert(code, bytes.clone());
        self.byte_to_codes.insert(bytes, code);
        self.symbols[code as usize] = Some(symbol);
    }

    pub fn get_by_code(&self, code: u8) -> Option<&Vec<u8>> {
        self.code_to_symbol.get(&code)
    }

    pub fn get_by_bytes(&self, bytes: &[u8]) -> Option<u8> {
        self.byte_to_codes.get(bytes).copied()
    }

    pub fn len(&self) -> usize {
        self.code_to_symbol.len()
    }

    pub fn is_empty(&self) -> bool {
        self.code_to_symbol.is_empty()
    }

    pub fn memory_usage(&self) -> usize {
        self.code_to_symbol.values().map(|v| v.len()).sum::<usize>()
            + self.byte_to_codes.keys().map(|k| k.len()).sum::<usize>()
            + std::mem::size_of::<Self>()
    }
}

impl Default for FsstSymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct FsstEncoder {
    table: FsstSymbolTable,
    encoded_count: usize,
    total_input_bytes: usize,
    total_output_bytes: usize,
}

impl FsstEncoder {
    pub fn new() -> Self {
        Self {
            table: FsstSymbolTable::new(),
            encoded_count: 0,
            total_input_bytes: 0,
            total_output_bytes: 0,
        }
    }

    pub fn train(strings: &[&str], max_symbols: usize) -> Self {
        let mut encoder = Self::new();
        encoder.build_symbol_table(strings, max_symbols);
        encoder
    }

    fn build_symbol_table(&mut self, strings: &[&str], max_symbols: usize) {
        let mut ngram_freq: HashMap<Vec<u8>, usize> = HashMap::new();

        for s in strings {
            let bytes = s.as_bytes();
            for len in 1..=MAX_SYMBOL_LEN.min(bytes.len()) {
                for i in 0..=bytes.len() - len {
                    let ngram: Vec<u8> = bytes[i..i + len].to_vec();
                    *ngram_freq.entry(ngram).or_insert(0) += 1;
                }
            }
        }

        let mut ngrams: Vec<(Vec<u8>, usize)> = ngram_freq.into_iter().collect();
        ngrams.sort_by(|a, b| {
            let score_a = a.1 * a.0.len();
            let score_b = b.1 * b.0.len();
            score_b.cmp(&score_a)
        });

        let mut code: u8 = 1;
        for (ngram, _freq) in ngrams {
            if code as usize >= max_symbols.min(SYMBOL_TABLE_SIZE) {
                break;
            }
            if ngram.len() > 1 {
                self.table.insert(ngram, code);
                code += 1;
            }
        }
    }

    pub fn encode(&self, s: &str) -> Vec<u8> {
        let bytes = s.as_bytes();
        let mut result = Vec::with_capacity(bytes.len());
        let mut i = 0;

        while i < bytes.len() {
            let mut best_match: Option<(u8, usize)> = None;

            for len in (1..=MAX_SYMBOL_LEN.min(bytes.len() - i)).rev() {
                let candidate: Vec<u8> = bytes[i..i + len].to_vec();
                if let Some(code) = self.table.get_by_bytes(&candidate) {
                    best_match = Some((code, len));
                    break;
                }
            }

            match best_match {
                Some((code, len)) => {
                    result.push(code);
                    i += len;
                }
                None => {
                    result.push(bytes[i]);
                    i += 1;
                }
            }
        }

        result
    }

    pub fn decode(&self, encoded: &[u8]) -> Vec<u8> {
        let mut result = Vec::new();

        for &code in encoded {
            if code == 0 {
                continue;
            }
            if let Some(symbol) = self.table.get_by_code(code) {
                result.extend_from_slice(symbol);
            } else {
                result.push(code);
            }
        }

        result
    }

    pub fn decode_to_string(&self, encoded: &[u8]) -> Option<String> {
        let bytes = self.decode(encoded);
        String::from_utf8(bytes).ok()
    }

    pub fn table(&self) -> &FsstSymbolTable {
        &self.table
    }

    pub fn compression_ratio(&self) -> f64 {
        if self.total_input_bytes == 0 {
            return 0.0;
        }
        (self.total_input_bytes - self.total_output_bytes) as f64 / self.total_input_bytes as f64
    }

    pub fn symbol_count(&self) -> usize {
        self.table.len()
    }
}

impl Default for FsstEncoder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct FsstColumn {
    encoder: FsstEncoder,
    encoded_data: Vec<Vec<u8>>,
    null_bitmap: Vec<bool>,
}

impl FsstColumn {
    pub fn new() -> Self {
        Self {
            encoder: FsstEncoder::new(),
            encoded_data: Vec::new(),
            null_bitmap: Vec::new(),
        }
    }

    pub fn train_and_build(strings: &[Option<&str>], max_symbols: usize) -> Self {
        let non_null: Vec<&str> = strings.iter().filter_map(|s| *s).collect();

        let encoder = FsstEncoder::train(&non_null, max_symbols);

        let mut column = Self {
            encoder,
            encoded_data: Vec::with_capacity(strings.len()),
            null_bitmap: Vec::with_capacity(strings.len()),
        };

        for s in strings {
            column.append(*s);
        }

        column
    }

    pub fn append(&mut self, value: Option<&str>) {
        match value {
            Some(s) => {
                let encoded = self.encoder.encode(s);
                self.encoded_data.push(encoded);
                self.null_bitmap.push(false);
            }
            None => {
                self.encoded_data.push(Vec::new());
                self.null_bitmap.push(true);
            }
        }
    }

    pub fn get(&self, row_idx: usize) -> Option<String> {
        if row_idx >= self.encoded_data.len() || self.null_bitmap[row_idx] {
            return None;
        }

        self.encoder.decode_to_string(&self.encoded_data[row_idx])
    }

    pub fn set(&mut self, row_idx: usize, value: Option<&str>) {
        if row_idx >= self.encoded_data.len() {
            return;
        }

        match value {
            Some(s) => {
                self.encoded_data[row_idx] = self.encoder.encode(s);
                self.null_bitmap[row_idx] = false;
            }
            None => {
                self.encoded_data[row_idx].clear();
                self.null_bitmap[row_idx] = true;
            }
        }
    }

    pub fn len(&self) -> usize {
        self.encoded_data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.encoded_data.is_empty()
    }

    pub fn is_null(&self, row_idx: usize) -> bool {
        row_idx < self.null_bitmap.len() && self.null_bitmap[row_idx]
    }

    pub fn memory_usage(&self) -> usize {
        let data_size: usize = self.encoded_data.iter().map(|v| v.len()).sum();
        let null_size = self.null_bitmap.len();
        let table_size = self.encoder.table().memory_usage();

        data_size + null_size + table_size
    }

    pub fn compression_ratio(&self) -> f64 {
        let original_size: usize = self
            .encoded_data
            .iter()
            .enumerate()
            .filter(|(i, _)| !self.null_bitmap[*i])
            .map(|(_, v)| {
                self.encoder.decode(v).len()
            })
            .sum();

        let compressed_size: usize = self.encoded_data.iter().map(|v| v.len()).sum();

        if original_size == 0 {
            return 0.0;
        }

        (original_size - compressed_size) as f64 / original_size as f64
    }

    pub fn encoder(&self) -> &FsstEncoder {
        &self.encoder
    }
}

impl Default for FsstColumn {
    fn default() -> Self {
        Self::new()
    }
}

pub fn select_fsst(strings: &[&str]) -> bool {
    if strings.len() < 100 {
        return false;
    }

    let total_len: usize = strings.iter().map(|s| s.len()).sum();
    let avg_len = total_len / strings.len();

    let unique_count = strings.iter().collect::<std::collections::HashSet<_>>().len();
    let cardinality_ratio = unique_count as f64 / strings.len() as f64;

    avg_len >= 20 && cardinality_ratio > 0.5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fsst_symbol_table() {
        let mut table = FsstSymbolTable::new();

        table.insert(b"hello".to_vec(), 1);
        table.insert(b"world".to_vec(), 2);

        assert_eq!(table.len(), 2);
        assert_eq!(table.get_by_code(1), Some(&b"hello".to_vec()));
        assert_eq!(table.get_by_bytes(b"world"), Some(2));
    }

    #[test]
    fn test_fsst_encoder_basic() {
        let strings = vec!["hello world", "hello rust", "hello code"];
        let encoder = FsstEncoder::train(&strings, 100);

        let encoded = encoder.encode("hello world");
        let decoded = encoder.decode_to_string(&encoded);

        assert_eq!(decoded, Some("hello world".to_string()));
    }

    #[test]
    fn test_fsst_encoder_compression() {
        let strings: Vec<&str> = (0..100)
            .map(|i| {
                if i % 3 == 0 {
                    "prefix_common_data_suffix"
                } else if i % 3 == 1 {
                    "prefix_other_data_suffix"
                } else {
                    "prefix_extra_data_suffix"
                }
            })
            .collect();

        let encoder = FsstEncoder::train(&strings, 200);

        let original_len: usize = strings.iter().map(|s| s.len()).sum();
        let compressed_len: usize = strings.iter().map(|s| encoder.encode(s).len()).sum();

        assert!(compressed_len < original_len);
    }

    #[test]
    fn test_fsst_column() {
        let strings = vec![
            Some("hello world"),
            None,
            Some("hello rust"),
            Some("hello code"),
        ];

        let column = FsstColumn::train_and_build(&strings, 100);

        assert_eq!(column.len(), 4);
        assert_eq!(column.get(0), Some("hello world".to_string()));
        assert!(column.is_null(1));
        assert_eq!(column.get(2), Some("hello rust".to_string()));
    }

    #[test]
    fn test_fsst_column_set() {
        let strings = vec![Some("hello world")];
        let mut column = FsstColumn::train_and_build(&strings, 100);

        column.set(0, Some("hello rust"));
        assert_eq!(column.get(0), Some("hello rust".to_string()));

        column.set(0, None);
        assert!(column.is_null(0));
    }

    #[test]
    fn test_select_fsst() {
        let short_strings: Vec<String> = (0..100).map(|i| format!("s{}", i)).collect();
        let short_refs: Vec<&str> = short_strings.iter().map(|s| s.as_str()).collect();
        assert!(!select_fsst(&short_refs));

        let long_strings: Vec<String> = (0..100)
            .map(|i| format!("very_long_string_with_common_prefix_{}", i))
            .collect();
        let long_refs: Vec<&str> = long_strings.iter().map(|s| s.as_str()).collect();
        assert!(select_fsst(&long_refs));
    }

    #[test]
    fn test_fsst_roundtrip() {
        let strings: Vec<&str> = vec![
            "https://example.com/page/1",
            "https://example.com/page/2",
            "https://example.com/page/3",
            "https://example.com/page/4",
            "https://example.com/page/5",
        ];

        let encoder = FsstEncoder::train(&strings, 200);

        for s in &strings {
            let encoded = encoder.encode(s);
            let decoded = encoder.decode_to_string(&encoded);
            assert_eq!(decoded, Some(s.to_string()));
        }
    }
}
