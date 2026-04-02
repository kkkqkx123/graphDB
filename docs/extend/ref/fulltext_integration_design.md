# 全文检索服务集成设计方案

## 概述

本方案采用 **gRPC 服务架构** 集成全文检索功能。直接复用现有的 `ref/bm25` 和 `ref/inversearch` 服务，GraphDB 核心仅实现客户端通信和数据同步逻辑。

## 为什么采用 gRPC 服务架构

### 1. 存储结构本质差异

| 组件 | 存储引擎 | 事务支持 | 索引类型 |
|------|----------|----------|----------|
| GraphDB | Redb (LSM-Tree) | 完整 MVCC | B-tree |
| BM25 | Tantivy | 无 | 倒排索引 |
| Inversearch | 内存哈希 | 无 | 自定义倒排 |

**强行内嵌的问题**:
- 两套 WAL 机制冲突
- 缓存策略互相干扰
- 备份恢复无法统一
- 破坏轻量设计原则

### 2. gRPC 方案的优势

- **本地性能**: localhost 通信 < 1ms，开销可忽略
- **资源隔离**: 服务崩溃不影响数据库核心
- **存储独立**: 索引与数据完全分离
- **可选组件**: 不需要时可不部署
- **即插即用**: 复用现有服务，无需重新实现

## 系统架构

```
┌─────────────────────────────────────────────────────────────────┐
│                        用户应用层                                │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      GraphDB 核心进程                            │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────────────┐ │
│  │   HTTP API  │────│ GraphService│────│   查询执行引擎       │ │
│  └─────────────┘    └─────────────┘    └─────────────────────┘ │
│                                                 │               │
│  ┌──────────────────────────────────────────────┘               │
│  │                                                               │
│  │  ┌─────────────────┐    ┌─────────────────┐                  │
│  │  │ 全文检索客户端   │    │ 数据同步管理器   │                  │
│  │  │ (gRPC Client)   │    │ (SyncManager)   │                  │
│  │  └─────────────────┘    └─────────────────┘                  │
│  │           │                        │                         │
│  └───────────┼────────────────────────┼─────────────────────────┘
│              │                        │
│              │ gRPC                   │ 数据变更通知
│              ▼                        ▼
│  ┌───────────────────────────────────────────────────────────┐  │
│  │ Redb 存储 (图数据 + 普通索引)                               │  │
│  └───────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
              │
              │ localhost gRPC
              ▼
┌─────────────────────────────────────────────────────────────────┐
│              全文检索服务进程 (独立部署)                          │
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │ 方案 A: ref/bm25 (Tantivy)                               │    │
│  │  - 成熟的全文搜索引擎                                     │    │
│  │  - 支持 BM25 评分                                         │    │
│  │  - 适合长文本搜索                                         │    │
│  └─────────────────────────────────────────────────────────┘    │
│                              或                                  │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │ 方案 B: ref/inversearch (自定义)                         │    │
│  │  - 轻量级实现                                             │    │
│  │  - 支持多种分词策略                                       │    │
│  │  - 适合关键词搜索                                         │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
│  独立存储目录: data/fulltext/                                    │
│  独立配置: configs/fulltext.toml                                 │
└─────────────────────────────────────────────────────────────────┘
```

## 核心组件设计

### 1. 全文检索客户端

```rust
// src/storage/fulltext/client.rs

use tonic::transport::Channel;
use bm25_proto::bm25_service_client::Bm25ServiceClient;
use bm25_proto::{SearchRequest, IndexDocumentRequest};

pub struct FulltextClient {
    client: Bm25ServiceClient<Channel>,
    config: FulltextClientConfig,
}

#[derive(Debug, Clone)]
pub struct FulltextClientConfig {
    pub endpoint: String,           // 如: "http://127.0.0.1:50051"
    pub timeout_ms: u64,
    pub retry_count: u32,
}

impl FulltextClient {
    pub async fn connect(config: FulltextClientConfig) -> Result<Self> {
        let endpoint = Endpoint::from_shared(config.endpoint.clone())?
            .timeout(Duration::from_millis(config.timeout_ms));
        
        let client = Bm25ServiceClient::connect(endpoint).await?;
        
        Ok(Self { client, config })
    }
    
    /// 索引文档
    pub async fn index_document(
        &mut self,
        index_name: &str,
        doc_id: &str,
        fields: HashMap<String, String>,
    ) -> Result<()> {
        let request = IndexDocumentRequest {
            index_name: index_name.to_string(),
            document_id: doc_id.to_string(),
            fields,
        };
        
        self.client.index_document(request).await?;
        Ok(())
    }
    
    /// 搜索
    pub async fn search(
        &mut self,
        index_name: &str,
        query: &str,
        limit: i32,
    ) -> Result<SearchResults> {
        let request = SearchRequest {
            index_name: index_name.to_string(),
            query: query.to_string(),
            limit,
            offset: 0,
            field_weights: HashMap::new(),
            highlight: true,
        };
        
        let response = self.client.search(request).await?;
        Ok(SearchResults::from(response.into_inner()))
    }
}
```

