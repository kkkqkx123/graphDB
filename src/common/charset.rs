use encoding_rs;

/// Represents different character encodings
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Encoding {
    Utf8,
    Utf16,
    Latin1,
    Gbk,
    Big5,
    Utf8Bom,
}

impl Encoding {
    /// Convert encoding to encoding_rs encoding
    pub fn to_encoding_rs(self) -> &'static encoding_rs::Encoding {
        match self {
            Encoding::Utf8 => encoding_rs::UTF_8,
            Encoding::Utf16 => encoding_rs::UTF_16LE, // Using little endian as default
            Encoding::Latin1 => encoding_rs::WINDOWS_1252, // Use WINDOWS_1252 which is similar to Latin1
            Encoding::Gbk => encoding_rs::GBK,
            Encoding::Big5 => encoding_rs::BIG5,
            Encoding::Utf8Bom => encoding_rs::UTF_8,
        }
    }
}

/// Character set utilities
pub struct CharsetUtils;

impl CharsetUtils {
    /// Check if a byte sequence is valid UTF-8
    pub fn is_valid_utf8(bytes: &[u8]) -> bool {
        std::str::from_utf8(bytes).is_ok()
    }

    /// Convert bytes to string with specified encoding
    pub fn decode_with_encoding(
        bytes: &[u8],
        encoding: Encoding,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        match encoding {
            Encoding::Utf8 => {
                let s = std::str::from_utf8(bytes)?;
                Ok(s.to_string())
            }
            Encoding::Utf16 => {
                // For UTF-16, we need to handle endianness and BOM
                if bytes.len() % 2 != 0 {
                    return Err("UTF-16 byte sequence must have even length".into());
                }

                // Check for BOM
                let has_bom = bytes.len() >= 2
                    && ((bytes[0] == 0xFF && bytes[1] == 0xFE) || // little endian
                     (bytes[0] == 0xFE && bytes[1] == 0xFF)); // big endian

                let slice = if has_bom { &bytes[2..] } else { bytes };

                // Convert to u16 array and then to string
                let mut utf16_vec = Vec::new();
                for chunk in slice.chunks_exact(2) {
                    let code_unit = u16::from_le_bytes([chunk[0], chunk[1]]); // assuming little endian
                    utf16_vec.push(code_unit);
                }

                String::from_utf16(&utf16_vec).map_err(|_| "Invalid UTF-16 sequence".into())
            }
            Encoding::Latin1 => {
                // Latin1 (ISO-8859-1) maps directly to Unicode U+0000-U+00FF
                Ok(bytes.iter().map(|&b| b as char).collect())
            }
            Encoding::Gbk => {
                // Using encoding_rs for GBK
                let (cow, _encoding_used, _had_errors) = encoding_rs::GBK.decode(bytes);
                Ok(cow.into_owned())
            }
            Encoding::Big5 => {
                // Using encoding_rs for Big5
                let (cow, _encoding_used, _had_errors) = encoding_rs::BIG5.decode(bytes);
                Ok(cow.into_owned())
            }
            Encoding::Utf8Bom => {
                // Check for UTF-8 BOM
                let slice =
                    if bytes.len() >= 3 && bytes[0] == 0xEF && bytes[1] == 0xBB && bytes[2] == 0xBF
                    {
                        &bytes[3..] // Skip BOM
                    } else {
                        bytes // No BOM present
                    };

                let s = std::str::from_utf8(slice)?;
                Ok(s.to_string())
            }
        }
    }

    /// Encode a string to bytes with specified encoding
    pub fn encode_with_encoding(text: &str, encoding: Encoding) -> Vec<u8> {
        match encoding {
            Encoding::Utf8 => text.as_bytes().to_vec(),
            Encoding::Utf16 => {
                // Convert to UTF-16 in little endian
                let utf16: Vec<u16> = text.encode_utf16().collect();
                let mut result = Vec::with_capacity(utf16.len() * 2);

                for code_unit in utf16 {
                    result.extend_from_slice(&code_unit.to_le_bytes());
                }

                result
            }
            Encoding::Latin1 => {
                // Latin1: all characters must be in 0-255 range
                text.chars()
                    .map(|c| {
                        if (c as u32) <= 255 {
                            c as u8
                        } else {
                            // For simplicity, using a replacement character
                            0x1A // ASCII substitute character
                        }
                    })
                    .collect()
            }
            Encoding::Gbk => {
                let (cow, _encoding_used, _had_errors) = encoding_rs::GBK.encode(text);
                cow.into_owned()
            }
            Encoding::Big5 => {
                let (cow, _encoding_used, _had_errors) = encoding_rs::BIG5.encode(text);
                cow.into_owned()
            }
            Encoding::Utf8Bom => {
                // Add UTF-8 BOM
                let mut result = vec![0xEF, 0xBB, 0xBF];
                result.extend_from_slice(text.as_bytes());
                result
            }
        }
    }

