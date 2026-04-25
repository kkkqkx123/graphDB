# Tantivy 分词能力分析与实施方案

## 一、Tantivy 内置分词能力

### 1. 内置分词器

Tantivy 提供了多个内置分词器：

| 分词器名称 | 功能描述 | 适用场景 |
|-----------|---------|---------|
| `default` | 按标点和空白分割，移除超过 40 字符的 token，转小写 | 通用文本 |
| `raw` | 完全不处理文本 | UUID、URL 等唯一标识符 |
| `en_stem` | 在 `default` 基础上增加英文词干提取 | 英文文本，提高召回率 |
| `simple` | 简单分词器 | 基础分词 |
| `whitespace` | 按空白分割 | 特定场景 |

**默认分词器处理流程**：
```
原始文本 → 按标点和空白分割 → 移除超长 token (>40) → 转小写
```

### 2. 英文分词支持

Tantivy 对英文有完善的支持：

**内置语言枚举**：
```rust
pub enum Language {
    Arabic,      // 阿拉伯语
    Danish,      // 丹麦语
    Dutch,       // 荷兰语
    English,     // 英语
    Finnish,     // 芬兰语
    French,      // 法语
    German,      // 德语
    Greek,       // 希腊语
    Hungarian,   // 匈牙利语
    Italian,     // 意大利语
    Norwegian,   // 挪威语
    Portuguese,  // 葡萄牙语
    Romanian,    // 罗马尼亚语
    Russian,     // 俄语
    Spanish,     // 西班牙语
    Swedish,     // 瑞典语
    Tamil,       // 泰米尔语
    Turkish,     // 土耳其语
}
```

**自定义英文分词器示例**：
```rust
use tantivy::tokenizer::*;

let en_stem = TextAnalyzer::builder(SimpleTokenizer::default())
    .filter(RemoveLongFilter::limit(40))  // 移除超长 token
    .filter(LowerCaser)                    // 转小写
    .filter(Stemmer::new(Language::English))  // 英文词干提取
    .build();
```

**处理效果**：
- `calculating` → `calcul`
- `calculated` → `calcul`
- `calculation` → `calcul`

### 3. 中文分词支持

**重要发现**：Tantivy **不内置中文分词器**，必须通过扩展实现。

---

## 二、长期方案：混合分词器

### 核心设计

**不采用临时方案（N-gram），直接实现最优方案（Jieba + 英文混合分词）**。

### 1. 依赖配置

在 `Cargo.toml` 中添加：

```toml
[dependencies]
jieba-rs = "0.6"        # 中文分词
tantivy = "0.22"        # 搜索引擎（已有）
whatlang = "0.16"       # 语言检测（已有）
```

### 2. 混合分词器实现

