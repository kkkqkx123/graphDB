use std::collections::{HashMap, HashSet};

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
    supported_charsets: HashSet<String>,
    supported_collations: HashSet<String>,
    charset_descriptions: HashMap<String, CharsetDesc>,
}

impl CharsetManager {
    pub fn new() -> Self {
        let mut manager = Self {
            supported_charsets: HashSet::new(),
            supported_collations: HashSet::new(),
            charset_descriptions: HashMap::new(),
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

    pub fn charset_and_collate_match(&self, charset_name: &str, collation_name: &str) -> bool {
        if let Some(desc) = self.charset_descriptions.get(&charset_name.to_lowercase()) {
            desc.supported_collations
                .iter()
                .any(|c| c.to_lowercase() == collation_name.to_lowercase())
        } else {
            false
        }
    }

    pub fn get_default_collation_by_charset(&self, charset_name: &str) -> Option<String> {
        self.charset_descriptions
            .get(&charset_name.to_lowercase())
            .map(|desc| desc.default_collation.clone())
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

/// 字符集工具类
pub struct CharsetUtils;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_charset_validation() {
        assert!(CharsetUtils::validate_charset_and_collation("utf8mb4", "utf8mb4_bin").is_ok());
        assert!(CharsetUtils::validate_charset_and_collation("utf8", "utf8_bin").is_ok());
        assert!(CharsetUtils::validate_charset_and_collation("invalid", "utf8mb4_bin").is_err());
    }

    #[test]
    fn test_supported_charsets() {
        let charsets = CharsetUtils::get_supported_charsets();
        assert!(charsets.contains(&"utf8".to_string()));
        assert!(charsets.contains(&"utf8mb4".to_string()));
        assert!(charsets.contains(&"latin1".to_string()));
        assert!(charsets.contains(&"gbk".to_string()));
        assert!(charsets.contains(&"big5".to_string()));
    }

    #[test]
    fn test_default_collation() {
        assert_eq!(
            CharsetUtils::get_default_collation("utf8mb4"),
            Some("utf8mb4_bin".to_string())
        );
        assert_eq!(
            CharsetUtils::get_default_collation("utf8"),
            Some("utf8_bin".to_string())
        );
    }
}