    pub fn detect_encoding(bytes: &[u8]) -> Option<Encoding> {
        if bytes.is_empty() {
            return Some(Encoding::Utf8);
        }

        if let Some(encoding) = Self::detect_bom(bytes) {
            return Some(encoding);
        }

        Self::detect_by_heuristics(bytes)
    }

    fn detect_bom(bytes: &[u8]) -> Option<Encoding> {
        if bytes.len() >= 3 && bytes[0] == 0xEF && bytes[1] == 0xBB && bytes[2] == 0xBF {
            return Some(Encoding::Utf8Bom);
        }

        if bytes.len() >= 2 && bytes[0] == 0xFF && bytes[1] == 0xFE {
            return Some(Encoding::Utf16);
        }

        if bytes.len() >= 2 && bytes[0] == 0xFE && bytes[1] == 0xFF {
            return Some(Encoding::Utf16);
        }

        None
    }

    fn detect_by_heuristics(bytes: &[u8]) -> Option<Encoding> {
        if Self::is_valid_utf8(bytes) {
            if Self::has_multibyte_sequences(bytes) {
                return Some(Encoding::Utf8);
            }
        }

        if Self::is_likely_gbk(bytes) {
            return Some(Encoding::Gbk);
        }

        if Self::is_likely_big5(bytes) {
            return Some(Encoding::Big5);
        }

        if Self::is_likely_latin1(bytes) {
            return Some(Encoding::Latin1);
        }

        Some(Encoding::Utf8)
    }

    fn has_multibyte_sequences(bytes: &[u8]) -> bool {
        let mut multibyte_count = 0;
        let mut i = 0;

        while i < bytes.len() {
            let b = bytes[i];

            if b & 0x80 == 0 {
                i += 1;
            } else if b & 0xE0 == 0xC0 {
                if i + 1 >= bytes.len() {
                    return false;
                }
                multibyte_count += 1;
                i += 2;
            } else if b & 0xF0 == 0xE0 {
                if i + 2 >= bytes.len() {
                    return false;
                }
                multibyte_count += 1;
                i += 3;
            } else if b & 0xF8 == 0xF0 {
                if i + 3 >= bytes.len() {
                    return false;
                }
                multibyte_count += 1;
                i += 4;
            } else {
                return false;
            }
        }

        multibyte_count > 0
    }

    fn is_likely_gbk(bytes: &[u8]) -> bool {
        if bytes.is_empty() {
            return false;
        }

        let mut high_bit_count = 0;
        let mut gbk_sequences = 0;

        for i in 0..bytes.len() {
            if bytes[i] & 0x80 != 0 {
                high_bit_count += 1;

                if bytes[i] >= 0x81 && bytes[i] <= 0xFE {
                    if i + 1 < bytes.len() {
                        let trail = bytes[i + 1];
                        if trail >= 0x40 && trail <= 0xFE {
                            gbk_sequences += 1;
                        }
                    }
                }
            }
        }

        let ratio = high_bit_count as f64 / bytes.len() as f64;
        ratio > 0.1 && gbk_sequences > 0
    }

    fn is_likely_big5(bytes: &[u8]) -> bool {
        if bytes.is_empty() {
            return false;
        }

        let mut high_bit_count = 0;
        let mut big5_sequences = 0;

        for i in 0..bytes.len() {
            if bytes[i] & 0x80 != 0 {
                high_bit_count += 1;

                if bytes[i] >= 0x81 && bytes[i] <= 0xFE {
                    if i + 1 < bytes.len() {
                        let trail = bytes[i + 1];
                        if (trail >= 0x40 && trail <= 0x7E) || (trail >= 0xA1 && trail <= 0xFE) {
                            big5_sequences += 1;
                        }
                    }
                }
            }
        }

        let ratio = high_bit_count as f64 / bytes.len() as f64;
        ratio > 0.1 && big5_sequences > 0
    }

