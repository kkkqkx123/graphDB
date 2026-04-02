# 全文检索 API 参考文档

## 概述

本文档描述 GraphDB 全文检索功能的 SQL 语法和 Rust API 接口。

全文检索通过 gRPC 与外部服务（crates/bm25 或 crates/inversearch）通信实现。

---

## SQL 语法

### 1. 创建全文索引

```sql
-- 基本语法
CREATE FULLTEXT INDEX <index_name> ON <tag_name>(<field_name>)

-- 指定服务提供者 (bm25 或 inversearch)
CREATE FULLTEXT INDEX idx_post_content ON Post(content) USING 'bm25'

-- 指定分词器 (仅 inversearch 支持)
CREATE FULLTEXT INDEX idx_post_content ON Post(content) 
USING 'inversearch' WITH TOKENIZER = 'cjk'
```

**参数说明**:
- `index_name`: 索引名称（唯一）
- `tag_name`: 标签名称
- `field_name`: 字段名称
- `USING`: 服务提供者，可选 'bm25' 或 'inversearch'
- `TOKENIZER`: 分词器类型，可选 'standard', 'cjk', 'whitespace'

**示例**:
```sql
CREATE FULLTEXT INDEX idx_article_title ON Article(title)
CREATE FULLTEXT INDEX idx_product_desc ON Product(description) USING 'bm25'
```

---

### 2. 删除全文索引

```sql
-- 基本语法
DROP FULLTEXT INDEX <index_name>

-- 示例
DROP FULLTEXT INDEX idx_post_content
```

---

### 3. 重建全文索引

```sql
-- 基本语法
REBUILD FULLTEXT INDEX <index_name>

-- 示例
REBUILD FULLTEXT INDEX idx_post_content
```

**说明**: 重建索引会清空现有索引并重新索引所有数据，用于数据修复。

---

### 4. 全文搜索

#### 4.1 MATCH 表达式

```sql
-- 基本搜索
MATCH (v:Post)
WHERE v.content MATCH "关键词"
RETURN v

-- 多词搜索（OR 关系）
MATCH (v:Post)
WHERE v.content MATCH "数据库 图数据库"
RETURN v

-- 短语搜索（精确匹配）
MATCH (v:Post)
WHERE v.content MATCH ""图数据库""
RETURN v

-- 带评分排序
MATCH (v:Article)
WHERE v.content MATCH "全文检索"
RETURN v, score(v) as relevance
ORDER BY relevance DESC
LIMIT 10

-- 设置评分阈值
MATCH (v:Post)
WHERE v.content MATCH "关键词" AND score(v) > 0.5
RETURN v
```

#### 4.2 CONTAINS 表达式

```sql
-- 基本搜索（与 MATCH 类似）
MATCH (v:Post)
WHERE v.content CONTAINS "关键词"
RETURN v

-- 多字段搜索
MATCH (p:Product)
WHERE p.name CONTAINS "手机" OR p.description CONTAINS "5G"
RETURN p
```

#### 4.3 LOOKUP 语法

```sql
-- 使用索引直接搜索（不查询图数据）
LOOKUP ON idx_post_content WHERE QUERY("搜索文本")
RETURN *

-- 带限制
LOOKUP ON idx_post_content WHERE QUERY("搜索文本")
RETURN *
LIMIT 20

-- 返回评分
LOOKUP ON idx_post_content WHERE QUERY("搜索文本")
RETURN doc_id, score
ORDER BY score DESC
```

---

### 5. 查看索引信息

```sql
-- 查看所有全文索引
SHOW FULLTEXT INDEXES

-- 查看指定索引状态
SHOW FULLTEXT INDEX STATUS idx_post_content

-- 查看索引统计
SHOW FULLTEXT INDEX STATS idx_post_content
```

---

## Rust API 接口

### 1. 配置

```rust
use graphdb::storage::fulltext::{FulltextConfig, SyncMode};

let config = FulltextConfig {
    enabled: true,
    endpoint: "http://127.0.0.1:50051".to_string(),
    timeout_ms: 5000,
    retry_count: 3,
    sync: SyncConfig {
        mode: SyncMode::Async,
        queue_size: 10000,
        batch_size: 100,
    },
};
```

### 2. 创建索引

```rust
use graphdb::storage::StorageClient;

// 创建全文索引
storage.create_fulltext_index(
    space_id,           // 空间 ID
    "Post",            // 标签名
    "content",         // 字段名
    Some("bm25"),      // 服务提供者
).await?;
```

### 3. 搜索

```rust
use graphdb::storage::fulltext::FulltextResult;

// 执行全文搜索
let results: Vec<FulltextResult> = storage
    .fulltext_search("idx_post_content", "图数据库", 10)
    .await?;

for result in results {
    println!("Doc ID: {:?}", result.doc_id);
    println!("Score: {}", result.score);
    
    if let Some(highlights) = result.highlights {
        for h in highlights {
            println!("Highlight: {}", h);
        }
    }
}
```

### 4. 手动同步

```rust
use graphdb::storage::fulltext::FulltextSyncManager;

// 获取同步管理器
let sync_manager = storage.fulltext_sync_manager();

// 手动触发同步
sync_manager.sync_vertex(
    space_id,
    "Post",
    &vertex_id,
    &properties,
).await?;

// 批量同步
let tasks = vec![
    SyncTask::IndexDocument { ... },
    SyncTask::IndexDocument { ... },
];
sync_manager.batch_sync(tasks).await?;
```

---

## 配置选项

### GraphDB 配置 (config.toml)

