use graphdb::common::charset::*;
use graphdb::common::charset::validation;
use graphdb::common::charset::MultibyteUtils;
use graphdb::common::charset::TextNormalizer;

#[test]
fn test_encoding_to_encoding_rs() {
    assert_eq!(Encoding::Utf8.to_encoding_rs(), encoding_rs::UTF_8);
    assert_eq!(Encoding::Utf16.to_encoding_rs(), encoding_rs::UTF_16LE);
    assert_eq!(Encoding::Latin1.to_encoding_rs(), encoding_rs::WINDOWS_1252);
    assert_eq!(Encoding::Gbk.to_encoding_rs(), encoding_rs::GBK);
    assert_eq!(Encoding::Big5.to_encoding_rs(), encoding_rs::BIG5);
    assert_eq!(Encoding::Utf8Bom.to_encoding_rs(), encoding_rs::UTF_8);
}

#[test]
fn test_encoding_debug() {
    let encoding = Encoding::Utf8;
    let debug_format = format!("{:?}", encoding);
    assert_eq!(debug_format, "Utf8");
}

#[test]
fn test_encoding_clone() {
    let encoding1 = Encoding::Latin1;
    let encoding2 = encoding1.clone();
    assert_eq!(encoding1, encoding2);
}

#[test]
fn test_encoding_partial_eq() {
    assert_eq!(Encoding::Utf8, Encoding::Utf8);
    assert_ne!(Encoding::Utf8, Encoding::Utf16);
}

#[test]
fn test_charset_utils_is_valid_utf8_valid() {
    let result = CharsetUtils::is_valid_utf8("Hello World".as_bytes());
    assert!(result);
}

#[test]
fn test_charset_utils_is_valid_utf8_invalid() {
    let invalid_utf8 = [0xFF, 0xFE];
    let result = CharsetUtils::is_valid_utf8(&invalid_utf8);
    assert!(!result);
}

#[test]
fn test_charset_utils_decode_utf8() {
    let bytes = "Hello".as_bytes();
    let result = CharsetUtils::decode_with_encoding(bytes, Encoding::Utf8);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Hello");
}

#[test]
fn test_charset_utils_decode_utf16() {
    let utf16_bytes = [
        0x48, 0x00, 0x65, 0x00, 0x6C, 0x00, 0x6C, 0x00, 0x6F, 0x00,
    ];
    let result = CharsetUtils::decode_with_encoding(&utf16_bytes, Encoding::Utf16);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Hello");
}

#[test]
fn test_charset_utils_decode_latin1() {
    let latin1_bytes = [0x48, 0x65, 0x6C, 0x6C, 0x6F];
    let result = CharsetUtils::decode_with_encoding(&latin1_bytes, Encoding::Latin1);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Hello");
}

#[test]
fn test_charset_utils_decode_gbk() {
    let gbk_bytes = [0xC4, 0xE3, 0xBA, 0xC3];
    let result = CharsetUtils::decode_with_encoding(&gbk_bytes, Encoding::Gbk);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "你好");
}

#[test]
fn test_charset_utils_decode_big5() {
    let big5_bytes = [0xA7, 0x41];
    let result = CharsetUtils::decode_with_encoding(&big5_bytes, Encoding::Big5);
    assert!(result.is_ok());
    assert!(!result.unwrap().is_empty());
}

#[test]
fn test_charset_utils_decode_utf8_bom() {
    let with_bom = [0xEF, 0xBB, 0xBF, 0x48, 0x65, 0x6C, 0x6C, 0x6F];
    let result = CharsetUtils::decode_with_encoding(&with_bom, Encoding::Utf8Bom);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Hello");
}

#[test]
fn test_charset_utils_decode_utf8_bom_no_bom() {
    let no_bom = "Hello".as_bytes();
    let result = CharsetUtils::decode_with_encoding(no_bom, Encoding::Utf8Bom);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Hello");
}

#[test]
fn test_charset_utils_decode_invalid_utf16() {
    let odd_length = [0xFF];
    let result = CharsetUtils::decode_with_encoding(&odd_length, Encoding::Utf16);
    assert!(result.is_err());
}

#[test]
fn test_charset_utils_encode_utf8() {
    let text = "Hello";
    let result = CharsetUtils::encode_with_encoding(text, Encoding::Utf8);
    assert_eq!(result, "Hello".as_bytes());
}

#[test]
fn test_charset_utils_encode_utf16() {
    let text = "Hi";
    let result = CharsetUtils::encode_with_encoding(text, Encoding::Utf16);
    assert_eq!(result.len(), 4);
    assert_eq!(result[0], 0x48);
    assert_eq!(result[2], 0x69);
}

#[test]
fn test_charset_utils_encode_latin1() {
    let text = "Café";
    let result = CharsetUtils::encode_with_encoding(text, Encoding::Latin1);
    assert_eq!(result[0], 0x43);
    assert_eq!(result[1], 0x61);
    assert_eq!(result[2], 0x66);
    assert_eq!(result[3], 0xE9);
}

#[test]
fn test_charset_utils_convert_encoding() {
    let text = "Test";
    let utf8_bytes = CharsetUtils::encode_with_encoding(text, Encoding::Utf8);
    let result = CharsetUtils::decode_with_encoding(&utf8_bytes, Encoding::Utf8);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), text);
}