    fn is_likely_latin1(bytes: &[u8]) -> bool {
        if bytes.is_empty() {
            return false;
        }

        let mut printable_count = 0;

        for &b in bytes {
            if (b >= 0x20 && b <= 0x7E) || b == 0x0A || b == 0x0D {
                printable_count += 1;
            } else if b >= 0xA0 {
                printable_count += 1;
            }
        }

        let ratio = printable_count as f64 / bytes.len() as f64;
        ratio > 0.8
    }

    /// Convert text from one encoding to another
    pub fn convert_encoding(
        bytes: &[u8],
        from_encoding: Encoding,
        to_encoding: Encoding,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        // First decode from the source encoding
        let text = Self::decode_with_encoding(bytes, from_encoding)?;

        // Then encode to the target encoding
        Ok(Self::encode_with_encoding(&text, to_encoding))
    }

    /// Get the number of characters (not bytes) in a UTF-8 string
    pub fn char_count(text: &str) -> usize {
        text.chars().count()
    }

    /// Get the number of bytes in a UTF-8 string
    pub fn byte_count(text: &str) -> usize {
        text.len()
    }

    /// Convert a string to lowercase respecting locale-specific rules (simplified)
    pub fn to_lowercase(text: &str) -> String {
        text.to_lowercase()
    }

    /// Convert a string to uppercase respecting locale-specific rules (simplified)
    pub fn to_uppercase(text: &str) -> String {
        text.to_uppercase()
    }

    /// Check if a string is palindromic (reads the same forwards and backwards)
    pub fn is_palindrome(text: &str) -> bool {
        let normalized = Self::to_lowercase(text);
        let reversed: String = normalized.chars().rev().collect();
        normalized == reversed
    }
}

/// Text normalizer for consistent text processing
pub struct TextNormalizer;

impl TextNormalizer {
    /// Normalize text for comparison (case-insensitive, trimmed)
    pub fn normalize_for_comparison(text: &str) -> String {
        text.trim().to_lowercase()
    }

    /// Normalize text by removing extra whitespace
    pub fn normalize_whitespace(text: &str) -> String {
        let mut result = String::new();
        let mut last_was_space = true; // To trim leading spaces

        for c in text.chars() {
            if c.is_whitespace() {
                if !last_was_space {
                    result.push(' ');
                    last_was_space = true;
                }
            } else {
                result.push(c);
                last_was_space = false;
            }
        }

        // Remove trailing space if added
        if result.ends_with(' ') {
            result.pop();
        }

        result
    }

    /// Normalize text by removing diacritics (simplified)
    pub fn normalize_diacritics(text: &str) -> String {
        // This is a simplified implementation
        // In a real implementation, we'd use a more comprehensive approach
        text.chars()
            .map(|c| match c {
                'á' | 'à' | 'â' | 'ä' | 'ă' | 'ắ' | 'ằ' | 'ẵ' | 'ẳ' | 'ặ' | 'ǟ' | 'ǻ' | 'ǎ'
                | 'ȁ' | 'ȃ' | 'ȧ' | 'ḁ' | 'ą' | 'ⱥ' | 'ɐ' => 'a',
                'é' | 'è' | 'ê' | 'ë' | 'ĕ' | 'ě' | 'ȅ' | 'ȇ' | 'ȩ' | 'ḝ' | 'ę' | 'ḙ' | 'ḛ'
                | 'ɇ' | 'ɛ' | 'ǝ' => 'e',
                'í' | 'ì' | 'î' | 'ï' | 'ĭ' | 'ǐ' | 'ȉ' | 'ȋ' | 'į' | 'ḭ' | 'ɨ' | 'ı' => {
                    'i'
                }
                'ó' | 'ò' | 'ô' | 'ö' | 'ő' | 'ŏ' | 'ǒ' | 'ȍ' | 'ȏ' | 'ơ' | 'ǫ' | 'ǭ' | 'ø'
                | 'ǿ' | 'ɔ' | 'œ' | 'ɶ' | 'ɵ' | 'ȯ' | 'ȱ' | 'ọ' | 'ỏ' | 'ồ' | 'ố' | 'ỗ' | 'ộ'
                | 'ớ' | 'ờ' | 'ở' | 'ỡ' | 'ợ' => 'o',
                'ú' | 'ù' | 'û' | 'ü' | 'ŭ' | 'ů' | 'ű' | 'ǔ' | 'ȕ' | 'ȗ' | 'ų' | 'ṷ' | 'ʉ' => {
                    'u'
                }
                'ñ' | 'ń' | 'ň' | 'ņ' | 'ŉ' | 'ŋ' | 'ɲ' | 'ƞ' | 'ɳ' | 'ȵ' => 'n',
                'ç' | 'ć' | 'č' | 'ĉ' | 'ċ' | 'ƈ' | 'ȼ' | 'ɕ' | 'ʗ' | 'ḉ' => 'c',
                _ => c,
            })
            .collect()
    }

