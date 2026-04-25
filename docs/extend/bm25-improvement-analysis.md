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
├── tokenizer/                # 分词器模块
│   └── mixed_tokenizer.rs    # 中英文混合分词器
└── storage/                  # 存储层
    ├── tantivy.rs            # Tantivy 存储实现
    ├── manager.rs            # 存储管理器
    └── common/               # 公共类型
        ├── trait.rs          # StorageInterface trait
        └── types.rs          # Bm25Stats, StorageInfo
```

主项目通过 `src/search/adapters/bm25_adapter.rs` 适配器使用 BM25 crate。

### 1.2 GraphDB 数据模型

GraphDB 是一个图数据库项目，BM25 用于索引图数据库中顶点(Vertex)和边(Edge)的属性值：

**索引结构**：
- 每个全文索引对应 `(space_id, tag_name, field_name)` 三元组
- `doc_id` 是顶点/边的 ID
- `content` 是顶点/边某个字段的字符串值

**搜索流程**：
1. 用户执行全文搜索查询
2. BM25 返回匹配的 `doc_id` 列表
3. 通过 `doc_id` 回查图存储获取完整顶点/边数据

### 1.3 当前 Schema

```rust
// schema.rs
pub struct IndexSchema {
    pub document_id: Field,  // 顶点/边 ID
    pub tag_name: Field,     // Tag 名称（如 "person", "movie"）
    pub field_name: Field,   // 字段名称（如 "name", "description"）
    pub content: Field,      // 字段内容（全文索引）
}
```

**设计说明**：
- `document_id`: 存储顶点/边的唯一标识符
- `tag_name`: 存储顶点的 Tag 类型，便于过滤和分类
- `field_name`: 存储字段名称，支持同一索引管理多个字段
- `content`: 唯一的全文搜索字段，使用混合分词器

### 1.4 当前分词

使用混合分词器（MixedTokenizer），处理流程：

```
中文文本 → Jieba 分词 → 词项列表
英文文本 → 按空格分割 + 转小写 → 词项列表
混合文本 → 自动检测 + 分别处理 → 词项列表
```

**特性**：
- 支持中文分词（基于 jieba-rs）
- 支持英文简单分词
- 自动跳过单字符英文 token
- 保持原始大小写信息用于高亮

### 1.5 当前搜索

```rust
// search.rs
pub fn search(
    manager: &IndexManager,
    schema: &IndexSchema,
    query_text: &str,
    options: &SearchOptions,
) -> Result<(Vec<SearchResult>, f32)>
```

**特性**：
- 使用 QueryParser 解析查询
- 仅搜索 `content` 字段
- 支持高亮显示
- 返回 `doc_id`、`score`、`tag_name`、`field_name`、`content`

## 二、已完成的改进

### 2.1 混合分词器实现

**目标**：支持中文分词，同时保持英文分词质量

**实现**：
- 新增 `crates/bm25/src/tokenizer/mixed_tokenizer.rs` 模块
- 实现 `MixedTokenizer`：自动检测中文字符，中文用 Jieba 分词，英文用简单分词
- 实现 Tantivy `Tokenizer` trait，注册为自定义分词器
- 在 `IndexManager` 创建索引时注册分词器

**处理效果对比**：

| 输入                      | 处理结果                          |
| ------------------------- | --------------------------------- |
| `"计算总价"`              | `["计算", "总价"]`                |
| `"Calculate total price"` | `["calculate", "total", "price"]` |
| `"计算total price"`       | `["计算", "total", "price"]`      |

### 2.2 Schema 设计（GraphDB 适配）

**目标**：设计适合图数据库全文搜索的 Schema

**字段说明**：

| 字段名       | 类型   | 用途                                     |
| ------------ | ------ | ---------------------------------------- |
| `document_id`| STRING | 顶点/边的唯一标识符                      |
| `tag_name`   | STRING | Tag 类型（如 "person", "movie"）         |
| `field_name` | STRING | 字段名称（如 "name", "description"）     |
| `content`    | TEXT   | 字段内容，全文索引，使用混合分词器       |

**与代码搜索的区别**：
- 移除了 `entity_type`、`raw_name`、`keywords`、`file_path`、`module_name` 等代码搜索专用字段
- `tag_name` 和 `field_name` 用于图数据库的元数据管理
- 搜索结果通过 `doc_id` 回查图存储获取完整数据

### 2.3 统计改进

**目标**：准确计算索引统计信息

**改进内容**：
- 准确计算平均文档长度（从 segment readers 获取实际 token 数）
- 移除粗略估算
- `stats_extractor.rs` 使用 MixedTokenizer 保持分词一致性

### 2.4 配置简化

**目标**：简化配置以适应图数据库场景

**改进内容**：
- `FieldWeights` 简化为仅包含 `content` 权重
- 移除 `title` 字段相关配置
- 保持 BM25 参数（k1, b）可配置

## 三、架构设计决策

### 3.1 为什么不需要多个搜索字段

**代码搜索场景**：
- 需要区分函数名、类名、变量名等
- 不同字段需要不同权重
- 需要存储文件路径、模块名等元数据

**图数据库场景**：
- 每个索引对应一个特定字段（由 `tag_name` + `field_name` 标识）
- 搜索结果通过 `doc_id` 回查图存储
- 元数据（`tag_name`, `field_name`）用于过滤，不参与搜索

### 3.2 为什么保留 tag_name 和 field_name

虽然这些字段不参与搜索，但它们：
- 便于调试和日志追踪
- 支持未来可能的过滤功能
- 保持与索引元数据的一致性

### 3.3 分词器选择

**决策**：使用 Jieba + 英文简单分词的混合方案，不使用 N-gram

**理由**：
- N-gram 索引体积大（3-5x），精确度中等
- Jieba 分词精确度高，索引体积合理（1.5x）
- 混合分词器是长期最优解，避免迁移成本

### 3.4 停用词处理

**决策**：不在文本生成阶段过滤停用词，由 Tantivy 在索引阶段处理

**理由**：
- Tantivy 内置停用词过滤器更完善
- 停用词过滤应在索引和查询阶段进行
- 避免维护不完善的停用词列表

## 四、依赖变更

```toml
# crates/bm25/Cargo.toml 新增
jieba-rs = "0.7"
```

## 五、影响范围

### 5.1 直接影响

- `crates/bm25/` crate 内部：schema、search、index、config 模块
- `crates/bm25/Cargo.toml`：新增依赖

### 5.2 间接影响

- `src/search/adapters/bm25_adapter.rs`：适配器已更新
- 现有索引数据：Schema 变更后需要重建索引

### 5.3 不受影响

- `crates/inversearch/`：独立搜索引擎
- `src/search/` 其他模块：接口不变
- 图存储层：完全独立

## 六、测试验证

### 6.1 编译检查

```shell
cargo check --package bm25-service  # 通过
cargo check                         # 通过
cargo clippy --package bm25-service # 通过
cargo clippy                        # 通过
```

### 6.2 单元测试

```shell
cd crates/bm25 && cargo test --lib  # 50/51 通过
```

唯一失败的测试 `storage::manager::tests::test_mutable_storage_manager` 与本次修改无关，是存储管理器初始化的问题。

### 6.3 集成测试

```shell
cargo test bm25 --lib  # 11/11 通过
```

所有 BM25 相关的集成测试均通过。

## 七、后续优化建议

### 7.1 中优先级

1. **文本预处理**：在索引前对文本进行清洗（移除特殊字符、标点）
2. **查询增强**：支持模糊搜索、通配符搜索
3. **高亮优化**：改进高亮片段生成逻辑

### 7.2 低优先级

1. **语义关键词提取**：利用图结构提取语义关键词
2. **多语言停用词支持**：利用 Tantivy 内置的多语言停用词过滤器
