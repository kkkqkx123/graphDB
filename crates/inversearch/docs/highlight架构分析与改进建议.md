# Highlight 模块架构分析与改进建议

## 概述

本文档分析了 inversearch 中 highlight 模块的当前架构,识别存在的问题,并提出引入专门数据结构传递匹配结果的改进方案,以支持无前端场景的灵活使用。

---

## 一、当前架构分析

### 1.1 模块结构

```
highlight/
├── mod.rs        # 模块入口,定义公共导出
├── types.rs      # 类型定义 (HighlightConfig, HighlightOptions 等)
├── core.rs       # 核心高亮逻辑
├── matcher.rs    # 匹配算法
├── processor.rs  # 处理器 (封装完整流程)
├── boundary.rs   # 边界处理 (文本截断/省略号)
└── tests.rs      # 单元测试
```

### 1.2 外部模块依赖关系

#### 依赖 `crate::encoder::Encoder`
- **使用位置**: `types.rs`, `core.rs`, `matcher.rs`, `processor.rs`
- **作用**: 对查询词和文档内容进行编码转换,用于模糊匹配和语音匹配
- **集成方式**: 
  ```rust
  // matcher.rs 中对文档词项编码
  let doc_enc = encode_and_join(doc_term_trimmed, encoder)?;
  
  // 将编码后的查询词与文档词项进行匹配
  find_best_match(doc_term, &doc_enc, &query_enc, ...)
  ```

#### 依赖 `crate::error::Result` 和错误类型
- **使用位置**: 所有 highlight 子模块
- **作用**: 统一错误处理
- **集成方式**:
  ```rust
  // error.rs 中定义了专门的 Highlight 错误变体
  #[error("Highlight error: {0}")]
  Highlight(String),
  
  // highlight 模块中使用 Result<T> 别名
  use crate::error::Result;
  ```

#### 依赖 `crate::common::parse_simple`
- **使用位置**: `core.rs`, `processor.rs`
- **作用**: 从 JSON 文档中解析指定字段的内容
- **集成方式**:
  ```rust
  let content = crate::common::parse_simple(document, field_path)?;
  ```

#### 依赖 `regex` crate
- **使用位置**: `core.rs`, `processor.rs`
- **作用**: 实现合并逻辑 (merge),将相邻的高亮标签合并
- **集成方式**: 直接使用 `regex::Regex::new()` 处理合并模式

### 1.3 内部模块依赖关系

```
mod.rs
  ├── 导出 types.rs 的所有公开类型 (pub use types::*)
  ├── 导出 core.rs 的核心函数
  ├── 导出 processor.rs 的处理器
  └── 导出 boundary.rs 的边界处理功能

processor.rs (最上层封装)
  ├── 使用 types.rs::HighlightConfig, HighlightOptions
  ├── 使用 boundary.rs::apply_advanced_boundary, BoundaryTerm
  ├── 使用 matcher.rs::find_best_match, encode_and_join
  └── 调用 crate::common::parse_simple

core.rs (独立高亮函数)
  ├── 使用 types.rs::HighlightConfig, HighlightedTerm
  ├── 使用 matcher.rs::find_best_match, encode_and_join
  └── 调用 crate::common::parse_simple

matcher.rs (匹配算法)
  └── 使用 crate::encoder::Encoder

boundary.rs (边界处理)
  └── 使用 types.rs::HighlightConfig, BoundaryTerm
```

### 1.4 对外导出 (lib.rs 集成)

在 `lib.rs` 中,highlight 模块被集成并重新导出:

```rust
pub mod highlight;

// 重新导出常用函数到 crate 根级别
pub use highlight::{
    highlight_fields, highlight_document, highlight_single_document,
    HighlightProcessor
};
```

这使得调用方可以:
```rust
// 方式1: 通过 highlight 模块访问
use inversearch::highlight::HighlightProcessor;

// 方式2: 直接从 crate 根访问
use inversearch::HighlightProcessor;
```

---

## 二、当前架构存在的问题

### 2.1 高亮结果与搜索结果强耦合

当前 `EnrichedSearchResult` 结构:
```rust
pub struct EnrichedSearchResult {
    pub id: DocId,
    pub doc: Option<serde_json::Value>,
    pub highlight: Option<String>,  // ❌ 问题: 只有字符串,无结构化信息
}
```

