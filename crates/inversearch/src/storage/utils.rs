//! 存储模块工具函数
//!
//! 提供各存储实现共享的辅助函数

use crate::r#type::{SearchResults, DocId};

/// 应用限制和偏移的辅助函数
pub fn apply_limit_offset(results: &[DocId], limit: usize, offset: usize) -> SearchResults {
    if results.is_empty() {
        return Vec::new();
    }

    let start = offset.min(results.len());
    let end = if limit > 0 {
        (start + limit).min(results.len())
    } else {
        results.len()
    };

    results[start..end].to_vec()
}
