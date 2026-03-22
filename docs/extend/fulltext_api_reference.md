# 全文检索 API 参考文档

## 概述

本文档描述 GraphDB 全文检索功能的 API 接口和使用方法。

---

## SQL 语法扩展

### 1. 创建全文索引

```sql
-- 基本语法
CREATE FULLTEXT INDEX <index_name> ON <tag_name>(<field_name>)

-- 示例
CREATE FULLTEXT INDEX idx_post_content ON Post(content)
CREATE FULLTEXT INDEX idx_article_title ON Article(title)
```

**参数说明**:
- `index_name`: 索引名称（唯一）
- `tag_name`: 标签名称
- `field_name`: 字段名称

---

### 2. 删除全文索引

```sql
-- 基本语法
DROP FULLTEXT INDEX <index_name>

-- 示例
DROP FULLTEXT INDEX idx_post_content
```

---

### 3. 全文搜索

#### 3.1 CONTAINS 表达式

```sql
-- 基本搜索
MATCH (v:Post)
WHERE v.content CONTAINS "关键词"
RETURN v

-- 多词搜索（OR 关系）
MATCH (v:Post)
WHERE v.content CONTAINS "数据库 图数据库"
RETURN v

-- 短语搜索
MATCH (v:Post)
WHERE v.content CONTAINS ""图数据库""
RETURN v
```

#### 3.2 MATCH 表达式

```sql
-- 基本搜索
MATCH (v:Article)
WHERE v.title MATCH "BM25 算法"
RETURN v

-- 带评分排序
MATCH (v:Article)
WHERE v.content MATCH "全文检索"
RETURN v, score(v) as relevance
ORDER BY relevance DESC
LIMIT 10
```

#### 3.3 LOOKUP 语法

```sql
-- 使用索引直接搜索
LOOKUP ON idx_post_content WHERE QUERY("搜索文本")
RETURN *

-- 带限制
LOOKUP ON idx_post_content WHERE QUERY("搜索文本")
RETURN *
LIMIT 20
```

---

### 4. 评分函数

```sql
-- 获取相关性评分
MATCH (v:Post)
WHERE v.content MATCH "关键词"
RETURN v, score(v) as score
ORDER BY score DESC

-- 设置评分阈值
MATCH (v:Post)
WHERE v.content MATCH "关键词" AND score(v) > 0.5
RETURN v
```

---

## Rust API 接口

### 1. 全文索引管理

```rust
use graphdb::storage::fulltext::{FulltextIndexConfig, FulltextOptions};
use graphdb::storage::StorageClient;

// 创建全文索引
let config = FulltextIndexConfig {
    index_name: "idx_post_content".to_string(),
    space_id: 1,
    schema_name: "Post".to_string(),
    field_name: "content".to_string(),
    provider: FulltextProviderType::Tantivy,
    tokenizer: TokenizerType::Cjk,
    options: FulltextOptions::default(),
};

storage.create_fulltext_index(config).await?;
```

### 2. 文档索引

```rust
use graphdb::core::Value;

// 索引单个文档
let doc_id = Value::String("post_001".to_string());
let content = "这是一篇关于图数据库的文章...";

storage.index_document("idx_post_content", &doc_id, content).await?;

// 批量索引
let documents = vec![
    (Value::String("post_001".to_string()), "内容1...".to_string()),
    (Value::String("post_002".to_string()), "内容2...".to_string()),
];

storage.batch_index_documents("idx_post_content", documents).await?;
```

### 3. 全文搜索

```rust
use graphdb::storage::fulltext::{SearchOptions, SearchResults};

// 基本搜索
let options = SearchOptions {
    limit: 10,
    offset: 0,
    highlight: true,
    field_weights: None,
};

let results: SearchResults = storage
    .fulltext_search("idx_post_content", "图数据库", &options)
    .await?;

// 处理结果
for result in results.results {
    println!("Doc ID: {:?}, Score: {}", result.doc_id, result.score);
    if let Some(highlights) = result.highlights {
        for highlight in highlights {
            println!("Highlight: {}", highlight);
        }
    }
}
```

### 4. 索引统计

```rust
// 获取索引统计信息
let stats = storage.get_fulltext_stats("idx_post_content").await?;

println!("文档数: {}", stats.doc_count);
println!("词项数: {}", stats.term_count);
println!("平均文档长度: {}", stats.avg_doc_length);
```

---

## 配置选项

### 1. 索引配置

```rust
FulltextOptions {
    store_doc: true,            // 是否存储文档内容
    store_positions: true,      // 是否存储词位置（用于高亮）
    bm25_k1: 1.2,              // BM25 k1 参数（控制词频饱和度）
    bm25_b: 0.75,              // BM25 b 参数（控制文档长度归一化）
}
```

**参数说明**:
- `bm25_k1`: 通常取值 1.2-2.0，值越大词频影响越大
- `bm25_b`: 通常取值 0.0-1.0，0.0 表示不考虑文档长度

### 2. 分词器选择

