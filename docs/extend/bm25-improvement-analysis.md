# BM25 改进分析

## 一、当前实现现状

### 1.1 架构概览

BM25 功能位于 `crates/bm25/` crate 中，核心模块结构如下：

```
crates/bm25/src/
├── lib.rs                    # 库入口，re-export 核心类型
├── error.rs                  # 错误类型定义
├── config/                   # 配置模块
│   ├── mod.rs                # Bm25Config, FieldWeights, SearchConfig
│   ├── builder.rs            # 构建器模式
│   ├── loader.rs             # 环境变量/文件加载
│   └── validator.rs          # 配置校验
├── api/
│   ├── core/                 # 核心 API
│   │   ├── index.rs          # IndexManager（索引管理）
│   │   ├── schema.rs         # IndexSchema（Schema 定义）
│   │   ├── document.rs       # 文档 CRUD
│   │   ├── search.rs         # 搜索逻辑
│   │   ├── stats.rs          # 索引统计
│   │   ├── stats_extractor.rs # TF/DF 统计提取
│   │   ├── batch.rs          # 批量操作
│   │   ├── delete.rs         # 删除操作
│   │   └── persistence.rs    # 持久化/备份
│   └── embedded/             # 嵌入式 API
│       └── index.rs          # Bm25Index 高层封装
└── storage/                  # 存储层
    ├── tantivy.rs            # Tantivy 存储实现
    ├── manager.rs            # 存储管理器
    └── common/               # 公共类型
        ├── trait.rs          # StorageInterface trait
        └── types.rs          # Bm25Stats, StorageInfo
```

主项目通过 `src/search/adapters/bm25_adapter.rs` 适配器使用 BM25 crate。

### 1.2 当前 Schema

当前 Schema 仅有 3 个字段：

```rust
// schema.rs
schema_builder.add_text_field("document_id", STRING | STORED);
schema_builder.add_text_field("title", TEXT | STORED);
schema_builder.add_text_field("content", TEXT | STORED);
```

**问题**：

- 字段过少，无法表达代码实体的结构化信息
- 缺少实体类型、原始名称、关键词等对代码搜索至关重要的字段
- `title` 和 `content` 使用默认分词器，无中文支持

### 1.3 当前分词

使用 Tantivy 默认分词器（`default` tokenizer），处理流程为：

```
原始文本 → 按标点和空白分割 → 移除超长 token (>40) → 转小写
```

**问题**：

- **无中文分词支持**：中文文本被当作整句处理，无法按词分割
- **无词干提取**：`calculating` 和 `calculate` 被视为不同词
- **无停用词过滤**：常见停用词（the, is, a 等）参与索引，增加噪声
- `stats_extractor.rs` 中使用简单 `split_whitespace()` 分词，与 Tantivy 分词不一致

### 1.4 当前搜索

```rust
// search.rs - parse_query
// 对每个查询词项，在 title 和 content 字段中搜索
// 使用 BooleanQuery (Should) 组合
```

**问题**：

- 无字段权重（虽然 `FieldWeights` 配置存在，但未在查询中使用）
- 无模糊搜索支持
- 无短语搜索支持
- 查询解析过于简单，无法处理复杂查询

### 1.5 当前统计

```rust
// stats.rs
let total_terms = searcher.num_docs() * 100; // 粗略估算
```

**问题**：

- 平均文档长度为粗略估算，不准确
- `stats_extractor.rs` 使用 `split_whitespace()` 分词，与 Tantivy 分词不一致

## 二、改进方案

### 2.1 高优先级改进

#### 2.1.1 混合分词器（中文+英文）

**目标**：支持中文分词，同时保持英文分词质量

**方案**：集成 `jieba-rs` 实现混合分词器

**实现要点**：

- 新增 `crates/bm25/src/tokenizer/` 模块
- 实现 `MixedTokenizer`：自动检测中文字符，中文用 Jieba 分词，英文用简单分词+小写
- 实现 Tantivy `Tokenizer` trait，注册为自定义分词器
- 在 `IndexManager` 创建索引时注册分词器

**处理效果对比**：

| 输入                      | 当前处理                          | 混合分词器                        |
| ------------------------- | --------------------------------- | --------------------------------- |
| `"计算总价"`              | `["计算总价"]`                    | `["计算", "总价"]`                |
| `"Calculate total price"` | `["calculate", "total", "price"]` | `["calculate", "total", "price"]` |
| `"计算total price"`       | `["计算total", "price"]`          | `["计算", "total", "price"]`      |

