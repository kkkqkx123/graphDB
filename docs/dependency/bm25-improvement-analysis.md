# BM25 功能改进分析

## 概述

本文档分析了当前项目的 BM25 功能实现，基于 Tantivy 库的功能特性，提出了改进建议，包括关键词功能补充和无意义标记符号的去除。

## 当前 BM25 功能分析

### 已实现功能

#### 1. 文本生成模块 (`src/ast_to_nl/bm25/`)

**Bm25Generator** (`generator.rs`):
- 为不同类型的代码实体生成 BM25 优化的文本
- 支持的实体类型：Function, Method, Constructor, Destructor, Class, Struct, Enum, Interface, Trait, Variable, Constant, Field, Parameter
- 生成内容包括：
  - 原始名称（精确匹配）
  - 标准化名称（模糊匹配）
  - 参数和类型
  - 文件路径和模块信息
  - 关键词

**KeywordExtractor** (`keyword_extractor.rs`):
- 提取实体名称（原始和标准化）
- 提取参数名称和类型
- 提取返回类型
- 从文档字符串中提取关键词
- 过滤常见停用词（英文）
- 去重并保持顺序

**模板系统** (`templates/`):
- 为不同实体类型生成格式化的描述文本
- 包含丰富的元数据（文件路径、模块名等）
- 支持关键词附加

#### 2. 文本处理工具 (`src/ast_to_nl/common/`)

**SymbolCleaner** (`symbol_cleaner.rs`):
- 转换操作符为自然语言（如 `->` → `returns`, `&&` → `and`）
- 移除括号、大括号、方括号
- 移除标点符号（分号、逗号、冒号）
- 标准化空白字符
- 支持显示模式（`clean_for_display`）

**NameNormalizer** (`normalizer.rs`):
- 转换命名约定为自然语言：
  - `snake_case` → `snake case`
  - `camelCase` → `camel case`
  - `PascalCase` → `pascal case`
  - 处理缩写词（如 `XMLParser` → `xml parser`）

**DocstringCleaner** (`docstring_cleaner.rs`):
- 移除注释标记（`///`, `/**`, `//`, `#` 等）
- 提取摘要（第一段或第一行）
- 移除特殊标签（`@param`, `# Arguments` 等）
- 标准化空白字符

### 当前 BM25 文本格式示例

```
Function 'calculate_total' (normalized: 'calculate total') that does 'Calculates the total price' with parameters 'price: f64, quantity: i32' that returns 'f64', defined in file 'calculator.rs' within module 'math'. Keywords: calculate, total, price.
```

## Tantivy 库功能对照

### Tantivy 提供的功能

1. **BM25 评分算法**: 内置实现，支持自定义统计
2. **分词器**:
   - `default`: 基本分词（标点分割、小写、长度限制）
   - `en_stem`: 包含词干提取
   - 支持自定义分词器和过滤器链
3. **多语言支持**: 支持多种语言的词干提取
4. **查询类型**: TermQuery, BooleanQuery, PhraseQuery, FuzzyQuery 等
5. **索引选项**: Basic, WithFreqsAndPositions

### 与当前项目的互补性

| 功能 | 当前项目 | Tantivy | 互补性 |
|------|---------|---------|--------|
| 文本生成 | ✅ 有 | ❌ 无 | 完全互补 |
| 关键词提取 | ✅ 有 | ❌ 无 | 完全互补 |
| 符号清理 | ✅ 有 | ✅ 有 | 部分重叠 |
| 名称标准化 | ✅ 有 | ❌ 无 | 完全互补 |
| 词干提取 | ❌ 无 | ✅ 有 | 完全互补 |
| BM25 评分 | ❌ 无 | ✅ 有 | 完全互补 |
| 停用词过滤 | ✅ 有（仅英文） | ✅ 有 | 部分重叠 |

## 改进建议

### 1. 补充关键词功能

#### 1.1 停用词处理

**当前问题**:
- `KeywordExtractor` 实现了不完善的英文停用词过滤
- 停用词列表不完整，且只支持英文
- 重复了搜索引擎（如 Tantivy）的已有功能

**建议**:
- **移除当前的停用词实现**，避免重复和不完善的实现
- **复用搜索引擎的停用词功能**（如 Tantivy 的内置停用词过滤器）
- 在文本生成阶段保留所有有意义的词，让搜索引擎在索引时处理停用词

**原因**:
1. Tantivy 等搜索引擎已经内置了完善的多语言停用词支持
2. 停用词过滤应该在索引和查询阶段进行，而不是文本生成阶段
3. 避免维护不完善的停用词列表
4. 搜索引擎的停用词处理更加高效和准确