| 分词器 | 适用场景 | 示例 |
|--------|----------|------|
| `Standard` | 英文文本 | "Hello world" → ["hello", "world"] |
| `Cjk` | 中日韩文本 | "图数据库" → ["图", "数", "据", "库"] |
| `Whitespace` | 简单分词 | "a b c" → ["a", "b", "c"] |
| `Raw` | 不分词 | "keyword" → ["keyword"] |

---

## 错误处理

### 常见错误码

| 错误码 | 描述 | 解决方案 |
|--------|------|----------|
| `IndexNotFound` | 索引不存在 | 检查索引名称是否正确 |
| `IndexAlreadyExists` | 索引已存在 | 删除旧索引或使用新名称 |
| `DocumentNotFound` | 文档不存在 | 检查文档 ID 是否正确 |
| `InvalidQuery` | 查询语法错误 | 检查查询字符串格式 |
| `IndexCorrupted` | 索引损坏 | 重建索引 |

### 错误示例

```rust
use graphdb::storage::fulltext::FulltextError;

match storage.fulltext_search("idx_name", "query", &options).await {
    Ok(results) => {
        // 处理结果
    }
    Err(FulltextError::IndexNotFound(name)) => {
        eprintln!("索引不存在: {}", name);
    }
    Err(FulltextError::InvalidQuery(msg)) => {
        eprintln!("查询语法错误: {}", msg);
    }
    Err(e) => {
        eprintln!("搜索失败: {:?}", e);
    }
}
```

---

## 性能优化建议

### 1. 索引优化

```rust
// 批量索引（推荐）
let batch_size = 1000;
for chunk in documents.chunks(batch_size) {
    storage.batch_index_documents("idx_name", chunk.to_vec()).await?;
}

// 定期提交
storage.commit_fulltext_index("idx_name").await?;
```

### 2. 查询优化

```sql
-- 使用 LIMIT 限制结果数
MATCH (v:Post)
WHERE v.content MATCH "关键词"
RETURN v
LIMIT 100

-- 使用评分阈值过滤
MATCH (v:Post)
WHERE v.content MATCH "关键词" AND score(v) > 0.3
RETURN v
```

### 3. 硬件建议

- **内存**: 至少 4GB 可用内存
- **磁盘**: SSD 推荐，索引文件可能较大
- **CPU**: 多核有助于并发查询

---

## 完整示例

### 示例 1：博客文章搜索

```rust
use graphdb::storage::fulltext::*;
use graphdb::storage::StorageClient;
use graphdb::core::Value;

async fn blog_search_example(storage: &impl StorageClient) -> Result<(), Box<dyn std::error::Error>> {
    // 1. 创建全文索引
    let config = FulltextIndexConfig {
        index_name: "idx_blog_content".to_string(),
        space_id: 1,
        schema_name: "BlogPost".to_string(),
        field_name: "content".to_string(),
        provider: FulltextProviderType::Tantivy,
        tokenizer: TokenizerType::Cjk,
        options: FulltextOptions {
            store_doc: true,
            store_positions: true,
            bm25_k1: 1.5,
            bm25_b: 0.75,
        },
    };
    
    storage.create_fulltext_index(config).await?;
    
    // 2. 索引文章
    let articles = vec![
        (Value::String("post_001".to_string()), 
         "图数据库是一种专门用于存储和查询图结构数据的数据库系统...".to_string()),
        (Value::String("post_002".to_string()), 
         "Rust 是一种系统级编程语言，具有内存安全和并发安全特性...".to_string()),
    ];
    
    storage.batch_index_documents("idx_blog_content", articles).await?;
    
    // 3. 搜索文章
    let options = SearchOptions {
        limit: 10,
        offset: 0,
        highlight: true,
        field_weights: None,
    };
    
    let results = storage
        .fulltext_search("idx_blog_content", "图数据库 Rust", &options)
        .await?;
    
    println!("找到 {} 个结果", results.total);
    
    for result in results.results {
        println!("文档: {:?}, 评分: {:.4}", result.doc_id, result.score);
    }
    
    Ok(())
}
```

### 示例 2：产品搜索

```sql
-- 创建产品描述索引
CREATE FULLTEXT INDEX idx_product_desc ON Product(description);

-- 搜索产品
MATCH (p:Product)
WHERE p.description MATCH "无线耳机 降噪"
RETURN p.name, p.price, score(p) as relevance
ORDER BY relevance DESC
LIMIT 20;

-- 带价格过滤的搜索
MATCH (p:Product)
WHERE p.description MATCH "无线耳机" 
  AND p.price < 1000
  AND score(p) > 0.5
RETURN p.name, p.price
ORDER BY score(p) DESC;
```

---

## 注意事项

1. **索引更新延迟**: 全文索引更新可能有短暂延迟，不适用于实时性要求极高的场景
2. **存储空间**: 全文索引可能占用较多磁盘空间，建议定期监控
3. **并发写入**: 高并发写入时建议批量处理，避免频繁提交
4. **查询复杂度**: 复杂查询可能影响性能，建议使用简单的关键词查询
