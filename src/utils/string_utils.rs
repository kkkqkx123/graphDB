/// Escapes special characters in a string for use in queries
pub fn escape_for_query(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('\"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// Unescapes special characters in a string
pub fn unescape_for_query(s: &str) -> String {
    s.replace("\\t", "\t")
        .replace("\\r", "\r")
        .replace("\\n", "\n")
        .replace("\\\"", "\"")
        .replace("\\'", "'")
        .replace("\\\\", "\\")
}

/// Normalizes identifier names (table names, column names, etc.)
pub fn normalize_identifier(name: &str) -> String {
    // Replace spaces with underscores and convert to lowercase
    name.trim()
        .replace(' ', "_")
        .replace('-', "_")
        .to_lowercase()
}

/// Sanitizes input to prevent injection attacks
pub fn sanitize_input(input: &str) -> String {
    // Remove potentially dangerous characters/sequences
    input
        .replace(";", "")
        .replace("--", "")
        .replace("/*", "")
        .replace("*/", "")
        .replace("xp_", "") // Prevent calls to extended procedures
        .replace("sp_", "") // Prevent calls to stored procedures
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_utils() {
        let original = "Hello\nWorld\t\"Test\"";
        let escaped = escape_for_query(original);
        assert_eq!(escaped, "Hello\\nWorld\\t\\\"Test\\\"");

        let unescaped = unescape_for_query(&escaped);
        assert_eq!(unescaped, original);
    }

    #[test]
    fn test_normalize_identifier() {
        assert_eq!(normalize_identifier("User Name"), "user_name");
        assert_eq!(normalize_identifier("  Table-Name  "), "table_name");
        assert_eq!(normalize_identifier("CamelCase"), "camelcase");
        assert_eq!(normalize_identifier("UPPER_CASE"), "upper_case");
    }

    #[test]
    fn test_sanitize_input() {
        let malicious = "SELECT * FROM users; DROP TABLE users; --";
        let sanitized = sanitize_input(malicious);
        assert_eq!(sanitized, "SELECT * FROM users DROP TABLE users ");

        let with_procedures = "EXEC xp_cmdshell 'dir'; EXEC sp_who2;";
        let sanitized2 = sanitize_input(with_procedures);
        assert_eq!(sanitized2, "EXEC cmdshell 'dir' EXEC who2");
    }

    #[test]
    fn test_escape_unescape_roundtrip() {
        let test_cases = vec![
            "Simple string",
            "String with 'quotes'",
            "String with \"double quotes\"",
            "String with\\backslash",
            "String with\nnewline",
            "String with\ttab",
            "String with\rcarriage return",
            "Complex 'string' with \"multiple\" \\special\\ characters\nand\ttabs",
        ];

        for original in test_cases {
            let escaped = escape_for_query(original);
            let unescaped = unescape_for_query(&escaped);
            assert_eq!(original, unescaped, "Roundtrip failed for: {}", original);
        }
    }
}
