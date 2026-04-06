# Inversearch 集成测试方案

## 概述

本文档定义了 Inversearch 服务的集成测试方案。Inversearch 是一个高性能的倒排索引搜索服务，支持关键词搜索、模糊匹配、短语查询等功能。

## 测试目标

1. 验证核心搜索功能的正确性
2. 确保 gRPC 服务接口的稳定性
3. 测试多语言（CJK、拉丁语系等）支持
4. 验证高亮、建议等高级功能
5. 确保索引操作的可靠性

## 测试目录结构

```
inversearch/
└── tests/
    ├── integration/           # 集成测试
    │   ├── mod.rs            # 测试模块入口
    │   ├── search_test.rs    # 搜索功能测试
    │   ├── index_test.rs     # 索引操作测试
    │   ├── service_test.rs   # gRPC 服务测试
    │   ├── highlight_test.rs # 高亮功能测试
    │   └── charset_test.rs   # 字符集测试
    └── fixtures/             # 测试数据
        ├── documents.rs      # 测试文档数据
        └── queries.rs        # 测试查询数据
```

## 测试模块设计

### 1. 搜索功能测试 (search_test.rs)

测试搜索核心功能的正确性。

```rust
//! 搜索功能集成测试
//!
//! 测试范围：
//! - 基本搜索
//! - 多词搜索
//! - 分页搜索
//! - 空查询处理
//! - 无结果查询

use inversearch_service::{Index, IndexOptions, SearchOptions};

/// 测试基本搜索功能
/// 验证：添加文档后，可以正确搜索到
#[test]
fn test_basic_search() {
    // 创建索引
    // 添加测试文档
    // 执行搜索
    // 验证结果
}

/// 测试多词搜索
/// 验证：多个关键词的 AND/OR 搜索
#[test]
fn test_multi_term_search() {
    // 测试多词交集搜索
    // 测试多词并集搜索
}

/// 测试分页功能
/// 验证：limit 和 offset 参数正常工作
#[test]
fn test_pagination() {
    // 添加大量文档
    // 测试不同 limit 值
    // 测试 offset 偏移
}

/// 测试上下文搜索
/// 验证：context 选项返回正确的上下文信息
#[test]
fn test_context_search() {
    // 启用上下文索引
    // 执行带上下文的搜索
    // 验证上下文信息
}
```

### 2. 索引操作测试 (index_test.rs)

测试索引的增删改查操作。

```rust
//! 索引操作集成测试
//!
//! 测试范围：
//! - 文档添加
//! - 文档更新
//! - 文档删除
//! - 索引清空
//! - 批量操作

use inversearch_service::{Index, IndexOptions, Document};

/// 测试文档添加
/// 验证：添加的文档可以被搜索到
#[test]
fn test_add_document() {
    // 创建索引
    // 添加文档
    // 验证文档存在
    // 验证可以搜索到
}

/// 测试文档更新
/// 验证：更新后搜索返回新内容
#[test]
fn test_update_document() {
    // 添加文档
    // 更新文档内容
    // 验证旧内容搜索不到
    // 验证新内容可以搜索到
}

/// 测试文档删除
/// 验证：删除后文档不再出现在搜索结果中
#[test]
fn test_remove_document() {
    // 添加文档
    // 删除文档
    // 验证搜索不到
}

/// 测试索引清空
/// 验证：清空后所有文档都被移除
#[test]
fn test_clear_index() {
    // 添加多个文档
    // 清空索引
    // 验证所有文档都不存在
}

/// 测试批量添加
/// 验证：批量操作的原子性和性能
#[test]
fn test_batch_add() {
    // 准备批量文档
    // 执行批量添加
    // 验证所有文档都存在
}
```

### 3. gRPC 服务测试 (service_test.rs)

测试 gRPC 服务接口的正确性。

```rust
//! gRPC 服务集成测试
//!
//! 测试范围：
//! - AddDocument
//! - UpdateDocument
//! - RemoveDocument
//! - Search
//! - ClearIndex
//! - GetStats

use inversearch_service::service::InversearchService;
use inversearch_service::proto::*;
use tonic::Request;

/// 测试添加文档接口
#[tokio::test]
async fn test_grpc_add_document() {
    // 创建服务实例
    // 构建 AddDocumentRequest
    // 调用 add_document
    // 验证响应
}

/// 测试搜索接口
#[tokio::test]
async fn test_grpc_search() {
    // 添加测试文档
    // 构建 SearchRequest
    // 调用 search
    // 验证返回结果
}

/// 测试统计接口
#[tokio::test]
async fn test_grpc_get_stats() {
    // 添加文档
    // 调用 get_stats
    // 验证统计数据
}
```

### 4. 高亮功能测试 (highlight_test.rs)

测试搜索结果高亮功能。

```rust
//! 高亮功能集成测试
//!
//! 测试范围：
//! - 基本高亮
//! - 多字段高亮
//! - 高亮边界处理
//! - 不同字符集高亮

use inversearch_service::highlight::{highlight_fields, HighlightProcessor};

/// 测试基本高亮
#[test]
fn test_basic_highlight() {
    // 创建高亮处理器
    // 处理测试文本
    // 验证高亮标签
}

/// 测试多字段高亮
#[test]
fn test_multi_field_highlight() {
    // 准备多字段文档
    // 执行高亮
    // 验证各字段高亮结果
}

/// 测试 CJK 文本高亮
#[test]
fn test_cjk_highlight() {
    // 准备中文/日文/韩文文本
    // 执行高亮
    // 验证结果正确性
}
```