### 2. 数据同步管理器

```rust
// src/storage/fulltext/sync.rs

/// 数据同步管理器
/// 负责将图数据的变更同步到全文检索服务
pub struct FulltextSyncManager {
    client: FulltextClient,
    /// 索引配置: (space_id, tag_name, field_name) -> index_name
    index_mappings: HashMap<(u64, String, String), String>,
}

impl FulltextSyncManager {
    /// 顶点变更时同步到全文索引
    pub async fn on_vertex_inserted(
        &self,
        space_id: u64,
        tag_name: &str,
        vertex_id: &Value,
        properties: &HashMap<String, Value>,
    ) -> Result<()> {
        // 查找需要索引的字段
        for (field_name, value) in properties {
            let key = (space_id, tag_name.to_string(), field_name.clone());
            
            if let Some(index_name) = self.index_mappings.get(&key) {
                // 只索引字符串类型的字段
                if let Value::String(text) = value {
                    let mut fields = HashMap::new();
                    fields.insert("doc_id".to_string(), vertex_id.to_string());
                    fields.insert("content".to_string(), text.clone());
                    
                    self.client.index_document(
                        index_name,
                        &vertex_id.to_string(),
                        fields,
                    ).await?;
                }
            }
        }
        
        Ok(())
    }
    
    /// 顶点删除时从全文索引移除
    pub async fn on_vertex_deleted(
        &self,
        space_id: u64,
        tag_name: &str,
        vertex_id: &Value,
    ) -> Result<()> {
        // 获取该标签的所有索引字段
        let prefix = (space_id, tag_name.to_string(), String::new());
        
        for (key, index_name) in self.index_mappings.iter() {
            if key.0 == space_id && key.1 == tag_name {
                self.client.delete_document(
                    index_name,
                    &vertex_id.to_string(),
                ).await?;
            }
        }
        
        Ok(())
    }
}
```

### 3. 存储层集成

```rust
// src/storage/fulltext/mod.rs

pub mod client;
pub mod sync;
pub mod types;

use crate::storage::StorageClient;

/// 扩展 StorageClient trait 支持全文检索
#[async_trait::async_trait]
pub trait FulltextStorage: StorageClient {
    /// 创建全文索引
    async fn create_fulltext_index(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
    ) -> Result<String>;
    
    /// 删除全文索引
    async fn drop_fulltext_index(&self, index_name: &str) -> Result<()>;
    
    /// 全文搜索
    async fn fulltext_search(
        &self,
        index_name: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<FulltextResult>>;
}

/// 全文搜索结果
#[derive(Debug, Clone)]
pub struct FulltextResult {
    pub doc_id: Value,
    pub score: f32,
    pub highlights: Option<Vec<String>>,
}
```

## 数据流设计

### 1. 创建全文索引

```
用户
  │ CREATE FULLTEXT INDEX idx_content ON Post(content)
  ▼
GraphDB 查询引擎
  │ 1. 解析 SQL
  │ 2. 创建索引元数据 (Redb)
  │ 3. 通知全文服务创建索引
  ▼
FulltextSyncManager
  │ 调用 bm25.CreateIndex("idx_content")
  ▼
BM25 服务
  │ 创建 Tantivy 索引
  ▼
返回成功
```

### 2. 插入数据同步

```
用户
  │ INSERT VERTEX Post(content) VALUES "图数据库文章"
  ▼
GraphDB 存储层
  │ 1. 写入 Redb (事务)
  │ 2. 提交事务
  │ 3. 触发同步钩子
  ▼
FulltextSyncManager
  │ 检查字段是否有全文索引
  │ 是 → 调用 bm25.IndexDocument()
  ▼
BM25 服务
  │ 索引文档
  ▼
异步完成
```

### 3. 全文搜索

```
用户
  │ MATCH (p:Post) WHERE p.content MATCH "图数据库"
  ▼
GraphDB 查询引擎
  │ 1. 解析 MATCH 表达式
  │ 2. 调用 FulltextClient.search()
  ▼
BM25 服务
  │ 执行搜索，返回 doc_ids
  ▼
GraphDB
  │ 根据 doc_ids 查询完整数据 (Redb)
  ▼
返回结果给用户
```

## 配置设计

### GraphDB 配置 (config.toml)

```toml
[fulltext]
# 是否启用全文检索
enabled = true

# 服务连接配置
endpoint = "http://127.0.0.1:50051"
timeout_ms = 5000
retry_count = 3

# 同步策略
[fulltext.sync]
# 同步模式: sync(同步) / async(异步) / off(关闭)
mode = "async"
# 异步队列大小
queue_size = 10000
# 批量处理大小
batch_size = 100

# 索引配置
[[fulltext.index]]
space = "default"
tag = "Post"
field = "content"
index_name = "idx_post_content"
provider = "bm25"  # 或 "inversearch"
```

