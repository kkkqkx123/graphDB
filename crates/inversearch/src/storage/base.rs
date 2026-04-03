//! 存储基类实现
//!
//! 提供各存储实现共享的数据结构和核心逻辑

use crate::r#type::{SearchResults, EnrichedSearchResults, DocId};
use crate::Index;

use crate::storage::utils::apply_limit_offset;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

/// 存储基类
///
/// 封装所有内存存储实现共享的数据结构和核心逻辑：
/// - 索引数据存储
/// - 上下文数据存储
/// - 文档内容存储
/// - 性能指标统计
#[derive(Debug)]
pub struct StorageBase {
    /// 主索引数据：词项 -> 文档ID列表
    pub data: HashMap<String, Vec<DocId>>,
    /// 上下文索引数据：上下文 -> 词项 -> 文档ID列表
    pub context_data: HashMap<String, HashMap<String, Vec<DocId>>>,
    /// 文档内容存储：文档ID -> 内容
    pub documents: HashMap<DocId, String>,
    /// 内存使用量（字节）
    pub(crate) memory_usage: AtomicUsize,
    /// 操作计数
    pub(crate) operation_count: AtomicUsize,
    /// 总延迟（微秒）
    pub(crate) total_latency: AtomicUsize,
}

impl StorageBase {
    /// 创建新的存储基类实例
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            context_data: HashMap::new(),
            documents: HashMap::new(),
            memory_usage: AtomicUsize::new(0),
            operation_count: AtomicUsize::new(0),
            total_latency: AtomicUsize::new(0),
        }
    }

    /// 从索引提交数据到存储
    ///
    /// 将索引中的数据导出到存储基类中
    pub fn commit_from_index(&mut self, index: &Index) {
        // 从主索引导出数据
        for doc_ids in index.map.index.values() {
            for (term_str, ids) in doc_ids {
                self.data.insert(term_str.clone(), ids.clone());
            }
        }

        // 从上下文索引导出数据
        for ctx_map in index.ctx.index.values() {
            for (ctx_term, doc_ids) in ctx_map {
                self.context_data
                    .entry("default".to_string())
                    .or_default()
                    .insert(ctx_term.clone(), doc_ids.clone());
            }
        }

        self.update_memory_usage();
    }

    /// 获取指定键的搜索结果
    ///
    /// # 参数
    /// - `key`: 搜索词项
    /// - `ctx`: 可选的上下文名称
    /// - `limit`: 返回结果数量限制（0表示无限制）
    /// - `offset`: 结果偏移量
    pub fn get(&self, key: &str, ctx: Option<&str>, limit: usize, offset: usize) -> SearchResults {
        let results = if let Some(ctx_key) = ctx {
            // 上下文搜索
            if let Some(ctx_map) = self.context_data.get(ctx_key) {
                ctx_map.get(key).cloned().unwrap_or_default()
            } else {
                Vec::new()
            }
        } else {
            // 普通搜索
            self.data.get(key).cloned().unwrap_or_default()
        };

        apply_limit_offset(&results, limit, offset)
    }

    /// 富化搜索结果
    ///
    /// 根据文档ID列表获取完整的文档内容
    pub fn enrich(&self, ids: &[DocId]) -> EnrichedSearchResults {
        let mut results = Vec::new();

        for &id in ids {
            if let Some(content) = self.documents.get(&id) {
                results.push(crate::r#type::EnrichedSearchResult {
                    id,
                    doc: Some(serde_json::json!({
                        "content": content,
                        "id": id
                    })),
                    highlight: None,
                });
            }
        }

        results
    }

    /// 检查文档ID是否存在
    ///
    /// 在索引数据和上下文数据中搜索指定ID
    pub fn has(&self, id: DocId) -> bool {
        // 检查主索引数据
        for doc_ids in self.data.values() {
            if doc_ids.contains(&id) {
                return true;
            }
        }

        // 检查上下文数据
        for ctx_map in self.context_data.values() {
            for doc_ids in ctx_map.values() {
                if doc_ids.contains(&id) {
                    return true;
                }
            }
        }

        false
    }

    /// 移除指定文档
    ///
    /// 从文档存储、索引数据和上下文数据中删除指定ID
    pub fn remove(&mut self, ids: &[DocId]) {
        for &id in ids {
            self.documents.remove(&id);

            // 从主索引数据中移除
            for doc_ids in self.data.values_mut() {
                doc_ids.retain(|&doc_id| doc_id != id);
            }

            // 从上下文数据中移除
            for ctx_map in self.context_data.values_mut() {
                for doc_ids in ctx_map.values_mut() {
                    doc_ids.retain(|&doc_id| doc_id != id);
                }
            }
        }
    }

    /// 清空所有数据
    pub fn clear(&mut self) {
        self.data.clear();
        self.context_data.clear();
        self.documents.clear();
    }

    /// 获取内存使用量（字节）
    pub fn get_memory_usage(&self) -> usize {
        self.memory_usage.load(Ordering::Relaxed)
    }

    /// 获取操作计数
    pub fn get_operation_count(&self) -> usize {
        self.operation_count.load(Ordering::Relaxed)
    }

    /// 获取总延迟（微秒）
    pub fn get_total_latency(&self) -> usize {
        self.total_latency.load(Ordering::Relaxed)
    }

    /// 计算平均延迟（微秒）
    pub fn get_average_latency(&self) -> usize {
        let count = self.get_operation_count();
        if count > 0 {
            self.get_total_latency() / count
        } else {
            0
        }
    }

    /// 更新内存使用量统计
    ///
    /// 计算所有数据结构的内存占用
    pub fn update_memory_usage(&self) {
        let mut total_size = 0;

        // 计算主索引数据大小
        total_size += std::mem::size_of_val(&self.data);
        for (k, v) in &self.data {
            total_size += k.len() + v.len() * std::mem::size_of::<DocId>();
        }

        // 计算上下文数据大小
        total_size += std::mem::size_of_val(&self.context_data);
        for (ctx_key, ctx_map) in &self.context_data {
            total_size += ctx_key.len();
            total_size += std::mem::size_of_val(ctx_map);
            for (term, ids) in ctx_map {
                total_size += term.len() + ids.len() * std::mem::size_of::<DocId>();
            }
        }

        // 计算文档存储大小
        total_size += std::mem::size_of_val(&self.documents);
        for (id, content) in &self.documents {
            total_size += std::mem::size_of_val(id) + content.len();
        }

        self.memory_usage.store(total_size, Ordering::Relaxed);
    }

    /// 记录操作开始时间
    ///
    /// 返回当前时间戳，用于后续计算操作延迟
    pub fn record_operation_start(&self) -> Instant {
        Instant::now()
    }

    /// 记录操作完成
    ///
    /// 根据开始时间计算并记录操作延迟
    pub fn record_operation_completion(&self, start_time: Instant) {
        let latency = start_time.elapsed().as_micros() as usize;
        self.operation_count.fetch_add(1, Ordering::Relaxed);
        self.total_latency.fetch_add(latency, Ordering::Relaxed);
    }

    /// 获取文档数量
    pub fn get_document_count(&self) -> usize {
        self.documents.len()
    }

    /// 获取索引项数量
    pub fn get_index_count(&self) -> usize {
        self.data.len()
    }
}