**局限性**:
- `highlight` 字段只是 `Option<String>`,丢失了匹配位置、匹配度、多字段高亮等结构化信息
- 无前端的场景(API/数据分析)无法获取原始匹配数据
- 无法支持多字段独立高亮展示
- 前端无法自定义高亮样式(只能使用服务端生成的 HTML 标签)

### 2.2 类型定义重复

存在两套几乎相同的类型定义:
- `highlight/types.rs` 中的 `HighlightOptions`, `EnrichedSearchResult` 等
- `type/mod.rs` 中的同名类型

这导致维护成本增加,且容易不一致。

### 2.3 gRPC 接口不支持高亮

`inversearch.proto` 的 `SearchResponse` 只返回 ID 列表:
```protobuf
message SearchResponse {
  repeated uint64 results = 1;
  uint32 total = 2;
  string error = 3;
}
```

没有高亮字段,无法通过 gRPC 传递高亮信息。

### 2.4 高亮计算与搜索流程耦合

当前 `HighlightProcessor` 直接修改 `FieldSearchResults`,导致:
- 搜索和高亮无法独立执行
- 无法在中间层缓存搜索结果而不缓存高亮结果
- 高亮计算开销无法按需避免

---

## 三、改进方案设计

### 3.1 核心设计思路

```
搜索流程:
  search() → SearchResults (纯ID+分数)
                  ↓
           上层模块自行决定:
           ├── 场景1: 直接返回 (无前端的API/数据分析)
           ├── 场景2: 调用 highlight() → HighlightResult (需要高亮)
           └── 场景3: 调用 enrich() → EnrichedSearchResult (需要完整文档)
```

### 3.2 建议的新数据结构

#### 单个匹配项的结构化信息

```rust
/// 单个匹配项的结构化信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighlightMatch {
    /// 匹配的原始文本
    pub text: String,
    /// 匹配的开始位置(字符级别)
    pub start_pos: usize,
    /// 匹配的结束位置
    pub end_pos: usize,
    /// 匹配的查询词
    pub matched_query: String,
    /// 匹配得分(可选,用于多匹配排序)
    pub score: Option<f64>,
}
```

#### 单个字段的高亮结果

```rust
/// 单个字段的高亮结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldHighlight {
    /// 字段名
    pub field: String,
    /// 所有匹配项
    pub matches: Vec<HighlightMatch>,
    /// 高亮后的完整文本(可选,方便前端直接使用)
    pub highlighted_text: Option<String>,
    /// 匹配的查询词列表
    pub matched_queries: Vec<String>,
}
```

#### 单个文档的高亮结果

```rust
/// 单个文档的高亮结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentHighlight {
    /// 文档ID
    pub id: DocId,
    /// 各字段的高亮结果
    pub fields: Vec<FieldHighlight>,
    /// 总匹配数
    pub total_matches: usize,
}
```

#### 搜索结果(不含高亮)

```rust
/// 搜索结果(不含高亮)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: DocId,
    pub score: Option<f64>,
    pub doc: Option<serde_json::Value>,
}
```

#### 完整的搜索结果(含高亮)

```rust
/// 完整的搜索结果(含高亮)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultWithHighlight {
    pub results: Vec<SearchResult>,
    pub highlights: Vec<DocumentHighlight>,
    pub total: usize,
    pub query: String,
}
```

---

## 四、架构改进方案对比

### 方案A: 并行架构(推荐)

```rust
// 1. 基础搜索 - 返回纯结果
pub fn search(index: &Index, options: &SearchOptions) -> Result<Vec<SearchResult>>;

// 2. 高亮处理 - 独立模块,按需调用
pub fn highlight_results(
    query: &str,
    results: &[SearchResult],
    encoders: &HashMap<String, Encoder>,
    options: &HighlightOptions,
) -> Result<Vec<DocumentHighlight>>;

// 3. 上层模块自行组合
// 场景1: 无前端的API
let results = search(&index, &options)?;
return results; // 直接返回

// 场景2: 需要高亮
let results = search(&index, &options)?;
let highlights = highlight_results(&query, &results, &encoders, &hl_options)?;
return SearchResultWithHighlight { results, highlights, ... };
```

