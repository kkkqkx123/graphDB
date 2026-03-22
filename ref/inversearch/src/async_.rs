//! 异步支持模块
//! 
//! 提供异步索引操作和搜索功能

use crate::r#type::{SearchOptions, SearchResults};
use crate::error::Result;
use crate::Index;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::task::JoinHandle;

/// 异步搜索任务
pub struct AsyncSearchTask {
    handle: JoinHandle<Result<crate::search::SearchResult>>,
}

impl AsyncSearchTask {
    /// 创建新的异步搜索任务
    pub fn new<F>(future: F) -> Self 
    where
        F: Future<Output = Result<crate::search::SearchResult>> + Send + 'static,
    {
        let handle = tokio::spawn(future);
        Self { handle }
    }
}

impl Future for AsyncSearchTask {
    type Output = Result<crate::search::SearchResult>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match Pin::new(&mut self.handle).poll(cx) {
            Poll::Ready(Ok(result)) => Poll::Ready(result),
            Poll::Ready(Err(join_error)) => {
                Poll::Ready(Err(crate::error::InversearchError::AsyncError(
                    join_error.to_string()
                )))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

/// 异步索引构建任务
pub struct AsyncIndexTask {
    handle: JoinHandle<Result<()>>,
}

impl AsyncIndexTask {
    /// 创建新的异步索引任务
    pub fn new<F>(future: F) -> Self 
    where
        F: Future<Output = Result<()>> + Send + 'static,
    {
        let handle = tokio::spawn(future);
        Self { handle }
    }
}

impl Future for AsyncIndexTask {
    type Output = Result<()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match Pin::new(&mut self.handle).poll(cx) {
            Poll::Ready(Ok(result)) => Poll::Ready(result),
            Poll::Ready(Err(join_error)) => {
                Poll::Ready(Err(crate::error::InversearchError::AsyncError(
                    join_error.to_string()
                )))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

/// 异步索引包装器
#[derive(Clone)]
pub struct AsyncIndex {
    pub(crate) index: std::sync::Arc<tokio::sync::RwLock<Index>>,
}

impl AsyncIndex {
    /// 创建新的异步索引
    pub fn new(index: Index) -> Self {
        Self {
            index: std::sync::Arc::new(tokio::sync::RwLock::new(index)),
        }
    }
    
    /// 异步添加文档
    pub async fn add_async(&self, id: u64, content: &str, append: bool) -> Result<()> {
        let content = content.to_string();
        let index = self.index.clone();
        
        tokio::task::spawn_blocking(move || {
            let mut index = index.blocking_write();
            index.add(id, &content, append)
        }).await?
    }
    
    /// 异步删除文档
    pub async fn remove_async(&self, id: u64) -> Result<()> {
        let index = self.index.clone();
        
        tokio::task::spawn_blocking(move || {
            let mut index = index.blocking_write();
            index.remove(id, false)
        }).await?
    }
    
    /// 异步搜索
    pub async fn search_async(&self, options: SearchOptions) -> Result<crate::search::SearchResult> {
        let index = self.index.clone();
        let options_clone = options.clone();
        
        tokio::task::spawn_blocking(move || {
            let index = index.blocking_read();
            index.search(&options_clone)
        }).await?
    }
    
    /// 异步带缓存搜索
    pub async fn search_cached_async(&self, options: SearchOptions) -> Result<crate::search::SearchResult> {
        let index = self.index.clone();
        let options_clone = options.clone();
        
        tokio::task::spawn_blocking(move || {
            let mut index = index.blocking_write();
            index.search_cached(&options_clone)
        }).await?
    }
    
    /// 异步更新文档
    pub async fn update_async(&self, id: u64, content: &str) -> Result<()> {
        let content = content.to_string();
        let index = self.index.clone();
        
        tokio::task::spawn_blocking(move || {
            let mut index = index.blocking_write();
            index.update(id, &content)
        }).await?
    }
    
    /// 异步清空索引
    pub async fn clear_async(&self) -> Result<()> {
        let index = self.index.clone();
        
        tokio::task::spawn_blocking(move || {
            let mut index = index.blocking_write();
            index.clear();
            Ok(())
        }).await?
    }
    
    /// 异步获取缓存统计
    pub async fn cache_stats_async(&self) -> Option<crate::search::CacheStats> {
        let index = self.index.read().await;
        index.cache_stats()
    }
    
    /// 异步清空缓存
    pub async fn clear_cache_async(&self) -> Result<()> {
        let mut index = self.index.write().await;
        index.clear_cache();
        Ok(())
    }
}

/// 异步搜索构建器
pub struct AsyncSearchBuilder {
    query: String,
    options: SearchOptions,
}

impl AsyncSearchBuilder {
    /// 创建新的异步搜索构建器
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            options: SearchOptions::default(),
        }
    }
    
    /// 设置限制
    pub fn limit(mut self, limit: usize) -> Self {
        self.options.limit = Some(limit);
        self
    }
    
    /// 设置偏移
    pub fn offset(mut self, offset: usize) -> Self {
        self.options.offset = Some(offset);
        self
    }
    
    /// 设置上下文
    pub fn context(mut self, context: bool) -> Self {
        self.options.context = Some(context);
        self
    }
    
    /// 设置建议
    pub fn suggest(mut self, suggest: bool) -> Self {
        self.options.suggest = Some(suggest);
        self
    }
    
    /// 执行搜索
    pub async fn execute(self, index: &AsyncIndex) -> Result<crate::search::SearchResult> {
        let mut options = self.options;
        options.query = Some(self.query);
        
        index.search_async(options).await
    }
}

/// 批量异步操作
pub struct BatchAsyncOperations {
    operations: Vec<BatchOperation>,
}

enum BatchOperation {
    Add { id: u64, content: String, append: bool },
    Remove { id: u64 },
    Update { id: u64, content: String },
}

impl BatchAsyncOperations {
    /// 创建新的批量操作
    pub fn new() -> Self {
        Self {
            operations: Vec::new(),
        }
    }
    
    /// 添加操作
    pub fn add_operation(&mut self, operation: BatchOperation) {
        self.operations.push(operation);
    }
    
    /// 添加文档
    pub fn add(&mut self, id: u64, content: impl Into<String>, append: bool) {
        self.add_operation(BatchOperation::Add {
            id,
            content: content.into(),
            append,
        });
    }
    
    /// 删除文档
    pub fn remove(&mut self, id: u64) {
        self.add_operation(BatchOperation::Remove { id });
    }
    
    /// 更新文档
    pub fn update(&mut self, id: u64, content: impl Into<String>) {
        self.add_operation(BatchOperation::Update {
            id,
            content: content.into(),
        });
    }
    
    /// 执行批量操作
    pub async fn execute(self, index: &AsyncIndex) -> Result<Vec<Result<()>>> {
        let mut results = Vec::new();
        
        for operation in self.operations {
            let result = match operation {
                BatchOperation::Add { id, content, append } => {
                    index.add_async(id, &content, append).await
                }
                BatchOperation::Remove { id } => {
                    index.remove_async(id).await
                }
                BatchOperation::Update { id, content } => {
                    index.update_async(id, &content).await
                }
            };
            results.push(result);
        }
        
        Ok(results)
    }
}

impl Default for BatchAsyncOperations {
    fn default() -> Self {
        Self::new()
    }
}

/// 并发搜索
pub struct ConcurrentSearch {
    searches: Vec<(String, SearchOptions)>,
}

impl ConcurrentSearch {
    /// 创建新的并发搜索
    pub fn new() -> Self {
        Self {
            searches: Vec::new(),
        }
    }
    
    /// 添加搜索
    pub fn add_search(&mut self, query: impl Into<String>, options: SearchOptions) {
        self.searches.push((query.into(), options));
    }
    
    /// 执行并发搜索
    pub async fn execute(self, index: &AsyncIndex) -> Result<Vec<crate::search::SearchResult>> {
        let mut handles = Vec::new();
        
        for (query, mut options) in self.searches {
            options.query = Some(query);
            let index_clone = index.clone();
            let options_clone = options.clone();
            
            let handle = tokio::spawn(async move {
                index_clone.search_async(options_clone).await
            });
            
            handles.push(handle);
        }
        
        let mut results = Vec::new();
        for handle in handles {
            results.push(handle.await??);
        }
        
        Ok(results)
    }
}

impl Default for ConcurrentSearch {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Index;

    #[tokio::test]
    async fn test_async_add_and_search() {
        let index = Index::default();
        let async_index = AsyncIndex::new(index);
        
        // 异步添加文档
        async_index.add_async(1, "hello world", false).await.unwrap();
        async_index.add_async(2, "rust programming", false).await.unwrap();
        
        // 异步搜索
        let mut options = SearchOptions::default();
        options.query = Some("hello".to_string());
        let result = async_index.search_async(options).await.unwrap();
        
        assert_eq!(result.results.len(), 1);
        assert!(result.results.contains(&1));
    }
    
    #[tokio::test]
    async fn test_async_search_builder() {
        let index = Index::default();
        let async_index = AsyncIndex::new(index);
        
        async_index.add_async(1, "test document", false).await.unwrap();
        
        // 使用搜索构建器
        let result = AsyncSearchBuilder::new("test")
            .limit(10)
            .offset(0)
            .execute(&async_index)
            .await
            .unwrap();
        
        assert_eq!(result.results.len(), 1);
        assert!(result.results.contains(&1));
    }
    
    #[tokio::test]
    async fn test_batch_operations() {
        let index = Index::default();
        let async_index = AsyncIndex::new(index);
        
        let mut batch = BatchAsyncOperations::new();
        batch.add(1, "first document", false);
        batch.add(2, "second document", false);
        batch.update(1, "updated first document");
        
        let results = batch.execute(&async_index).await.unwrap();
        assert_eq!(results.len(), 3);
        
        // 验证结果
        for result in results {
            assert!(result.is_ok());
        }
    }
    
    #[tokio::test]
    async fn test_concurrent_search() {
        let index = Index::default();
        let async_index = AsyncIndex::new(index);
        
        async_index.add_async(1, "hello world", false).await.unwrap();
        async_index.add_async(2, "rust world", false).await.unwrap();
        
        let mut concurrent = ConcurrentSearch::new();
        concurrent.add_search("hello", SearchOptions::default());
        concurrent.add_search("world", SearchOptions::default());
        concurrent.add_search("rust", SearchOptions::default());
        
        let results = concurrent.execute(&async_index).await.unwrap();
        assert_eq!(results.len(), 3);
        
        // hello应该找到文档1
        assert_eq!(results[0].results.len(), 1);
        assert!(results[0].results.contains(&1));
        
        // world应该找到文档1和2
        assert_eq!(results[1].results.len(), 2);
        assert!(results[1].results.contains(&1));
        assert!(results[1].results.contains(&2));
        
        // rust应该找到文档2
        assert_eq!(results[2].results.len(), 1);
        assert!(results[2].results.contains(&2));
    }
}