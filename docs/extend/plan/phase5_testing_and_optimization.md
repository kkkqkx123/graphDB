# Phase 5: 测试与优化方案

## 阶段目标

完成全文检索功能的全面测试，包括单元测试、集成测试和性能测试，并进行必要的性能优化。

**预计工期**: 5-7 天  
**前置依赖**: Phase 4 (数据同步机制)

---

## 测试策略

### 1. 单元测试

#### 1.1 引擎适配器测试

**文件**: `src/search/adapters/bm25_adapter_test.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_bm25_lifecycle() {
        let temp_dir = TempDir::new().unwrap();
        let engine = Bm25SearchEngine::open_or_create(temp_dir.path()).unwrap();
        
        // 测试索引
        engine.index("1", "Rust programming language").await.unwrap();
        engine.index("2", "Graph database implementation").await.unwrap();
        engine.index("3", "Rust graph database").await.unwrap();
        
        // 测试提交
        engine.commit().await.unwrap();
        
        // 测试搜索
        let results = engine.search("Rust", 10).await.unwrap();
        assert_eq!(results.len(), 2);
        assert!(results[0].score >= results[1].score);
        
        // 测试删除
        engine.delete("1").await.unwrap();
        engine.commit().await.unwrap();
        
        let results = engine.search("Rust", 10).await.unwrap();
        assert_eq!(results.len(), 1);
        
        // 测试关闭
        engine.close().await.unwrap();
    }
    
    #[tokio::test]
    async fn test_bm25_batch_operations() {
        let temp_dir = TempDir::new().unwrap();
        let engine = Bm25SearchEngine::open_or_create(temp_dir.path()).unwrap();
        
        let docs: Vec<(String, String)> = (0..100)
            .map(|i| (i.to_string(), format!("Document content {}", i)))
            .collect();
        
        // 批量索引
        engine.index_batch(docs).await.unwrap();
        engine.commit().await.unwrap();
        
        // 验证
        let results = engine.search("Document", 100).await.unwrap();
        assert_eq!(results.len(), 100);
    }
    
    #[tokio::test]
    async fn test_bm25_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_path_buf();
        
        // 第一次打开，索引数据
        {
            let engine = Bm25SearchEngine::open_or_create(&path).unwrap();
            engine.index("1", "Persistent data").await.unwrap();
            engine.commit().await.unwrap();
            engine.close().await.unwrap();
        }
        
        // 第二次打开，验证数据
        {
            let engine = Bm25SearchEngine::open_or_create(&path).unwrap();
            let results = engine.search("Persistent", 10).await.unwrap();
            assert_eq!(results.len(), 1);
            assert_eq!(results[0].doc_id, Value::from("1"));
        }
    }
}
```

#### 1.2 协调器测试

**文件**: `src/coordinator/fulltext_test.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_coordinator_create_and_search() {
        let (coordinator, _temp) = create_test_coordinator().await;
        
        // 创建索引
        let index_id = coordinator
            .create_index(1, "Article", "title", Some(EngineType::Bm25))
            .await
            .unwrap();
        
        assert!(!index_id.is_empty());
        
        // 模拟顶点插入
        let vertex = create_test_vertex(1, "Article", vec![("title", "Hello World")]);
        coordinator.on_vertex_inserted(1, &vertex).await.unwrap();
        coordinator.commit_all().await.unwrap();
        
        // 搜索
        let results = coordinator.search(1, "Article", "title", "Hello", 10).await.unwrap();
        assert_eq!(results.len(), 1);
    }
    
    #[tokio::test]
    async fn test_coordinator_multiple_indexes() {
        let (coordinator, _temp) = create_test_coordinator().await;
        
        // 创建多个索引
        coordinator.create_index(1, "Post", "title", None).await.unwrap();
        coordinator.create_index(1, "Post", "content", None).await.unwrap();
        
        // 插入数据
        let vertex = create_test_vertex(1, "Post", vec![
            ("title", "Rust Tutorial"),
            ("content", "Learn Rust programming"),
        ]);
        coordinator.on_vertex_inserted(1, &vertex).await.unwrap();
        coordinator.commit_all().await.unwrap();
        
        // 分别搜索
        let title_results = coordinator.search(1, "Post", "title", "Rust", 10).await.unwrap();
        let content_results = coordinator.search(1, "Post", "content", "programming", 10).await.unwrap();
        
        assert_eq!(title_results.len(), 1);
        assert_eq!(content_results.len(), 1);
    }
}
```