    /// Normalize text to NFD (decomposed) form (simplified)
    pub fn normalize_decomposed(text: &str) -> String {
        // This would use proper Unicode normalization in a full implementation
        // For now, just returning the original text
        text.to_string()
    }
}

/// Multibyte character utilities
pub struct MultibyteUtils;

impl MultibyteUtils {
    /// Get the byte length of the first character in a UTF-8 string
    pub fn first_char_byte_len(text: &str) -> Option<usize> {
        text.chars().next().map(|c| c.len_utf8())
    }

    /// Get the byte length of a specific character in a UTF-8 string
    pub fn char_byte_len(c: char) -> usize {
        c.len_utf8()
    }

    /// Get the byte position of a character at index (in characters, not bytes)
    pub fn char_to_byte_index(text: &str, char_index: usize) -> Option<usize> {
        let mut byte_pos = 0;
        let mut char_pos = 0;

        for ch in text.chars() {
            if char_pos == char_index {
                return Some(byte_pos);
            }

            byte_pos += ch.len_utf8();
            char_pos += 1;
        }

        if char_pos == char_index {
            Some(byte_pos)
        } else {
            None // Index out of bounds
        }
    }

    /// Get substring by character indices (not byte indices)
    pub fn substring_by_chars(text: &str, start: usize, end: usize) -> Option<String> {
        if start > end {
            return None;
        }

        let char_indices: Vec<(usize, char)> = text.char_indices().collect();

        if start > char_indices.len() || end > char_indices.len() {
            return None;
        }

        if start == end {
            return Some(String::new());
        }

        let start_byte = char_indices[start].0;
        let end_byte = if end == char_indices.len() {
            text.len()
        } else {
            char_indices[end].0
        };

        Some(text[start_byte..end_byte].to_string())
    }

    /// Replace all occurrences of a substring with another, respecting character boundaries
    pub fn replace_chars(text: &str, from: &str, to: &str) -> String {
        text.replace(from, to)
    }
}

/// Character set validation utilities
pub mod validation {
    use super::*;

    /// Check if a string contains valid UTF-8
    pub fn is_valid_utf8_string(s: &str) -> bool {
        s.is_ascii() || CharsetUtils::is_valid_utf8(s.as_bytes())
    }

    /// Check if a string contains only ASCII characters
    pub fn is_ascii_only(s: &str) -> bool {
        s.is_ascii()
    }

    /// Check if a string contains only Latin-1 characters (U+0000 to U+00FF)
    pub fn is_latin1_only(s: &str) -> bool {
        s.chars().all(|c| (c as u32) <= 0xFF)
    }

    /// Check if a string contains only printable ASCII characters
    pub fn is_printable_ascii_only(s: &str) -> bool {
        s.chars()
            .all(|c| c.is_ascii() && c.is_ascii_graphic() || c == ' ')
    }

    /// Sanitize a string by replacing non-Latin1 characters
    pub fn sanitize_to_latin1(s: &str) -> String {
        s.chars()
            .map(|c| if (c as u32) <= 0xFF { c } else { '?' })
            .collect()
    }
}

/// Character set configuration
#[derive(Debug, Clone)]
pub struct CharsetConfig {
    pub default_encoding: Encoding,
    pub fallback_encoding: Encoding,
    pub enable_multibyte_support: bool,
    pub max_string_length: usize,
}