```toml
[fulltext]
# 是否启用全文检索
enabled = true

# gRPC 服务端点
endpoint = "http://127.0.0.1:50051"

# 请求超时（毫秒）
timeout_ms = 5000

# 重试次数
retry_count = 3

# 同步配置
[fulltext.sync]
# 同步模式: sync(同步) / async(异步) / off(关闭)
mode = "async"

# 异步队列大小
queue_size = 10000

# 批量处理大小
batch_size = 100

# 失败重试间隔（毫秒）
retry_interval_ms = 100
```

---

## 完整示例

### 示例 1：博客系统搜索

```rust
use graphdb::storage::fulltext::*;
use graphdb::storage::StorageClient;

async fn blog_search_example(storage: &impl StorageClient) -> Result<(), Box<dyn std::error::Error>> {
    // 1. 创建全文索引
    storage.create_fulltext_index(
        1,                      // space_id
        "BlogPost",            // tag_name
        "content",             // field_name
        Some("bm25"),          // provider
    ).await?;
    
    println!("全文索引创建成功");
    
    // 2. 插入数据（自动同步到全文索引）
    let query = r#"
        INSERT VERTEX BlogPost(title, content) VALUES 
        ("post_001", "图数据库入门指南", "图数据库是一种专门用于存储和查询图结构数据的数据库系统..."),
        ("post_002", "Rust 编程语言", "Rust 是一种系统级编程语言，具有内存安全和并发安全特性...")
    "#;
    
    storage.execute(query).await?;
    println!("数据插入成功，已自动同步到全文索引");
    
    // 3. 执行全文搜索
    let results = storage
        .fulltext_search("idx_blogpost_content", "图数据库", 10)
        .await?;
    
    println!("找到 {} 个结果", results.len());
    
    for result in results {
        println!("文档: {:?}, 评分: {:.4}", result.doc_id, result.score);
    }
    
    Ok(())
}
```

### 示例 2：产品搜索（带过滤）

```sql
-- 创建产品描述索引
CREATE FULLTEXT INDEX idx_product_desc ON Product(description) USING 'bm25';

-- 搜索产品（带价格过滤）
MATCH (p:Product)
WHERE p.description MATCH "无线耳机" 
  AND p.price < 1000
  AND score(p) > 0.3
RETURN p.name, p.price, score(p) as relevance
ORDER BY relevance DESC
LIMIT 20;

-- 搜索结果高亮
MATCH (p:Product)
WHERE p.description MATCH "降噪耳机"
RETURN p.name, 
       highlight(p.description) as highlighted_desc,
       score(p) as relevance
ORDER BY relevance DESC;
```

### 示例 3：多字段搜索

```sql
-- 创建多个索引
CREATE FULLTEXT INDEX idx_article_title ON Article(title) USING 'bm25';
CREATE FULLTEXT INDEX idx_article_content ON Article(content) USING 'bm25';

-- 多字段搜索（标题权重更高）
MATCH (a:Article)
WHERE a.title MATCH "Rust" OR a.content MATCH "Rust"
RETURN a, 
       score(a.title) * 2.0 + score(a.content) as weighted_score
ORDER BY weighted_score DESC;
```

---

## 错误处理

### 常见错误

| 错误码 | 描述 | 解决方案 |
|--------|------|----------|
| `FulltextServiceUnavailable` | 全文检索服务不可用 | 检查服务是否启动 |
| `IndexNotFound` | 索引不存在 | 检查索引名称或创建索引 |
| `IndexAlreadyExists` | 索引已存在 | 删除旧索引或使用新名称 |
| `InvalidQuery` | 查询语法错误 | 检查查询字符串 |
| `SyncTimeout` | 同步超时 | 增加超时时间或检查服务状态 |

### 错误处理示例

```rust
use graphdb::storage::fulltext::FulltextError;

match storage.fulltext_search("idx_name", "query", 10).await {
    Ok(results) => {
        // 处理结果
    }
    Err(FulltextError::ServiceUnavailable(msg)) => {
        eprintln!("全文检索服务不可用: {}", msg);
        eprintln!("请确保服务已启动: ./crates/bm25/target/release/bm25-service");
    }
    Err(FulltextError::IndexNotFound(name)) => {
        eprintln!("索引不存在: {}", name);
        eprintln!("请先创建索引: CREATE FULLTEXT INDEX {} ON ...", name);
    }
    Err(e) => {
        eprintln!("搜索失败: {:?}", e);
    }
}
```

---

## 性能优化建议

### 1. 批量操作

```rust
// 批量索引（推荐）
let documents = vec![
    ("doc_1".to_string(), fields_1),
    ("doc_2".to_string(), fields_2),
    // ...
];

storage.batch_index_documents("idx_name", documents).await?;
```

### 2. 异步同步

```toml
# config.toml
[fulltext.sync]
mode = "async"      # 使用异步模式提高写入性能
queue_size = 10000  # 增大队列缓冲区
batch_size = 100    # 批量处理大小
```

### 3. 查询优化

```sql
-- 使用 LIMIT 限制结果数
MATCH (v:Post)
WHERE v.content MATCH "关键词"
RETURN v
LIMIT 100

-- 使用评分阈值过滤低质量结果
MATCH (v:Post)
WHERE v.content MATCH "关键词" AND score(v) > 0.3
RETURN v
```

---

## 注意事项

1. **服务依赖**: 使用全文检索前需确保外部服务已启动
2. **最终一致性**: 异步模式下数据同步有短暂延迟
3. **存储空间**: 全文索引占用额外磁盘空间，需定期监控
4. **网络通信**: 虽然 localhost 开销小，但仍需处理超时和重试