impl Default for StorageBase {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_base_new() {
        let base = StorageBase::new();
        assert_eq!(base.get_memory_usage(), 0);
        assert_eq!(base.get_operation_count(), 0);
        assert_eq!(base.get_document_count(), 0);
        assert_eq!(base.get_index_count(), 0);
    }

    #[test]
    fn test_storage_base_clear() {
        let mut base = StorageBase::new();
        base.data.insert("test".to_string(), vec![1, 2, 3]);
        base.documents.insert(1, "content".to_string());

        base.clear();

        assert!(base.data.is_empty());
        assert!(base.documents.is_empty());
    }

    #[test]
    fn test_storage_base_has() {
        let mut base = StorageBase::new();
        base.data.insert("test".to_string(), vec![1, 2, 3]);

        assert!(base.has(1));
        assert!(base.has(2));
        assert!(!base.has(999));
    }

    #[test]
    fn test_storage_base_remove() {
        let mut base = StorageBase::new();
        base.data.insert("test".to_string(), vec![1, 2, 3]);
        base.documents.insert(1, "doc1".to_string());
        base.documents.insert(2, "doc2".to_string());

        base.remove(&[1]);

        assert!(!base.has(1));
        assert!(base.has(2));
        assert!(!base.documents.contains_key(&1));
        assert!(base.documents.contains_key(&2));
    }

    #[test]
    fn test_storage_base_operation_timing() {
        let base = StorageBase::new();

        let start = base.record_operation_start();
        std::thread::sleep(std::time::Duration::from_millis(1));
        base.record_operation_completion(start);

        assert_eq!(base.get_operation_count(), 1);
        assert!(base.get_total_latency() > 0);
        assert!(base.get_average_latency() > 0);
    }
}