**已实现的改进**:
```rust
// 已修改：移除停用词过滤，只保留基本的词提取
fn extract_docstring_keywords(&self, doc: &str) -> Vec<String> {
    let mut keywords = Vec::new();

    // Remove comment markers
    let cleaned = doc
        .replace("///", "")
        .replace("/**", "")
        .replace("*/", "")
        .replace("/*", "")
        .replace("//", "")
        .replace('#', "");

    // Extract significant words (length >= 3)
    // Stop words will be filtered by the search engine during indexing
    for word in cleaned.split_whitespace() {
        let word = word.trim_matches(|c: char| !c.is_alphanumeric());
        if word.len() >= 3 {
            keywords.push(word.to_string());
        }
    }

    keywords
}
```

**与 Tantivy 集成时的停用词处理**:
```rust
use tantivy::tokenizer::*;

// 使用 Tantivy 内置的停用词过滤器
let tokenizer = TextAnalyzer::builder(SimpleTokenizer::default())
    .filter(RemoveLongFilter::limit(40))
    .filter(LowerCaser)
    .filter(StopWordFilter::default())  // 内置停用词过滤
    .filter(Stemmer::new(Language::English))
    .build();
```

#### 1.2 语义关键词提取

**当前问题**:
- 关键词提取主要基于简单的频率和长度过滤
- 缺乏语义理解

**建议**:
- 利用代码上下文提取语义关键词
- 识别函数/类的核心用途
- 提取领域特定术语

**实现示例**:
```rust
// 添加语义关键词提取
impl KeywordExtractor {
    fn extract_semantic_keywords(&self, entity: &Entity) -> Vec<String> {
        let mut keywords = Vec::new();

        // 识别动词（函数名中的动作）
        if entity.kind.is_function_like() {
            keywords.extend(self.extract_verbs(&entity.name));
        }

        // 识别领域术语（类型名称）
        if entity.kind.is_type_like() {
            keywords.extend(self.extract_domain_terms(&entity.name));
        }

        keywords
    }
}
```

#### 1.3 关键词权重

**当前问题**:
- 所有关键词权重相同
- 缺乏重要性区分

**建议**:
- 为关键词分配权重
- 根据关键词来源、位置等因素确定权重
- 在生成 BM25 文本时突出重要关键词

**实现示例**:
```rust
pub struct WeightedKeyword {
    pub keyword: String,
    pub weight: f32,
}

impl KeywordExtractor {
    pub fn extract_weighted(&self, entity: &Entity) -> Vec<WeightedKeyword> {
        let mut keywords = Vec::new();

        // 函数名权重最高
        keywords.push(WeightedKeyword {
            keyword: entity.name.clone(),
            weight: 1.0,
        });

        // 参数名权重中等
        for (param_name, _) in &entity.parameters {
            keywords.push(WeightedKeyword {
                keyword: param_name.clone(),
                weight: 0.7,
            });
        }

        // 文档字符串关键词权重较低
        if let Some(doc) = &entity.doc_comment {
            for kw in self.extract_docstring_keywords(doc) {
                keywords.push(WeightedKeyword {
                    keyword: kw,
                    weight: 0.5,
                });
            }
        }

        keywords
    }
}
```

### 2. 去除无意义的标记符号

#### 2.1 当前符号清理分析

**SymbolCleaner 当前处理**:
- ✅ 操作符转换（`->`, `&&`, `||`, `==`, 等）
- ✅ 括号移除（`{}`, `[]`, `()`）
- ✅ 标点符号移除（`;`, `,`）
- ✅ 空白标准化

**存在的问题**:
1. **引号残留**: 生成的文本中包含大量单引号（如 `'calculate_total'`），这些引号对 BM25 搜索无意义
2. **括号残留**: 在某些情况下，括号可能仍有残留
3. **特殊字符**: 某些特殊字符可能未被清理
4. **格式化符号**: 如 `(normalized: '...')` 这样的格式化标记对搜索无帮助

#### 2.2 改进建议

##### 2.2.1 移除引号

**问题**: 当前生成的文本中大量使用单引号包裹标识符
```text
Function 'calculate_total' (normalized: 'calculate total') that does '...'
```

**建议**: 移除标识符周围的引号，因为它们对 BM25 搜索无意义
```text
Function calculate_total (normalized: calculate total) that does ...
```

**实现**:
```rust
// 在 symbol_cleaner.rs 中添加
pub fn clean(&self, text: &str) -> String {
    let mut result = text.to_string();

    // 现有的清理逻辑...
    result = self.replace_operators(&result);
    result = self.remove_brackets(&result);
    result = self.remove_punctuation(&result);

    // 新增：移除标识符周围的引号
    result = self.remove_identifier_quotes(&result);

    result = self.normalize_whitespace(&result);
    result
}

/// 移除标识符周围的引号
/// 注意：保留必要的引号（如字符串字面量）
fn remove_identifier_quotes(&self, text: &str) -> String {
    // 移除 'xxx' 格式的引号（标识符）
    text.replace("'", "")
}
```

