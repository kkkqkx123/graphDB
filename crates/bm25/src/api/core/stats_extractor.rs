use crate::storage::Bm25Stats;
use crate::tokenizer::MixedTokenizer;
use std::collections::HashMap;

pub fn extract_tf_df_stats(
    fields: &HashMap<String, String>,
    total_docs: u64,
    avg_doc_length: f32,
) -> Bm25Stats {
    let mut tf_map: HashMap<String, f32> = HashMap::new();
    let mut df_set: HashMap<String, ()> = HashMap::new();

    let tokenizer = MixedTokenizer::new();

    for content in fields.values() {
        let raw_tokens = tokenizer.tokenize_text(content);
        let terms: Vec<String> = raw_tokens.into_iter().map(|(t, _, _)| t).collect();

        let mut term_counts: HashMap<String, u64> = HashMap::new();
        for term in terms {
            *term_counts.entry(term).or_insert(0) += 1;
        }

        for (term, count) in term_counts {
            *tf_map.entry(term.clone()).or_insert(0.0) += count as f32;
            df_set.entry(term).or_insert(());
        }
    }

    let df_map: HashMap<String, u64> = df_set.into_keys().map(|term| (term, 1)).collect();

    Bm25Stats {
        tf: tf_map,
        df: df_map,
        total_docs,
        avg_doc_length,
    }
}

pub fn extract_batch_tf_df_stats(
    documents: &[(String, HashMap<String, String>)],
    total_docs: u64,
    avg_doc_length: f32,
) -> Bm25Stats {
    let mut combined_stats = Bm25Stats::default();

    for (_doc_id, fields) in documents {
        let doc_stats = extract_tf_df_stats(fields, total_docs, avg_doc_length);

        for (term, tf) in doc_stats.tf {
            *combined_stats.tf.entry(term).or_insert(0.0) += tf;
        }

        for (term, df) in doc_stats.df {
            *combined_stats.df.entry(term).or_insert(0) += df;
        }

        combined_stats.total_docs = total_docs;
        combined_stats.avg_doc_length = avg_doc_length;
    }

    combined_stats
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_single_doc_stats() {
        let mut fields = HashMap::new();
        fields.insert("content".to_string(), "Hello Rust World".to_string());

        let stats = extract_tf_df_stats(&fields, 10, 100.0);

        assert_eq!(stats.tf.get("hello"), Some(&1.0));
        assert_eq!(stats.tf.get("world"), Some(&1.0));
        assert_eq!(stats.tf.get("rust"), Some(&1.0));

        assert_eq!(stats.df.get("hello"), Some(&1));
        assert_eq!(stats.df.get("world"), Some(&1));
        assert_eq!(stats.df.get("rust"), Some(&1));

        assert_eq!(stats.total_docs, 10);
        assert_eq!(stats.avg_doc_length, 100.0);
    }

    #[test]
    fn test_extract_chinese_stats() {
        let mut fields = HashMap::new();
        fields.insert("content".to_string(), "计算总价的方法".to_string());

        let stats = extract_tf_df_stats(&fields, 10, 100.0);

        assert!(
            stats.tf.contains_key("计算"),
            "Expected '计算' in tf: {:?}",
            stats.tf
        );
        assert!(
            stats.tf.contains_key("总价"),
            "Expected '总价' in tf: {:?}",
            stats.tf
        );
    }

    #[test]
    fn test_extract_empty_fields() {
        let fields: HashMap<String, String> = HashMap::new();
        let stats = extract_tf_df_stats(&fields, 5, 50.0);

        assert!(stats.tf.is_empty());
        assert!(stats.df.is_empty());
        assert_eq!(stats.total_docs, 5);
        assert_eq!(stats.avg_doc_length, 50.0);
    }
}