impl Default for CharsetConfig {
    fn default() -> Self {
        Self {
            default_encoding: Encoding::Utf8,
            fallback_encoding: Encoding::Latin1,
            enable_multibyte_support: true,
            max_string_length: 1024 * 1024, // 1MB
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_utf8_validation() {
        let valid_utf8 = "Hello, 世界";
        let invalid_utf8 = [0, 159, 146, 150]; // Invalid UTF-8 sequence

        assert!(CharsetUtils::is_valid_utf8(valid_utf8.as_bytes()));
        assert!(!CharsetUtils::is_valid_utf8(&invalid_utf8));
    }

    #[test]
    fn test_encoding_decoding() {
        let original = "Hello, 世界!";

        // Test UTF-8 encoding/decoding
        let encoded = CharsetUtils::encode_with_encoding(original, Encoding::Utf8);
        let decoded = CharsetUtils::decode_with_encoding(&encoded, Encoding::Utf8)
            .expect("Failed to decode UTF-8");
        assert_eq!(original, decoded);

        // Test with Latin-1 (will lose non-Latin1 chars)
        let encoded_latin1 = CharsetUtils::encode_with_encoding(original, Encoding::Latin1);
        let decoded_latin1 = CharsetUtils::decode_with_encoding(&encoded_latin1, Encoding::Latin1)
            .expect("Failed to decode Latin-1");
        assert_ne!(original, decoded_latin1); // The non-Latin1 chars will be replaced
    }

    #[test]
    fn test_encoding_conversion() {
        let original = "Test";

        // Encode in UTF-8
        let utf8_bytes = CharsetUtils::encode_with_encoding(original, Encoding::Utf8);

        // Convert to Latin-1
        let latin1_bytes =
            CharsetUtils::convert_encoding(&utf8_bytes, Encoding::Utf8, Encoding::Latin1)
                .expect("Failed to convert encoding");

        // Decode back to string
        let result = CharsetUtils::decode_with_encoding(&latin1_bytes, Encoding::Latin1)
            .expect("Failed to decode converted bytes");

        assert_eq!(original, result);
    }

    #[test]
    fn test_text_normalization() {
        let original = "  Hello   World  ";
        let normalized = TextNormalizer::normalize_whitespace(original);
        assert_eq!(normalized, "Hello World");

        let original_case = "hEllO";
        let lower_normalized = TextNormalizer::normalize_for_comparison(original_case);
        assert_eq!(lower_normalized, "hello");
    }

    #[test]
    fn test_multibyte_utils() {
        let text = "Hello, 世界";

        // Test byte length of first character
        assert_eq!(MultibyteUtils::first_char_byte_len(text), Some(1));

        // Test byte position of character at index 7 (the '世')
        if let Some(byte_pos) = MultibyteUtils::char_to_byte_index(text, 7) {
            // The '世' character should be at byte position 7
            assert_eq!(byte_pos, 7);
        }

        // Test substring by character indices
        if let Some(substring) = MultibyteUtils::substring_by_chars(text, 7, 9) {
            assert_eq!(substring, "世界");
        }
    }

    #[test]
    fn test_validation() {
        let ascii_text = "Hello";
        let unicode_text = "Hello, 世界";
        let latin1_text = "Café"; // Contains non-ASCII Latin1 character

        assert!(validation::is_ascii_only(ascii_text));
        assert!(!validation::is_ascii_only(unicode_text));

        assert!(validation::is_latin1_only(latin1_text));
        assert!(!validation::is_latin1_only(unicode_text));

        assert!(validation::is_printable_ascii_only(ascii_text));
        assert!(!validation::is_printable_ascii_only(unicode_text));
    }
}

/// 字符集描述信息
#[derive(Debug, Clone)]
pub struct CharsetDesc {
    pub charset_name: String,
    pub default_collation: String,
    pub supported_collations: Vec<String>,
    pub description: String,
    pub max_char_length: i32,
}

/// 字符集管理器
pub struct CharsetManager {
    supported_charsets: std::collections::HashSet<String>,
    supported_collations: std::collections::HashSet<String>,
    charset_descriptions: std::collections::HashMap<String, CharsetDesc>,
}

impl CharsetManager {
    pub fn new() -> Self {
        let mut manager = Self {
            supported_charsets: std::collections::HashSet::new(),
            supported_collations: std::collections::HashSet::new(),
            charset_descriptions: std::collections::HashMap::new(),
        };

        manager.register_charset(CharsetDesc {
            charset_name: "utf8".to_string(),
            default_collation: "utf8_bin".to_string(),
            supported_collations: vec!["utf8_bin".to_string()],
            description: "UTF-8 Unicode".to_string(),
            max_char_length: 4,
        });

        manager.register_charset(CharsetDesc {
            charset_name: "utf8mb4".to_string(),
            default_collation: "utf8mb4_bin".to_string(),
            supported_collations: vec!["utf8mb4_bin".to_string()],
            description: "UTF-8 Unicode with 4-byte support".to_string(),
            max_char_length: 4,
        });

        manager.register_charset(CharsetDesc {
            charset_name: "latin1".to_string(),
            default_collation: "latin1_bin".to_string(),
            supported_collations: vec!["latin1_bin".to_string()],
            description: "Latin1 (ISO-8859-1)".to_string(),
            max_char_length: 1,
        });

        manager.register_charset(CharsetDesc {
            charset_name: "gbk".to_string(),
            default_collation: "gbk_bin".to_string(),
            supported_collations: vec!["gbk_bin".to_string()],
            description: "GBK Chinese".to_string(),
            max_char_length: 2,
        });

        manager.register_charset(CharsetDesc {
            charset_name: "big5".to_string(),
            default_collation: "big5_bin".to_string(),
            supported_collations: vec!["big5_bin".to_string()],
            description: "Big5 Traditional Chinese".to_string(),
            max_char_length: 2,
        });

        manager
    }

    fn register_charset(&mut self, desc: CharsetDesc) {
        self.supported_charsets.insert(desc.charset_name.clone());
        self.supported_collations.insert(desc.default_collation.clone());
        for collation in &desc.supported_collations {
            self.supported_collations.insert(collation.clone());
        }
        self.charset_descriptions.insert(desc.charset_name.clone(), desc);
    }

    pub fn is_support_charset(&self, charset_name: &str) -> bool {
        self.supported_charsets.contains(&charset_name.to_lowercase())
    }

    pub fn is_support_collate(&self, collate_name: &str) -> bool {
        self.supported_collations.contains(&collate_name.to_lowercase())
    }

    pub fn charset_and_collate_match(&self, charset_name: &str, collate_name: &str) -> bool {
        if let Some(desc) = self.charset_descriptions.get(&charset_name.to_lowercase()) {
            desc.supported_collations
                .iter()
                .any(|c| c.to_lowercase() == collate_name.to_lowercase())
        } else {
            false
        }
    }

    pub fn get_default_collation_by_charset(&self, charset_name: &str) -> Option<String> {
        self.charset_descriptions
            .get(&charset_name.to_lowercase())
            .map(|desc| desc.default_collation.clone())
    }

    pub fn get_charset_by_collation(&self, collation_name: &str) -> Option<String> {
        for (charset_name, desc) in &self.charset_descriptions {
            if desc.supported_collations
                .iter()
                .any(|c| c.to_lowercase() == collation_name.to_lowercase())
            {
                return Some(charset_name.clone());
            }
        }
        None
    }

    pub fn get_charset_desc(&self) -> &std::collections::HashMap<String, CharsetDesc> {
        &self.charset_descriptions
    }

    pub fn get_supported_charsets(&self) -> Vec<String> {
        self.supported_charsets.iter().cloned().collect()
    }

    pub fn get_supported_collations(&self) -> Vec<String> {
        self.supported_collations.iter().cloned().collect()
    }
}

impl Default for CharsetManager {
    fn default() -> Self {
        Self::new()
    }
}

impl CharsetUtils {
    pub fn is_supported_charset(charset_name: &str) -> bool {
        let manager = CharsetManager::new();
        manager.is_support_charset(charset_name)
    }

    pub fn is_supported_collation(collation_name: &str) -> bool {
        let manager = CharsetManager::new();
        manager.is_support_collate(collation_name)
    }

    pub fn validate_charset_and_collation(
        charset_name: &str,
        collation_name: &str,
    ) -> Result<(), String> {
        let manager = CharsetManager::new();

        if !manager.is_support_charset(charset_name) {
            return Err(format!("不支持的字符集: {}", charset_name));
        }

        if !manager.is_support_collate(collation_name) {
            return Err(format!("不支持的排序规则: {}", collation_name));
        }

        if !manager.charset_and_collate_match(charset_name, collation_name) {
            return Err(format!(
                "字符集 {} 与排序规则 {} 不匹配",
                charset_name, collation_name
            ));
        }

        Ok(())
    }

    pub fn get_default_collation(charset_name: &str) -> Option<String> {
        let manager = CharsetManager::new();
        manager.get_default_collation_by_charset(charset_name)
    }

    pub fn get_supported_charsets() -> Vec<String> {
        let manager = CharsetManager::new();
        manager.get_supported_charsets()
    }

    pub fn get_supported_collations() -> Vec<String> {
        let manager = CharsetManager::new();
        manager.get_supported_collations()
    }
}