#### 2.1.2 Schema 增强

**目标**：增加结构化字段，支持代码实体搜索

**新增字段**：

| 字段名        | 类型   | 用途                                   |
| ------------- | ------ | -------------------------------------- |
| `entity_type` | STRING | 实体类型（function, class, struct 等） |
| `raw_name`    | TEXT   | 原始名称（精确+模糊匹配）              |
| `keywords`    | TEXT   | 关键词（多值，空格分隔）               |
| `file_path`   | STRING | 文件路径                               |
| `module_name` | STRING | 模块名                                 |

**优化字段**：

- `content` 字段使用混合分词器
- `raw_name` 字段使用英文词干分词器
- `keywords` 字段使用默认分词器

#### 2.1.3 查询改进

**目标**：利用字段权重和多种查询类型提升搜索质量

**改进内容**：

- 使用 `QueryParser` 替代手动构建查询
- 配置字段权重（`raw_name` > `keywords` > `title` > `content`）
- 支持模糊搜索（`FuzzyQuery`）
- 支持短语搜索（`PhraseQuery`）

#### 2.1.4 统计改进

**目标**：准确计算索引统计信息

**改进内容**：

- 移除 `stats_extractor.rs` 中的简单分词，统一使用 Tantivy 分词
- 准确计算平均文档长度
- 移除粗略估算

### 2.2 中优先级改进

#### 2.2.1 文本预处理

**目标**：在索引前对文本进行清洗和标准化

**改进内容**：

- 符号清理：移除引号、括号、标点等对搜索无意义的字符
- 名称标准化：将 `snake_case`、`camelCase`、`PascalCase` 转为空格分隔的自然语言
- 冗余词移除：移除模板中的连接词（`that does`, `with parameters` 等）

#### 2.2.2 关键词权重

**目标**：为不同来源的关键词分配不同权重

**改进内容**：

- 实体名称权重最高（1.0）
- 参数名权重中等（0.7）
- 文档字符串关键词权重较低（0.5）
- 通过字段权重和词项位置实现

### 2.3 低优先级改进

#### 2.3.1 语义关键词提取

利用代码上下文提取语义关键词，识别函数/类的核心用途和领域术语。

#### 2.3.2 多语言停用词支持

利用 Tantivy 内置的多语言停用词过滤器，支持更多语言的停用词过滤。

## 三、实施计划

### 阶段一：核心改进（高优先级）

1. **添加 jieba-rs 依赖**
2. **实现混合分词器** `tokenizer/mixed_tokenizer.rs`
3. **增强 Schema** 添加结构化字段
4. **改进查询逻辑** 使用 QueryParser + 字段权重
5. **修复统计计算** 移除粗略估算
6. **注册自定义分词器** 到 IndexManager

### 阶段二：质量提升（中优先级）

7. **文本预处理** 符号清理、名称标准化
8. **关键词权重** 通过字段权重实现

### 阶段三：高级功能（低优先级）

9. **语义关键词提取**
10. **多语言停用词支持**

## 四、关键设计决策

### 4.1 分词器选择

**决策**：使用 Jieba + 英文简单分词的混合方案，不使用 N-gram

**理由**：

- N-gram 索引体积大（3-5x），精确度中等
- Jieba 分词精确度高，索引体积合理（1.5x）
- 混合分词器是长期最优解，避免迁移成本

### 4.2 停用词处理

**决策**：不在文本生成阶段过滤停用词，由 Tantivy 在索引阶段处理

**理由**：

- Tantivy 内置停用词过滤器更完善
- 停用词过滤应在索引和查询阶段进行
- 避免维护不完善的停用词列表

### 4.3 Schema 兼容性

**决策**：新增字段，保持 `document_id`、`title`、`content` 向后兼容

**理由**：

- 主项目适配器依赖现有字段
- 新增字段为可选，不影响现有功能
- 渐进式迁移，降低风险

## 五、依赖变更

```toml
# crates/bm25/Cargo.toml 新增
jieba-rs = "0.7"
```

## 六、影响范围

### 6.1 直接影响

- `crates/bm25/` crate 内部：schema、search、index、stats_extractor 模块
- `crates/bm25/Cargo.toml`：新增依赖

### 6.2 间接影响

- `src/search/adapters/bm25_adapter.rs`：适配器可能需要适配新字段
- 现有索引数据：Schema 变更后需要重建索引

### 6.3 不受影响

- `crates/inversearch/`：独立搜索引擎
- `src/search/` 其他模块：接口不变