### 5. 字符集测试 (charset_test.rs)

测试不同字符集的处理能力。

```rust
//! 字符集集成测试
//!
//! 测试范围：
//! - 拉丁字符
//! - CJK 字符（中/日/韩）
//! - 阿拉伯字符
//! - 西里尔字符
//! - 印地语字符

use inversearch_service::charset::*;

/// 测试拉丁字符处理
#[test]
fn test_latin_charset() {
    // 测试拉丁字符分词
    // 测试大小写处理
    // 测试归一化
}

/// 测试 CJK 字符处理
#[test]
fn test_cjk_charset() {
    // 测试中文分词
    // 测试日文分词
    // 测试韩文分词
}

/// 测试阿拉伯字符处理
#[test]
fn test_arabic_charset() {
    // 测试阿拉伯字符归一化
    // 测试 RTL 支持
}
```

## 测试数据设计

### 文档数据 (fixtures/documents.rs)

```rust
//! 测试文档数据

pub struct TestDocument {
    pub id: u64,
    pub content: &'static str,
    pub metadata: Vec<(&'static str, &'static str)>,
}

/// 编程语言相关文档
pub const PROGRAMMING_DOCS: &[TestDocument] = &[
    TestDocument {
        id: 1,
        content: "Rust is a systems programming language focused on safety and performance",
        metadata: vec![("category", "language"), ("type", "systems")],
    },
    TestDocument {
        id: 2,
        content: "Python is a high-level programming language known for its simplicity",
        metadata: vec![("category", "language"), ("type", "scripting")],
    },
    // ...
];

/// CJK 文档
pub const CJK_DOCS: &[TestDocument] = &[
    TestDocument {
        id: 100,
        content: "Rust是一种系统编程语言，专注于安全和性能",
        metadata: vec![("lang", "zh"), ("category", "技术")],
    },
    TestDocument {
        id: 101,
        content: "Pythonは高水準プログラミング言語です",
        metadata: vec![("lang", "ja"), ("category", "技術")],
    },
    // ...
];
```

### 查询数据 (fixtures/queries.rs)

```rust
//! 测试查询数据

pub struct TestQuery {
    pub query: &'static str,
    pub expected_doc_ids: Vec<u64>,
    pub description: &'static str,
}

/// 基本搜索查询
pub const BASIC_QUERIES: &[TestQuery] = &[
    TestQuery {
        query: "Rust",
        expected_doc_ids: vec![1],
        description: "单关键词搜索",
    },
    TestQuery {
        query: "programming language",
        expected_doc_ids: vec![1, 2],
        description: "多关键词搜索",
    },
    // ...
];

/// 边界情况查询
pub const EDGE_CASE_QUERIES: &[TestQuery] = &[
    TestQuery {
        query: "",
        expected_doc_ids: vec![],
        description: "空查询",
    },
    TestQuery {
        query: "xyz123nonexistent",
        expected_doc_ids: vec![],
        description: "无结果查询",
    },
    // ...
];
```

## 测试配置

### Cargo.toml 配置

```toml
[dev-dependencies]
tempfile = "3.10"
tokio-test = "0.4"
serial_test = "3.0"

[[test]]
name = "integration_tests"
path = "tests/integration/mod.rs"
```

### 测试辅助模块

```rust
//! 测试辅助工具

use inversearch_service::{Index, IndexOptions};
use tempfile::TempDir;

/// 创建临时测试索引
pub fn create_test_index() -> (TempDir, Index) {
    let temp_dir = TempDir::new().unwrap();
    let options = IndexOptions::default();
    let index = Index::new(options).unwrap();
    (temp_dir, index)
}

/// 添加测试文档集合
pub fn seed_test_data(index: &mut Index) {
    // 添加标准测试文档
}

/// 断言搜索结果包含指定文档
pub fn assert_contains(results: &[u64], doc_id: u64) {
    assert!(
        results.contains(&doc_id),
        "Expected results to contain document {}",
        doc_id
    );
}
```

## 测试执行策略

### 1. 单元测试

```bash
# 运行所有单元测试
cargo test --lib

# 运行特定模块测试
cargo test --lib index::
```

### 2. 集成测试

```bash
# 运行所有集成测试
cargo test --test integration_tests

# 运行特定集成测试
cargo test --test integration_tests search_test
```

### 3. 功能测试

```bash
# 运行带特定功能的测试
cargo test --features "service cache"
```

## 持续集成配置

```yaml
# .github/workflows/test.yml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          
      - name: Run unit tests
        run: cargo test --lib
        
      - name: Run integration tests
        run: cargo test --test integration_tests
        
      - name: Run feature tests
        run: cargo test --all-features
```

## 测试覆盖率目标

| 模块 | 目标覆盖率 |
|------|-----------|
| index | 90% |
| search | 90% |
| charset | 85% |
| highlight | 85% |
| service | 80% |
| intersect | 85% |

## 注意事项

1. **并发测试**：使用 `serial_test` 确保测试串行执行，避免索引状态冲突
2. **临时资源**：使用 `tempfile` 创建临时目录，确保测试后清理
3. **超时设置**：为异步测试设置合理的超时时间
4. **错误处理**：测试应验证错误情况，而不仅是成功路径
5. **性能基准**：记录关键操作的性能基准，防止性能回归
