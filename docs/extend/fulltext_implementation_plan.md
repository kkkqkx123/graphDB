# 全文检索实现计划

## 项目概述

基于 `fulltext_search_design.md` 设计方案，制定详细的实现计划和任务分解。

---

## 阶段一：基础框架搭建（第 1-3 天）

### 任务 1.1：添加依赖配置

```toml
# Cargo.toml 修改
[dependencies]
tantivy = { version = "0.24", optional = true }

[features]
default = ["redb", "embedded", "server", "c-api", "fulltext-tantivy"]
fulltext-tantivy = ["dep:tantivy"]
fulltext-builtin = []
```

**验收标准**:
- [ ] `cargo check` 通过
- [ ] 特性开关工作正常

---

### 任务 1.2：创建模块结构

**目录结构**:
```
src/storage/fulltext/
├── mod.rs              # 模块入口
├── types.rs            # 类型定义
├── provider.rs         # Provider trait
├── error.rs            # 错误类型
├── tantivy_impl/       # Tantivy 实现
│   ├── mod.rs
│   ├── index_manager.rs
│   ├── searcher.rs
│   └── schema.rs
└── builtin_impl/       # 内置实现（占位）
    └── mod.rs
```

**验收标准**:
- [ ] 目录结构创建完成
- [ ] 所有文件包含基础框架代码
- [ ] 模块能正常编译

---

### 任务 1.3：定义核心类型

**文件**: `src/storage/fulltext/types.rs`

**需要定义的类型**:
1. `FulltextIndexConfig` - 全文索引配置
2. `FulltextProviderType` - 提供者类型枚举
3. `TokenizerType` - 分词器类型
4. `FulltextOptions` - 索引选项
5. `SearchOptions` - 搜索选项
6. `SearchResults` - 搜索结果
7. `SearchResult` - 单个结果
8. `IndexStats` - 索引统计

**验收标准**:
- [ ] 所有类型定义完成
- [ ] 实现 `Serialize`/`Deserialize`
- [ ] 实现 `Debug`/`Clone`
- [ ] 单元测试通过

---

### 任务 1.4：扩展索引类型枚举

**文件**: `src/core/types/index.rs`

**修改内容**:
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
pub enum IndexType {
    #[serde(rename = "tag")]
    TagIndex,
    #[serde(rename = "edge")]
    EdgeIndex,
    #[serde(rename = "fulltext")]      // 新增
    FulltextIndex,
}
```

**验收标准**:
- [ ] 枚举扩展完成
- [ ] 序列化/反序列化测试通过
- [ ] 不影响现有功能

---

## 阶段二：Tantivy 实现（第 4-10 天）

### 任务 2.1：实现 IndexManager

**文件**: `src/storage/fulltext/tantivy_impl/index_manager.rs`

**功能列表**:
- [ ] `new(base_path: PathBuf)` - 构造函数
- [ ] `create_index(name: &str)` - 创建索引
- [ ] `open_index(name: &str)` - 打开索引
- [ ] `delete_index(name: &str)` - 删除索引
- [ ] `get_writer(name: &str)` - 获取写入器
- [ ] `get_reader(name: &str)` - 获取读取器
- [ ] `commit(name: &str)` - 提交变更

**验收标准**:
- [ ] 所有功能实现
- [ ] 错误处理完善
- [ ] 单元测试覆盖率 > 80%

---

### 任务 2.2：实现 Schema 定义

**文件**: `src/storage/fulltext/tantivy_impl/schema.rs`

**功能列表**:
- [ ] 定义文档 Schema
- [ ] 支持动态字段
- [ ] 字段类型映射

**Schema 结构**:
```rust
pub struct FulltextSchema {
    pub doc_id_field: Field,
    pub content_field: Field,
    pub schema: Schema,
}
```

**验收标准**:
- [ ] Schema 定义正确
- [ ] 字段类型映射正确
- [ ] 单元测试通过

---

### 任务 2.3：实现 Searcher

**文件**: `src/storage/fulltext/tantivy_impl/searcher.rs`

**功能列表**:
- [ ] `search()` - 基本搜索
- [ ] `build_query()` - 查询构建
- [ ] `extract_results()` - 结果提取
- [ ] `calculate_score()` - 评分计算

**支持的查询类型**:
- 词项查询
- 短语查询
- 布尔查询
- 前缀查询

**验收标准**:
- [ ] 所有查询类型支持
- [ ] 搜索结果正确
- [ ] 性能测试通过

---

### 任务 2.4：实现 FulltextProvider Trait
**负责人**: 开发团队
**时间**: 2 天
**优先级**: 高

**文件**: `src/storage/fulltext/tantivy_impl/mod.rs`

**功能列表**:
- [ ] `create_index()` - 创建索引
- [ ] `drop_index()` - 删除索引
- [ ] `index_document()` - 索引文档
- [ ] `batch_index_documents()` - 批量索引
- [ ] `delete_document()` - 删除文档
- [ ] `search()` - 搜索
- [ ] `get_stats()` - 获取统计

**验收标准**:
- [ ] Trait 完整实现
- [ ] 异步接口正确
- [ ] 错误处理完善

---

### 任务 2.5：集成到 StorageClient

**文件**: `src/storage/storage_client.rs`

**修改内容**:
```rust
pub trait StorageClient: Send + Sync {
    // 现有接口...
    
    // 新增全文检索接口
    async fn create_fulltext_index(
        &self,
        config: FulltextIndexConfig,
    ) -> Result<()>;
    