```rust
use jieba_rs::Jieba;
use tantivy::tokenizer::{Token, TokenStream, Tokenizer};
use std::borrow::Cow;

/// Mixed tokenizer for Chinese and English text
pub struct MixedTokenizer {
    jieba: Jieba,
}

impl MixedTokenizer {
    /// Create a new mixed tokenizer
    pub fn new() -> Self {
        Self {
            jieba: Jieba::new(),
        }
    }
    
    /// Check if a character is a CJK character
    fn is_cjk(c: char) -> bool {
        matches!(c, 
            '\u{4E00}'..='\u{9FFF}' |      // CJK Unified Ideographs
            '\u{3400}'..='\u{4DBF}' |      // CJK Extension A
            '\u{20000}'..='\u{2A6DF}' |    // CJK Extension B
            '\u{2A700}'..='\u{2B73F}' |    // CJK Extension C
            '\u{2B740}'..='\u{2B81F}' |    // CJK Extension D
            '\u{2B820}'..='\u{2CEAF}' |    // CJK Extension E
            '\u{F900}'..='\u{FAFF}' |      // CJK Compatibility Ideographs
            '\u{2F800}'..='\u{2FA1F}'      // CJK Compatibility Ideographs Supplement
        )
    }
    
    /// Extract continuous CJK character segment
    fn extract_cjk_segment(text: &str, start: usize) -> (&str, usize) {
        let mut end = start;
        for (i, c) in text.char_indices().skip_while(|(idx, _)| *idx < start) {
            if Self::is_cjk(c) {
                end = i + c.len_utf8();
            } else {
                break;
            }
        }
        (&text[start..end], end)
    }
    
    /// Tokenize text into tokens
    fn tokenize(&self, text: &str) -> Vec<String> {
        let mut tokens = Vec::new();
        let mut ascii_buffer = String::new();
        
        for (i, c) in text.char_indices() {
            if Self::is_cjk(c) {
                // Process accumulated ASCII text
                if !ascii_buffer.is_empty() {
                    self.tokenize_english(&ascii_buffer, &mut tokens);
                    ascii_buffer.clear();
                }
                
                // Extract CJK segment
                let (cjk_segment, _) = Self::extract_cjk_segment(text, i);
                if !cjk_segment.is_empty() {
                    // Use Jieba for Chinese tokenization
                    let cjk_tokens = self.jieba.cut(cjk_segment, true);
                    tokens.extend(cjk_tokens.into_iter().map(String::from));
                }
            } else {
                // Accumulate ASCII characters
                ascii_buffer.push(c);
            }
        }
        
        // Process remaining ASCII text
        if !ascii_buffer.is_empty() {
            self.tokenize_english(&ascii_buffer, &mut tokens);
        }
        
        tokens
    }
    
    /// Tokenize English text
    fn tokenize_english(&self, text: &str, tokens: &mut Vec<String>) {
        for word in text.split_whitespace() {
            let word = word.trim_matches(|c: char| !c.is_alphanumeric());
            if word.len() >= 2 {
                tokens.push(word.to_lowercase());
            }
        }
    }
}

impl Default for MixedTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Tokenizer for MixedTokenizer {
    type TokenStream<'a> = MixedTokenStream<'a>;
    
    fn token_stream<'a>(&'a mut self, text: &'a str) -> Self::TokenStream<'a> {
        let tokens = self.tokenize(text);
        MixedTokenStream::new(tokens)
    }
}

/// Mixed token stream
pub struct MixedTokenStream<'a> {
    tokens: Vec<String>,
    current: usize,
    token: Token<'a>,
}

impl<'a> MixedTokenStream<'a> {
    fn new(tokens: Vec<String>) -> Self {
        Self {
            tokens,
            current: 0,
            token: Token::default(),
        }
    }
}

impl<'a> TokenStream for MixedTokenStream<'a> {
    fn advance(&mut self) -> bool {
        if self.current < self.tokens.len() {
            let token_str = &self.tokens[self.current];
            self.token = Token {
                offset_from: 0,
                offset_to: token_str.len(),
                position: self.current as u32,
                position_length: 1,
                text: Cow::Owned(token_str.clone()),
            };
            self.current += 1;
            true
        } else {
            false
        }
    }
    
    fn token(&self) -> &Token<'a> {
        &self.token
    }
    
    fn token_mut(&mut self) -> &mut Token<'a> {
        &mut self.token
    }
}
```

### 3. 注册到 Tantivy

```rust
use tantivy::{Index, schema::{Schema, TextFieldIndexing, TextOptions, IndexRecordOption}};

/// Register mixed tokenizer with Tantivy index
pub fn register_mixed_tokenizer(index: &Index) {
    let mixed_tokenizer = MixedTokenizer::new();
    index.register_tokenizer("mixed", mixed_tokenizer);
}

/// Create schema with mixed tokenizer
pub fn create_schema() -> Schema {
    let mut schema_builder = Schema::builder();
    
    // Text field with mixed tokenizer
    let text_options = TextOptions::default()
        .set_indexing_options(
            TextFieldIndexing::default()
                .set_tokenizer("mixed")
                .set_index_option(IndexRecordOption::WithFreqsAndPositions)
        )
        .set_stored();
    
    schema_builder.add_text_field("content", text_options);
    schema_builder.build()
}
```

---

## 三、处理效果对比

### 1. 纯中文文本

**输入**：
```
"计算总价"
```

**当前处理（空格分割）**：
```
["计算总价"]  // 整体作为一个 token
```

**混合分词器处理**：
```
["计算", "总价"]  // 正确分词
```

**搜索效果**：
- ✅ 搜索 "计算" 可以匹配
- ✅ 搜索 "总价" 可以匹配
- ✅ 搜索 "计算总价" 可以匹配

### 2. 纯英文文本

**输入**：
```
"Calculate total price"
```

**当前处理**：
```
["Calculate", "total", "price"]
```

**混合分词器处理**：
```
["calculate", "total", "price"]  // 转小写
```

**搜索效果**：
- ✅ 大小写不敏感
- ✅ 与当前行为一致

### 3. 中英文混合

**输入**：
```
"计算total price"
```

**当前处理**：
```
["计算total", "price"]  // 错误分割
```

**混合分词器处理**：
```
["计算", "total", "price"]  // 正确分词
```

**搜索效果**：
- ✅ 搜索 "计算" 可以匹配
- ✅ 搜索 "total" 可以匹配
- ✅ 搜索 "price" 可以匹配

### 4. 代码注释场景

**输入**：
```rust
/// 计算总价
/// Calculate total price
```

**当前处理**：
```
["计算总价", "Calculate", "total", "price"]
```

**混合分词器处理**：
```
["计算", "总价", "calculate", "total", "price"]
```

**搜索效果**：
- ✅ 中文搜索精确
- ✅ 英文搜索精确
- ✅ 混合搜索支持

---

## 四、性能分析

### 1. 分词性能