#[test]
fn test_charset_utils_convert_utf8_to_gbk() {
    let text = "测试";
    let utf8_bytes = CharsetUtils::encode_with_encoding(text, Encoding::Utf8);
    let gbk_bytes = CharsetUtils::convert_encoding(&utf8_bytes, Encoding::Utf8, Encoding::Gbk);
    assert!(gbk_bytes.is_ok());
    let result = CharsetUtils::decode_with_encoding(&gbk_bytes.unwrap(), Encoding::Gbk);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), text);
}

#[test]
fn test_charset_utils_convert_gbk_to_utf8() {
    let gbk_bytes = [0xC4, 0xE3, 0xBA, 0xC3];
    let utf8_bytes = CharsetUtils::convert_encoding(&gbk_bytes, Encoding::Gbk, Encoding::Utf8);
    assert!(utf8_bytes.is_ok());
    let result = CharsetUtils::decode_with_encoding(&utf8_bytes.unwrap(), Encoding::Utf8);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "你好");
}

#[test]
fn test_charset_utils_detect_encoding() {
    let utf8_bytes = "Hello".as_bytes();
    let detected = CharsetUtils::detect_encoding(utf8_bytes);
    assert!(detected.is_some());
}

#[test]
fn test_charset_utils_detect_encoding_gbk() {
    let gbk_bytes = [0xC4, 0xE3, 0xBA, 0xC3];
    let detected = CharsetUtils::detect_encoding(&gbk_bytes);
    assert!(detected.is_some());
}

#[test]
fn test_charset_utils_encode_decode_roundtrip() {
    let original = "Hello, World! こんにちは 你好";
    let encoded = CharsetUtils::encode_with_encoding(original, Encoding::Utf8);
    let decoded = CharsetUtils::decode_with_encoding(&encoded, Encoding::Utf8);
    assert!(decoded.is_ok());
    assert_eq!(decoded.unwrap(), original);
}

#[test]
fn test_charset_utils_char_count() {
    assert_eq!(CharsetUtils::char_count("Hello"), 5);
    assert_eq!(CharsetUtils::char_count("你好"), 2);
    assert_eq!(CharsetUtils::char_count(""), 0);
}

#[test]
fn test_charset_utils_byte_count() {
    assert_eq!(CharsetUtils::byte_count("Hello"), 5);
    assert_eq!(CharsetUtils::byte_count("你好"), 6);
}

#[test]
fn test_charset_utils_to_lowercase() {
    assert_eq!(CharsetUtils::to_lowercase("HELLO"), "hello");
    assert_eq!(CharsetUtils::to_lowercase("Hello"), "hello");
}

#[test]
fn test_charset_utils_to_uppercase() {
    assert_eq!(CharsetUtils::to_uppercase("hello"), "HELLO");
    assert_eq!(CharsetUtils::to_uppercase("Hello"), "HELLO");
}

#[test]
fn test_charset_utils_is_palindrome() {
    assert!(CharsetUtils::is_palindrome("racecar"));
    assert!(!CharsetUtils::is_palindrome("hello"));
    assert!(CharsetUtils::is_palindrome("a"));
}

#[test]
fn test_text_normalizer_normalize_for_comparison() {
    assert_eq!(
        TextNormalizer::normalize_for_comparison("café"),
        TextNormalizer::normalize_for_comparison("CAFÉ")
    );
}

#[test]
fn test_text_normalizer_normalize_whitespace() {
    assert_eq!(
        TextNormalizer::normalize_whitespace("  hello   world  "),
        "hello world"
    );
}

#[test]
fn test_multibyte_utils_first_char_byte_len() {
    assert_eq!(MultibyteUtils::first_char_byte_len("a"), Some(1));
    assert_eq!(MultibyteUtils::first_char_byte_len("你"), Some(3));
}

#[test]
fn test_multibyte_utils_char_byte_len() {
    assert_eq!(MultibyteUtils::char_byte_len('a'), 1);
    assert_eq!(MultibyteUtils::char_byte_len('你'), 3);
}

#[test]
fn test_multibyte_utils_char_to_byte_index() {
    assert_eq!(MultibyteUtils::char_to_byte_index("hello", 0), Some(0));
    assert_eq!(MultibyteUtils::char_to_byte_index("hello", 1), Some(1));
    assert_eq!(MultibyteUtils::char_to_byte_index("你好", 1), Some(3));
}

#[test]
fn test_multibyte_utils_substring_by_chars() {
    let result = MultibyteUtils::substring_by_chars("hello", 0, 3);
    assert_eq!(result, Some("hel".to_string()));

    let result = MultibyteUtils::substring_by_chars("你好世界", 0, 2);
    assert_eq!(result, Some("你好".to_string()));
}

#[test]
fn test_multibyte_utils_replace_chars() {
    let result = MultibyteUtils::replace_chars("foo bar", "o", "x");
    assert_eq!(result, "fxx bar");
}

#[test]
fn test_validation_is_valid_utf8_string() {
    assert!(validation::is_valid_utf8_string("Hello"));
}

#[test]
fn test_validation_is_ascii_only() {
    assert!(validation::is_ascii_only("Hello"));
    assert!(!validation::is_ascii_only("你好"));
}

#[test]
fn test_validation_is_latin1_only() {
    assert!(validation::is_latin1_only("Café"));
    assert!(!validation::is_latin1_only("你好"));
}

#[test]
fn test_validation_is_printable_ascii_only() {
    assert!(validation::is_printable_ascii_only("Hello"));
    assert!(!validation::is_printable_ascii_only("Hello\t"));
}

#[test]
fn test_validation_sanitize_to_latin1() {
    let result = validation::sanitize_to_latin1("你好");
    assert!(result.is_ascii());
}