### 2. 集成测试

**文件**: `tests/fulltext_integration_test.rs`

```rust
use graphdb::test_utils::*;

#[tokio::test]
async fn test_fulltext_end_to_end() {
    let (db, _temp) = setup_test_database().await;
    
    // 1. 创建全文索引
    db.execute("CREATE FULLTEXT INDEX idx_title ON Article(title) USING bm25")
        .await
        .unwrap();
    
    // 2. 插入数据
    db.execute("INSERT VERTEX Article(title, content) VALUES ('Hello World', 'First article')")
        .await
        .unwrap();
    db.execute("INSERT VERTEX Article(title, content) VALUES ('Rust Programming', 'Second article')")
        .await
        .unwrap();
    db.execute("INSERT VERTEX Article(title, content) VALUES ('Hello Rust', 'Third article')")
        .await
        .unwrap();
    
    // 3. 等待索引（实际实现可能需要同步等待机制）
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    // 4. 执行全文搜索
    let results = db.execute("MATCH (a:Article) WHERE a.title MATCH 'Hello' RETURN a")
        .await
        .unwrap();
    
    assert_eq!(results.rows.len(), 2);
    
    // 5. 带评分排序
    let results = db.execute(
        "MATCH (a:Article) WHERE a.title MATCH 'Rust' RETURN a, score(a) as relevance ORDER BY relevance DESC"
    ).await.unwrap();
    
    assert_eq!(results.rows.len(), 2);
    // 验证排序："Rust Programming" 应该比 "Hello Rust" 评分更高
    let first_score: f32 = results.rows[0].get("relevance").unwrap().as_f32().unwrap();
    let second_score: f32 = results.rows[1].get("relevance").unwrap().as_f32().unwrap();
    assert!(first_score >= second_score);
}

#[tokio::test]
async fn test_fulltext_with_updates() {
    let (db, _temp) = setup_test_database().await;
    
    // 创建索引
    db.execute("CREATE FULLTEXT INDEX idx_content ON Post(content) USING bm25")
        .await
        .unwrap();
    
    // 插入
    db.execute("INSERT VERTEX Post(content) VALUES 'Original content'")
        .await
        .unwrap();
    
    tokio::time::sleep(Duration::from_millis(200)).await;
    
    // 验证
    let results = db.execute("MATCH (p:Post) WHERE p.content MATCH 'Original' RETURN p")
        .await
        .unwrap();
    assert_eq!(results.rows.len(), 1);
    
    // 更新
    db.execute("UPDATE VERTEX Post SET content = 'Updated content' WHERE id = 1")
        .await
        .unwrap();
    
    tokio::time::sleep(Duration::from_millis(200)).await;
    
    // 验证旧内容搜不到
    let results = db.execute("MATCH (p:Post) WHERE p.content MATCH 'Original' RETURN p")
        .await
        .unwrap();
    assert_eq!(results.rows.len(), 0);
    
    // 验证新内容可以搜到
    let results = db.execute("MATCH (p:Post) WHERE p.content MATCH 'Updated' RETURN p")
        .await
        .unwrap();
    assert_eq!(results.rows.len(), 1);
}

#[tokio::test]
async fn test_fulltext_rebuild() {
    let (db, _temp) = setup_test_database().await;
    
    // 创建索引并插入数据
    db.execute("CREATE FULLTEXT INDEX idx_content ON Article(content) USING bm25")
        .await
        .unwrap();
    db.execute("INSERT VERTEX Article(content) VALUES 'Test content'")
        .await
        .unwrap();
    
    tokio::time::sleep(Duration::from_millis(200)).await;
    
    // 验证数据存在
    let results = db.execute("MATCH (a:Article) WHERE a.content MATCH 'Test' RETURN a")
        .await
        .unwrap();
    assert_eq!(results.rows.len(), 1);
    
    // 重建索引
    db.execute("REBUILD FULLTEXT INDEX idx_content").await.unwrap();
    
    // 重建后数据应该为空（需要重新索引）
    let results = db.execute("MATCH (a:Article) WHERE a.content MATCH 'Test' RETURN a")
        .await
        .unwrap();
    assert_eq!(results.rows.len(), 0);
}
```

