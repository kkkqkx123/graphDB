mod lcg;
mod radix;
mod cache;

pub use lcg::{lcg, lcg64, lcg_for_number};
pub use radix::{to_radix, RadixTable};
pub use cache::{CompressCache, compress_with_cache};

pub const DEFAULT_CACHE_SIZE: usize = 200_000;

pub fn compress_string(input: &str) -> String {
    compress_with_cache(input, DEFAULT_CACHE_SIZE)
}

pub fn compress_string_with_options(input: &str, cache_size: usize) -> String {
    compress_with_cache(input, cache_size)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_string() {
        let result = compress_string("hello");
        assert!(!result.is_empty());
        assert!(result.len() <= 8);
    }

    #[test]
    fn test_compress_number() {
        let result = compress_string("123");
        assert!(!result.is_empty());
    }

    #[test]
    fn test_compress_unicode() {
        let result = compress_string("你好");
        assert!(!result.is_empty());
    }

    #[test]
    fn test_compress_empty() {
        let result = compress_string("");
        assert!(!result.is_empty());
    }

    #[test]
    fn test_radix_basic() {
        let result = to_radix(255);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_radix_custom_radix() {
        let table = "0123456789ABCDEF";
        let result = radix::to_radix_with_table(16, table);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_lcg_string() {
        let hash = lcg("test");
        assert!(hash > 0);
    }

    #[test]
    fn test_lcg_number() {
        let hash = lcg_for_number(42u64, 32);
        assert_eq!(hash, 42);
    }

    #[test]
    fn test_lcg64() {
        let hash = lcg64("test");
        assert!(hash > 0);
    }

    #[test]
    fn test_compress_deterministic() {
        let result1 = compress_string("hello");
        let result2 = compress_string("hello");
        assert_eq!(result1, result2);
    }
}