**优点**:
- ✅ 搜索和高亮解耦,职责单一
- ✅ 无前端的场景不需要支付高亮计算开销
- ✅ 上层模块灵活控制是否启用高亮
- ✅ 结构化数据支持多种展示方式(前端渲染/数据分析/API返回)

**缺点**:
- ❌ 调用方需要多一步操作(但可通过封装简化)

### 方案B: 可选参数架构

```rust
pub struct SearchOptions {
    // ... 现有字段
    pub highlight: Option<HighlightOptions>, // 新增: 可选高亮配置
}

pub fn search(index: &Index, options: &SearchOptions) -> Result<SearchResponse> {
    // 1. 执行搜索
    let results = perform_search(index, options)?;
    
    // 2. 根据配置决定是否高亮
    let response = if let Some(hl_opts) = &options.highlight {
        let highlights = highlight_results(&results, hl_opts)?;
        SearchResponse::WithHighlight(SearchResultWithHighlight { highlights, ... })
    } else {
        SearchResponse::Simple(results)
    };
    
    Ok(response)
}

pub enum SearchResponse {
    Simple(Vec<SearchResult>),
    WithHighlight(SearchResultWithHighlight),
}
```

**优点**:
- ✅ API 统一,调用方只需设置一个参数
- ✅ 内部自动优化,无需手动组合

**缺点**:
- ❌ 搜索和高亮仍有耦合
- ❌ 枚举类型增加使用复杂度
- ❌ 难以独立缓存搜索结果和高亮结果

---

## 五、实施建议

### 5.1 推荐方案: 方案A(并行架构)

### 5.2 实施步骤

#### 步骤1: 新增结构化类型

在 `highlight/types.rs` 中新增:
- `HighlightMatch`
- `FieldHighlight`
- `DocumentHighlight`
- `SearchResult` (不含高亮的基础结果)
- `SearchResultWithHighlight` (完整结果)

#### 步骤2: 重构核心高亮函数

修改 `highlight_single_document` 返回结构化结果:
```rust
pub fn highlight_single_document(
    query: &str,
    content: &str,
    encoder: &Encoder,
    config: &HighlightConfig,
) -> Result<DocumentHighlight> {
    // 返回包含匹配位置和高亮文本的结构
}
```

**注意**: 保持向后兼容,可同时保留返回 `String` 的版本。

#### 步骤3: 新增批量处理函数

```rust
pub fn highlight_results(
    query: &str,
    results: &[SearchResult],
    encoders: &HashMap<String, Encoder>,
    options: &HighlightOptions,
) -> Result<Vec<DocumentHighlight>>;
```

#### 步骤4: 修改 SearchResponse

在 `search/mod.rs` 中支持可选高亮:
```rust
pub enum SearchResponse {
    Simple(Vec<SearchResult>),
    WithHighlight(SearchResultWithHighlight),
}
```

#### 步骤5: 更新 gRPC 接口

修改 `inversearch.proto`:
```protobuf
message SearchRequest {
  string query = 1;
  uint32 limit = 2;
  uint32 offset = 3;
  bool context = 4;
  bool suggest = 5;
  bool resolve = 6;
  bool enrich = 7;
  bool cache = 8;
  bool highlight = 9;  // 新增: 是否返回高亮信息
  HighlightOptions highlight_options = 10;  // 新增: 高亮配置
}

message SearchResponse {
  repeated uint64 results = 1;
  uint32 total = 2;
  string error = 3;
  repeated DocumentHighlight highlights = 4;  // 新增: 高亮结果
}

message DocumentHighlight {
  uint64 id = 1;
  repeated FieldHighlight fields = 2;
  uint32 total_matches = 3;
}

message FieldHighlight {
  string field = 1;
  repeated HighlightMatch matches = 2;
  string highlighted_text = 3;
  repeated string matched_queries = 4;
}

message HighlightMatch {
  string text = 1;
  uint32 start_pos = 2;
  uint32 end_pos = 3;
  string matched_query = 4;
  double score = 5;
}
```

#### 步骤6: 添加单元测试

- 测试 `HighlightMatch` 位置计算准确性
- 测试多字段高亮结果
- 测试边界条件下的结构化输出
- 测试向后兼容性

### 5.3 向后兼容策略

1. 保留现有的 `highlight_single_document` 返回 `String` 的版本
2. 新增 `highlight_single_document_structured` 返回结构化结果
3. 在下一个大版本中废弃旧版本

