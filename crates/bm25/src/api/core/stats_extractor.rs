//! TF/DF 统计信息提取工具
//!
//! 从文档内容中提取词项频率（TF）和文档频率（DF）统计信息

use crate::storage::Bm25Stats;
use std::collections::HashMap;

/// 从文档字段中提取 TF/DF 统计信息
///
/// # Arguments
///
/// * `fields` - 文档字段映射
/// * `total_docs` - 当前总文档数
/// * `avg_doc_length` - 平均文档长度
///
/// # Returns
///
/// * `Bm25Stats` - 提取的统计信息
pub fn extract_tf_df_stats(
    fields: &HashMap<String, String>,
    total_docs: u64,
    avg_doc_length: f32,
) -> Bm25Stats {
    let mut tf_map: HashMap<String, f32> = HashMap::new();
    let mut df_set: HashMap<String, ()> = HashMap::new(); // 使用 HashSet 的替代方案

    // 遍历所有字段，提取词项
    for content in fields.values() {
        // 简单的分词：按空白字符分割并转为小写
        // TODO: 支持多语言分词（中文、日文等）
        let terms: Vec<String> = content
            .split_whitespace()
            .map(|t| t.to_lowercase())
            .collect();

        // 统计词项频率
        let mut term_counts: HashMap<String, u64> = HashMap::new();
        for term in terms {
            *term_counts.entry(term).or_insert(0) += 1;
        }

        // 更新 TF
        for (term, count) in term_counts {
            // TF: 词项在文档中的出现次数（跨字段累加）
            *tf_map.entry(term.clone()).or_insert(0.0) += count as f32;

            // 记录该词项在当前文档中出现（用于计算 DF）
            df_set.entry(term).or_insert(());
        }
    }

    // 将 df_set 转换为 df_map
    let df_map: HashMap<String, u64> = df_set.into_keys().map(|term| (term, 1)).collect();

    Bm25Stats {
        tf: tf_map,
        df: df_map,
        total_docs,
        avg_doc_length,
    }
}

/// 批量从多个文档中提取 TF/DF 统计信息
///
/// # Arguments
///
/// * `documents` - 文档列表，每个文档包含 (document_id, fields)
/// * `total_docs` - 当前总文档数
/// * `avg_doc_length` - 平均文档长度
///
/// # Returns
///
/// * `Bm25Stats` - 合并后的统计信息
pub fn extract_batch_tf_df_stats(
    documents: &[(String, HashMap<String, String>)],
    total_docs: u64,
    avg_doc_length: f32,
) -> Bm25Stats {
    let mut combined_stats = Bm25Stats::default();

    for (_doc_id, fields) in documents {
        let doc_stats = extract_tf_df_stats(fields, total_docs, avg_doc_length);

        // 合并 TF
        for (term, tf) in doc_stats.tf {
            *combined_stats.tf.entry(term).or_insert(0.0) += tf;
        }

        // 合并 DF
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
        fields.insert("title".to_string(), "Hello World".to_string());
        fields.insert("content".to_string(), "Hello Rust World".to_string());

        let stats = extract_tf_df_stats(&fields, 10, 100.0);

        // TF 统计
        assert_eq!(stats.tf.get("hello"), Some(&2.0));
        assert_eq!(stats.tf.get("world"), Some(&2.0));
        assert_eq!(stats.tf.get("rust"), Some(&1.0));

        // DF 统计
        assert_eq!(stats.df.get("hello"), Some(&1));
        assert_eq!(stats.df.get("world"), Some(&1));
        assert_eq!(stats.df.get("rust"), Some(&1));

        assert_eq!(stats.total_docs, 10);
        assert_eq!(stats.avg_doc_length, 100.0);
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
