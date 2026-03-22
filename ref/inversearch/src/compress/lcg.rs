const DEFAULT_BIT: u32 = 32;
const DEFAULT_BIT_PLUS: u32 = DEFAULT_BIT + 1;
const DEFAULT_RANGE: u64 = 2_u64.pow(DEFAULT_BIT) - 1;
const UINT32_MAX: u64 = u32::MAX as u64;

pub fn lcg(input: &str) -> u64 {
    if input.is_empty() {
        return DEFAULT_RANGE;
    }

    let mut crc: u64 = 0;
    let bit = DEFAULT_BIT_PLUS;

    for c in input.chars() {
        crc = (crc.wrapping_mul(bit as u64)).wrapping_add(c as u64) ^ crc;
    }

    crc.wrapping_add(2_u64.pow(DEFAULT_BIT - 1))
}

pub fn lcg64(input: &str) -> u64 {
    lcg(input)
}

pub fn lcg_for_number<T: Into<u64>>(num: T, _bit: u32) -> u64 {
    let num_val: u64 = num.into();
    num_val & DEFAULT_RANGE
}

pub fn lcg_for_u32(num: u32) -> u64 {
    num as u64 & DEFAULT_RANGE
}

pub fn lcg_result_to_u32(hash: u64) -> u32 {
    (hash & DEFAULT_RANGE) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lcg_basic() {
        let hash = lcg("hello");
        assert!(hash > 0);
        assert!(hash <= u64::MAX);
    }

    #[test]
    fn test_lcg_deterministic() {
        let hash1 = lcg("test");
        let hash2 = lcg("test");
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_lcg_different_inputs() {
        let hash1 = lcg("a");
        let hash2 = lcg("b");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_lcg_empty() {
        let hash = lcg("");
        assert_eq!(hash, DEFAULT_RANGE);
    }

    #[test]
    fn test_lcg_number() {
        let hash = lcg_for_number(42u32, DEFAULT_BIT);
        assert_eq!(hash, 42);
    }

    #[test]
    fn test_lcg_max_number() {
        let hash = lcg_for_number(u32::MAX, DEFAULT_BIT);
        assert_eq!(hash, u32::MAX as u64);
    }

    #[test]
    fn test_lcg64() {
        let hash1 = lcg64("hello");
        let hash2 = lcg("hello");
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_lcg_unicode() {
        let hash = lcg("你好");
        assert!(hash > 0);
    }

    #[test]
    fn test_lcg_special_chars() {
        let hash = lcg("hello\n\t\r");
        assert!(hash > 0);
    }
}
