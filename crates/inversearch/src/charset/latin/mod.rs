use std::collections::HashMap;
use crate::r#type::EncoderOptions;

// ========== 基础字符映射 ==========
fn get_soundex_map() -> HashMap<char, char> {
    let mut map = HashMap::new();
    
    // 声音相似字符映射
    map.insert('b', 'p');
    map.insert('v', 'f');
    map.insert('w', 'f');
    map.insert('z', 's');
    map.insert('x', 's');
    map.insert('d', 't');
    map.insert('n', 'm');
    map.insert('c', 'k');
    map.insert('g', 'k');
    map.insert('j', 'k');
    map.insert('q', 'k');
    map.insert('i', 'e');
    map.insert('y', 'e');
    map.insert('u', 'o');
    
    map
}

// ========== 高级匹配规则 ==========
fn get_advanced_matcher() -> HashMap<String, String> {
    let mut matcher = HashMap::new();
    
    matcher.insert("ae".to_string(), "a".to_string());
    matcher.insert("oe".to_string(), "o".to_string());
    matcher.insert("sh".to_string(), "s".to_string());
    matcher.insert("kh".to_string(), "k".to_string());
    matcher.insert("th".to_string(), "t".to_string());
    matcher.insert("ph".to_string(), "f".to_string());
    matcher.insert("pf".to_string(), "f".to_string());
    
    matcher
}

// ========== 替换规则 ==========
fn get_advanced_replacer() -> Vec<(String, String)> {
    vec![
        (r"([^aeo])h(.)", "$1$2"),
        (r"([aeo])h([^aeo]|$)", "$1$2"),
    ].into_iter().map(|(a, b)| (a.to_string(), b.to_string())).collect()
}

fn get_compact_replacer() -> Vec<(String, String)> {
    vec![
        (r"(?!^)[aeo]", ""),
    ].into_iter().map(|(a, b)| (a.to_string(), b.to_string())).collect()
}

// ========== Soundex 编码 ==========
pub fn soundex_encode(string_to_encode: &str) -> String {
    let codes = get_soundex_codes();
    
    if string_to_encode.is_empty() {
        return String::new();
    }
    
    let first_char = string_to_encode.chars().next().unwrap();
    let mut encoded_string = first_char.to_string();
    let mut last = codes.get(&first_char.to_ascii_lowercase()).copied().unwrap_or(0);
    
    for (_i, char) in string_to_encode.chars().enumerate().skip(1) {
        // Remove all occurrences of "h" and "w"
        if char.to_ascii_lowercase() != 'h' && char.to_ascii_lowercase() != 'w' {
            // Replace all consonants with digits
            let char_code = codes.get(&char.to_ascii_lowercase()).copied().unwrap_or(0);
            
            // Remove all occurrences of a,e,i,o,u,y except first letter
            if char_code != 0 {
                // Replace all adjacent same digits with one digit
                if char_code != last {
                    encoded_string.push_str(&char_code.to_string());
                    last = char_code;
                    if encoded_string.len() == 4 {
                        break;
                    }
                }
            }
        }
    }
    
    encoded_string
}

fn get_soundex_codes() -> HashMap<char, i32> {
    let mut codes = HashMap::new();
    
    // Vowels and y get 0
    codes.insert('a', 0);
    codes.insert('e', 0);
    codes.insert('i', 0);
    codes.insert('o', 0);
    codes.insert('u', 0);
    codes.insert('y', 0);
    
    // Group 1: b, f, p, v
    codes.insert('b', 1);
    codes.insert('f', 1);
    codes.insert('p', 1);
    codes.insert('v', 1);
    
    // Group 2: c, g, j, k, q, s, x, z, ß
    codes.insert('c', 2);
    codes.insert('g', 2);
    codes.insert('j', 2);
    codes.insert('k', 2);
    codes.insert('q', 2);
    codes.insert('s', 2);
    codes.insert('x', 2);
    codes.insert('z', 2);
    codes.insert('ß', 2);
    
    // Group 3: d, t
    codes.insert('d', 3);
    codes.insert('t', 3);
    
    // Group 4: l
    codes.insert('l', 4);
    
    // Group 5: m, n
    codes.insert('m', 5);
    codes.insert('n', 5);
    
    // Group 6: r
    codes.insert('r', 6);
    
    codes
}

