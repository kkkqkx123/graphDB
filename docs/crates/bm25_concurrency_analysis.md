# BM25 并发设计与多线程分析

## 概述

本文档分析 `crates/bm25` 的并发设计，识别潜在问题并提供改进建议。

**分析日期**: 2026-04-03  
**分析范围**: `crates/bm25/src/index/`

---

## 1. 当前架构分析

### 1.1 IndexManager 设计

```rust
pub struct IndexManager {
    index: Index,
    schema: Schema,
    config: IndexManagerConfig,
    cached_reader: Arc<RwLock<Option<IndexReader>>>,
}
```

**特点**:
- ✅ `Index` 是线程安全的（tantivy 内部使用 Arc）
- ✅ `cached_reader` 使用 `Arc<RwLock<>>` 实现线程共享
- ⚠️ **每次调用 `writer()` 都创建新的 `IndexWriter`**

### 1.2 当前调用链

```
GraphDB::index() 
  └─> tokio::task::spawn_blocking()
       └─> bm25_service::add_document()
            └─> manager.writer()  // 创建新 writer
            └─> writer.add_document()
            └─> writer.commit()   // 立即提交
```

---

## 2. 核心问题

### 2.1 ❌ 每次操作都创建新的 IndexWriter

**问题代码**:
```rust
// crates/bm25/src/index/document.rs
pub fn add_document(
    manager: &IndexManager,
    schema: &IndexSchema,
    document_id: &str,
    fields: &HashMap<String, String>,
) -> Result<()> {
    let mut writer = manager.writer()?;  // ❌ 每次都创建新 writer
    let doc = schema.to_document(document_id, fields);
    writer.add_document(doc)?;
    writer.commit()?;  // ❌ 每次都提交
    Ok(())
}
```

**影响分析**:

| 问题 | 影响 | 严重性 |
|------|------|--------|
| **资源浪费** | 每次创建 writer 需要分配缓冲区（默认 50MB） | 🔴 高 |
| **性能开销** | 频繁 commit 导致频繁磁盘 I/O | 🔴 高 |
| **并发冲突** | 多个 writer 同时 commit 可能导致索引损坏 | 🔴 高 |
| **内存膨胀** | 并发场景下多个 writer 同时存在 | 🟡 中 |

### 2.2 ❌ 每个文档单独提交

**当前行为**:
```rust
// 每次 index() 调用都会 commit
writer.commit()?;
```

**问题**:
1. **磁盘 I/O 爆炸**: 1000 个文档 = 1000 次磁盘提交
2. **Segment 文件过多**: tantivy 每个 commit 创建新 segment
3. **Merge 压力**: 后台需要频繁 merge segment

**性能对比**:
```
批量提交 1000 个文档:
- 当前设计：1000 次 commit, ~10 秒
- 优化设计：1 次 commit, ~0.1 秒
```

### 2.3 ⚠️ spawn_blocking 可能耗尽线程池

**当前代码**:
```rust
// src/search/adapters/bm25_adapter.rs
async fn index(&self, doc_id: &str, content: &str) -> Result<(), SearchError> {
    tokio::task::spawn_blocking(move || {
        add_document(&manager, &schema, &doc_id, &fields)
    })
    .await?
}
```

**问题**:
- Tokio 默认阻塞线程池大小 = CPU 核心数
- 高并发场景（如 1000 QPS）会耗尽线程池
- 阻塞其他需要 `spawn_blocking` 的任务

---

## 3. 根本原因

### 3.1 Tantivy 的设计哲学

Tantivy 是一个**同步**搜索引擎库，设计假设：
1. **单线程索引**: 通常由单个线程管理索引
2. **批量操作**: 推荐批量添加文档后统一 commit
3. **长生命周期 Writer**: Writer 应该复用，而不是每次创建

### 3.2 当前设计违背了 Tantivy 的最佳实践

| Tantivy 最佳实践 | 当前实现 | 结果 |
|------------------|----------|------|
| 复用 IndexWriter | 每次创建新 writer | ❌ |
| 批量 commit | 每个文档 commit | ❌ |
| 单线程索引更新 | 多线程并发访问 | ⚠️ |