---

## 六、使用场景示例

### 场景1: 无前端的 API 服务

```rust
// 只需要文档ID和内容,不需要高亮
let results = search(&index, &options)?;
Ok(Json(results))
```

### 场景2: 需要高亮的前端服务

```rust
let results = search(&index, &options)?;
let highlights = highlight_results(
    &query,
    &results,
    &encoders,
    &HighlightOptions {
        template: "<mark>$1</mark>".to_string(),
        boundary: Some(HighlightBoundaryOptions {
            before: Some(50),
            after: Some(50),
            total: Some(200),
        }),
        ..Default::default()
    }
)?;

Ok(Json(SearchResultWithHighlight {
    results,
    highlights,
    total: results.len(),
    query,
}))
```

### 场景3: 数据分析场景

```rust
// 获取结构化匹配信息用于分析
let highlights = highlight_results(&query, &results, &encoders, &options)?;

for doc_highlight in &highlights {
    println!("文档 {} 匹配了 {} 次", doc_highlight.id, doc_highlight.total_matches);
    for field in &doc_highlight.fields {
        println!("  字段 '{}':", field.field);
        for m in &field.matches {
            println!("    位置 {}-{}: '{}'", m.start_pos, m.end_pos, m.text);
        }
    }
}
```

### 场景4: 前端自定义渲染

```rust
// 前端根据结构化数据自行渲染
let highlights = highlight_results(&query, &results, &encoders, &options)?;

for doc in &highlights {
    for field in &doc.fields {
        // 前端可以使用自己的高亮样式
        for m in &field.matches {
            console.log(`匹配: ${m.text} at ${m.start_pos}-${m.end_pos}`);
        }
    }
}
```

---

## 七、性能优化建议

### 7.1 按需计算

```rust
// 搜索结果缓存时,只缓存基础结果
// 高亮结果按需计算,不缓存或单独缓存
```

### 7.2 批量处理优化

```rust
// highlight_results 可以并行处理多个文档
pub async fn highlight_results_async(...) -> Result<Vec<DocumentHighlight>> {
    let futures: Vec<_> = results
        .iter()
        .map(|r| highlight_single_doc_async(r, ...))
        .collect();
    futures::future::join_all(futures).await
}
```

### 7.3 编码器缓存

```rust
// 对同一查询词的编码结果进行缓存
// 避免在多字段高亮时重复编码
```

---

## 八、总结

### 核心建议

**应该补充专门的数据结构传递匹配结果**,理由如下:

1. **当前 `Option<String>` 设计信息丢失严重**,无法支持:
   - 多字段独立高亮
   - 匹配位置信息
   - 匹配度评分
   - 前端自定义渲染逻辑

2. **并行架构更灵活**:
   ```
   search() → 纯结果 → 上层决定 → 无前端场景直接返回
                                  → 需要高亮时调用 highlight()
   ```

3. **性能优化**: 无前端的API/数据分析场景可以跳过昂贵的高亮计算

4. **gRPC 支持**: 需要在 proto 中增加高亮字段,当前只返回 ID 列表

### 推荐实施路径

1. 新增结构化类型 (`HighlightMatch`, `FieldHighlight`, `DocumentHighlight`)
2. 保持现有 `highlight_single_document` 向后兼容
3. 新增 `highlight_results` 批量处理函数返回结构化结果
4. 修改 `SearchResponse` 支持可选高亮
5. 更新 proto 定义
6. 添加完整的单元测试

---

## 附录: 关键文件清单

### 需要修改的文件

- `inversearch/src/highlight/types.rs` - 新增结构化类型
- `inversearch/src/highlight/core.rs` - 重构核心高亮逻辑
- `inversearch/src/highlight/processor.rs` - 新增批量处理函数
- `inversearch/src/highlight/mod.rs` - 导出新类型
- `inversearch/src/type/mod.rs` - 统一类型定义
- `inversearch/proto/inversearch.proto` - 更新 gRPC 接口
- `inversearch/src/lib.rs` - 导出新函数

### 参考文件

- `inversearch/src/error.rs` - 错误处理机制
- `inversearch/src/encoder/mod.rs` - 编码器集成
- `inversearch/src/search/mod.rs` - 搜索流程
- `inversearch/src/resolver/enrich.rs` - 结果丰富化逻辑
