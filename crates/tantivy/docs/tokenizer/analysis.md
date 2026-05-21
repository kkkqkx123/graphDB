# Tantivy 分词系统分析

## 整体架构

Tantivy 的分词系统采用 **Pipeline/Decorator 模式**，由四层核心 trait 在 `tokenizer-api` 中定义：

### 核心 Trait

**`Token`** — 最小分词单元：
- `offset_from/to`: 字节偏移（保留原文映射能力）
- `position`: 词在序列中的位置
- `text`: 实际词内容
- `position_length`: 原始词长（默认 1）
- 默认 `position = usize::MAX` 作为哨兵值

**`Tokenizer`** — 从原始文本生产 `TokenStream` 的工厂：
```rust
pub trait Tokenizer: 'static + Clone + Send + Sync {
    type TokenStream<'a>: TokenStream;
    fn token_stream<'a>(&'a mut self, text: &'a str) -> Self::TokenStream<'a>;
}
```

**`TokenStream`** — 可消费的 token 流：
- `advance()` -> `bool`：前进到下一个 token
- `token()` / `token_mut()`：访问当前 token
- `process(sink)`：消费全部 token 到回调

**`TokenFilter`** — 装饰器，包装一个 `Tokenizer` 产生新 `Tokenizer`：
```rust
pub trait TokenFilter: 'static + Send + Sync {
    type Tokenizer<T: Tokenizer>: Tokenizer;
    fn transform<T: Tokenizer>(self, tokenizer: T) -> Self::Tokenizer<T>;
}
```

### Pipeline 装配

`TextAnalyzerBuilder` 通过类型系统在编译期嵌套过滤器：

```rust
TextAnalyzer::builder(SimpleTokenizer::default())
    .filter(RemoveLongFilter::limit(40))
    .filter(LowerCaser)
    .filter(Stemmer::new(Language::English))
    .build();
```

- 每个 `.filter()` 产生 `FilteredTokenizer<F, T>` 泛型嵌套
- 全编译期展开，零成本抽象
- `.dynamic()` 或 `.filter_dynamic()` 兜底，使用 `Box<dyn BoxableTokenizer>` 实现运行时多态

## 分词器组件

### 基础 Tokenizer（6 种）

| Tokenizer | 策略 | 特点 |
|---|---|---|
| `SimpleTokenizer` | 按空白 + 标点拆分，只保留字母数字 | UTF-8 安全，使用 `char_indices()` |
| `WhitespaceTokenizer` | 仅按 ASCII 空白拆分 | 标点保留在 token 内 |
| `RawTokenizer` | 整个输入作为一个 token | position=0, position_length=1 |
| `RegexTokenizer` | 用户自定义正则匹配 | 支持相对锚点 |
| `NgramTokenizer` | n-gram 生成 | 可配 min_gram/max_gram/prefix_only |
| `FacetTokenizer` | 层级分面路径 | 产出所有祖先路径（如 `/a/b/c` -> `/a`, `/a/b`, `/a/b/c`） |
| `EmptyTokenizer` | 内部使用，零 token | 默认 TextAnalyzer |

### Token Filter（8 个）

| Filter | 作用 | 备注 |
|---|---|---|
| `LowerCaser` | 转小写 | ASCII 快路径 + Unicode 慢路径 |
| `RemoveLongFilter` | 移除超长 token | 可配字节数上限 |
| `AsciiFoldingFilter` | Unicode -> ASCII | ~1500 行字符映射，覆盖拉丁/希腊/西里尔等 |
| `AlphaNumOnlyFilter` | 移除含非字母数字的 token | 严格过滤 |
| `Stemmer` | 词干提取 | feature-gated(`rust-stemmers`)，18 种语言 |
| `StopWordFilter` | 停用词过滤 | feature-gated，12 种欧洲语言内置词典 |
| `SplitCompoundWords` | 复合词拆分 | 基于 Aho-Corasick 自动机 |
| `PreTokenizedString` | 外部预分词注入 | JSON 序列化/反序列化 |

### 默认注册 Tokenizer

- `default`: `SimpleTokenizer -> RemoveLongFilter(40) -> LowerCaser`
- `en_stem`: `SimpleTokenizer -> RemoveLongFilter(40) -> LowerCaser -> Stemmer(English)`
- `raw`: `RawTokenizer`
- `whitespace`: `WhitespaceTokenizer`

## 与 BM25 Scoring 的集成

分词器通过索引层面的统计数据与 BM25 评分间接关联：

```
Raw Text
  -> Tokenizer (生产 TokenStream)
  -> Filter Chain (变换/过滤)
  -> Indexing (fieldnorm + term frequencies + positions)
  -> Inverted Index + Fieldnorm Store
     |
     v  Bm25StatisticsProvider
     |   total_num_tokens: 所有 fieldnorm 之和
     |   total_num_docs: 文档总数 N
     |   doc_freq: 包含某 term 的文档数 n
     v
Bm25Weight
  IDF = log(1 + (N - n + 0.5) / (n + 0.5))
  TF_factor = term_freq / (term_freq + k1 * (1 - b + b * fieldnorm / avg_fieldnorm))
  Score = weight * TF_factor
```

- 默认参数: k1 = 1.8, b = 0.4
- 256 槽 fieldnorm ID 缓存加速 TF 计算

## 设计评价

### 优点

1. **Pipeline 模式** — 关注点分离清晰，每个 filter 职责单一
2. **零成本抽象** — `FilteredTokenizer<F, T>` 编译期泛型嵌套，无动态分发
3. **双模式支持** — 编译期类型安全 + 运行时 `dynamic()` 兜底
4. **并发正确** — `Tokenizer` 是 `Send + Sync` 工厂，可共享；`TokenStream` 无需同步
5. **扩展性好** — 实现 `Tokenizer` trait + `TokenizerManager::register()` 即可添加自定义分词器

### 存在的问题

1. **`position_length` 形同虚设** — 全部写死为 1（除 RawTokenizer 外），显然是短语查询预留字段但从未被利用

2. **`NgramTokenizer` 破坏位置信息** — 所有 ngram position = 0，无法用于短语查询，可能误导依赖位置的 filter

3. **`Token.reset()` 使用 `usize::MAX` 作为哨兵值** — 不如 `Option<usize>` 清晰，MAX 作为有效值在边界计算中可能出错

4. **`BoxableTokenizer::box_clone()`** — 为绕过 Rust 对象安全限制手动实现克隆分发，易错且增加维护负担

5. **`MAX_TOKEN_LEN` 静默丢弃** — 超长 token（> 65530 字节）在索引层被丢弃而非在 tokenizer 层警告

6. **`LowerCaser` Unicode 缺陷** — 代码自认 "fails to handle the special sigma case"（希腊字母 sigma），土耳其语 I/İ 等也未被处理

7. **输入验证缺失** — `PreTokenizedString` 不作校验，恶意 position/offset 可能破坏索引

8. **语言覆盖偏欧洲中心** — 内置停用词仅 12 种欧洲语言，无中文/日文/韩文等支持

### 总结

设计整体合理，工程实现成熟。Pipeline + trait 的组合模式是搜索库分词的经典方案，与 Lucene 的 Analyzer 体系异曲同工。上述问题主要集中在边界状态处理和国际化支持上，而非架构级缺陷。对于以英文为主要目标语言的场景，当前设计是恰当的。

---

*本文档对应 tantivy v0.26.0 版本的分析结果。*