// ========== 预设配置 ==========

/// 基础拉丁字符集 - 简单的字符映射
pub fn get_charset_latin_balance() -> EncoderOptions {
    EncoderOptions {
        mapper: Some(get_soundex_map()),
        rtl: Some(false),
        dedupe: Some(true),
        split: None,
        numeric: Some(true),
        normalize: Some(true),
        prepare: None,
        finalize: None,
        filter: None,
        matcher: None,
        stemmer: None,
        replacer: None,
        minlength: Some(1),
        maxlength: Some(1024),
        cache: Some(true),
    }
}

/// 高级拉丁字符集 - 包含匹配和替换规则
pub fn get_charset_latin_advanced() -> EncoderOptions {
    let mut mapper = HashMap::new();
    mapper.insert('t', 't');
    mapper.insert('e', 'e');
    mapper.insert('s', 's');
    
    // 合并基础声音映射
    mapper.extend(get_soundex_map());
    
    EncoderOptions {
        mapper: Some(mapper),
        matcher: Some(get_advanced_matcher()),
        replacer: Some(get_advanced_replacer()),
        rtl: Some(false),
        dedupe: Some(true),
        split: None,
        numeric: Some(true),
        normalize: Some(true),
        prepare: None,
        finalize: None,
        filter: None,
        stemmer: None,
        minlength: Some(1),
        maxlength: Some(1024),
        cache: Some(true),
    }
}

/// 扩展拉丁字符集 - 最完整的规则集合
pub fn get_charset_latin_extra() -> EncoderOptions {
    let mut replacer = get_advanced_replacer();
    replacer.extend(get_compact_replacer());
    
    let mut mapper = HashMap::new();
    mapper.extend(get_soundex_map());
    
    EncoderOptions {
        mapper: Some(mapper),
        replacer: Some(replacer),
        matcher: Some(get_advanced_matcher()),
        rtl: Some(false),
        dedupe: Some(true),
        split: None,
        numeric: Some(true),
        normalize: Some(true),
        prepare: None,
        finalize: None,
        filter: None,
        stemmer: None,
        minlength: Some(1),
        maxlength: Some(1024),
        cache: Some(true),
    }
}

/// Soundex 编码配置
pub fn get_charset_latin_soundex() -> EncoderOptions {
    EncoderOptions {
        dedupe: Some(false),
        ..Default::default()
    }
}

// ========== 测试 ==========
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_soundex_encode() {
        assert_eq!(soundex_encode("Smith"), "S53");
        assert_eq!(soundex_encode("Smythe"), "S53");
        assert_eq!(soundex_encode("Schmidt"), "S53");
    }

    #[test]
    fn test_charset_latin_balance() {
        let options = get_charset_latin_balance();
        assert!(options.mapper.is_some());
        let mapper = options.mapper.unwrap();
        assert_eq!(mapper.get(&'b'), Some(&'p'));
        assert_eq!(mapper.get(&'v'), Some(&'f'));
    }

    #[test]
    fn test_charset_latin_advanced() {
        let options = get_charset_latin_advanced();
        assert!(options.mapper.is_some());
        assert!(options.matcher.is_some());
        assert!(options.replacer.is_some());
        
        let matcher = options.matcher.unwrap();
        assert_eq!(matcher.get("ae"), Some(&"a".to_string()));
        assert_eq!(matcher.get("oe"), Some(&"o".to_string()));
    }

    #[test]
    fn test_charset_latin_extra() {
        let options = get_charset_latin_extra();
        assert!(options.mapper.is_some());
        assert!(options.replacer.is_some());
        assert!(options.matcher.is_some());
        
        let replacer = options.replacer.unwrap();
        // Should have more replacers than just advanced due to compact
        assert!(replacer.len() > 2);
    }

    #[test]
    fn test_charset_latin_soundex() {
        let options = get_charset_latin_soundex();
        assert_eq!(options.dedupe, Some(false));
    }

    #[test]
    fn test_latin_polyfill() {
        let polyfill = get_latin_polyfill();
        assert_eq!(polyfill.get(&'à'), Some(&"a"));
        assert_eq!(polyfill.get(&'é'), Some(&"e"));
        assert_eq!(polyfill.get(&'ñ'), Some(&"n"));
        // β is not in the latin mapping, so it should return None
        assert_eq!(polyfill.get(&'β'), None);
    }

    #[test]
    fn test_normalize_latin() {
        let result = normalize_latin("Héllo Wörld");
        assert_eq!(result, "Hello World");
    }

    #[test]
    fn test_normalize_latin_multiple() {
        let result = normalize_latin("café naïve");
        assert_eq!(result, "cafe naive");
    }
}