##### 2.2.2 优化格式化标记

**问题**: `(normalized: '...')` 这样的格式化标记对搜索无帮助

**建议**:
- 选项 1: 完全移除格式化标记
- 选项 2: 简化格式化标记
- 选项 3: 使用更简洁的格式

**选项 2 示例**:
```text
// 改进前
Function 'calculate_total' (normalized: 'calculate total') that does 'Calculates the total price'

// 改进后
Function calculate_total (normalized: calculate total) does Calculates the total price
```

**选项 3 示例**:
```text
// 改进后
Function calculate_total normalized as calculate total does Calculates the total price
```

##### 2.2.3 移除冗余词汇

**问题**: 某些词汇对搜索无帮助，增加噪声

**建议**: 移除以下类型的冗余词汇：
- 冗余的连接词（`that does`, `with parameters`, `that returns`）
- 冗余的介词（`in`, `within`, `of`）

**改进示例**:
```text
// 改进前
Function calculate_total normalized as calculate total that does Calculates the total price with parameters price f64 quantity i32 that returns f64 defined in file calculator.rs within module math

// 改进后（移除冗余词汇）
Function calculate_total normalized calculate total Calculates total price parameters price f64 quantity i32 returns f64 file calculator.rs module math
```

##### 2.2.4 改进后的 SymbolCleaner 实现

```rust
impl SymbolCleaner {
    pub fn clean(&self, text: &str) -> String {
        if text.is_empty() {
            return String::new();
        }

        let mut result = text.to_string();

        // Step 1: 替换操作符
        result = self.replace_operators(&result);

        // Step 2: 移除括号
        result = self.remove_brackets(&result);

        // Step 3: 移除标点符号（包括引号）
        result = self.remove_punctuation_and_quotes(&result);

        // Step 4: 移除冗余词汇
        result = self.remove_redundant_words(&result);

        // Step 5: 标准化空白
        result = self.normalize_whitespace(&result);

        result
    }

    /// 移除标点符号和引号
    fn remove_punctuation_and_quotes(&self, text: &str) -> String {
        text.chars()
            .filter(|&c| !matches!(c, ';' | ',' | ':' | '\'' | '"' | '`'))
            .collect()
    }

    /// 移除冗余词汇
    fn remove_redundant_words(&self, text: &str) -> String {
        let redundant_words = [
            "that does",
            "that does the",
            "with parameters",
            "with",
            "that returns",
            "returns",
            "defined in file",
            "defined in",
            "within module",
            "within",
            "in file",
            "in",
            "of class",
            "of type",
            "of",
        ];

        let mut result = text.to_string();
        for word in redundant_words {
            result = result.replace(word, " ");
        }
        result
    }
}
```

### 3. 模板格式优化

#### 3.1 简化模板结构

**当前模板问题**:
- 句子结构过于复杂
- 包含大量连接词
- 不利于 BM25 搜索

**建议**: 使用更简洁的模板格式

**改进示例**:
```rust
// 改进前的模板
pub fn generate_function_description(data: &Bm25FunctionTemplateData) -> String {
    let mut text = format!(
        "Function '{}' (normalized: '{}')",
        data.original_name, data.normalized_name
    );

    if let Some(desc) = data.description {
        if !desc.is_empty() {
            text.push_str(&format!(" that does '{}'", desc));
        }
    }

    if !data.parameters.is_empty() {
        let params: Vec<String> = data
            .parameters
            .iter()
            .map(|(name, ty)| match ty {
                Some(t) => format!("{}: {}", name, t),
                None => name.to_string(),
            })
            .collect();
        text.push_str(&format!(" with parameters '{}'", params.join(", ")));
    }

    // ...
}

// 改进后的模板
pub fn generate_function_description(data: &Bm25FunctionTemplateData) -> String {
    let mut parts = Vec::new();

    // 实体类型
    parts.push(format!("Function {}", data.original_name));

    // 标准化名称
    if data.normalized_name != data.original_name {
        parts.push(format!("normalized {}", data.normalized_name));
    }

    // 描述
    if let Some(desc) = data.description {
        if !desc.is_empty() {
            parts.push(desc.to_string());
        }
    }

    // 参数
    if !data.parameters.is_empty() {
        let params: Vec<String> = data
            .parameters
            .iter()
            .map(|(name, ty)| match ty {
                Some(t) => format!("{} {}", name, t),
                None => name.to_string(),
            })
            .collect();
        parts.push(format!("parameters {}", params.join(" ")));
    }

    // 返回类型
    if let Some(ret) = data.return_type {
        if !ret.is_empty() {
            parts.push(format!("returns {}", ret));
        }
    }

    // 文件和模块
    parts.push(format!("file {}", data.file_path));
    parts.push(format!("module {}", data.module_name));

    // 关键词
    if !data.keywords.is_empty() {
        parts.push(format!("keywords {}", data.keywords.join(" ")));
    }

    parts.join(" ")
}
```

#### 3.2 改进后的输出示例

```text
// 改进前
Function 'calculate_total' (normalized: 'calculate total') that does 'Calculates the total price' with parameters 'price: f64, quantity: i32' that returns 'f64', defined in file 'calculator.rs' within module 'math'. Keywords: calculate, total, price.

