/// String algorithms
pub struct StringAlgorithms;

impl StringAlgorithms {
    /// Compute the Levenshtein distance (edit distance) between two strings
    pub fn levenshtein_distance(s1: &str, s2: &str) -> usize {
        let s1_chars: Vec<char> = s1.chars().collect();
        let s2_chars: Vec<char> = s2.chars().collect();
        let s1_len = s1_chars.len();
        let s2_len = s2_chars.len();

        if s1_len == 0 {
            return s2_len;
        }
        if s2_len == 0 {
            return s1_len;
        }

        let mut matrix = vec![vec![0; s2_len + 1]; s1_len + 1];

        for i in 0..=s1_len {
            matrix[i][0] = i;
        }
        for j in 0..=s2_len {
            matrix[0][j] = j;
        }

        for i in 1..=s1_len {
            for j in 1..=s2_len {
                let cost = if s1_chars[i - 1] == s2_chars[j - 1] {
                    0
                } else {
                    1
                };
                matrix[i][j] = std::cmp::min(
                    std::cmp::min(
                        matrix[i - 1][j] + 1, // deletion
                        matrix[i][j - 1] + 1, // insertion
                    ),
                    matrix[i - 1][j - 1] + cost, // substitution
                );
            }
        }

        matrix[s1_len][s2_len]
    }

    /// Find all occurrences of a pattern in a text using naive string matching
    pub fn find_pattern_naive(text: &str, pattern: &str) -> Vec<usize> {
        let mut matches = Vec::new();

        if pattern.is_empty() {
            return matches;
        }

        let text_chars: Vec<char> = text.chars().collect();
        let pattern_chars: Vec<char> = pattern.chars().collect();

        for i in 0..=(text_chars.len() - pattern_chars.len()) {
            let mut found = true;
            for j in 0..pattern_chars.len() {
                if text_chars[i + j] != pattern_chars[j] {
                    found = false;
                    break;
                }
            }
            if found {
                matches.push(i);
            }
        }

        matches
    }

    /// KMP (Knuth-Morris-Pratt) algorithm for pattern matching
    pub fn kmp_search(text: &str, pattern: &str) -> Vec<usize> {
        if pattern.is_empty() {
            return vec![];
        }

        let text_chars: Vec<char> = text.chars().collect();
        let pattern_chars: Vec<char> = pattern.chars().collect();

        // Preprocess the pattern to create the LPS array
        let lps = Self::compute_lps_array(&pattern_chars);

        let mut matches = Vec::new();
        let mut text_idx = 0;
        let mut pattern_idx = 0;

        while text_idx < text_chars.len() {
            if pattern_chars[pattern_idx] == text_chars[text_idx] {
                text_idx += 1;
                pattern_idx += 1;
            }

            if pattern_idx == pattern_chars.len() {
                matches.push(text_idx - pattern_idx);
                pattern_idx = lps[pattern_idx - 1];
            } else if text_idx < text_chars.len()
                && pattern_chars[pattern_idx] != text_chars[text_idx]
            {
                if pattern_idx != 0 {
                    pattern_idx = lps[pattern_idx - 1];
                } else {
                    text_idx += 1;
                }
            }
        }

        matches
    }

    fn compute_lps_array(pattern: &[char]) -> Vec<usize> {
        let mut lps = vec![0; pattern.len()];
        let mut len = 0;
        let mut idx = 1;

        while idx < pattern.len() {
            if pattern[idx] == pattern[len] {
                len += 1;
                lps[idx] = len;
                idx += 1;
            } else {
                if len != 0 {
                    len = lps[len - 1];
                } else {
                    lps[idx] = 0;
                    idx += 1;
                }
            }
        }

        lps
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(
            StringAlgorithms::levenshtein_distance("kitten", "sitting"),
            3
        );
        assert_eq!(StringAlgorithms::levenshtein_distance("hello", "hello"), 0);
    }

    #[test]
    fn test_find_pattern_naive() {
        let text = "ABABDABACDABABCABCABCABCABC";
        let pattern = "ABABC";
        let matches = StringAlgorithms::find_pattern_naive(text, pattern);
        assert_eq!(matches, vec![10]);
    }

    #[test]
    fn test_kmp_search() {
        let text = "ABABDABACDABABCABCABCABCABC";
        let pattern = "ABABCABCAB";
        let matches = StringAlgorithms::kmp_search(text, pattern);
        assert!(!matches.is_empty());
    }
}
