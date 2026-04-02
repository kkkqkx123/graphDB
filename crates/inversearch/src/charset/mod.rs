pub mod exact;
pub mod normalize;
pub mod cjk;
pub mod latin;

// ========== 主要导出 ==========

// Basic charsets
pub use exact::get_charset_exact as charset_exact;
pub use normalize::get_charset_normalize as charset_normalize;
pub use cjk::get_charset_cjk as charset_cjk;

// Latin charsets  
pub use latin::get_charset_latin_balance as charset_latin_balance;
pub use latin::get_charset_latin_advanced as charset_latin_advanced;
pub use latin::get_charset_latin_extra as charset_latin_extra;
pub use latin::get_charset_latin_soundex as charset_latin_soundex;

// ========== 便捷函数 ==========

pub fn get_charset_exact() -> crate::r#type::EncoderOptions {
    exact::get_charset_exact()
}

pub fn get_charset_default() -> crate::r#type::EncoderOptions {
    normalize::get_charset_normalize()
}

pub fn get_charset_normalize() -> crate::r#type::EncoderOptions {
    normalize::get_charset_normalize()
}

pub fn get_charset_cjk() -> crate::r#type::EncoderOptions {
    cjk::get_charset_cjk()
}

// Latin charset functions
pub fn get_charset_latin_balance() -> crate::r#type::EncoderOptions {
    latin::get_charset_latin_balance()
}

pub fn get_charset_latin_advanced() -> crate::r#type::EncoderOptions {
    latin::get_charset_latin_advanced()
}

pub fn get_charset_latin_extra() -> crate::r#type::EncoderOptions {
    latin::get_charset_latin_extra()
}

pub fn get_charset_latin_soundex() -> crate::r#type::EncoderOptions {
    latin::get_charset_latin_soundex()
}

// ========== 测试 ==========
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_charset_polyfill() {
        let polyfill = latin::get_latin_polyfill();
        assert_eq!(polyfill.get(&'à'), Some(&"a"));
        assert_eq!(polyfill.get(&'é'), Some(&"e"));
        assert_eq!(polyfill.get(&'ñ'), Some(&"n"));
        assert_eq!(polyfill.get(&'β'), None);
    }

    #[test]
    fn test_normalize_charset() {
        let result = latin::normalize_latin("Héllo Wörld");
        assert_eq!(result, "Hello World");
    }

    #[test]
    fn test_normalize_charset_multiple() {
        let result = latin::normalize_latin("café naïve");
        assert_eq!(result, "cafe naive");
    }

    #[test]
    fn test_normalize_charset_cyrillic() {
        let result = latin::normalize_latin("йё");
        assert_eq!(result, "йё");
    }

    #[test]
    fn test_normalize_charset_greek() {
        let result = latin::normalize_latin("άέή");
        assert_eq!(result, "άέή");
    }
}