### 服务启动脚本

```powershell
# start_services.ps1

# 1. 启动 BM25 服务
Start-Process -FilePath "ref/bm25/target/release/bm25-service.exe" `
    -ArgumentList "--config", "configs/bm25.toml" `
    -WindowStyle Hidden

# 2. 等待服务就绪
Start-Sleep -Seconds 2

# 3. 启动 GraphDB
Start-Process -FilePath "target/release/graphdb-server.exe" `
    -ArgumentList "--config", "config.toml"
```

## SQL 语法扩展

### 创建全文索引

```sql
-- 创建全文索引
CREATE FULLTEXT INDEX idx_post_content ON Post(content);

-- 指定分词器 (针对 inversearch)
CREATE FULLTEXT INDEX idx_post_content ON Post(content) 
WITH TOKENIZER = 'cjk';

-- 指定服务提供者
CREATE FULLTEXT INDEX idx_post_content ON Post(content) 
USING 'bm25';
```

### 全文搜索

```sql
-- 基本搜索
MATCH (p:Post)
WHERE p.content MATCH "图数据库"
RETURN p

-- 带评分排序
MATCH (p:Post)
WHERE p.content MATCH "图数据库"
RETURN p, score(p) as relevance
ORDER BY relevance DESC
LIMIT 10;

-- 多字段搜索
MATCH (p:Post)
WHERE p.title MATCH "Rust" OR p.content MATCH "图数据库"
RETURN p

-- 使用索引直接搜索
LOOKUP ON idx_post_content WHERE QUERY("搜索文本")
RETURN *
```

## 错误处理与容错

### 1. 服务不可用处理

```rust
pub enum FulltextError {
    /// 服务连接失败
    ServiceUnavailable(String),
    /// 索引不存在
    IndexNotFound(String),
    /// 查询语法错误
    InvalidQuery(String),
    /// 超时
    Timeout,
}

impl FulltextSyncManager {
    /// 带重试的同步操作
    pub async fn sync_with_retry<F, Fut>(
        &self,
        operation: F,
    ) -> Result<()>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<()>>,
    {
        let mut retries = 0;
        loop {
            match operation().await {
                Ok(()) => return Ok(()),
                Err(e) if retries < self.config.retry_count => {
                    retries += 1;
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
                Err(e) => {
                    // 记录错误，但不影响主流程
                    log::error!("全文索引同步失败: {:?}", e);
                    return Ok(());  // 静默失败
                }
            }
        }
    }
}
```

### 2. 数据一致性策略

| 策略 | 说明 | 适用场景 |
|------|------|----------|
| **最终一致性** | 异步同步，允许短暂不一致 | 默认推荐 |
| **强一致性** | 同步等待索引完成 | 关键业务 |
| **手动同步** | 提供重建索引命令 | 修复数据 |

```sql
-- 手动重建全文索引
REBUILD FULLTEXT INDEX idx_post_content;

-- 查看索引同步状态
SHOW FULLTEXT INDEX STATUS;
```

## 部署方案

### 单机部署 (推荐)

```
┌─────────────────────────────────────┐
│           单机服务器                 │
│                                     │
│  ┌─────────────┐  ┌─────────────┐  │
│  │ GraphDB     │  │ BM25 Service│  │
│  │ :8080       │  │ :50051      │  │
│  └─────────────┘  └─────────────┘  │
│                                     │
│  数据目录:                          │
│  - data/graphdb/   (Redb)          │
│  - data/fulltext/  (Tantivy)       │
│                                     │
└─────────────────────────────────────┘
```

### 开发环境

```powershell
# 一键启动开发环境
./scripts/start-dev.ps1

# 该脚本会:
# 1. 检查 BM25 服务是否已编译
# 2. 启动 BM25 服务
# 3. 启动 GraphDB
# 4. 初始化测试数据
```

## 性能预期

| 指标 | 预期值 | 说明 |
|------|--------|------|
| 单次搜索延迟 | 5-15ms | 含网络往返 |
| 批量索引速度 | 3000-5000 doc/s | 异步模式 |
| 服务启动时间 | < 2s | 冷启动 |
| 内存占用 | +200-500MB | 相比纯 GraphDB |

## 总结

本方案的核心思想：

1. **复用现有服务**: 直接使用 ref/bm25 和 ref/inversearch，无需重新实现
2. **轻量集成**: GraphDB 核心只增加 gRPC 客户端逻辑
3. **可选依赖**: 全文检索作为可选组件，不影响核心功能
4. **本地优先**: localhost 通信，性能开销可忽略

下一步：参考 `fulltext_implementation_plan.md` 开始具体实现。