### 3. 性能测试

**文件**: `benches/fulltext_benchmark.rs`

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use graphdb::test_utils::*;
use tokio::runtime::Runtime;

fn bench_indexing(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("indexing");
    
    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::new("bm25", size), size, |b, &size| {
            b.iter(|| {
                rt.block_on(async {
                    let (coordinator, _temp) = setup_test_coordinator().await;
                    coordinator.create_index(1, "Doc", "content", Some(EngineType::Bm25))
                        .await
                        .unwrap();
                    
                    for i in 0..size {
                        let vertex = create_test_vertex(
                            i as i64,
                            "Doc",
                            vec![("content", &format!("Document content number {}", i))],
                        );
                        coordinator.on_vertex_inserted(1, &vertex).await.unwrap();
                    }
                    
                    coordinator.commit_all().await.unwrap();
                });
            });
        });
    }
    
    group.finish();
}

fn bench_search(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("search");
    
    for doc_count in [1000, 10000, 100000].iter() {
        group.bench_with_input(
            BenchmarkId::new("single_term", doc_count),
            doc_count,
            |b, &doc_count| {
                let coordinator = rt.block_on(async {
                    let (coordinator, _temp) = setup_test_coordinator().await;
                    coordinator.create_index(1, "Doc", "content", Some(EngineType::Bm25))
                        .await
                        .unwrap();
                    
                    // 准备数据
                    for i in 0..doc_count {
                        let vertex = create_test_vertex(
                            i as i64,
                            "Doc",
                            vec![("content", &format!("Content {}", i))],
                        );
                        coordinator.on_vertex_inserted(1, &vertex).await.unwrap();
                    }
                    coordinator.commit_all().await.unwrap();
                    coordinator
                });
                
                b.iter(|| {
                    rt.block_on(async {
                        let _ = coordinator.search(1, "Doc", "content", "Content", 10).await;
                    });
                });
            },
        );
    }
    
    group.finish();
}

fn bench_engine_comparison(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("engine_comparison");
    
    for engine_type in [EngineType::Bm25, EngineType::Inversearch].iter() {
        let engine_name = format!("{:?}", engine_type);
        
        group.bench_function(format!("{}_index", engine_name), |b| {
            b.iter(|| {
                rt.block_on(async {
                    let (coordinator, _temp) = setup_test_coordinator().await;
                    coordinator.create_index(1, "Doc", "content", Some(*engine_type))
                        .await
                        .unwrap();
                    
                    for i in 0..1000 {
                        let vertex = create_test_vertex(
                            i as i64,
                            "Doc",
                            vec![("content", &format!("Test content {}", i))],
                        );
                        coordinator.on_vertex_inserted(1, &vertex).await.unwrap();
                    }
                    coordinator.commit_all().await.unwrap();
                });
            });
        });
    }
    
    group.finish();
}

criterion_group!(benches, bench_indexing, bench_search, bench_engine_comparison);
criterion_main!(benches);
```

---

## 性能优化

### 1. 索引优化

#### 1.1 批量提交优化

```rust
// src/sync/batch.rs

impl BatchProcessor {
    /// 优化后的批量提交
    pub async fn optimized_commit(&mut self) -> Result<(), BatchError> {
        // 按索引分组
        let mut batches: HashMap<_, Vec<_>> = HashMap::new();
        
        for ((space_id, tag, field), docs) in &self.buffers {
            if !docs.is_empty() {
                batches.insert((*space_id, tag.clone(), field.clone()), docs.clone());
            }
        }
        
        // 并行提交
        let futures: Vec<_> = batches.into_iter()
            .map(|(key, docs)| {
                let coordinator = self.coordinator.clone();
                async move {
                    let (space_id, tag, field) = key;
                    if let Some(engine) = coordinator.get_engine(space_id, &tag, &field) {
                        engine.index_batch(docs).await?;
                        engine.commit().await?;
                    }
                    Ok::<_, SearchError>(())
                }
            })
            .collect();
        
        // 等待所有提交完成
        for result in futures::future::join_all(futures).await {
            result.map_err(|e| BatchError::CommitError(e.to_string()))?;
        }
        
        self.buffers.clear();
        Ok(())
    }
}
```

#### 1.2 内存使用优化

```rust
// src/search/manager.rs

