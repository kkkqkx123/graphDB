//! 查询指纹模块
//!
//! 提供查询归一化和指纹生成功能。
//! 参考PostgreSQL的pg_stat_statements模块实现。

/// 查询指纹归一化
///
/// 将查询字符串归一化为标准形式，用于生成查询指纹。
/// 参考PostgreSQL的pg_stat_statements模块实现。
///
/// # 归一化规则
/// 1. 去除首尾空白
/// 2. 将多个空白字符替换为单个空格
/// 3. 将字符串常量替换为占位符($1, $2, ...)
/// 4. 将数字常量替换为占位符
/// 5. 统一转换为小写
///
/// # 示例
/// ```
/// use graphdb::query::optimizer::stats::feedback::fingerprint::normalize_query;
///
/// let query = "SELECT * FROM users WHERE id = 123";
/// let normalized = normalize_query(query);
/// assert_eq!(normalized, "select * from users where id = $1");
/// ```
pub fn normalize_query(query: &str) -> String {
    // 1. 去除首尾空白
    let trimmed = query.trim();

    // 2. 将多个空白字符替换为单个空格
    let normalized_whitespace = trimmed.split_whitespace().collect::<Vec<_>>().join(" ");

    // 3. 将字符串常量替换为占位符
    let mut result = String::new();
    let mut in_string = false;
    let mut param_count = 0;

    let chars: Vec<char> = normalized_whitespace.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];

        if !in_string {
            // 检查字符串开始
            if c == '\'' || c == '"' {
                in_string = true;
                param_count += 1;
                result.push_str(&format!("${}", param_count));
                // 跳过直到字符串结束
                let string_char = c;
                i += 1;
                while i < chars.len() {
                    if chars[i] == string_char {
                        // 检查是否是转义
                        if i + 1 < chars.len() && chars[i + 1] == string_char {
                            i += 2;
                            continue;
                        }
                        break;
                    }
                    i += 1;
                }
            } else if c.is_ascii_digit() {
                // 将数字常量替换为占位符
                param_count += 1;
                result.push_str(&format!("${}", param_count));
                // 跳过连续的数字
                while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                    i += 1;
                }
                continue;
            } else {
                // 统一转换为小写
                result.push(c.to_ascii_lowercase());
            }
        }

        i += 1;
    }

    result
}

/// 生成查询指纹
///
/// 基于归一化后的查询字符串生成唯一指纹。
/// 使用FNV-1a哈希算法。
///
/// # 示例
/// ```
/// use graphdb::query::optimizer::stats::feedback::fingerprint::generate_query_fingerprint;
///
/// let query1 = "SELECT * FROM users WHERE id = 1";
/// let query2 = "SELECT * FROM users WHERE id = 2";
/// let fp1 = generate_query_fingerprint(query1);
/// let fp2 = generate_query_fingerprint(query2);
/// // 相同结构的不同查询应该有相同的指纹
/// assert_eq!(fp1, fp2);
/// ```
pub fn generate_query_fingerprint(query: &str) -> String {
    let normalized = normalize_query(query);
    // 使用简单的FNV-1a哈希算法
    const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;

    let mut hash = FNV_OFFSET_BASIS;
    for byte in normalized.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }

    format!("{:016x}", hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_query() {
        let query1 = "SELECT * FROM users WHERE age > 25 AND name = 'John'";
        let normalized1 = normalize_query(query1);
        assert!(normalized1.contains("$1")); // 数字常量被替换
        assert!(normalized1.contains("$2")); // 字符串常量被替换
        assert!(normalized1.starts_with("select * from users where"));

        let query2 = "  SELECT   id  FROM   t   WHERE  x = 100  ";
        let normalized2 = normalize_query(query2);
        assert_eq!(normalized2, "select id from t where x = $1");
    }

    #[test]
    fn test_normalize_query_with_escaped_quotes() {
        let query = "SELECT * FROM t WHERE name = 'O''Brien'";
        let normalized = normalize_query(query);
        assert!(normalized.contains("$1"));
        assert!(normalized.starts_with("select * from t where"));
    }

    #[test]
    fn test_generate_query_fingerprint() {
        let query1 = "SELECT * FROM users WHERE id = 1";
        let query2 = "SELECT * FROM users WHERE id = 2";
        let fp1 = generate_query_fingerprint(query1);
        let fp2 = generate_query_fingerprint(query2);
        // 相同结构的不同查询应该有相同的指纹
        assert_eq!(fp1, fp2);

        // 不同结构的查询应该有不同的指纹
        let query3 = "SELECT * FROM orders WHERE id = 1";
        let fp3 = generate_query_fingerprint(query3);
        assert_ne!(fp1, fp3);
    }
}