---

## 4. 解决方案

### 4.1 ✅ 方案 A：单例 Writer 模式（推荐）

**设计思路**:
```rust
pub struct Bm25SearchEngine {
    manager: Arc<IndexManager>,
    schema: IndexSchema,
    index_path: PathBuf,
    writer: Arc<Mutex<IndexWriter>>,  // ✅ 单例 writer
}

impl Bm25SearchEngine {
    pub fn open_or_create(path: &Path) -> Result<Self, SearchError> {
        // ... 创建 manager ...
        
        let writer = manager.writer()?;
        
        Ok(Self {
            manager: Arc::new(manager),
            schema,
            index_path: path.to_path_buf(),
            writer: Arc::new(Mutex::new(writer)),
        })
    }
}
```

**索引操作**:
```rust
async fn index(&self, doc_id: &str, content: &str) -> Result<(), SearchError> {
    let mut writer_guard = self.writer.lock().await;
    
    let mut fields = HashMap::new();
    fields.insert("content".to_string(), content.to_string());
    
    let doc = self.schema.to_document(doc_id, &fields);
    writer_guard.add_document(doc);
    // ✅ 不立即 commit，等待批量提交或 close
    
    Ok(())
}

async fn commit(&self) -> Result<(), SearchError> {
    let mut writer_guard = self.writer.lock().await;
    writer_guard.commit()
        .map_err(|e| SearchError::Bm25Error(format!("Commit failed: {}", e)))?;
    Ok(())
}
```

**优点**:
- ✅ Writer 复用，无创建开销
- ✅ 支持批量 commit
- ✅ 线程安全（Mutex 保护）
- ✅ 符合 Tantivy 最佳实践

**缺点**:
- ⚠️ 写操作串行化（但这是合理的，因为索引本身就是串行的）

### 4.2 ✅ 方案 B：通道批量处理

**设计思路**:
```rust
pub struct Bm25SearchEngine {
    manager: Arc<IndexManager>,
    schema: IndexSchema,
    tx: mpsc::Sender<IndexOperation>,
}

enum IndexOperation {
    Add { doc_id: String, fields: HashMap<String, String> },
    Delete { doc_id: String },
    Commit,
}

impl Bm25SearchEngine {
    pub fn open_or_create(path: &Path) -> Result<Self, SearchError> {
        let (tx, rx) = mpsc::channel(1000);
        
        // 启动后台索引线程
        let manager_clone = manager.clone();
        let schema_clone = schema.clone();
        std::thread::spawn(move || {
            let mut writer = manager_clone.writer().unwrap();
            let mut batch_size = 0;
            const BATCH_COMMIT_THRESHOLD: usize = 100;
            
            while let Ok(op) = rx.recv() {
                match op {
                    IndexOperation::Add { doc_id, fields } => {
                        let doc = schema_clone.to_document(&doc_id, &fields);
                        writer.add_document(doc).unwrap();
                        batch_size += 1;
                        
                        // 自动批量提交
                        if batch_size >= BATCH_COMMIT_THRESHOLD {
                            writer.commit().unwrap();
                            batch_size = 0;
                        }
                    }
                    IndexOperation::Delete { doc_id } => {
                        let term = Term::from_field_text(schema_clone.document_id, &doc_id);
                        writer.delete_term(term);
                    }
                    IndexOperation::Commit => {
                        writer.commit().unwrap();
                        batch_size = 0;
                    }
                }
            }
        });
        
        Ok(Self { manager, schema, tx })
    }
    
    async fn index(&self, doc_id: &str, content: &str) -> Result<(), SearchError> {
        let mut fields = HashMap::new();
        fields.insert("content".to_string(), content.to_string());
        
        self.tx.send(IndexOperation::Add {
            doc_id: doc_id.to_string(),
            fields,
        }).await
        .map_err(|e| SearchError::Internal(e.to_string()))?;
        
        Ok(())
    }
}
```