/// 索引缓存管理
pub struct IndexCache {
    /// LRU 缓存
    cache: lru::LruCache<IndexKey, Arc<dyn SearchEngine>>,
    /// 最大缓存索引数
    max_indexes: usize,
}

impl IndexCache {
    /// 获取索引（带缓存）
    pub fn get(&mut self, key: &IndexKey) -> Option<Arc<dyn SearchEngine>> {
        self.cache.get(key).cloned()
    }
    
    /// 插入索引
    pub fn put(&mut self, key: IndexKey, engine: Arc<dyn SearchEngine>) {
        // 如果缓存已满，关闭最久未使用的索引
        if self.cache.len() >= self.max_indexes && !self.cache.contains(&key) {
            if let Some((lru_key, lru_engine)) = self.cache.pop_lru() {
                // 异步关闭引擎
                tokio::spawn(async move {
                    let _ = lru_engine.close().await;
                });
            }
        }
        
        self.cache.put(key, engine);
    }
}
```

### 2. 搜索优化

#### 2.1 结果缓存

```rust
// src/search/cache.rs

use moka::future::Cache;

/// 搜索结果缓存
pub struct SearchCache {
    cache: Cache<String, Vec<SearchResult>>,
}

impl SearchCache {
    pub fn new(max_capacity: u64, ttl_secs: u64) -> Self {
        Self {
            cache: Cache::builder()
                .max_capacity(max_capacity)
                .time_to_live(Duration::from_secs(ttl_secs))
                .build(),
        }
    }
    
    /// 生成缓存键
    fn cache_key(space_id: u64, tag: &str, field: &str, query: &str, limit: usize) -> String {
        format!("{}:{}:{}:{}:{}", space_id, tag, field, query, limit)
    }
    
    /// 获取缓存结果
    pub async fn get(&self, space_id: u64, tag: &str, field: &str, query: &str, limit: usize) 
        -> Option<Vec<SearchResult>> {
        let key = Self::cache_key(space_id, tag, field, query, limit);
        self.cache.get(&key).await
    }
    
    /// 缓存结果
    pub async fn set(&self, space_id: u64, tag: &str, field: &str, query: &str, limit: usize, results: Vec<SearchResult>) {
        let key = Self::cache_key(space_id, tag, field, query, limit);
        self.cache.insert(key, results).await;
    }
    
    /// 使缓存失效
    pub async fn invalidate(&self, space_id: u64, tag: &str, field: &str) {
        // 使用前缀匹配清除相关缓存
        let prefix = format!("{}:{}:{}", space_id, tag, field);
        self.cache.invalidate_entries_if(move |key, _| key.starts_with(&prefix))
            .expect("Failed to invalidate cache");
    }
}
```

#### 2.2 查询预热

```rust
// src/search/warmup.rs

/// 索引预热器
pub struct IndexWarmer {
    coordinator: Arc<FulltextCoordinator>,
}

impl IndexWarmer {
    /// 预热常用查询
    pub async fn warm_common_queries(&self) {
        let common_queries = vec![
            ("Post", "content", "tutorial"),
            ("Article", "title", "Rust"),
            ("User", "name", "admin"),
        ];
        
        for (tag, field, query) in common_queries {
            // 执行搜索以加载索引到内存
            let _ = self.coordinator.search(1, tag, field, query, 10).await;
        }
    }
    
    /// 预热特定索引
    pub async fn warm_index(&self, space_id: u64, tag: &str, field: &str) {
        if let Some(engine) = self.coordinator.get_engine(space_id, tag, field) {
            // 执行空查询以加载索引结构
            let _ = engine.search("*", 1).await;
        }
    }
}
```

### 3. 监控指标

```rust
// src/search/metrics.rs

use metrics::{counter, gauge, histogram, Counter, Gauge, Histogram};

/// 全文检索指标
pub struct FulltextMetrics {
    /// 索引操作计数
    index_ops: Counter,
    /// 搜索操作计数
    search_ops: Counter,
    /// 搜索延迟
    search_latency: Histogram,
    /// 索引文档数
    indexed_docs: Gauge,
    /// 队列大小
    queue_size: Gauge,
}