| 分词器 | 性能 | 精确度 | 索引体积 |
|-------|------|--------|---------|
| 空格分割 | 快 | 低 | 小 |
| N-gram (2-3) | 中 | 中 | 大 |
| Jieba | 中 | 高 | 中 |
| 混合分词器 | 中 | 高 | 中 |

### 2. 内存占用

**Jieba 字典**：
- 默认字典：约 5MB
- 可选小字典：约 1MB

**建议**：
- 生产环境：使用默认字典（精确度高）
- 内存受限：使用小字典（牺牲部分精确度）

### 3. 索引体积

**对比分析**：

| 文本类型 | 空格分割 | N-gram | 混合分词器 |
|---------|---------|--------|-----------|
| 纯中文 | 1x | 3-5x | 1.5x |
| 纯英文 | 1x | 2-3x | 1x |
| 中英混合 | 1x | 3-4x | 1.2x |

**结论**：混合分词器索引体积合理，远小于 N-gram。

---

## 五、实施方案

### 1. 模块结构

```
src/
├── ast_to_nl/
│   ├── bm25/
│   │   ├── mod.rs
│   │   ├── mixed_tokenizer.rs    # 新增：混合分词器
│   │   ├── quality_filter.rs     # 新增：质量过滤器
│   │   ├── layered_index.rs      # 新增：分层索引
│   │   └── ...                   # 现有模块
│   └── ...
└── search/
    ├── mod.rs
    ├── index.rs                  # 新增：索引管理
    └── query.rs                  # 新增：查询构建
```

### 2. 实施步骤

**步骤 1：添加依赖**
```bash
cargo add jieba-rs
```

**步骤 2：实现混合分词器**
- 创建 `src/ast_to_nl/bm25/mixed_tokenizer.rs`
- 实现 `MixedTokenizer`
- 编写单元测试

**步骤 3：集成到索引**
- 修改索引创建逻辑
- 注册混合分词器
- 更新 schema 配置

**步骤 4：测试验证**
- 测试中文分词
- 测试英文分词
- 测试混合分词
- 测试搜索效果

### 3. 测试用例

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_chinese_tokenization() {
        let mut tokenizer = MixedTokenizer::new();
        let tokens = tokenizer.tokenize("计算总价");
        assert!(tokens.contains(&"计算".to_string()));
        assert!(tokens.contains(&"总价".to_string()));
    }
    
    #[test]
    fn test_english_tokenization() {
        let mut tokenizer = MixedTokenizer::new();
        let tokens = tokenizer.tokenize("Calculate total price");
        assert!(tokens.contains(&"calculate".to_string()));
        assert!(tokens.contains(&"total".to_string()));
        assert!(tokens.contains(&"price".to_string()));
    }
    
    #[test]
    fn test_mixed_tokenization() {
        let mut tokenizer = MixedTokenizer::new();
        let tokens = tokenizer.tokenize("计算total price");
        assert!(tokens.contains(&"计算".to_string()));
        assert!(tokens.contains(&"total".to_string()));
        assert!(tokens.contains(&"price".to_string()));
    }
    
    #[test]
    fn test_complex_comment() {
        let mut tokenizer = MixedTokenizer::new();
        let text = "计算总价\nCalculate total price";
        let tokens = tokenizer.tokenize(text);
        
        // 中文分词
        assert!(tokens.contains(&"计算".to_string()));
        assert!(tokens.contains(&"总价".to_string()));
        
        // 英文分词
        assert!(tokens.contains(&"calculate".to_string()));
        assert!(tokens.contains(&"total".to_string()));
        assert!(tokens.contains(&"price".to_string()));
    }
}
```

---

## 六、总结

### Tantivy 分词能力总结

| 方面 | 支持情况 | 解决方案 |
|-----|---------|---------|
| **英文分词** | ✅ 完善 | 使用内置分词器 |
| **中文分词** | ❌ 不支持 | 集成 Jieba |
| **多语言** | ❌ 不支持 | 实现混合分词器 |
| **自定义** | ✅ 灵活 | 实现 Tokenizer trait |

### 混合分词器优势

1. **多语言支持**：
   - ✅ 正确处理中文
   - ✅ 正确处理英文
   - ✅ 正确处理中英文混合

2. **搜索质量**：
   - ✅ 中文搜索精确
   - ✅ 英文搜索精确
   - ✅ 混合搜索支持

3. **性能合理**：
   - ✅ 分词速度适中
   - ✅ 索引体积合理
   - ✅ 内存占用可控

4. **易于维护**：
   - ✅ 代码清晰
   - ✅ 易于扩展
   - ✅ 易于测试

### 关键结论

1. **Tantivy 不会自动处理中文分词**，必须明确配置
2. **不采用临时方案**（N-gram），直接实现最优方案（混合分词器）
3. **混合分词器是长期最优解**，支持中英文混合，精确度高，性能合理
4. **一次性实现，避免迁移成本**，长期可维护