**优点**:
- ✅ 完全异步，不阻塞 Tokio 线程池
- ✅ 自动批量提交
- ✅ 写操作串行化，避免并发冲突
- ✅ 可扩展（可添加队列、背压等机制）

**缺点**:
- ⚠️ 实现复杂度稍高
- ⚠️ 需要管理后台线程生命周期

### 4.3 📊 方案对比

| 特性 | 方案 A（单例 Writer） | 方案 B（通道批量） |
|------|---------------------|-------------------|
| 实现复杂度 | ⭐⭐ 简单 | ⭐⭐⭐ 中等 |
| 性能 | ⭐⭐⭐⭐ 好 | ⭐⭐⭐⭐⭐ 最佳 |
| 内存占用 | ⭐⭐⭐⭐ 低 | ⭐⭐⭐ 中 |
| 并发安全 | ✅ 是 | ✅ 是 |
| 批量优化 | ⭐⭐⭐ 手动 | ⭐⭐⭐⭐⭐ 自动 |
| 推荐场景 | 中小规模 | 大规模高并发 |

---

## 5. 其他问题

### 5.1 ⚠️ 缺少自动 Merge 策略

**问题**: Tantivy 需要定期 merge segment，当前没有配置。

**建议**:
```rust
pub fn writer(&self) -> Result<IndexWriter> {
    let mut writer = self.index.writer(self.config.writer_buffer_size)?;
    
    // 配置自动 merge 策略
    writer.set_merge_policy(
        tantivy::merge_policy::LogMergePolicy::default()
    );
    
    Ok(writer)
}
```

### 5.2 ⚠️ 缺少错误恢复

**问题**: 如果 commit 失败，writer 可能处于不一致状态。

**建议**:
```rust
async fn commit(&self) -> Result<(), SearchError> {
    let mut writer_guard = self.writer.lock().await;
    
    match writer_guard.commit() {
        Ok(_) => Ok(()),
        Err(e) => {
            // 重新创建 writer
            *writer_guard = self.manager.writer()
                .map_err(|create_err| SearchError::Bm25Error(
                    format!("Failed to recreate writer: {}", create_err)
                ))?;
            Err(SearchError::Bm25Error(format!("Commit failed: {}", e)))
        }
    }
}
```

---

## 6. 总结

### 6.1 核心问题

1. **每次操作创建新 Writer** - 资源浪费，性能差
2. **每个文档单独提交** - 磁盘 I/O 爆炸
3. **spawn_blocking 可能耗尽线程池** - 高并发风险

### 6.2 推荐修复优先级

| 优先级 | 问题 | 建议方案 |
|--------|------|----------|
| 🔴 **高** | 每次创建新 Writer | 采用单例 Writer 模式 |
| 🔴 **高** | 每个文档单独提交 | 批量 commit（100 文档或定时） |
| 🟡 **中** | spawn_blocking 线程池 | 采用通道批量处理 |
| 🟡 **中** | 缺少 merge 策略 | 配置 LogMergePolicy |
| 🟢 **低** | 错误恢复 | 添加重试/重建机制 |

### 6.3 当前已修复

✅ **GraphDB 层已实现 Writer 缓存**（本次修改）:
```rust
pub struct Bm25SearchEngine {
    writer: Arc<Mutex<Option<IndexWriter>>>,
}
```

⚠️ **但 bm25-service crate 层仍需改进**:
- `add_document()` 仍创建新 writer 并立即 commit
- 需要修改为接收 writer 参数或内部不 commit

### 6.4 下一步行动

1. **修改 bm25-service crate**:
   - `add_document()` 不创建 writer，改为接收 writer 参数
   - 或者添加 `add_document_no_commit()` 变体

2. **实现批量提交策略**:
   - 基于文档数量（如 100 文档）
   - 基于时间间隔（如 1 秒）
   - 基于内存大小（如 10MB）

3. **添加监控指标**:
   - Writer 创建次数
   - Commit 频率
   - Segment 数量
   - Merge 状态