// ========== 字符集工具函数 ==========

/// 获取拉丁字符填充映射表 - 用于字符标准化
pub fn get_latin_polyfill() -> HashMap<char, &'static str> {
    let mut map = HashMap::new();

    // Latin letters with diacritics
    map.insert('ª', "a");
    map.insert('º', "o");
    map.insert('à', "a");
    map.insert('á', "a");
    map.insert('â', "a");
    map.insert('ã', "a");
    map.insert('ä', "a");
    map.insert('å', "a");
    map.insert('ç', "c");
    map.insert('è', "e");
    map.insert('é', "e");
    map.insert('ê', "e");
    map.insert('ë', "e");
    map.insert('ì', "i");
    map.insert('í', "i");
    map.insert('î', "i");
    map.insert('ï', "i");
    map.insert('ñ', "n");
    map.insert('ò', "o");
    map.insert('ó', "o");
    map.insert('ô', "o");
    map.insert('õ', "o");
    map.insert('ö', "o");
    map.insert('ù', "u");
    map.insert('ú', "u");
    map.insert('û', "u");
    map.insert('ü', "u");
    map.insert('ý', "y");
    map.insert('ÿ', "y");
    
    // Extended Latin
    map.insert('ā', "a");
    map.insert('ă', "a");
    map.insert('ą', "a");
    map.insert('ć', "c");
    map.insert('ĉ', "c");
    map.insert('ċ', "c");
    map.insert('č', "c");
    map.insert('ď', "d");
    map.insert('ē', "e");
    map.insert('ĕ', "e");
    map.insert('ė', "e");
    map.insert('ę', "e");
    map.insert('ě', "e");
    map.insert('ĝ', "g");
    map.insert('ğ', "g");
    map.insert('ġ', "g");
    map.insert('ģ', "g");
    map.insert('ĥ', "h");
    map.insert('ĩ', "i");
    map.insert('ī', "i");
    map.insert('ĭ', "i");
    map.insert('į', "i");
    map.insert('ĳ', "ij");
    map.insert('ĵ', "j");
    map.insert('ķ', "k");
    map.insert('ĺ', "l");
    map.insert('ļ', "l");
    map.insert('ľ', "l");
    map.insert('ŀ', "l");
    map.insert('ń', "n");
    map.insert('ņ', "n");
    map.insert('ň', "n");
    map.insert('ŉ', "n");
    map.insert('ō', "o");
    map.insert('ŏ', "o");
    map.insert('ő', "o");
    map.insert('ŕ', "r");
    map.insert('ŗ', "r");
    map.insert('ř', "r");
    map.insert('ś', "s");
    map.insert('ŝ', "s");
    map.insert('ş', "s");
    map.insert('š', "s");
    map.insert('ţ', "t");
    map.insert('ť', "t");
    map.insert('ũ', "u");
    map.insert('ū', "u");
    map.insert('ŭ', "u");
    map.insert('ů', "u");
    map.insert('ű', "u");
    map.insert('ų', "u");
    map.insert('ŵ', "w");
    map.insert('ŷ', "y");
    map.insert('ź', "z");
    map.insert('ż', "z");
    map.insert('ž', "z");
    map.insert('ſ', "s");

    map
}

/// 标准化拉丁字符串中的特殊字符
pub fn normalize_latin(str: &str) -> String {
    let polyfill = get_latin_polyfill();
    let mut result = String::new();

    for char in str.chars() {
        if let Some(replacement) = polyfill.get(&char) {
            result.push_str(replacement);
        } else {
            result.push(char);
        }
    }

    result
}