    async fn fulltext_search(
        &self,
        index_name: &str,
        query: &str,
        options: &SearchOptions,
    ) -> Result<SearchResults>;
}
```

**验收标准**:
- [ ] 接口添加完成
- [ ] 实现类更新
- [ ] 集成测试通过

---

## 阶段三：查询层集成（第 11-14 天）

### 任务 3.1：实现 FulltextScanExecutor

**文件**: `src/query/executor/data_access/fulltext_scan.rs`

**功能列表**:
- [ ] 执行器结构定义
- [ ] `execute()` 方法实现
- [ ] 结果转换逻辑

**验收标准**:
- [ ] 执行器能正确执行
- [ ] 结果格式正确
- [ ] 单元测试通过

---

### 任务 3.2：扩展查询解析器

**支持的语法**:
```sql
-- CONTAINS 表达式
WHERE field CONTAINS "text"

-- MATCH 表达式
WHERE field MATCH "text"

-- 评分函数
score(vertex) as relevance
```

**文件修改**:
- `src/query/parser/ast/expr.rs` - 添加表达式类型
- `src/query/parser/parser/expr_parser.rs` - 添加解析逻辑

**验收标准**:
- [ ] 新语法能正确解析
- [ ] 解析器测试通过
- [ ] 错误提示友好

---

### 任务 3.3：实现查询计划生成

**文件**: `src/query/planner/`

**功能列表**:
- [ ] 识别全文检索条件
- [ ] 生成 FulltextScan 计划节点
- [ ] 成本估算

**验收标准**:
- [ ] 计划生成正确
- [ ] 成本估算合理
- [ ] 集成测试通过

---

## 阶段四：测试与优化（第 15-19 天）

### 任务 4.1：单元测试

**测试范围**:
- [ ] 类型定义测试
- [ ] IndexManager 测试
- [ ] Searcher 测试
- [ ] Provider 测试
- [ ] Executor 测试

**验收标准**:
- [ ] 测试覆盖率 > 80%
- [ ] 所有测试通过
- [ ] 边界条件覆盖

---

### 任务 4.2：集成测试

**测试场景**:
- [ ] 创建全文索引
- [ ] 索引文档
- [ ] 执行搜索
- [ ] 删除文档
- [ ] 删除索引
- [ ] 并发访问

**验收标准**:
- [ ] 所有场景测试通过
- [ ] 性能指标达标
- [ ] 无内存泄漏

---

### 任务 4.3：性能测试

**测试指标**:
- [ ] 索引速度: > 1000 文档/秒
- [ ] 查询延迟: < 100ms (P95)
- [ ] 并发查询: 支持 100+ QPS

**验收标准**:
- [ ] 性能指标达标
- [ ] 性能报告生成

---

### 任务 4.4：文档编写

**文档内容**:
- [ ] API 文档
- [ ] 使用指南
- [ ] 性能调优指南
- [ ] 故障排查指南

**验收标准**:
- [ ] 文档完整
- [ ] 示例代码可运行

---

## 阶段五：内置实现（可选，第 20-26 天）

### 任务 5.1：实现倒排索引
**负责人**: 开发团队
**时间**: 3 天
**优先级**: 低

**文件**: `src/storage/fulltext/builtin_impl/inverted_index.rs`

**功能列表**:
- [ ] 倒排索引结构
- [ ] 文档添加/删除
- [ ] BM25 评分计算
- [ ] 持久化存储

---

### 任务 5.2：实现分词器
**负责人**: 开发团队
**时间**: 2 天
**优先级**: 低

**文件**: `src/storage/fulltext/builtin_impl/tokenizer.rs`

**功能列表**:
- [ ] 标准分词器
- [ ] CJK 分词器
- [ ] 空格分词器

---

### 任务 5.3：实现 BuiltinProvider
**负责人**: 开发团队
**时间**: 2 天
**优先级**: 低

**文件**: `src/storage/fulltext/builtin_impl/mod.rs`

**功能列表**:
- [ ] 实现 FulltextProvider trait
- [ ] 集成到存储层

---

## 里程碑

| 里程碑 | 日期 | 交付物 |
|--------|------|--------|
| M1: 基础框架完成 | 第 3 天 | 模块结构、类型定义 |
| M2: Tantivy 实现完成 | 第 10 天 | 完整全文检索功能 |
| M3: 查询层集成完成 | 第 14 天 | 支持 SQL 语法 |
| M4: 测试完成 | 第 19 天 | 测试报告、性能报告 |
| M5: 内置实现完成 | 第 26 天 | 备选实现（可选） |

---

## 资源需求

### 人力资源
- 核心开发: 1-2 人
- 代码审查: 1 人
- 测试: 1 人

### 技术资源
- 开发环境: Rust 1.88+
- 测试数据: 100万+ 文档数据集
- 测试环境: 8核16G 服务器

---

## 风险与缓解

| 风险 | 可能性 | 影响 | 缓解措施 |
|------|--------|------|----------|
| Tantivy API 变更 | 低 | 高 | 锁定版本，及时更新 |
| 性能不达标 | 中 | 高 | 提前进行原型验证 |
| 内存泄漏 | 中 | 高 | 使用 valgrind/miri 检测 |
| 并发问题 | 中 | 高 | 充分的压力测试 |

---

## 附录

### A. 参考文档
- [Tantivy 文档](https://docs.rs/tantivy/)
- [BM25 算法论文](https://www.emerald.com/insight/content/doi/10.1108/eb026526/full/html)

### B. 相关文件
- `docs/extend/fulltext_search_design.md` - 设计方案
- `ref/bm25/` - BM25 参考实现
- `ref/inversearch/` - Inversearch 参考实现