impl FulltextMetrics {
    pub fn new() -> Self {
        Self {
            index_ops: counter!("fulltext_index_ops_total"),
            search_ops: counter!("fulltext_search_ops_total"),
            search_latency: histogram!("fulltext_search_latency_seconds"),
            indexed_docs: gauge!("fulltext_indexed_docs"),
            queue_size: gauge!("fulltext_queue_size"),
        }
    }
    
    pub fn record_index(&self, count: usize) {
        self.index_ops.increment(count as u64);
        self.indexed_docs.increment(count as f64);
    }
    
    pub fn record_search(&self, latency: Duration) {
        self.search_ops.increment(1);
        self.search_latency.record(latency.as_secs_f64());
    }
    
    pub fn set_queue_size(&self, size: usize) {
        self.queue_size.set(size as f64);
    }
}
```

---

## 性能目标

| 指标 | 目标值 | 测试方法 |
|------|--------|----------|
| 单次搜索延迟 | < 5ms (P95) | 10万文档基准测试 |
| 批量索引速度 | > 5000 doc/s | 1万文档批量测试 |
| 内存占用 | < 300MB | 10万文档索引 |
| 索引构建速度 | > 2000 doc/s | 全量索引测试 |
| 并发搜索 | > 1000 QPS | 并发压力测试 |

---

## 故障排查指南

### 常见问题

#### 1. 搜索结果为空

```rust
// 排查步骤
pub async fn diagnose_search_empty(
    coordinator: &FulltextCoordinator,
    space_id: u64,
    tag: &str,
    field: &str,
    query: &str,
) -> Vec<String> {
    let mut issues = Vec::new();
    
    // 1. 检查索引是否存在
    if !coordinator.has_index(space_id, tag, field) {
        issues.push(format!("索引不存在: {}.{}.{}", space_id, tag, field));
        return issues;
    }
    
    // 2. 检查索引状态
    if let Some(metadata) = coordinator.get_index_info(space_id, tag, field) {
        if metadata.doc_count == 0 {
            issues.push("索引为空，可能尚未同步数据".to_string());
        }
        issues.push(format!("索引文档数: {}", metadata.doc_count));
    }
    
    // 3. 尝试搜索所有文档
    let all_results = coordinator.search(space_id, tag, field, "*", 100).await;
    match all_results {
        Ok(results) => issues.push(format!("通配符搜索结果数: {}", results.len())),
        Err(e) => issues.push(format!("搜索错误: {}", e)),
    }
    
    issues
}
```

#### 2. 索引性能下降

```rust
// 性能诊断
pub async fn diagnose_performance(
    coordinator: &FulltextCoordinator,
    space_id: u64,
    tag: &str,
    field: &str,
) -> PerformanceReport {
    let start = Instant::now();
    
    // 测试搜索性能
    let search_start = Instant::now();
    let _ = coordinator.search(space_id, tag, field, "test", 10).await;
    let search_latency = search_start.elapsed();
    
    // 获取统计信息
    let stats = coordinator.get_index_stats(space_id, tag, field).await;
    
    PerformanceReport {
        search_latency_ms: search_latency.as_millis(),
        doc_count: stats.as_ref().map(|s| s.doc_count).unwrap_or(0),
        index_size_mb: stats.as_ref().map(|s| s.index_size / 1024 / 1024).unwrap_or(0),
        recommendations: generate_recommendations(&stats),
    }
}
```

---

## 验收标准

- [ ] 单元测试覆盖率 > 80%
- [ ] 集成测试全部通过
- [ ] 性能测试达到目标值
- [ ] 内存使用在预期范围内
- [ ] 支持 10万+ 文档索引
- [ ] 支持并发搜索
- [ ] 代码通过 `cargo clippy` 检查
- [ ] 文档完整

---

## 风险与缓解措施

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| 性能不达标 | 高 | 提前进行性能测试，预留优化时间 |
| 测试覆盖不足 | 中 | 使用 tarpaulin 检查覆盖率 |
| 内存泄漏 | 中 | 使用 valgrind/miri 检查 |
| 并发问题 | 高 | 使用 loom 进行并发测试 |

---

## 交付物

1. **测试代码**
   - 单元测试
   - 集成测试
   - 性能测试

2. **性能报告**
   - 基准测试结果
   - 优化前后对比

3. **运维文档**
   - 监控指标说明
   - 故障排查指南
   - 性能调优建议
