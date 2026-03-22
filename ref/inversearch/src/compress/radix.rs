use std::sync::OnceLock;

const DEFAULT_RADIX: usize = 255;

static RADIX_TABLE: OnceLock<String> = OnceLock::new();

fn get_radix_table() -> &'static str {
    RADIX_TABLE.get_or_init(|| {
        let mut table = Vec::with_capacity(DEFAULT_RADIX);
        for i in 0..DEFAULT_RADIX {
            table.push((i + 1) as u8);
        }
        String::from_utf8(table).unwrap_or_else(|_| {
            let mut fallback = String::with_capacity(DEFAULT_RADIX);
            for i in 0..DEFAULT_RADIX {
                fallback.push((i as u8 + 1) as char);
            }
            fallback
        })
    })
}

pub fn to_radix_u64(mut number: u64, radix: usize) -> String {
    let table = get_radix_table();
    
    if radix > DEFAULT_RADIX {
        panic!("Radix {} exceeds maximum allowed ({})", radix, DEFAULT_RADIX);
    }

    if number == 0 {
        return table.chars().next().unwrap().to_string();
    }

    let mut result = String::new();
    
    while number > 0 {
        let rixit = number % radix as u64;
        result.insert(0, table.chars().nth(rixit as usize).unwrap_or('\0'));
        number /= radix as u64;
    }

    result
}

pub fn to_radix_u32(number: u32, radix: usize) -> String {
    to_radix_u64(number as u64, radix)
}

pub fn to_radix_usize(number: usize, radix: usize) -> String {
    to_radix_u64(number as u64, radix)
}

pub fn to_radix(number: u64) -> String {
    to_radix_u64(number, DEFAULT_RADIX)
}

pub fn to_radix_with_table(number: u64, custom_table: &str) -> String {
    if number == 0 {
        return custom_table.chars().next().unwrap().to_string();
    }

    let mut result = String::new();
    let mut num = number;
    let radix = custom_table.len();

    while num > 0 {
        let rixit = num % radix as u64;
        result.insert(0, custom_table.chars().nth(rixit as usize).unwrap_or('\0'));
        num /= radix as u64;
    }

    result
}

pub struct RadixTable {
    table: String,
    radix: usize,
}

impl RadixTable {
    pub fn new(radix: usize) -> Self {
        if radix > DEFAULT_RADIX {
            panic!("Radix {} exceeds maximum allowed ({})", radix, DEFAULT_RADIX);
        }

        let mut table = Vec::with_capacity(radix);
        for i in 0..radix {
            table.push((i + 1) as u8);
        }

        RadixTable {
            table: String::from_utf8(table).unwrap_or_else(|_| String::with_capacity(radix)),
            radix,
        }
    }

    pub fn from_custom_table(table: &str) -> Self {
        RadixTable {
            table: table.to_string(),
            radix: table.len(),
        }
    }

    pub fn encode(&self, mut number: u64) -> String {
        if number == 0 {
            return self.table.chars().next().unwrap().to_string();
        }

        let mut result = String::new();
        
        while number > 0 {
            let rixit = number % self.radix as u64;
            result.insert(0, self.table.chars().nth(rixit as usize).unwrap_or('\0'));
            number /= self.radix as u64;
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_radix_basic() {
        let result = to_radix(255);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_to_radix_zero() {
        let result = to_radix(0);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_to_radix_one() {
        let result = to_radix(1);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_to_radix_custom_radix() {
        let result = to_radix_u64(16, 16);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_to_radix_binary() {
        let result = to_radix_u64(255, 2);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_to_radix_hex() {
        let result = to_radix_u64(255, 16);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_to_radix_with_table() {
        let table = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
        let result = to_radix_with_table(255, table);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_radix_table_basic() {
        let table = RadixTable::new(255);
        let result = table.encode(255);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_radix_table_custom() {
        let table = RadixTable::from_custom_table("01");
        let result = table.encode(255);
        assert!(!result.contains('2'));
    }

    #[test]
    #[should_panic(expected = "exceeds maximum")]
    fn test_radix_too_large() {
        let _ = RadixTable::new(256);
    }
}