// 改进后
Function calculate_total normalized calculate total Calculates total price parameters price f64 quantity i32 returns f64 file calculator.rs module math keywords calculate total price
```

### 4. 与 Tantivy 集成建议

#### 4.1 Schema 设计

```rust
use tantivy::schema::*;

// 为代码搜索优化的 Schema
let mut schema_builder = Schema::builder();

// 主文本字段（使用自定义分词器）
let text_options = TextOptions::default()
    .set_indexing_options(
        TextFieldIndexing::default()
            .set_tokenizer("code_tokenizer")
            .set_index_option(IndexRecordOption::WithFreqsAndPositions)
    )
    .set_stored();

// 原始名称字段（精确匹配）
let raw_name_options = TextOptions::default()
    .set_indexing_options(
        TextFieldIndexing::default()
            .set_tokenizer("raw")
            .set_index_option(IndexRecordOption::Basic)
    )
    .set_stored();

// 关键词字段（多值）
let keywords_options = TextOptions::default()
    .set_indexing_options(
        TextFieldIndexing::default()
            .set_tokenizer("default")
            .set_index_option(IndexRecordOption::Basic)
    )
    .set_stored();

schema_builder.add_text_field("text", text_options);
schema_builder.add_text_field("raw_name", raw_name_options);
schema_builder.add_text_field("keywords", keywords_options);
schema_builder.add_text_field("file_path", STRING | STORED);
schema_builder.add_text_field("module_name", STRING | STORED);
schema_builder.add_text_field("entity_type", STRING | STORED);

let schema = schema_builder.build();
```

#### 4.2 自定义分词器

```rust
use tantivy::tokenizer::*;

// 为代码搜索优化的分词器
let code_tokenizer = TextAnalyzer::builder(SimpleTokenizer::default())
    .filter(RemoveLongFilter::limit(40))
    .filter(LowerCaser)
    // 可选：添加代码特定的过滤器
    .filter(AlphaNumOnlyFilter)
    .build();

index
    .tokenizers()
    .register("code_tokenizer", code_tokenizer);
```

#### 4.3 查询策略

```rust
// 组合查询策略
let sub_queries = vec![
    // 1. 精确匹配原始名称
    Box::new(TermQuery::new(
        Term::from_field_text(raw_name_field, query_str),
        IndexRecordOption::Basic,
    )),

    // 2. BM25 主文本搜索
    Box::new(query_parser.parse_query(query_str)?),

    // 3. 关键词匹配
    Box::new(BooleanQuery::from(
        keywords
            .iter()
            .map(|kw| {
                (
                    Occur::Should,
                    Box::new(TermQuery::new(
                        Term::from_field_text(keywords_field, kw),
                        IndexRecordOption::Basic,
                    )) as Box<dyn Query>,
                )
            })
            .collect::<Vec<_>>(),
    )),
];

let final_query = BooleanQuery::from(
    sub_queries
        .into_iter()
        .map(|q| (Occur::Should, q))
        .collect(),
);
```

## 实施优先级

### 高优先级
1. ✅ 移除引号和冗余标点符号
2. ✅ 简化模板格式，移除冗余词汇
3. ✅ 优化 SymbolCleaner 的清理逻辑

### 中优先级
4. 实现多语言关键词支持
5. 添加关键词权重功能
6. 优化模板结构

### 低优先级
7. 与 Tantivy 集成（如果项目需要）
8. 实现语义关键词提取
9. 添加代码特定的分词器

## 总结

当前项目的 BM25 功能已经具备了良好的基础，包括文本生成、关键词提取和符号清理等功能。通过以下改进可以进一步提升 BM25 搜索效果：

1. **去除无意义标记符号**: 移除引号、冗余词汇和格式化标记，使文本更简洁、更利于搜索
2. **补充关键词功能**: 添加多语言支持、关键词权重和语义提取
3. **优化模板格式**: 使用更简洁的模板结构，提高搜索效率

这些改进将使生成的 BM25 文本更加优化，与 Tantivy 等搜索引擎的集成将更加顺畅，最终提升代码搜索的准确性和效率。
